; NSIS installer for quassel-dump-all.
;
; Built by release.yml via:
;   makensis -DVERSION=<version> -DBITS=<32|64> -DSRCDIR=<dir with exe/README.md/LICENSE> \
;            -DOUTFILE=<output .exe path> installer.nsi

!include "MUI2.nsh"
!include "WinMessages.nsh"
!include "StrFunc.nsh"

${Using:StrFunc} StrRep
${Using:StrFunc} UnStrRep

!ifndef VERSION
  !define VERSION "0.0.0"
!endif
!ifndef BITS
  !define BITS "64"
!endif
!ifndef SRCDIR
  !define SRCDIR ".."
!endif
!ifndef OUTFILE
  !define OUTFILE "quassel-dump-all-v${VERSION}-installer.exe"
!endif

Name "quassel-dump-all"
OutFile "${OUTFILE}"
Unicode true
RequestExecutionLevel admin

!if "${BITS}" == "64"
  InstallDir "$PROGRAMFILES64\quassel-dump-all"
!else
  InstallDir "$PROGRAMFILES\quassel-dump-all"
!endif

!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${SRCDIR}\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "quassel-dump-all"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "Graham Ollis"
VIAddVersionKey "FileDescription" "quassel-dump-all installer"

Section "quassel-dump-all" SecMain
  SetOutPath "$INSTDIR"
  File "${SRCDIR}\quassel-dump-all.exe"
  File "${SRCDIR}\README.md"
  File "${SRCDIR}\LICENSE"

  WriteUninstaller "$INSTDIR\uninstall.exe"

  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\quassel-dump-all" \
    "DisplayName" "quassel-dump-all"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\quassel-dump-all" \
    "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\quassel-dump-all" \
    "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\quassel-dump-all" \
    "InstallLocation" "$INSTDIR"
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\quassel-dump-all" \
    "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\quassel-dump-all" \
    "NoRepair" 1

  ; Append the install dir to the system PATH so quassel-dump-all works from any shell.
  ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
  ${StrRep} $1 "$0" "$INSTDIR" ""
  StrCmp $0 "$1" 0 PathAlreadyPresent
    StrCpy $1 "$0;$INSTDIR"
    WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$1"
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
  PathAlreadyPresent:
SectionEnd

Section "Uninstall"
  ; Remove the install dir from the system PATH.
  ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
  ${UnStrRep} $1 "$0" ";$INSTDIR" ""
  ${UnStrRep} $1 "$1" "$INSTDIR;" ""
  ${UnStrRep} $1 "$1" "$INSTDIR" ""
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$1"
  SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000

  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\quassel-dump-all"

  Delete "$INSTDIR\quassel-dump-all.exe"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
