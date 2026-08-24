#!/bin/bash
set -e

APP_NAME="redcloud"
DISPLAY_NAME="RedCloud VPN"
VERSION="3.5"
OUTPUT_INSTALLER="RedCloud-Linux-Installer-v${VERSION}.run"

echo -e "\e[1;34m====================================================\e[0m"
echo -e "\e[1;32m   در حال ساخت پکیج نصاب تک‌فایلی RedCloud برای لینوکس   \e[0m"
echo -e "\e[1;34m====================================================\e[0m"

# ۱. بررسی وجود پوشه بیلد فلاتر
BUNDLE_DIR="build/linux/x64/release/bundle"

if [ ! -d "$BUNDLE_DIR" ]; then
    echo -e "\e[1;33m[!] پوشه بیلد پیدا نشد. در حال اجرای بیلد فلاتر...\e[0m"
    flutter build linux --release
fi

if [ ! -d "$BUNDLE_DIR" ]; then
    echo -e "\e[1;31m[X] خطای بیلد فلاتر! لطفاً ابتدا flutter build linux --release را بررسی کنید.\e[0m"
    exit 1
fi

# ۲. ایجاد دایرکتوری آماده‌سازی (Staging)
STAGE_DIR="/tmp/redcloud_staging"
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"

echo -e "\e[1;36m[*] در حال کپی فایل‌های برنامه و باینری‌ها...\e[0m"
cp -r "$BUNDLE_DIR"/* "$STAGE_DIR/"

# کپی فایل‌های باینری هسته
[ -f "sing-box" ] && cp "sing-box" "$STAGE_DIR/"
[ -f "aether" ] && cp "aether" "$STAGE_DIR/"
[ -f "tor" ] && cp "tor" "$STAGE_DIR/"
[ -f "psiphon-tunnel-core" ] && cp "psiphon-tunnel-core" "$STAGE_DIR/"
[ -f "cloudflare_IPs.txt" ] && cp "cloudflare_IPs.txt" "$STAGE_DIR/"

# اطمینان از وجود آیکون
mkdir -p "$STAGE_DIR/assets"
if [ -f "assets/app_icon.png" ]; then
    cp "assets/app_icon.png" "$STAGE_DIR/assets/"
elif [ -f "assets/app_icon.ico" ]; then
    cp "assets/app_icon.ico" "$STAGE_DIR/assets/app_icon.png"
fi

# ساخت فایل فشرده payload
PAYLOAD_TAR="/tmp/redcloud_payload.tar.gz"
rm -f "$PAYLOAD_TAR"
tar -czf "$PAYLOAD_TAR" -C "$STAGE_DIR" .
rm -rf "$STAGE_DIR"

# ۳. ساخت اسکریپت اکسترکتور و نصاب تک‌فایلی (.run)
cat << 'EOF' > "$OUTPUT_INSTALLER"
#!/bin/bash
set -e

if [ "$EUID" -ne 0 ]; then
  echo -e "\e[1;31m[!] لطفاً این نصاب را با دسترسی روت اجرا کنید: sudo $0\e[0m"
  exit 1
fi

INSTALL_DIR="/opt/redcloud"
APP_NAME="redcloud"
DISPLAY_NAME="RedCloud VPN"

echo -e "\e[1;32m===========================================\e[0m"
echo -e "\e[1;32m     در حال نصب نرم‌افزار RedCloud VPN...     \e[0m"
echo -e "\e[1;32m===========================================\e[0m"

# ۱. متوقف کردن پروسه‌های قدیمی در صورت وجود
pkill -9 -f "sing-box" 2>/dev/null || true
pkill -9 -f "aether" 2>/dev/null || true
pkill -9 -f "client" 2>/dev/null || true

# ۲. ایجاد پوشه مقصد و اکسترکت فایل‌ها
mkdir -p "$INSTALL_DIR"
ARCHIVE_LINE=$(awk '/^__PAYLOAD_BEGINS__/ {print NR + 1; exit 0; }' "$0")
tail -n +"$ARCHIVE_LINE" "$0" | tar -xz -C "$INSTALL_DIR"

# ۳. اعطای دسترسی‌های اجرایی کامل
chmod -R 755 "$INSTALL_DIR"
[ -f "$INSTALL_DIR/client" ] && chmod +x "$INSTALL_DIR/client"
[ -f "$INSTALL_DIR/sing-box" ] && chmod +x "$INSTALL_DIR/sing-box"
[ -f "$INSTALL_DIR/aether" ] && chmod +x "$INSTALL_DIR/aether"
[ -f "$INSTALL_DIR/psiphon-tunnel-core" ] && chmod +x "$INSTALL_DIR/psiphon-tunnel-core"
[ -f "$INSTALL_DIR/tor" ] && chmod +x "$INSTALL_DIR/tor"

# ۴. اعطای دسترسی‌های دائمی شبکه به هسته‌ها (TUN Mode دائمی بدون پسورد)
echo -e "\e[1;36m[*] در حال تنظیم دسترسی‌های کارت شبکه مجازی (TUN Capabilities)...\e[0m"
setcap cap_net_admin,cap_net_bind_service,cap_net_raw=+ep "$INSTALL_DIR/sing-box" 2>/dev/null || true
setcap cap_net_admin,cap_net_bind_service,cap_net_raw=+ep "$INSTALL_DIR/aether" 2>/dev/null || true
[ -f "$INSTALL_DIR/client" ] && setcap cap_net_admin,cap_net_bind_service=+ep "$INSTALL_DIR/client" 2>/dev/null || true

# ۵. نصب آیکون سیستمی
mkdir -p /usr/share/icons/hicolor/256x256/apps/
if [ -f "$INSTALL_DIR/assets/app_icon.png" ]; then
    cp "$INSTALL_DIR/assets/app_icon.png" /usr/share/icons/hicolor/256x256/apps/redcloud.png
fi

# ۶. ساخت فایل Desktop Shortcut در منوی برنامه‌های لینوکس
cat << DESKTOP_EOF > /usr/share/applications/redcloud.desktop
[Desktop Entry]
Name=RedCloud VPN
Comment=Next-Gen Anti-Censorship Client
Exec=/opt/redcloud/client
Icon=redcloud
Terminal=false
Type=Application
Categories=Network;Security;VPN;
StartupWMClass=client
DESKTOP_EOF

chmod +x /usr/share/applications/redcloud.desktop

# ۷. ساخت خودکار شورت‌کات روی صفحه دسکتاپ تمام کاربران سیستم
for USER_HOME in /home/*; do
    if [ -d "$USER_HOME" ]; then
        USERNAME=$(basename "$USER_HOME")
        DESKTOP_DIR="$USER_HOME/Desktop"
        [ ! -d "$DESKTOP_DIR" ] && DESKTOP_DIR="$USER_HOME/دسکتاپ"
        
        if [ -d "$DESKTOP_DIR" ]; then
            cp /usr/share/applications/redcloud.desktop "$DESKTOP_DIR/"
            chown "$USERNAME:$USERNAME" "$DESKTOP_DIR/redcloud.desktop"
            chmod +x "$DESKTOP_DIR/redcloud.desktop"
            # معتبرسازی شورت‌کات در دسکتاپ گنوم/کالی/اوبونتو
            su - "$USERNAME" -c "gio set '$DESKTOP_DIR/redcloud.desktop' metadata::trusted true" 2>/dev/null || true
        fi
    fi
done

# ۸. ساخت دستور ترمینال جهانی 'redcloud'
cat << 'CMD_EOF' > /usr/local/bin/redcloud
#!/bin/bash
cd /opt/redcloud
exec /opt/redcloud/client "$@" >/dev/null 2>&1 &
CMD_EOF

chmod +x /usr/local/bin/redcloud

# ۹. ساخت اسکریپت حذف تمیز (Uninstaller)
cat << 'UNINSTALL_EOF' > /opt/redcloud/uninstall.sh
#!/bin/bash
if [ "$EUID" -ne 0 ]; then
  echo "لطفاً با دسترسی روت اجرا کنید: sudo $0"
  exit 1
fi
pkill -9 -f "sing-box" 2>/dev/null || true
pkill -9 -f "aether" 2>/dev/null || true
pkill -9 -f "client" 2>/dev/null || true
rm -rf /opt/redcloud
rm -f /usr/local/bin/redcloud
rm -f /usr/share/applications/redcloud.desktop
rm -f /usr/share/icons/hicolor/256x256/apps/redcloud.png
for USER_HOME in /home/*; do
    rm -f "$USER_HOME/Desktop/redcloud.desktop" 2>/dev/null || true
    rm -f "$USER_HOME/دسکتاپ/redcloud.desktop" 2>/dev/null || true
done
echo "نرم‌افزار RedCloud با موفقیت و به‌طور کامل از سیستم حذف شد."
UNINSTALL_EOF

chmod +x /opt/redcloud/uninstall.sh

echo -e "\e[1;32m========================================================\e[0m"
echo -e "\e[1;32m   نصب با موفقیت انجام شد! 🎉                         \e[0m"
echo -e "\e[1;33m • اجرای آسان در ترمینال با تایپ: \e[1;37mredcloud\e[0m"
echo -e "\e[1;33m • یا کلیک روی آیکون RedCloud VPN در صفحه دسکتاپ و منو \e[0m"
echo -e "\e[1;32m========================================================\e[0m"
exit 0

__PAYLOAD_BEGINS__
EOF

# الصاق فایل باینری به اسکریپت نصاب
cat "$PAYLOAD_TAR" >> "$OUTPUT_INSTALLER"
rm -f "$PAYLOAD_TAR"
chmod +x "$OUTPUT_INSTALLER"

echo -e "\e[1;32m[✓] فایل نصاب تک‌فایلی آماده شد: \e[1;33m$OUTPUT_INSTALLER\e[0m"