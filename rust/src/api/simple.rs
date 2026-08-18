use std::fs::File;
use std::io::{Write, BufReader, BufRead};
use std::process::{Command, Child, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use std::net::{TcpStream, SocketAddr, ToSocketAddrs, IpAddr};
use std::path::PathBuf;
use url::Url;
use base64::{Engine as _, engine::general_purpose};
use native_tls::TlsConnector;
use std::sync::mpsc;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

static PROXY_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
static TOR_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
static PSIPHON_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
static AETHER_PROCESS: Mutex<Option<Child>> = Mutex::new(None);
static ACTIVE_DNS: Mutex<Option<(String, String)>> = Mutex::new(None);

static TOR_BOOTSTRAP_PERCENT: Mutex<i32> = Mutex::new(0);
static AETHER_BOOTSTRAP_PERCENT: Mutex<i32> = Mutex::new(0);

static PSIPHON_CONNECTED: Mutex<bool> = Mutex::new(false);
static AETHER_CONNECTED: Mutex<bool> = Mutex::new(false);
static AETHER_STATUS_MSG: Mutex<String> = Mutex::new(String::new());

fn resolve_binary_path(name: &str) -> PathBuf {
    let file_name = PathBuf::from(name)
        .file_name()
        .map(|f| f.to_os_string())
        .unwrap_or_else(|| std::ffi::OsString::from(name));

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(&file_name);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    let p = PathBuf::from(name);
    if p.exists() {
        return p;
    }

    if let Ok(cur) = std::env::current_dir() {
        let candidate = cur.join(&file_name);
        if candidate.exists() {
            return candidate;
        }
    }

    p
}

fn get_safe_work_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("RedCloud");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[cfg(target_os = "windows")]
fn notify_windows_proxy_change() {
    #[link(name = "wininet")]
    extern "system" {
        fn InternetSetOptionW(
            h_internet: *mut std::ffi::c_void,
            dw_option: u32,
            lp_buffer: *mut std::ffi::c_void,
            dw_buffer_length: u32,
        ) -> i32;
    }

    unsafe {
        const INTERNET_OPTION_SETTINGS_CHANGED: u32 = 39;
        const INTERNET_OPTION_REFRESH: u32 = 37;
        InternetSetOptionW(std::ptr::null_mut(), INTERNET_OPTION_SETTINGS_CHANGED, std::ptr::null_mut(), 0);
        InternetSetOptionW(std::ptr::null_mut(), INTERNET_OPTION_REFRESH, std::ptr::null_mut(), 0);
    }
}

#[cfg(target_os = "windows")]
fn get_global_job_object() -> Option<usize> {
    static WIN_JOB_OBJECT: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    *WIN_JOB_OBJECT.get_or_init(|| {
        unsafe {
            use windows_sys::Win32::System::JobObjects::{
                CreateJobObjectW, SetInformationJobObject, JobObjectExtendedLimitInformation,
                JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            };

            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job as usize == 0 || job as isize == -1 {
                return None;
            }

            let mut info = std::mem::zeroed::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            let size = std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32;
            let res = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                size,
            );

            if res == 0 {
                windows_sys::Win32::Foundation::CloseHandle(job);
                None
            } else {
                Some(job as usize)
            }
        }
    })
}

#[cfg(target_os = "windows")]
fn assign_child_to_job(child: &std::process::Child) {
    use std::os::windows::io::AsRawHandle;
    let child_handle = child.as_raw_handle();
    if let Some(job_handle_usize) = get_global_job_object() {
        unsafe {
            use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
            let h_job = job_handle_usize as windows_sys::Win32::Foundation::HANDLE;
            let h_proc = child_handle as windows_sys::Win32::Foundation::HANDLE;
            let _ = AssignProcessToJobObject(h_job, h_proc);
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[flutter_rust_bridge::frb(non_opaque)]
pub struct ProxyNode {
    pub name: String,
    pub protocol: String,
    pub raw_url: String,
}

pub fn is_connected() -> bool {
    let process_guard = PROXY_PROCESS.lock().unwrap_or_else(|e| e.into_inner());
    process_guard.is_some()
}

pub fn is_tor_connected() -> bool {
    let process_guard = TOR_PROCESS.lock().unwrap_or_else(|e| e.into_inner());
    process_guard.is_some()
}

pub fn is_psiphon_connected() -> bool {
    let process_guard = PSIPHON_PROCESS.lock().unwrap_or_else(|e| e.into_inner());
    process_guard.is_some()
}

pub fn is_aether_connected() -> bool {
    let process_guard = AETHER_PROCESS.lock().unwrap_or_else(|e| e.into_inner());
    process_guard.is_some()
}

pub fn is_hybrid_connected() -> bool {
    let proxy_guard = PROXY_PROCESS.lock().unwrap_or_else(|e| e.into_inner());
    let aether_guard = AETHER_PROCESS.lock().unwrap_or_else(|e| e.into_inner());
    proxy_guard.is_some() && aether_guard.is_some()
}

pub fn is_dns_active() -> bool {
    ACTIVE_DNS.lock().unwrap_or_else(|e| e.into_inner()).is_some()
}

pub fn get_tor_bootstrap_progress() -> i32 {
    *TOR_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn is_psiphon_bootstrap_done() -> bool {
    *PSIPHON_CONNECTED.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn get_aether_bootstrap_progress() -> i32 {
    *AETHER_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn is_aether_bootstrap_done() -> bool {
    *AETHER_CONNECTED.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn get_aether_status_text() -> String {
    AETHER_STATUS_MSG.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn ping_dns_server(ip: String) -> i32 {
    let addr = format!("{}:53", ip).parse::<SocketAddr>();
    if let Ok(socket_addr) = addr {
        let start = Instant::now();
        if TcpStream::connect_timeout(&socket_addr, Duration::from_millis(1500)).is_ok() {
            return start.elapsed().as_millis() as i32;
        }
    }
    -1
}

pub fn ping_proxy_server(host: String, port: u16) -> i32 {
    let addr = format!("{}:{}", host, port);
    let start = Instant::now();
    if let Ok(addrs) = addr.to_socket_addrs() {
        for socket_addr in addrs {
            if TcpStream::connect_timeout(&socket_addr, Duration::from_millis(1500)).is_ok() {
                return start.elapsed().as_millis() as i32;
            }
        }
    }
    -1
}

pub fn set_system_dns(primary: String, secondary: String) -> Result<String, String> {
    let mut process_guard = ACTIVE_DNS.lock().unwrap_or_else(|e| e.into_inner());

    if process_guard.is_some() {
        return Err("یک دی‌ان‌اس در حال حاضر فعال است. ابتدا آن را خاموش کنید.".to_string());
    }

    let primary_ip: IpAddr = primary.trim().parse()
        .map_err(|_| "آدرس آی‌پی اولیه نامعتبر است.".to_string())?;

    let secondary_ip: IpAddr = secondary.trim().parse()
        .map_err(|_| "آدرس آی‌پی ثانویه نامعتبر است.".to_string())?;

    let script = format!(
        "Get-NetAdapter | Where-Object {{$_.Status -eq 'Up'}} | Set-DnsClientServerAddress -ServerAddresses ('{}', '{}')",
        primary_ip, secondary_ip
    );

    let mut command = Command::new("powershell");
    command.args(&["-Command", &script])
           .stdin(Stdio::null())
           .stdout(Stdio::null())
           .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

    let output = command.output();

    match output {
        Ok(out) => {
            if out.status.success() {
                *process_guard = Some((primary, secondary));
                Ok("دی‌ان‌اس با موفقیت روی سیستم فعال شد.".to_string())
            } else {
                Err("خطا در اعمال تنظیمات دی‌ان‌اس. برنامه را به عنوان Administrator اجرا کنید.".to_string())
            }
        }
        Err(e) => Err(format!("خطا در اجرای اسکریپت پاورشل: {}", e)),
    }
}

pub fn reset_system_dns() -> Result<String, String> {
    let mut process_guard = ACTIVE_DNS.lock().unwrap_or_else(|e| e.into_inner());

    let script = "Get-NetAdapter | Where-Object {$_.Status -eq 'Up'} | Set-DnsClientServerAddress -ResetServerAddresses";

    let mut command = Command::new("powershell");
    command.args(&["-Command", script])
           .stdin(Stdio::null())
           .stdout(Stdio::null())
           .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

    let output = command.output();

    match output {
        Ok(out) => {
            if out.status.success() {
                *process_guard = None;
                Ok("تنظیمات دی‌ان‌اس سیستم به حالت خودکار (DHCP) بازگشت.".to_string())
            } else {
                Err("خطا در ریست دی‌ان‌اس. برنامه را به عنوان Administrator اجرا کنید.".to_string())
            }
        }
        Err(e) => Err(format!("خطا در ریست دی‌ان‌اس: {}", e))
    }
}

fn set_windows_system_proxy(enable: bool, host: String, port: u16) {
    if cfg!(target_os = "windows") {
        let enable_val = if enable { "1" } else { "0" };
        
        let mut cmd = Command::new("reg");
        cmd.args(&[
            "add", 
            "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", 
            "/v", "ProxyEnable", 
            "/t", "REG_DWORD", 
            "/d", enable_val, 
            "/f"
        ]);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(0x08000000);
        let _ = cmd.output();

        if enable {
            let proxy_server = format!("{}:{}", host, port);
            
            let mut cmd2 = Command::new("reg");
            cmd2.args(&[
                "add", 
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", 
                "/v", "ProxyServer", 
                "/t", "REG_SZ", 
                "/d", &proxy_server, 
                "/f"
            ]);
            #[cfg(target_os = "windows")]
            cmd2.creation_flags(0x08000000);
            let _ = cmd2.output();

            let mut cmd3 = Command::new("reg");
            cmd3.args(&[
                "add", 
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", 
                "/v", "ProxyOverride", 
                "/t", "REG_SZ", 
                "/d", "<local>;localhost;127.0.0.1", 
                "/f"
            ]);
            #[cfg(target_os = "windows")]
            cmd3.creation_flags(0x08000000);
            let _ = cmd3.output();
        }

        #[cfg(target_os = "windows")]
        notify_windows_proxy_change();
    }
}

fn process_aether_line(l: String) {
    let trimmed = l.trim().to_string();
    if trimmed.is_empty() { return; }

    {
        let mut status = AETHER_STATUS_MSG.lock().unwrap_or_else(|e| e.into_inner());
        *status = trimmed.clone();
    }

    let lower = trimmed.to_lowercase();

    if let Some(pos) = trimmed.find('%') {
        let start = trimmed[..pos].rfind(|c: char| !c.is_ascii_digit()).map(|p| p + 1).unwrap_or(0);
        if let Ok(p) = trimmed[start..pos].parse::<i32>() {
            let mut progress = AETHER_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
            *progress = p;
        }
    } else if lower.contains("discovering") || lower.contains("searching") || lower.contains("scanning") {
        let mut progress = AETHER_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
        if *progress < 45 { *progress = 45; }
    } else if lower.contains("probing") || lower.contains("testing") || lower.contains("handshake") {
        let mut progress = AETHER_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
        if *progress < 80 { *progress = 80; }
    }

    if lower.contains("connected") 
        || lower.contains("tunnel established")
        || lower.contains("listening") 
        || lower.contains("ready")
        || lower.contains("socks5")
        || lower.contains("validation passed") {
        let mut progress = AETHER_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
        *progress = 100;
        let mut connected = AETHER_CONNECTED.lock().unwrap_or_else(|e| e.into_inner());
        *connected = true;
    }
}

fn spawn_single_aether_mode(
    binary_path: &PathBuf, 
    mode: &str,
    noize: &str,
    warp_key: Option<&str>,
    team_token: Option<&str>,
) -> Result<Child, String> {
    let work_dir = get_safe_work_dir();
    let mut command = Command::new(binary_path);
    
    command.arg("--bind").arg("127.0.0.1:1819")
           .arg("--http-proxy").arg("127.0.0.1:1820")
           .arg("-4")
           .arg("--scan").arg("turbo");

    let selected_noize = if noize.trim().is_empty() { "firewall" } else { noize.trim() };
    command.arg("--noize").arg(selected_noize);

    if let Some(key) = warp_key {
        if !key.trim().is_empty() {
            command.arg("--key").arg(key.trim());
        }
    }

    if let Some(team) = team_token {
        if !team.trim().is_empty() {
            command.arg("--team").arg(team.trim());
        }
    }

    match mode {
        "masque_h2" => {
            command.arg("--masque").arg("--h2").arg("--fragment");
        },
        "gool" => {
            command.arg("--gool");
        },
        "wireguard" => {
            command.arg("--wg");
        },
        _ => {
            command.arg("--masque");
        }
    }

    command.current_dir(&work_dir)
           .stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

    let mut child = command.spawn()
        .map_err(|e| format!("خطا در اجرای aether.exe: {}", e))?;

    #[cfg(target_os = "windows")]
    assign_child_to_job(&child);

    if let Some(stdout) = child.stdout.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(l) = line {
                    process_aether_line(l);
                }
            }
        });
    }

    if let Some(stderr) = child.stderr.take() {
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(l) = line {
                    process_aether_line(l);
                }
            }
        });
    }

    Ok(child)
}

pub fn start_aether_core(
    binary_path: String,
    mode: String,
    noize: String,
    warp_key: Option<String>,
    team: Option<String>,
    use_system_proxy: bool,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let _ = Command::new("taskkill").args(&["/F", "/IM", "aether.exe"]).creation_flags(0x08000000).output();

    let mut process_guard = AETHER_PROCESS.lock().unwrap_or_else(|e| e.into_inner());
    if process_guard.is_some() {
        return Err("شبکه اتر در حال حاضر فعال است.".to_string());
    }

    let resolved_path = resolve_binary_path(&binary_path);
    
    let modes_to_try: Vec<&str> = if mode == "auto" || mode.is_empty() {
        vec!["masque_h3", "masque_h2", "gool", "wireguard"]
    } else {
        vec![mode.as_str()]
    };

    let mut connected_child: Option<Child> = None;
    let mut last_error = String::new();

    for current_mode in modes_to_try {
        let mode_persian_name = match current_mode {
            "masque_h3" => "MASQUE H3 (QUIC - سرعت بالا)",
            "masque_h2" => "MASQUE H2 + Fragment (ضد اختلال UDP)",
            "gool" => "Gool (WARP-in-WARP - تونل مضاعف)",
            "wireguard" => "WireGuard (وایرگارد)",
            _ => "MASQUE",
        };

        {
            let mut progress = AETHER_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
            *progress = 25;
            let mut connected = AETHER_CONNECTED.lock().unwrap_or_else(|e| e.into_inner());
            *connected = false;
            let mut status = AETHER_STATUS_MSG.lock().unwrap_or_else(|e| e.into_inner());
            *status = format!("در حال اسکن و آزمایش اتصال با پروتکل {}...", mode_persian_name);
        }

        match spawn_single_aether_mode(
            &resolved_path, 
            current_mode, 
            &noize, 
            warp_key.as_deref(), 
            team.as_deref()
        ) {
            Ok(mut child) => {
                let mut mode_success = false;
                for _ in 0..40 {
                    thread::sleep(Duration::from_millis(350));
                    
                    if let Ok(Some(exit_status)) = child.try_wait() {
                        last_error = format!("پروتکل {} با وضعیت {} بسته شد.", mode_persian_name, exit_status);
                        break;
                    }

                    if TcpStream::connect_timeout(&"127.0.0.1:1819".parse().unwrap(), Duration::from_millis(200)).is_ok() {
                        mode_success = true;
                        break;
                    }
                }

                if mode_success {
                    connected_child = Some(child);
                    let mut progress = AETHER_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
                    *progress = 100;
                    let mut connected = AETHER_CONNECTED.lock().unwrap_or_else(|e| e.into_inner());
                    *connected = true;
                    let mut status = AETHER_STATUS_MSG.lock().unwrap_or_else(|e| e.into_inner());
                    *status = format!("پل ارتباطی با پروتکل پایدار {} فعال شد!", mode_persian_name);
                    break;
                } else {
                    let _ = child.kill();
                    #[cfg(target_os = "windows")]
                    let _ = Command::new("taskkill").args(&["/F", "/IM", "aether.exe"]).creation_flags(0x08000000).output();
                }
            }
            Err(e) => {
                last_error = e;
            }
        }
    }

    if let Some(c) = connected_child {
        *process_guard = Some(c);
        if use_system_proxy {
            set_windows_system_proxy(true, "127.0.0.1".to_string(), 1820);
        }
        Ok("اتصال شبکه اتر با موفقیت برقرار شد.".to_string())
    } else {
        let mut status = AETHER_STATUS_MSG.lock().unwrap_or_else(|e| e.into_inner());
        *status = format!("خطا در تمام پروتکل‌ها: {}", last_error);
        Err(format!("امکان برقراری پل با هیچ یک از پروتکل‌ها فراهم نشد: {}", last_error))
    }
}

pub fn stop_aether_core() -> Result<String, String> {
    let mut process_guard = AETHER_PROCESS.lock().unwrap_or_else(|e| e.into_inner());

    if let Some(mut child) = process_guard.take() {
        match child.kill() {
            Ok(_) => {
                set_windows_system_proxy(false, String::new(), 0);
                
                let mut progress = AETHER_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
                *progress = 0;
                let mut connected = AETHER_CONNECTED.lock().unwrap_or_else(|e| e.into_inner());
                *connected = false;
                let mut status = AETHER_STATUS_MSG.lock().unwrap_or_else(|e| e.into_inner());
                *status = "اتصال قطع شد.".to_string();

                #[cfg(target_os = "windows")]
                let _ = Command::new("taskkill")
                    .args(&["/F", "/IM", "aether.exe"])
                    .creation_flags(0x08000000)
                    .output();

                Ok("اتصال شبکه اتر متوقف و سیستم به حالت عادی برگشت.".to_string())
            }
            Err(e) => Err(format!("خطا در متوقف کردن فرآیند اتر: {}", e)),
        }
    } else {
        Err("شبکه اتر در حال اجرا نیست.".to_string())
    }
}

/// شروع اتصال هیبریدی کاملاً منطبق بر استاندارد Sing-box 1.10 - 1.14+ با فیلد مدرن address آرایه‌ای
pub fn start_hybrid_connection(
    singbox_path: String,
    aether_path: String,
    selected_node: ProxyNode,
    aether_mode: String,
    aether_noize: String,
    aether_warp_key: Option<String>,
    aether_team: Option<String>,
    use_system_proxy: bool,
    use_tun_mode: bool,
    dns_type: String,
    dns_primary: String,
    _dns_secondary: String,
    dns_dot_host: Option<String>,
    _utls_fingerprint: Option<String>,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("taskkill").args(&["/F", "/IM", "sing-box.exe"]).creation_flags(0x08000000).output();
        let _ = Command::new("taskkill").args(&["/F", "/IM", "aether.exe"]).creation_flags(0x08000000).output();
    }

    let aether_res = start_aether_core(
        aether_path, 
        aether_mode, 
        aether_noize, 
        aether_warp_key, 
        aether_team, 
        false
    );
    if let Err(e) = aether_res {
        return Err(format!("خطا در راه‌اندازی پل اتر: {}", e));
    }

    let mut aether_ready = false;
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(350));
        if TcpStream::connect_timeout(&"127.0.0.1:1819".parse().unwrap(), Duration::from_millis(200)).is_ok() {
            aether_ready = true;
            break;
        }
    }

    if !aether_ready {
        let _ = stop_aether_core();
        return Err("پل ارتباطی اتر در زمان مقرر آماده نشد.".to_string());
    }

    let mut outbound_json = convert_link_to_outbound(
        selected_node,
        None,
        false,
        false,
        None,
        None,
        None,
    )?;

    outbound_json["detour"] = serde_json::json!("aether-bridge");

    let mut inbounds = serde_json::json!([
        {
            "type": "mixed",
            "tag": "mixed-in",
            "listen": "127.0.0.1",
            "listen_port": 2080
        }
    ]);

    if use_tun_mode {
        inbounds.as_array_mut().unwrap().push(serde_json::json!({
            "type": "tun",
            "tag": "tun-in",
            "interface_name": "RedCloud-TUN",
            "address": [
                "172.19.0.1/30"
            ],
            "auto_route": true,
            "strict_route": false,
            "stack": "mixed"
        }));
    }

    let dns_server_json = match dns_type.as_str() {
        "doh" => {
            let server_name = dns_dot_host.clone().unwrap_or_else(|| "cloudflare-dns.com".to_string());
            serde_json::json!({
                "type": "https",
                "tag": "dns_proxy",
                "server": dns_primary,
                "server_port": 443,
                "path": "/dns-query",
                "detour": "proxy-out",
                "tls": {
                    "enabled": true,
                    "server_name": server_name,
                    "insecure": true
                }
            })
        },
        _ => {
            serde_json::json!({
                "type": "https",
                "tag": "dns_proxy",
                "server": "1.1.1.1",
                "server_port": 443,
                "path": "/dns-query",
                "detour": "proxy-out",
                "tls": {
                    "enabled": true,
                    "server_name": "cloudflare-dns.com",
                    "insecure": true
                }
            })
        }
    };

    let final_config = serde_json::json!({
        "log": {
            "level": "info"
        },
        "experimental": {
            "clash_api": {
                "external_controller": "127.0.0.1:9090"
            }
        },
        "dns": {
            "servers": [
                dns_server_json,
                {
                    "type": "udp",
                    "tag": "dns_direct",
                    "server": "1.1.1.1",
                    "server_port": 53
                }
            ],
            "rules": [
                {
                    "query_type": ["A", "AAAA"],
                    "server": "dns_proxy"
                }
            ],
            "strategy": "ipv4_only",
            "final": "dns_proxy"
        },
        "inbounds": inbounds,
        "outbounds": [
            outbound_json,
            {
                "type": "socks",
                "tag": "aether-bridge",
                "server": "127.0.0.1",
                "server_port": 1819
            },
            {
                "type": "block",
                "tag": "block"
            },
            {
                "type": "direct",
                "tag": "direct"
            }
        ],
        "route": {
            "auto_detect_interface": true,
            "final": "proxy-out",
            "default_domain_resolver": "dns_proxy",
            "rules": [
                {
                    "action": "sniff"
                },
                {
                    "protocol": "dns",
                    "action": "hijack-dns"
                },
                {
                    "port": [53],
                    "action": "hijack-dns"
                },
                {
                    "process_name": [
                        "aether.exe", 
                        "tor.exe", 
                        "psiphon-tunnel-core.exe"
                    ],
                    "outbound": "direct"
                },
                {
                    "ip_is_private": true,
                    "outbound": "direct"
                },
                {
                    "network": "udp",
                    "port": [443],
                    "outbound": "block"
                }
            ]
        }
    });

    let work_dir = get_safe_work_dir();
    let temp_config_path = work_dir.join("redcloud_temp_hybrid_config.json");
    let mut file = File::create(&temp_config_path)
        .map_err(|e| format!("خطا در ساخت فایل پیکربندی هیبریدی: {}", e))?;
    
    file.write_all(final_config.to_string().as_bytes())
        .map_err(|e| format!("خطا در ذخیره‌سازی فایل هیبریدی: {}", e))?;

    let resolved_singbox = resolve_binary_path(&singbox_path);
    let mut command = Command::new(&resolved_singbox);
    command.arg("run").arg("-c").arg(&temp_config_path).current_dir(&work_dir);

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

    let child = command.spawn()
        .map_err(|e| format!("خطا در اجرای هسته Sing-box در مسیر {:?}: {}", resolved_singbox, e))?;

    #[cfg(target_os = "windows")]
    assign_child_to_job(&child);

    {
        let mut process_guard = PROXY_PROCESS.lock().unwrap_or_else(|e| e.into_inner());
        *process_guard = Some(child);
    }

    if use_system_proxy && !use_tun_mode {
        set_windows_system_proxy(true, "127.0.0.1".to_string(), 2080);
    }

    Ok("اتصال ترکیبی هیبریدی با موفقیت برقرار شد! هویت خارجی فعال است.".to_string())
}

