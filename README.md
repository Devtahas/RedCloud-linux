<div align="center">

# 🛡️ RedCloud VPN (Linux Edition)

### کلاینت فوق‌پیشرفته و چندموتوره ضدسانسور اینترنت برای لینوکس

**Next-Gen Anti-Censorship Client Powered by Flutter & Rust**

[![Release](https://img.shields.io/badge/Release-v3.5%20Linux-blue.svg?style=for-the-badge&logo=linux)](https://github.com/Devtahas/RedCloud-linux/releases/latest)
[![Flutter](https://img.shields.io/badge/Flutter-3.29-02569B?style=for-the-badge&logo=flutter&logoColor=white)](https://flutter.dev)
[![Rust](https://img.shields.io/badge/Rust-2021-DEA584?style=for-the-badge&logo=rust&logoColor=black)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/Platform-Debian%20|%20Kali%20|%20Arch%20|%20Fedora-black?style=for-the-badge&logo=linux)](https://github.com/Devtahas/RedCloud-linux)
[![Telegram](https://img.shields.io/badge/Telegram-Channel-2CA5E0?style=for-the-badge&logo=telegram&logoColor=white)](https://t.me/DevTaha_project)

<p align="center">
<b>RedCloud VPN</b> یک نرم‌افزار متن‌باز، فوق‌العاده سریع و مقاوم در برابر فیلترینگ شدید است که با ترکیب رابط گرافیکی <b>Flutter</b> و هسته باینری پرسرعت <b>Rust</b>، ارتباطی پایدار، امن و بدون افت سرعت را برای تمامی توزیع‌های لینوکس فراهم می‌کند.
</p>

</div>

---

## ✨ ویژگی‌های برجسته (Key Features)

- 🚀 **اتصال هیبریدی انقلابی (Hybrid Aether + Sing-box):** زنجیره‌سازی خودکار ترافیک از دل پل ضدسانسور MASQUE برای مخفی‌سازی کامل هویت ترافیک از سنسورهای DPI.
- ⚡ **پروتکل نسل جدید MASQUE (Aether Core):** اتصال مستقیم به زیرساخت ابری بر بستر **HTTP/3 QUIC (0-RTT)** و **H2 + Fragment** با پروفایل‌های پارازیت Noize (Firewall, Aggressive, Light).
- 💎 **پشتیبانی کامل از پروتکل‌های مدرن:** شامل **VLESS Reality**، **Hysteria 2 (Hy2)**، **Trojan** و هدرهای رمزنگاری‌شده **ECH (Encrypted Client Hello)**.
- 🛡️ **تکنیک‌های پیشرفته Anti-DPI:** شبیه‌ساز اثر انگشت مرورگر (uTLS Chrome/Firefox)، قطعه‌بندی پکت‌های امنیتی (TLS Fragmentation) و جعل تزریقی دامنه (TLS Spoofing).
- 🌐 **شبکه‌های پیاز تور (Tor) و سایفون (Psiphon):** با امکان انتخاب کشور خروجی (Exit Node) برای حداکثر گمنامی و عبور از شدیدترین محدودیت‌ها.
- 🎯 **اسکنر دوحالته لایه ۷ کلودفلر (Quick & Deep Scanner):** پایش موازی و زنده پینگ با تست واقعی دست‌دهی وب‌ساکت و استخراج خودکار آی‌پی‌های سفید.
- 🎮 **کارت شبکه مجازی سیستمی (Linux TUN Mode):** عبور ۱۰۰٪ کل ترافیک سیستم‌عامل (ترمینال، بازی‌ها، SSH و اپلیکیشن‌ها) با تنظیم دائمی دسترسی‌های `cap_net_admin`.
- 🔄 **چرخش خودکار اکانت‌ها (Auto-Rotation):** دریافت خودکار سرورهای تازه و رایگان از گیت‌هاب در صورت اتمام حجم.
- 🚦 **تغییردهنده هوشمند DNS (DNS Changer):** دور زدن تحریم‌ها و بازی‌های آنلاین با DNSهای شکن، الکترو، ۴۰۳ آنلاین، رادار گیم و پروتکل‌های رمزنگاری DoH / DoT.

---

## 🐧 توزیع‌های پشتیبانی‌شده (Compatibility)

پکیج نصاب تک‌فایلی RedCloud روی **تمامی توزیع‌های استاندارد لینوکس (۶۴ بیتی)** بدون نیاز به هیچ پیش‌نیاز قبلی اجرا می‌شود:

| خانواده توزیع | توزیع‌های تست‌شده |
|---|---|
| **Debian / Ubuntu** | Kali Linux, Debian 11/12, Ubuntu 20.04+, Linux Mint, Pop!_OS |
| **Arch Linux** | Arch Linux, BlackArch, Manjaro, EndeavourOS, Garuda |
| **RedHat / Fedora** | Fedora 38+, RHEL, AlmaLinux, Rocky Linux |
| **سایر توزیع‌ها** | openSUSE, Void Linux, Alpine, Zorin OS |

---

## 📥 راهنمای نصب سریع (Quick Installation)

کاربران نهایی برای نصب کامل برنامه، کافی است آخرین نسخه نصاب را از بخش **Releases** دانلود کرده و دستورات زیر را در ترمینال اجرا کنند:

```bash
# ۱. دانلود مستقیم نصاب
wget https://github.com/Devtahas/RedCloud-linux/releases/latest/download/RedCloud-Linux-Installer-v3.5.run

# ۲. اعطای دسترسی اجرایی
chmod +x RedCloud-Linux-Installer-v3.5.run

# ۳. نصب کامل با یک دستور
sudo ./RedCloud-Linux-Installer-v3.5.run
```

> **نکته:** فرآیند نصب کاملاً خودکار است و تمام دسترسی‌های لازم برای کارت شبکه TUN، میانبر دسکتاپ و دستور ترمینال را تنظیم می‌کند.

---

## 🚀 نحوه اجرای نرم‌افزار

پس از نصب، به دو روش آسان می‌توانید برنامه را اجرا کنید:

### از طریق ترمینال

```bash
redcloud
```

### از طریق محیط گرافیکی

روی آیکون **RedCloud VPN** در منوی برنامه‌ها یا Desktop کلیک کنید.

---

## 🗑️ راهنمای حذف کامل (Uninstallation)

در صورت تمایل به حذف نرم‌افزار:

```bash
sudo /opt/redcloud/uninstall.sh
```

---

## 🛠️ ساختار فنی پروژه (Architecture)

```text
RedCloud-linux/
├── .github/workflows/     # CI/CD
├── assets/                # آیکون‌ها و دارایی‌های گرافیکی
├── lib/
│   ├── main.dart
│   └── src/rust/
├── rust/
│   ├── src/api/simple.rs
│   └── Cargo.toml
├── build_installer.sh
└── pubspec.yaml
```

---

## ❤️ حمایت مالی از پروژه (Donate)

<div align="center">

| شبکه | ارز | آدرس کیف پول |
|--------|--------|--------|
| **BNB Smart Chain (BEP20)** | **USDT (Tether)** | `0xDeda28Aa73Ec089A77B3fC616E0011a8fce12900` |

</div>

---

## 📢 ارتباط با ما و اخبار آپدیت‌ها

- **کانال رسمی تلگرام:** https://t.me/DevTaha_project
- **مخزن گیت‌هاب:** https://github.com/Devtahas/RedCloud-linux

---

<div align="center">
<sub>توسعه‌یافته با ❤️ برای آزادی اینترنت و مبارزه با سانسور</sub>
</div>
