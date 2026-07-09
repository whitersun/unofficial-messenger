!macro CLEAN_UNOFFICIAL_MESSENGER_USER_DATA
  SetShellVarContext current

  Delete "$APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\Unofficial Messenger.lnk"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "UnofficialMessenger"

  SetOutPath "$TEMP"

  RMDir /r /REBOOTOK "$APPDATA\unofficial-messenger-next"
  RMDir /r /REBOOTOK "$LOCALAPPDATA\unofficial-messenger-next"
  RMDir /r /REBOOTOK "$APPDATA\unofficial-messenger"
  RMDir /r /REBOOTOK "$LOCALAPPDATA\unofficial-messenger"
  RMDir /r /REBOOTOK "$APPDATA\io.github.whitersun.unofficial-messenger-next"
  RMDir /r /REBOOTOK "$LOCALAPPDATA\io.github.whitersun.unofficial-messenger-next"
  RMDir /r /REBOOTOK "$APPDATA\io.github.whitersun.unofficial-messenger"
  RMDir /r /REBOOTOK "$LOCALAPPDATA\io.github.whitersun.unofficial-messenger"
!macroend

!macro RUN_DELAYED_UNOFFICIAL_MESSENGER_CLEANUP
  FileOpen $0 "$TEMP\unofficial-messenger-next-cleanup.cmd" w
  FileWrite $0 '@echo off$\r$\n'
  FileWrite $0 'timeout /T 3 /NOBREAK >NUL 2>NUL$\r$\n'
  FileWrite $0 'rmdir /S /Q "%APPDATA%\unofficial-messenger-next" 2>NUL$\r$\n'
  FileWrite $0 'rmdir /S /Q "%LOCALAPPDATA%\unofficial-messenger-next" 2>NUL$\r$\n'
  FileWrite $0 'rmdir /S /Q "%APPDATA%\unofficial-messenger" 2>NUL$\r$\n'
  FileWrite $0 'rmdir /S /Q "%LOCALAPPDATA%\unofficial-messenger" 2>NUL$\r$\n'
  FileWrite $0 'rmdir /S /Q "%APPDATA%\io.github.whitersun.unofficial-messenger-next" 2>NUL$\r$\n'
  FileWrite $0 'rmdir /S /Q "%LOCALAPPDATA%\io.github.whitersun.unofficial-messenger-next" 2>NUL$\r$\n'
  FileWrite $0 'rmdir /S /Q "%APPDATA%\io.github.whitersun.unofficial-messenger" 2>NUL$\r$\n'
  FileWrite $0 'rmdir /S /Q "%LOCALAPPDATA%\io.github.whitersun.unofficial-messenger" 2>NUL$\r$\n'
  FileWrite $0 'del "%~f0" >NUL 2>NUL$\r$\n'
  FileClose $0
  Exec '"$SYSDIR\cmd.exe" /C start "" /MIN "$TEMP\unofficial-messenger-next-cleanup.cmd"'
!macroend

!macro NSIS_HOOK_PREINSTALL
  ReadRegStr $0 SHCTX "${UNINSTKEY}" "InstallLocation"

  ${If} $UpdateMode <> 1
  ${AndIf} $0 == ""
    nsExec::ExecToLog 'taskkill /IM "unofficial-messenger-next.exe" /F'
    !insertmacro CLEAN_UNOFFICIAL_MESSENGER_USER_DATA
    SetOutPath "$INSTDIR"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'taskkill /IM "unofficial-messenger-next.exe" /F'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  !insertmacro CLEAN_UNOFFICIAL_MESSENGER_USER_DATA
  !insertmacro RUN_DELAYED_UNOFFICIAL_MESSENGER_CLEANUP
!macroend