pub fn stop_hybrid_connection() -> Result<String, String> {
    let _ = stop_proxy_core();
    let _ = stop_aether_core();
    set_windows_system_proxy(false, String::new(), 0);
    Ok("اتصال هیبریدی متوقف و سیستم به حالت عادی بازگشت.".to_string())
}

pub fn start_tor_core(binary_path: String, country_code: String, use_system_proxy: bool) -> Result<String, String> {
    let mut process_guard = TOR_PROCESS.lock().unwrap_or_else(|e| e.into_inner());

    if process_guard.is_some() {
        return Err("شبکه تور در حال حاضر فعال است.".to_string());
    }

    {
        let mut progress = TOR_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
        *progress = 0;
    }

    let work_dir = get_safe_work_dir();
    let temp_torrc_path = work_dir.join("redcloud_temp_torrc");
    let mut torrc_content = "SocksPort 9050\nHTTPTunnelPort 9051\n".to_string();

    if !country_code.is_empty() {
        torrc_content.push_str(&format!("ExitNodes {{{}}}\nStrictNodes 1\n", country_code));
    }

    let mut file = File::create(&temp_torrc_path)
        .map_err(|e| format!("خطا در ایجاد فایل پیکربندی تور در Temp: {}", e))?;
    
    file.write_all(torrc_content.as_bytes())
        .map_err(|e| format!("خطا در ذخیره فایل پیکربندی تور: {}", e))?;

    let resolved_path = resolve_binary_path(&binary_path);
    let mut command = Command::new(&resolved_path);
    command.arg("-f").arg(&temp_torrc_path)
           .current_dir(&work_dir)
           .stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

    let mut child = command.spawn()
        .map_err(|e| format!("خطا در اجرای فرآیند تور: {}", e))?;

    #[cfg(target_os = "windows")]
    assign_child_to_job(&child);

    let stdout = child.stdout.take().ok_or("خطا در دریافت خروجی متنی تور")?;

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(l) = line {
                if let Some(pos) = l.find("Bootstrapped ") {
                    let sub = &l[pos + 13..];
                    if let Some(percent_pos) = sub.find('%') {
                        if let Ok(percent) = sub[..percent_pos].parse::<i32>() {
                            let mut progress = TOR_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
                            *progress = percent;
                        }
                    }
                }
            }
        }
    });

    *process_guard = Some(child);
    
    if use_system_proxy {
        set_windows_system_proxy(true, "127.0.0.1".to_string(), 9051);
    }
    
    Ok("فرآیند تور آغاز شد. در حال اتصال به شبکه پیاز...".to_string())
}

pub fn stop_tor_core() -> Result<String, String> {
    let mut process_guard = TOR_PROCESS.lock().unwrap_or_else(|e| e.into_inner());

    if let Some(mut child) = process_guard.take() {
        match child.kill() {
            Ok(_) => {
                let work_dir = get_safe_work_dir();
                let temp_torrc_path = work_dir.join("redcloud_temp_torrc");
                let _ = std::fs::remove_file(temp_torrc_path);
                
                set_windows_system_proxy(false, String::new(), 0);
                
                let mut progress = TOR_BOOTSTRAP_PERCENT.lock().unwrap_or_else(|e| e.into_inner());
                *progress = 0;
                
                Ok("اتصال تور متوقف و سیستم به حالت عادی برگشت.".to_string())
            }
            Err(e) => Err(format!("خطا در متوقف کردن فرآیند تور: {}", e)),
        }
    } else {
        Err("شبکه تور در حال اجرا نیست.".to_string())
    }
}

pub fn start_psiphon_core(binary_path: String, country_code: String, use_system_proxy: bool) -> Result<String, String> {
    let mut process_guard = PSIPHON_PROCESS.lock().unwrap_or_else(|e| e.into_inner());

    if process_guard.is_some() {
        return Err("شبکه سایفون در حال حاضر فعال است.".to_string());
    }

    {
        let mut connected = PSIPHON_CONNECTED.lock().unwrap_or_else(|e| e.into_inner());
        *connected = false;
    }

    let work_dir = get_safe_work_dir();
    let temp_config_path = work_dir.join("redcloud_temp_psiphon_config.json");
    
    let mut config_json = serde_json::json!({
        "LocalSocksProxyPort": 9080,
        "LocalHttpProxyPort": 9081,
        "PropagationChannelId": "FFFFFFFFFFFFFFFF",
        "SponsorId": "FFFFFFFFFFFFFFFF"
    });

    if !country_code.is_empty() {
        config_json["EgressRegion"] = serde_json::json!(country_code);
    }

    let mut file = File::create(&temp_config_path)
        .map_err(|e| format!("خطا در ایجاد فایل تنظیمات سایفون: {}", e))?;
    
    file.write_all(config_json.to_string().as_bytes())
        .map_err(|e| format!("خطا در ذخیره فایل تنظیمات سایفون: {}", e))?;

    let resolved_path = resolve_binary_path(&binary_path);
    let mut command = Command::new(&resolved_path);
    command.arg("-config")
           .arg(&temp_config_path)
           .current_dir(&work_dir)
           .stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000); 

    let mut child = command.spawn()
        .map_err(|e| format!("خطا در اجرای فرآیند سایفون: {}", e))?;

    #[cfg(target_os = "windows")]
    assign_child_to_job(&child);

    let stdout = child.stdout.take().ok_or("خطا در دریافت خروجی متنی سایفون")?;

    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(l) = line {
                if l.contains("\"noticeType\":\"Tunnels\"") && l.contains("\"count\":1") {
                    let mut connected = PSIPHON_CONNECTED.lock().unwrap_or_else(|e| e.into_inner());
                    *connected = true; 
                }
            }
        }
    });

    *process_guard = Some(child);
    
    if use_system_proxy {
        set_windows_system_proxy(true, "127.0.0.1".to_string(), 9081);
    }
    
    Ok("در حال برقراری اتصال با سرورهای سایفون؛ لطفاً چند لحظه شکیبا باشید...".to_string())
}

pub fn stop_psiphon_core() -> Result<String, String> {
    let mut process_guard = PSIPHON_PROCESS.lock().unwrap_or_else(|e| e.into_inner());

    if let Some(mut child) = process_guard.take() {
        match child.kill() {
            Ok(_) => {
                let work_dir = get_safe_work_dir();
                let temp_config_path = work_dir.join("redcloud_temp_psiphon_config.json");
                let remote_server_list = work_dir.join("remote_server_list");
                
                let _ = std::fs::remove_file(temp_config_path);
                let _ = std::fs::remove_file(remote_server_list);
                
                let mut connected = PSIPHON_CONNECTED.lock().unwrap_or_else(|e| e.into_inner());
                *connected = false;

                set_windows_system_proxy(false, String::new(), 0);
                Ok("اتصال سایفون متوقف و سیستم به حالت عادی برگشت.".to_string())
            }
            Err(e) => Err(format!("خطا در متوقف کردن فرآیند سایفون: {}", e)),
        }
    } else {
        Err("شبکه سایفون در حال اجرا نیست.".to_string())
    }
}

fn scan_single_ip(ip: &str, port: u16, sni: &str, timeout_ms: u64) -> Option<u128> {
    let addr = format!("{}:{}", ip, port).parse::<SocketAddr>().ok()?;
    let start = Instant::now();
    
    let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).ok()?;
    
    let connector = TlsConnector::new().ok()?;
    let _tls_stream = connector.connect(sni, stream).ok()?;
    
    let duration = start.elapsed().as_millis();
    Some(duration)
}

