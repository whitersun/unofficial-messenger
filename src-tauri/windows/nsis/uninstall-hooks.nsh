!macro CLEAN_UNOFFICIAL_MESSENGER_USER_DATA
  SetShellVarContext current

  Delete "$APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\Unofficial Messenger.lnk"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "UnofficialMessenger"

  RMDir /r "$APPDATA\unofficial-messenger-next"
  RMDir /r "$LOCALAPPDATA\unofficial-messenger-next"
  RMDir /r "$APPDATA\io.github.whitersun.unofficial-messenger-next"
  RMDir /r "$LOCALAPPDATA\io.github.whitersun.unofficial-messenger-next"
!macroend

!macro NSIS_HOOK_PREINSTALL
  ReadRegStr $0 SHCTX "${UNINSTKEY}" "InstallLocation"

  ${If} $UpdateMode <> 1
  ${AndIf} $0 == ""
    nsExec::ExecToLog 'taskkill /IM "unofficial-messenger-next.exe" /F'
    !insertmacro CLEAN_UNOFFICIAL_MESSENGER_USER_DATA
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog 'taskkill /IM "unofficial-messenger-next.exe" /F'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  !insertmacro CLEAN_UNOFFICIAL_MESSENGER_USER_DATA
!macroend
