; HiveCode Windows Installer Script
; Inno Setup 6.x - Creates a polished Windows installer with WebView2 support
;
; To compile:
;   1. Install Inno Setup 6 from https://jrsoftware.org/isinfo.php
;   2. Open this file in Inno Setup IDE and click Build > Compile
;   3. Or: "C:\Program Files (x86)\Inno Setup 6\Compil32.exe" /cc installer.iss

[Setup]
; Application metadata
AppName=HiveCode
AppVersion=0.1.0
AppPublisher=HivePowered
AppPublisherURL=https://hivepowered.ai
AppSupportURL=https://hivepowered.ai
AppUpdatesURL=https://hivepowered.ai
AppContact=support@hivepowered.ai

; Install behavior
DefaultDirName={autopf}\HiveCode
DefaultGroupName=HiveCode
AllowNoIcons=yes
LicenseFile=..\..\LICENSE
SetupIconFile=..\..\crates\hivecode-tauri\icons\icon.ico
UninstallIconFile=..\..\crates\hivecode-tauri\icons\icon.ico

; Output configuration
OutputDir=.\output
OutputBaseFilename=HiveCode-0.1.0-Setup
Compression=lzma
SolidCompression=yes

; Windows requirements
MinVersion=10.0.10240

; Architecture
ArchitecturesInstallIn64BitMode=x64
ArchitecturesAllowed=x64

; User install options
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; Install UI
WizardStyle=modern
WizardSizePercent=100

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
; Create Start Menu shortcuts
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Main executable from Tauri release build
Source: "..\..\target\release\hivecode.exe"; DestDir: "{app}"; Flags: ignoreversion

; Example configuration file (optional)
Source: "..\..\config.example.toml"; DestDir: "{app}"; DestName: "config.example.toml"; Flags: ignoreversion

; Application icon for shortcuts
Source: "..\..\crates\hivecode-tauri\icons\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Dirs]
; Create data directory for user configs/cache
Name: "{localappdata}\HiveCode"; Flags: uninsalwaysuninitialize

[Icons]
; Start Menu shortcuts
Name: "{group}\HiveCode"; Filename: "{app}\hivecode.exe"; IconFilename: "{app}\icon.ico"; Comment: "Run HiveCode"; Flags: createonlyiffileexists
Name: "{group}\{cm:UninstallProgram,HiveCode}"; Filename: "{uninstallexe}"

; Desktop shortcut (if user selected it)
Name: "{autodesktop}\HiveCode"; Filename: "{app}\hivecode.exe"; IconFilename: "{app}\icon.ico"; Comment: "Run HiveCode"; Tasks: desktopicon; Flags: createonlyiffileexists

[Registry]
; Register hivecode:// URL protocol for deep linking
Root: HKCU; Subkey: "Software\Classes\hivecode"; ValueType: string; ValueName: ""; ValueData: "URL:HiveCode Protocol"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\hivecode"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\hivecode\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\hivecode.exe,0"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\hivecode\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\hivecode.exe"" ""%1"""; Flags: uninsdeletekey

[Run]
; Run WebView2 installer if needed
Filename: "{tmp}\MsEdgeWebview2Setup.exe"; Parameters: "/silent /install"; StatusMsg: "Installing WebView2 Runtime..."; Flags: skipifdoesntexist waituntilterminated

; Run main application after install
Filename: "{app}\hivecode.exe"; Description: "Launch HiveCode"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Clean up user data directories
Type: dirifempty; Name: "{localappdata}\HiveCode"
Type: dirifempty; Name: "{app}"

[Code]
{
  Custom code to handle WebView2 installation and verification
}

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssInstall then
  begin
    { Check for WebView2 runtime and download if needed }
    if not FileExists('C:\Program Files\Microsoft Edge WebView2 Runtime\msedgewebview2.exe') then
    begin
      MsgBox('WebView2 Runtime will be installed automatically.', mbInformation, MB_OK);
      if not DownloadFile('https://go.microsoft.com/fwlink/p/?LinkId=2124703', ExpandConstant('{tmp}\MsEdgeWebview2Setup.exe')) then
      begin
        MsgBox('Failed to download WebView2. Please install it manually from https://developer.microsoft.com/en-us/microsoft-edge/webview2/', mbError, MB_OK);
      end;
    end;
  end;
end;

procedure InitializeWizard();
begin
  { You can add custom initialization here if needed }
end;