pub fn run_cloudflare_scanner(uuid: String, path: String, worker: String) -> Vec<ProxyNode> {
    let ip_list = vec![
        "104.21.0.1", "104.22.0.1", "172.67.0.1", "104.27.110.232",
        "104.16.0.1", "104.18.0.1", "162.159.0.1", "104.26.0.1",
        "172.65.0.1", "104.24.0.1", "104.20.0.1", "104.25.0.1"
    ];

    let (tx, rx) = mpsc::channel();
    let mut handles = vec![];

    for ip in ip_list {
        let tx_clone = tx.clone();
        let worker_clone = worker.clone();
        let ip_str = ip.to_string();

        let handle = thread::spawn(move || {
            if let Some(latency) = scan_single_ip(&ip_str, 2053, &worker_clone, 1500) {
                let _ = tx_clone.send((ip_str, latency));
            }
        });
        handles.push(handle);
    }

    drop(tx);

    for h in handles {
        let _ = h.join();
    }

    let mut results = Vec::new();
    while let Ok((ip, latency)) = rx.try_recv() {
        results.push((ip, latency));
    }

    results.sort_by_key(|&(_, lat)| lat);

    let mut clean_nodes = Vec::new();
    for (ip, latency) in results {
        let encoded_path = urlencoding::encode(&path);
        let raw_url = format!(
            "vless://{}@{}:2053?encryption=none&security=tls&sni={}&fp=chrome&alpn=http%2F1.1&insecure=1&allowInsecure=1&type=ws&host={}&path={}#{}%3A2053%20%7C%20TLS%20%7C%20HTTP1.1%20%7C%20{}ms",
            uuid, ip, worker, worker, encoded_path, ip, latency
        );

        clean_nodes.push(ProxyNode {
            name: format!("Scanner | {} | {}ms", ip, latency),
            protocol: "vless".to_string(),
            raw_url,
        });
    }

    clean_nodes
}

