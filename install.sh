#!/usr/bin/env bash

# =====================================================================
#  RedCloud VPN - اسکریپت نصب خودکار و تنظیم دسترسی‌های دائمی لینوکس
# =====================================================================

set -e

# بررسی دسترسی روت
if [ "$EUID" -ne 0 ]; then
  echo -e "\e[31m[!] لطفاً این اسکریپت را با دسترسی روت اجرا کنید: sudo ./install.sh\e[0m"
  exit 1
fi

echo -e "\e[34m[*] در حال نصب RedCloud VPN روی سیستم‌عامل لینوکس...\e[0m"

# تشخیص یوزر اصلی سیستم (برای ساخت شورت‌کات دسکتاپ)
REAL_USER="${SUDO_USER:-$USER}"
USER_HOME=$(getent passwd "$REAL_USER" | cut -d: -f6)

INSTALL_DIR="/opt/redcloud"
BIN_LINK="/usr/local/bin/redcloud"
DESKTOP_ENTRY="/usr/share/applications/redcloud.desktop"
USER_DESKTOP="$USER_HOME/Desktop/redcloud.desktop"

# ۱. ساخت دایرکتوری برنامه در /opt/redcloud
echo -e "\e[32m[+] کپی فایل‌های برنامه در $INSTALL_DIR ...\e[0m"
mkdir -p "$INSTALL_DIR"
cp -r ./* "$INSTALL_DIR/"

# ۲. اعطای دسترسی‌های اجرایی
echo -e "\e[32m[+] اعطای دسترسی اجرایی (chmod +x) به فایل‌ها...\e[0m"
chmod +x "$INSTALL_DIR/client" 2>/dev/null || true
chmod +x "$INSTALL_DIR/sing-box" 2>/dev/null || true
chmod +x "$INSTALL_DIR/aether" 2>/dev/null || true
chmod +x "$INSTALL_DIR/tor" 2>/dev/null || true
chmod +x "$INSTALL_DIR/psiphon-tunnel-core" 2>/dev/null || true

# ۳. تنظیم قابلیت‌های کرنل (Capabilities) برای حالت TUN بدون نیاز به اجرای کل برنامه با sudo
# این قابلیت باعث می‌شود کارت شبکه مجازی /dev/net/tun همیشه بدون خطا باز شود
echo -e "\e[32m[+] تنظیم دائمی دسترسی‌های شبکه (setcap CAP_NET_ADMIN)...\e[0m"
if command -v setcap >/dev/null 2>&1; then
  if [ -f "$INSTALL_DIR/sing-box" ]; then
    setcap 'cap_net_admin,cap_net_bind_service=+ep' "$INSTALL_DIR/sing-box" 2>/dev/null || true
  fi
  if [ -f "$INSTALL_DIR/client" ]; then
    setcap 'cap_net_admin,cap_net_bind_service=+ep' "$INSTALL_DIR/client" 2>/dev/null || true
  fi
fi

# ۴. ساخت دستور سراسری در ترمینال (/usr/local/bin/redcloud)
echo -e "\e[32m[+] فعال‌سازی دستور 'redcloud' در ترمینال...\e[0m"
cat << 'EOF' > "$BIN_LINK"
#!/usr/bin/env bash
cd /opt/redcloud
exec ./client "$@"
EOF
chmod +x "$BIN_LINK"

# ۵. ساخت فایل شورت‌کات دسکتاپ و منوی برنامه‌ها
echo -e "\e[32m[+] ایجاد شورت‌کات در منوی برنامه‌ها و دسکتاپ...\e[0m"
cat << EOF > "$DESKTOP_ENTRY"
[Desktop Entry]
Name=RedCloud VPN
Comment=Next-Gen Anti-Censorship Client for Linux
Exec=/usr/local/bin/redcloud
Icon=/opt/redcloud/data/flutter_assets/assets/app_icon.png
Terminal=false
Type=Application
Categories=Network;Security;VPN;
StartupWMClass=client
EOF
chmod 644 "$DESKTOP_ENTRY"

# ساخت شورت‌کات روی میزکار کاربر (Desktop) در صورت وجود
if [ -d "$USER_HOME/Desktop" ]; then
  cp "$DESKTOP_ENTRY" "$USER_DESKTOP"
  chown "$REAL_USER:$REAL_USER" "$USER_DESKTOP"
  chmod +x "$USER_DESKTOP"
  
  # اعتماد به شورت‌کات در گنوم و کالی لینوکس
  if command -v gio >/dev/null 2>&1; then
    sudo -u "$REAL_USER" gio set "$USER_DESKTOP" metadata::trusted true 2>/dev/null || true
  fi
fi

echo -e "\e[32m=====================================================\e[0m"
echo -e "\e[32m[✔] نصب با موفقیت پایان یافت!\e[0m"
echo -e "\e[33m• از این به بعد در هر کجای ترمینال بنویسید: \e[1mredcloud\e[0m"
echo -e "\e[33m• یا روی آیکون RedCloud VPN در دسکتاپ یا منوی سیستم کلیک کنید.\e[0m"
echo -e "\e[32m=====================================================\e[0m"