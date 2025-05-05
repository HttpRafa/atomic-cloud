[Setup]
; Basic application information
AppName=Atomic Cloud CLI
AppVersion=0.0.0-nightly
DefaultDirName={\pf32}\Atomic Cloud
DefaultGroupName=Atomic Cloud
OutputBaseFilename=cli-windows-x86_64-setup
Compression=lzma2
overwriteinstallations=yes

SetupIconFile=logo.ico

[Languages]
Name: "english"; MessagesFile: "compiler\:Default.isl"

[Files]
Source: "cli-windows-x86\_64.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "logo.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
; Desktop shortcut
Name: "{userdesktop}\Atomic Cloud CLI"; Filename: "{app}\cli-windows-x86\_64.exe"; WorkingDir: "{app}"; IconFilename: "{app}\logo.ico"

; Start Menu shortcut
Name: "{group}\Atomic Cloud CLI"; Filename: "{app}\cli-windows-x86\_64.exe"; WorkingDir: "{app}"; IconFilename: "{app}\logo.ico"

[Run]
; Optionally launch the application after installation
Filename: "{app}\cli-windows-x86\_64.exe"; Description: "Launch Atomic Cloud CLI"; Flags: nowait postinstall skipifsilent