pub fn start_proxy_with_node(
    binary_path: String,
    selected_node: ProxyNode,
    use_system_proxy: bool,
    custom_sni: Option<String>,
    enable_fragment: bool,
    enable_record_fragment: bool,
    tls_spoof: Option<String>,
    use_tun_mode: bool,
    dns_type: String,
    dns_primary: String,
    _dns_secondary: String,
    _dns_doh_url: Option<String>,
    dns_dot_host: Option<String>,
    utls_fingerprint: Option<String>,
    fragment_fallback_delay: Option<String>,
) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let _ = Command::new("taskkill")
        .args(&["/F", "/IM", "sing-box.exe"])
        .creation_flags(0x08000000)
        .output();

    let mut process_guard = PROXY_PROCESS.lock().unwrap_or_else(|e| e.into_inner());

    if process_guard.is_some() {
        return Err("پروکسی در حال حاضر فعال است.".to_string());
    }

    let outbound_json = convert_link_to_outbound(
        selected_node,
        custom_sni,
        enable_fragment,
        enable_record_fragment,
        tls_spoof,
        utls_fingerprint,
        fragment_fallback_delay,
    )?;

    let mut inbounds = serde_json::json!([
        {
            "type": "mixed",
            "tag": "mixed-in",
            "listen": "127.0.0.1",
            "listen_port": 2080
        }
    ]);

    if use_tun_mode {
        inbounds.as_array_mut().unwrap().push(serde_json::json!({
            "type": "tun",
            "tag": "tun-in",
            "interface_name": "RedCloud-TUN",
            "address": [
                "172.19.0.1/30"
            ],
            "auto_route": true,
            "strict_route": false,
            "stack": "mixed"
        }));
    }

    let dns_server_json = match dns_type.as_str() {
        "doh" => {
            let server_name = dns_dot_host.clone().unwrap_or_else(|| "cloudflare-dns.com".to_string());
            serde_json::json!({
                "type": "https",
                "tag": "dns_proxy",
                "server": dns_primary,
                "server_port": 443,
                "path": "/dns-query",
                "detour": "proxy-out",
                "tls": {
                    "enabled": true,
                    "server_name": server_name,
                    "insecure": true
                }
            })
        },
        _ => {
            serde_json::json!({
                "type": "https",
                "tag": "dns_proxy",
                "server": "1.1.1.1",
                "server_port": 443,
                "path": "/dns-query",
                "detour": "proxy-out",
                "tls": {
                    "enabled": true,
                    "server_name": "cloudflare-dns.com",
                    "insecure": true
                }
            })
        }
    };

    let final_config = serde_json::json!({
        "log": {
            "level": "info"
        },
        "experimental": {
            "clash_api": {
                "external_controller": "127.0.0.1:9090"
            }
        },
        "dns": {
            "servers": [
                dns_server_json,
                {
                    "type": "udp",
                    "tag": "dns_direct",
                    "server": "1.1.1.1",
                    "server_port": 53
                }
            ],
            "rules": [
                {
                    "query_type": ["A", "AAAA"],
                    "server": "dns_proxy"
                }
            ],
            "strategy": "ipv4_only",
            "final": "dns_proxy"
        },
        "inbounds": inbounds,
        "outbounds": [
            outbound_json,
            {
                "type": "block",
                "tag": "block"
            },
            {
                "type": "direct",
                "tag": "direct"
            }
        ],
        "route": {
            "auto_detect_interface": true,
            "final": "proxy-out",
            "default_domain_resolver": "dns_proxy",
            "rules": [
                {
                    "action": "sniff"
                },
                {
                    "protocol": "dns",
                    "action": "hijack-dns"
                },
                {
                    "port": [53],
                    "action": "hijack-dns"
                },
                {
                    "process_name": [
                        "aether.exe", 
                        "tor.exe", 
                        "psiphon-tunnel-core.exe"
                    ],
                    "outbound": "direct"
                },
                {
                    "ip_is_private": true,
                    "outbound": "direct"
                },
                {
                    "network": "udp",
                    "port": [443],
                    "outbound": "block"
                }
            ]
        }
    });

    let work_dir = get_safe_work_dir();
    let temp_config_path = work_dir.join("redcloud_temp_config.json");
    let mut file = File::create(&temp_config_path)
        .map_err(|e| format!("خطا در ساخت فایل پیکربندی: {}", e))?;
    
    file.write_all(final_config.to_string().as_bytes())
        .map_err(|e| format!("خطا در ذخیره‌سازی فایل پیکربندی: {}", e))?;

    let resolved_path = resolve_binary_path(&binary_path);
    let mut command = Command::new(&resolved_path);
    command.arg("run").arg("-c").arg(&temp_config_path)
           .current_dir(&work_dir);

    let log_file_path = work_dir.join("redcloud_sing_box_log.txt");
    let log_file = File::create(&log_file_path)
        .map_err(|e| format!("خطا در ایجاد فایل لاگ: {}", e))?;

    command.stdin(Stdio::null())
           .stdout(Stdio::from(log_file.try_clone().map_err(|e| e.to_string())?))
           .stderr(Stdio::from(log_file));

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

    let child = command.spawn();

    match child {
        Ok(c) => {
            #[cfg(target_os = "windows")]
            assign_child_to_job(&c);

            *process_guard = Some(c);
            
            if use_system_proxy && !use_tun_mode {
                set_windows_system_proxy(true, "127.0.0.1".to_string(), 2080);
            }
            
            Ok("اتصال با موفقیت برقرار شد.".to_string())
        }
        Err(e) => Err(format!("خطا در اجرای فرآیند هسته: {}", e)),
    }
}

pub fn stop_proxy_core() -> Result<String, String> {
    let mut process_guard = PROXY_PROCESS.lock().unwrap_or_else(|e| e.into_inner());

    if let Some(mut child) = process_guard.take() {
        match child.kill() {
            Ok(_) => {
                let work_dir = get_safe_work_dir();
                let temp_config_path = work_dir.join("redcloud_temp_config.json");
                let _ = std::fs::remove_file(temp_config_path);
                
                set_windows_system_proxy(false, String::new(), 0);
                Ok("پروکسی متوقف و سیستم به حالت عادی برگشت.".to_string())
            }
            Err(e) => Err(format!("خطا در متوقف کردن فرآیند: {}", e)),
        }
    } else {
        Err("پروکسی در حال اجرا نیست.".to_string())
    }
}

pub fn parse_import_links(input: String) -> Result<Vec<ProxyNode>, String> {
    let mut nodes = Vec::new();

    if input.starts_with("http://") || input.starts_with("https://") {
        return Err("لطفاً متن دریافت شده از لینک ساب را وارد کنید.".to_string());
    }

    let sanitized_input = input.trim().replace(|c: char| c.is_whitespace(), "");

    let mut base64_str = sanitized_input.clone();
    while base64_str.len() % 4 != 0 {
        base64_str.push('=');
    }

    let decoded_content = if let Ok(decoded_bytes) = general_purpose::STANDARD.decode(&base64_str) {
        String::from_utf8(decoded_bytes).unwrap_or_else(|_| input.clone())
    } else {
        input.clone()
    };

    for line in decoded_content.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            continue;
        }

        if let Ok(url) = Url::parse(line_trimmed) {
            let protocol = url.scheme().to_lowercase();
            if protocol == "vless" || protocol == "trojan" || protocol == "hysteria2" || protocol == "hy2" {
                let name = url.fragment()
                    .map(|f| urlencoding::decode(f).unwrap_or_else(|_| f.into()).to_string())
                    .unwrap_or_else(|| "سرور ناشناس".to_string());

                let normalized_protocol = if protocol == "hy2" { "hysteria2".to_string() } else { protocol };

                nodes.push(ProxyNode {
                    name,
                    protocol: normalized_protocol,
                    raw_url: line_trimmed.to_string(),
                });
            }
        }
    }

    if nodes.is_empty() {
        return Err("هیچ سرور معتبری در ورودی یافت نشد.".to_string());
    }

    Ok(nodes)
}

