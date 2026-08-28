<div align="center">

# 🛡️ RedCloud VPN (Linux Edition)

### کلاینت چندموتوره و نسل جدید ضدسانسور اینترنت برای لینوکس

**Next-Generation Anti-Censorship Client Powered by Flutter & Rust**

[![Release](https://img.shields.io/badge/Release-v3.5%20Linux-00D2FF.svg?style=for-the-badge\&logo=linux\&logoColor=white)](https://github.com/Devtahas/RedCloud-linux/releases/latest)
[![Flutter](https://img.shields.io/badge/Flutter-3.29-02569B?style=for-the-badge\&logo=flutter\&logoColor=white)](https://flutter.dev)
[![Rust](https://img.shields.io/badge/Rust-2021-DEA584?style=for-the-badge\&logo=rust\&logoColor=black)](https://www.rust-lang.org)
[![Sing-box](https://img.shields.io/badge/Sing--box-1.13.13-6C5DD3?style=for-the-badge\&logo=codeforces\&logoColor=white)](https://github.com/SagerNet/sing-box)
[![Platform](https://img.shields.io/badge/Platform-Kali%20%7C%20Debian%20%7C%20Arch%20%7C%20Fedora-black?style=for-the-badge\&logo=linux)](https://github.com/Devtahas/RedCloud-linux)
[![Telegram](https://img.shields.io/badge/Telegram-Channel-2CA5E0?style=for-the-badge\&logo=telegram\&logoColor=white)](https://t.me/DevTaha_project)

<br>

<p align="center">
<b>RedCloud VPN</b> یک نرم‌افزار متن‌باز، فوق‌العاده سبک و پرسرعت برای توزیع‌های مختلف لینوکس است.<br>
این پروژه با تلفیق رابط کاربری مدرن <b>Flutter (با موتور رندرینگ Impeller)</b> و هسته باینری قدرتمند <b>Rust (FFI Bridge)</b>، اتصال پایدار، گمنام و بدون نشت داده را در شدیدترین شرایط محدودیت اینترنت فراهم می‌کند.
</p>

</div>

---

## 🌟 معماری شبکه و جریان ترافیک (Traffic Flow)

در حالت اتصال هیبریدی، ترافیک سیستم از دو لایه عبور می‌کند:

```text
[ کاربر / کل ترافیک سیستم‌عامل ]
              │
              ▼
[ کارت شبکه مجازی TUN یا پروکسی سیستمی ]
              │
              ▼
[ هسته ویتوری (Sing-box v1.13.13) ]
              │
              │  VLESS Reality / Hysteria 2 / Trojan / ECH
              │
              │  زنجیره‌سازی از SOCKS5: 127.0.0.1:1819
              ▼
[ پل ضدسانسور اِتر (Aether MASQUE) ]
              │
              │  HTTP/3 QUIC 0-RTT / H2 Fragment
              ▼
[ لایه ابری Cloudflare Edge ]
              │
              ▼
[ اینترنت آزاد بدون فیلتر ]
```

---

## ✨ قابلیت‌های کلیدی (Key Features)

### 🚀 ۱. اتصال هیبریدی Aether + Sing-box

* زنجیره‌سازی خودکار بین **پل ضدسانسور اِتر (MASQUE)** و **هسته Sing-box**.
* پشتیبانی از اتصال از طریق ورکرها و زیرساخت‌های ابری خارجی.
* تغییر مسیر و لایه‌بندی ترافیک برای افزایش پایداری اتصال در شبکه‌های محدودشده.

### ⚡ ۲. پشتیبانی از پروتکل نسل جدید MASQUE (Aether Engine)

* اتصال مستقیم و پایدار به زیرساخت **Cloudflare Zero Trust** بدون نیاز به داشتن دامنه یا سرور اختصاصی.
* پشتیبانی از حالت‌های:

  * **MASQUE H3 (QUIC)**
  * **MASQUE H2 + Fragment**
  * **Gool (WARP-in-WARP)**
  * **WireGuard**
* پروفایل‌های پارازیت هوشمند ضد DPI:

  * **Firewall** — مخصوص شبکه‌های دارای محدودیت شدید
  * **Light**
  * **Aggressive**

### 💎 ۳. پشتیبانی جامع از پروتکل‌های پیشرفته ویتوری

* سازگاری با **VLESS Reality**، **Hysteria 2 (Hy2)** و **Trojan**.
* پشتیبانی از **ECH (Encrypted Client Hello)** برای رمزنگاری Client Hello.
* تست پینگ سریع و هم‌زمان سرورها با استفاده از هسته Rust.
* مرتب‌سازی خودکار سرورها بر اساس وضعیت اتصال و پاسخ‌دهی.

### 🛡️ ۴. ابزارهای پیشرفته Anti-DPI

* **uTLS Fingerprint:** شبیه‌سازی اثر انگشت مرورگرهای واقعی مانند Google Chrome، Firefox، Safari و Edge.
* **TLS Fragmentation & Record Fragmentation:** تقسیم ClientHello و رکوردهای TLS برای کاهش قابلیت شناسایی ترافیک توسط سامانه‌های DPI.
* **TLS Spoofing:** امکان استفاده از SNI مجاز یا جایگزین پیش از ارسال ترافیک اصلی.

### 🎮 ۵. کارت شبکه مجازی بومی (Linux TUN Mode)

* هدایت ترافیک کل سیستم‌عامل، ترمینال، ابزارهای تست، بازی‌های آنلاین و کلاینت‌ها از طریق `/dev/net/tun`.
* جلوگیری از ایجاد Loop ترافیکی با استفاده از استک پرسرعت User-Space و حالت `mixed`.
* تنظیم دسترسی `CAP_NET_ADMIN` برای اجرای TUN بدون نیاز به وارد کردن مکرر رمز عبور `sudo`.

### 🧅 ۶. شبکه‌های گمنامی Tor و Psiphon

* **Tor over MASQUE:** عبور ترافیک Tor از طریق پل MASQUE برای افزایش مقاومت در برابر مسدودسازی.
* امکان انتخاب کشور خروجی (Exit Node) در صورت پشتیبانی هسته Tor.
* **Psiphon over MASQUE:** استفاده از هسته رسمی Psiphon در کنار لایه انتقال MASQUE.

### 🎯 ۷. اسکنر دوحالته لایه ۷ Cloudflare (IP Scanner)

#### Quick Scan

* تست موازی IPهای منتخب.
* ارسال Handshake در لایه ۷ وب‌سوکت.
* بررسی پاسخ `HTTP 101`.

#### Deep Scan

* اسکن رنج‌های CIDR مربوط به Cloudflare.
* استفاده از فایل `cloudflare_IPs.txt`.
* بررسی تعداد زیادی IP برای یافتن نقاط اتصال مناسب.

### 🚦 ۸. تغییر‌دهنده هوشمند DNS (DNS Changer)

* امکان تغییر DNS سیستم بدون نیاز به فعال‌سازی VPN.
* هماهنگی خودکار با **NetworkManager (`nmcli`)**.
* پشتیبانی از **systemd-resolved**.
* پشتیبانی از DNSهای عمومی و سرویس‌های DNS رمزنگاری‌شده:

  * DoH
  * DoT
  * شکن
  * الکترو
  * 403 Online
  * رادار گیم

---

## 🐧 توزیع‌های پشتیبانی‌شده (Compatibility)

RedCloud به‌صورت مستقل و بدون نیاز به نصب فریم‌ورک‌های اضافی روی توزیع‌های مختلف لینوکس ۶۴ بیتی قابل اجرا است.

| خانواده توزیع       | توزیع‌های آزمایش‌شده                                                     |
| ------------------- | ------------------------------------------------------------------------ |
| **Debian / Ubuntu** | Kali Linux، Debian 11 / 12، Ubuntu 20.04+، Linux Mint، Pop!_OS، Zorin OS |
| **Arch Linux**      | Arch Linux، BlackArch، Manjaro، EndeavourOS، Garuda Linux                |
| **RedHat / Fedora** | Fedora 38+، RHEL، AlmaLinux، Rocky Linux، CentOS Stream                  |
| **سایر توزیع‌ها**   | openSUSE Tumbleweed / Leap، Void Linux، Alpine Linux                     |

---

## 📥 راهنمای نصب سریع در لینوکس (Quick Install)

در ترمینال سیستم لینوکسی خود دستورات زیر را اجرا کنید:

```bash
# ۱. رفتن به پوشه Downloads
cd ~/Downloads

# ۲. دانلود آخرین نسخه لینوکس
wget https://github.com/Devtahas/RedCloud-linux/releases/latest/download/RedCloud-Linux-x86_64.zip

# ۳. استخراج فایل فشرده
unzip -o RedCloud-Linux-x86_64.zip

# ۴. ورود به پوشه برنامه
cd redcloud

# ۵. اجرای اسکریپت نصب
sudo ./install.sh
```

---

## 🚀 نحوه اجرای برنامه

پس از نصب، می‌توانید RedCloud را به دو روش اجرا کنید.

### روش ۱: اجرای مستقیم از ترمینال

```bash
redcloud
```

### روش ۲: اجرای گرافیکی

از طریق **Application Menu** لینوکس یا **Desktop Icon** روی **RedCloud VPN** کلیک کنید.

---

## 🗑️ راهنمای حذف کامل (Uninstall)

برای حذف کامل فایل‌های برنامه، شورت‌کات‌ها و دستور ترمینال:

```bash
sudo rm -rf /opt/redcloud \
/usr/local/bin/redcloud \
/usr/share/applications/redcloud.desktop \
~/Desktop/redcloud.desktop \
/etc/ld.so.conf.d/redcloud.conf && sudo ldconfig
```

---

## 🛠️ ساختار سورس‌کد پروژه (Project Structure)

```text
RedCloud-linux/
├── .github/
│   └── workflows/
│       └── build.yml
│           # پایپ‌لاین CI/CD برای بیلد و بسته‌بندی خودکار
│
├── assets/
│   ├── app_icon.png
│   │   # آیکون دسکتاپ و System Tray
│   └── app_icon.ico
│
├── lib/
│   ├── main.dart
│   │   # کنترلر اصلی، رابط کاربری Flutter و مدیریت تب‌ها
│   └── src/
│       └── rust/
│           # کدهای تولیدشده FFI Bridge (FRB)
│
├── rust/
│   ├── src/
│   │   └── api/
│   │       └── simple.rs
│   │           # هسته بک‌اند لینوکس و مدیریت پروسه‌ها و کانفیگ Sing-box
│   │
│   └── Cargo.toml
│       # وابستگی‌های Rust مانند libc و serde
│
├── pubspec.yaml
│   # پیکربندی پکیج‌های Flutter
│
└── README.md
```

---

## ❤️ حمایت مالی از توسعه و پایداری سرورها (Donate)

حمایت‌های شما مستقیماً صرف نگهداری زیرساخت‌ها، تست مداوم پروتکل‌ها و توسعه RedCloud می‌شود.

<div align="center">

| شبکه (Network)              | رمزارز (Currency) | آدرس کیف پول (Wallet Address)                |
| --------------------------- | ----------------- | -------------------------------------------- |
| **BNB Smart Chain (BEP20)** | **USDT (Tether)** | `0xDeda28Aa73Ec089A77B3fC616E0011a8fce12900` |

</div>

---

## 📢 ارتباط با ما و اخبار آپدیت‌ها

* 💬 **کانال رسمی تلگرام:** [@DevTaha_project](https://t.me/DevTaha_project)
* 🐙 **مخزن رسمی گیت‌هاب:** [Devtahas/RedCloud-linux](https://github.com/Devtahas/RedCloud-linux)
* 🐛 **گزارش باگ و پیشنهادات:** [GitHub Issues](https://github.com/Devtahas/RedCloud-linux/issues)

---

<div align="center">

<sub>توسعه‌یافته با ❤️ برای آزادی دسترسی به اینترنت آزاد و امن</sub>

</div>
