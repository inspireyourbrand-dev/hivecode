; HiveCode Installer - Inno Setup Script
; Requires Inno Setup 6.x: https://jrsoftware.org/isinfo.php
;
; To build the installer:
;   1. Install Inno Setup 6
;   2. Run: cargo tauri build   (generates the .exe in target/release/)
;   3. Open this file in Inno Setup Compiler and click Build
;   OR from command line: ISCC.exe scripts\installer.iss

#define MyAppName "HiveCode"
#define MyAppVersion "0.1.0"
#define MyAppPublisher "HivePowered"
#define MyAppURL "https://hivepowered.ai"
#define MyAppExeName "hivecode.exe"
#define MyAppDescription "The model-agnostic AI coding assistant"
#define MyAppId "ai.hivepowered.hivecode"

[Setup]
AppId={{{#MyAppId}}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
OutputDir=..\dist\installer
OutputBaseFilename=HiveCode-{#MyAppVersion}-Setup
SetupIconFile=..\ui\public\hivecode.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
UninstallDisplayIcon={app}\{#MyAppExeName}
UninstallDisplayName={#MyAppName}
VersionInfoVersion={#MyAppVersion}.0
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription={#MyAppDescription}
VersionInfoProductName={#MyAppName}
VersionInfoProductVersion={#MyAppVersion}
MinVersion=10.0.17763

; Visual customization
WizardImageFile=..\assets\installer-banner.bmp
WizardSmallImageFile=..\assets\installer-icon.bmp

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addtopath"; Description: "Add HiveCode to system PATH"; GroupDescription: "System Integration:"; Flags: checkedonce
Name: "registerprotocol"; Description: "Register hivecode:// URL protocol"; GroupDescription: "System Integration:"

[Files]
; Main executable - from Tauri build output
Source: "..\target\release\hivecode.exe"; DestDir: "{app}"; Flags: ignoreversion

; WebView2 bootstrapper (for systems without WebView2)
Source: "..\scripts\MicrosoftEdgeWebview2Setup.exe"; DestDir: "{tmp}"; Flags: deleteafterinstall; Check: not IsWebView2Installed

; Config template
Source: "..\config.example.toml"; DestDir: "{app}"; DestName: "config.example.toml"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Registry]
; Add to PATH
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}"; Tasks: addtopath; Check: NeedsAddPath(ExpandConstant('{app}'))

; Register URL protocol
Root: HKCU; Subkey: "Software\Classes\hivecode"; ValueType: string; ValueName: ""; ValueData: "URL:HiveCode Protocol"; Tasks: registerprotocol
Root: HKCU; Subkey: "Software\Classes\hivecode"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""; Tasks: registerprotocol
Root: HKCU; Subkey: "Software\Classes\hivecode\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: registerprotocol

; App registration for Windows search/start menu
Root: HKCU; Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; ValueType: string; ValueName: "InstallPath"; ValueData: "{app}"
Root: HKCU; Subkey: "Software\{#MyAppPublisher}\{#MyAppName}"; ValueType: string; ValueName: "Version"; ValueData: "{#MyAppVersion}"

[Run]
; Install WebView2 if needed (silent)
Filename: "{tmp}\MicrosoftEdgeWebview2Setup.exe"; Parameters: "/silent /install"; StatusMsg: "Installing WebView2 Runtime..."; Check: not IsWebView2Installed; Flags: waituntilterminated

; Launch after install
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}\logs"
Type: filesandordirs; Name: "{app}\cache"

[Code]
function IsWebView2Installed: Boolean;
var
  ResultCode: Integer;
  RegKey: String;
begin
  Result := False;
  RegKey := 'SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';
  if RegKeyExists(HKLM, RegKey) then
    Result := True
  else begin
    RegKey := 'SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}';
    if RegKeyExists(HKCU, RegKey) then
      Result := True;
  end;
end;

function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKCU, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    // Create default config directory if it doesn't exist
    if not DirExists(ExpandConstant('{userappdata}\hivecode')) then
      CreateDir(ExpandConstant('{userappdata}\hivecode'));
  end;
end;