fn convert_link_to_outbound(
    node: ProxyNode,
    custom_sni: Option<String>,
    enable_fragment: bool,
    enable_record_fragment: bool,
    tls_spoof: Option<String>,
    utls_fingerprint: Option<String>,
    fragment_fallback_delay: Option<String>,
) -> Result<serde_json::Value, String> {
    let parsed_url = Url::parse(&node.raw_url).map_err(|e| e.to_string())?;
    let host = parsed_url.host_str().ok_or("هاست یافت نشد")?;
    let port = parsed_url.port().ok_or("پورت یافت نشد")?;
    
    let protocol = if node.protocol == "hy2" { "hysteria2" } else { node.protocol.as_str() };

    // ساخت Outbound برای Hysteria 2
    if protocol == "hysteria2" {
        let auth = parsed_url.username();
        let mut outbound = serde_json::json!({
            "type": "hysteria2",
            "tag": "proxy-out",
            "server": host,
            "server_port": port,
            "password": auth,
        });

        let mut sni = host.to_string();
        let mut insecure = true;
        let mut obfs_type = String::new();
        let mut obfs_password = String::new();
        let mut ech_config = String::new();

        for (key, val) in parsed_url.query_pairs() {
            match key.as_ref() {
                "sni" | "peer" => sni = val.into_owned(),
                "insecure" | "allowInsecure" => insecure = val == "1" || val == "true",
                "obfs" => obfs_type = val.into_owned(),
                "obfs-password" => obfs_password = val.into_owned(),
                "ech" | "ech_config" => ech_config = val.into_owned(),
                _ => {}
            }
        }

        let final_sni = if let Some(ref cs) = custom_sni {
            if !cs.trim().is_empty() { cs.trim().to_string() } else { sni }
        } else {
            sni
        };

        let mut tls_obj = serde_json::json!({
            "enabled": true,
            "server_name": final_sni,
            "insecure": insecure,
        });

        if !ech_config.is_empty() {
            let configs: Vec<&str> = ech_config.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            tls_obj["ech"] = serde_json::json!({
                "enabled": true,
                "config": configs,
                "pq_signature_schemes_enabled": true
            });
        }

        outbound["tls"] = tls_obj;

        if !obfs_type.is_empty() && !obfs_password.is_empty() {
            outbound["obfs"] = serde_json::json!({
                "type": obfs_type,
                "password": obfs_password
            });
        }

        return Ok(outbound);
    }

    // ساخت Outbound برای VLESS و Trojan
    let mut outbound = serde_json::json!({
        "type": protocol,
        "tag": "proxy-out",
        "server": host,
        "server_port": port,
    });

    if protocol == "vless" {
        let uuid = parsed_url.username();
        outbound["uuid"] = serde_json::json!(uuid);
    } else if protocol == "trojan" {
        let password = parsed_url.username();
        outbound["password"] = serde_json::json!(password);
    }

    let mut sni = "".to_string();
    let mut path = "".to_string();
    let mut network = "tcp".to_string();
    let mut security = "none".to_string();
    let mut ws_host = "".to_string();
    let mut pbk = "".to_string();
    let mut sid = "".to_string();
    let mut spx = "".to_string();
    let mut ech_config = "".to_string();
    let mut insecure = true;

    for (key, val) in parsed_url.query_pairs() {
        match key.as_ref() {
            "sni" => sni = val.into_owned(),
            "path" => path = val.into_owned(),
            "type" => network = val.into_owned(),
            "security" => security = val.into_owned(),
            "host" => ws_host = val.into_owned(),
            "pbk" | "public_key" => pbk = val.into_owned(),
            "sid" | "short_id" => sid = val.into_owned(),
            "spx" | "spider_x" => spx = val.into_owned(),
            "ech" | "ech_config" => ech_config = val.into_owned(),
            "insecure" | "allowInsecure" => insecure = val == "1" || val == "true",
            _ => {}
        }
    }

    let final_sni = if let Some(ref cs) = custom_sni {
        if !cs.trim().is_empty() {
            cs.trim().to_string()
        } else {
            sni
        }
    } else {
        sni
    };

    let final_fingerprint = if let Some(ref fp) = utls_fingerprint {
        if !fp.trim().is_empty() {
            fp.trim().to_string()
        } else {
            "chrome".to_string()
        }
    } else {
        "chrome".to_string()
    };

    if security == "tls" || security == "reality" {
        let mut tls_obj = serde_json::json!({
            "enabled": true,
            "server_name": final_sni,
            "insecure": insecure
        });

        if security == "reality" {
            let mut reality_obj = serde_json::json!({
                "enabled": true,
                "public_key": pbk,
            });
            if !sid.is_empty() {
                reality_obj["short_id"] = serde_json::json!(sid);
            }
            if !spx.is_empty() {
                reality_obj["spider_x"] = serde_json::json!(spx);
            }
            tls_obj["reality"] = reality_obj;
            tls_obj["utls"] = serde_json::json!({
                "enabled": true,
                "fingerprint": if final_fingerprint == "none" { "chrome".to_string() } else { final_fingerprint.clone() }
            });
        } else {
            if let Some(ref fp) = utls_fingerprint {
                if !fp.trim().is_empty() && fp != "none" {
                    tls_obj["utls"] = serde_json::json!({
                        "enabled": true,
                        "fingerprint": fp.trim()
                    });
                }
            }
        }

        // افزودن تنظیمات ECH در صورت وجود
        if !ech_config.is_empty() {
            let configs: Vec<&str> = ech_config.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            tls_obj["ech"] = serde_json::json!({
                "enabled": true,
                "config": configs,
                "pq_signature_schemes_enabled": true
            });
        }

        if enable_fragment {
            tls_obj["fragment"] = serde_json::json!(true);
            if let Some(ref delay) = fragment_fallback_delay {
                if !delay.trim().is_empty() {
                    tls_obj["fragment_fallback_delay"] = serde_json::json!(delay.trim());
                }
            }
        }
        if enable_record_fragment {
            tls_obj["record_fragment"] = serde_json::json!(true);
        }

        if let Some(ref spoof) = tls_spoof {
            if !spoof.trim().is_empty() {
                tls_obj["spoof"] = serde_json::json!(spoof.trim());
                tls_obj["spoof_method"] = serde_json::json!("default");
            }
        }

        outbound["tls"] = tls_obj;
    }

    if network == "ws" || network == "grpc" || network == "http" {
        let mut transport = serde_json::json!({
            "type": network,
        });
        
        if network == "ws" {
            if !path.is_empty() {
                transport["path"] = serde_json::json!(path);
            }
            if !ws_host.is_empty() {
                transport["headers"] = serde_json::json!({
                    "Host": ws_host
                });
            }
        } else if network == "grpc" {
            if !path.is_empty() {
                transport["service_name"] = serde_json::json!(path);
            }
        }
        
        outbound["transport"] = transport;
    }

    Ok(outbound)
}