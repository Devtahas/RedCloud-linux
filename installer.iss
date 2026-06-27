; سناریوی ساخت اینستالر حرفه‌ای برای نرم‌افزار RedCloud VPN
#define AppName "RedCloud VPN"
#define AppVersion "2.5"
#define AppPublisher "RedCloud"
#define AppExeName "client.exe"

[Setup]
; شناسه منحصربه‌فرد نرم‌افزار شما در رجیستری ویندوز
AppId={{9F2C0E8D-D8A1-4F43-9831-C7D4E75A22E1}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
; مسیر پیش‌فرض نصب در Program Files ویندوز (کاربر می‌تواند آن را تغییر دهد)
DefaultDirName={autopf}\{#AppName}
DisableProgramGroupPage=yes
; ذخیره فایل نصبی نهایی در ریشه پروژه شما
OutputDir=.
OutputBaseFilename=RedCloud_Setup
; ست کردن آیکون نئونی شما برای خود فایل نصب کننده
SetupIconFile=assets\app_icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
; درخواست دسترسی ادمین (Administrator) برای کارهای سیستمی و نصب در Program Files
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
; گزینه انتخابی ساخت شورت‌کات دسکتاپ به صورت خودکار
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; پکیج کردن کل محتویات پوشه ریلیز (شامل sing-box.exe و tor.exe و دی‌ال‌ال‌ها)
Source: "build\windows\x64\runner\Release\*"; DestDir: "{app}"; Flags: recursesubdirs createallsubdirs

[Icons]
; ساخت شورت‌کات در منوی استارت و دسکتاپ
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
; ست کردن آیکون شیلد معروف UAC و اجرای خودکار با دسترسی ادمین برای کلاینت پس از نصب در سراسر سیستم‌عامل
Root: "HKLM"; Subkey: "SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers"; ValueType: string; ValueName: "{app}\{#AppExeName}"; ValueData: "~ RUNASADMIN"; Flags: uninsdeletevalue

[Run]
; گزینه اجرای خودکار نرم‌افزار پس از پایان نصب
Filename: "{app}\{#AppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(AppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent