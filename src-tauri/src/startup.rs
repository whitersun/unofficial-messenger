#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const STARTUP_REGISTRY_VALUE: &str = "UnofficialMessenger";
#[cfg(windows)]
const STARTUP_SHORTCUT_FILE: &str = "Unofficial Messenger.lnk";
#[cfg(windows)]
const STARTUP_ARGUMENT: &str = "--tauri-startup";
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub(crate) const STARTUP_LOAD_DELAY_MS: u64 = 8000;

#[cfg(windows)]
pub(crate) fn was_launched_from_windows_startup() -> bool {
    std::env::args().any(|arg| arg == STARTUP_ARGUMENT)
}

#[cfg(not(windows))]
pub(crate) fn was_launched_from_windows_startup() -> bool {
    false
}

#[cfg(windows)]
pub(crate) fn normalize_startup_working_directory() {
    if !was_launched_from_windows_startup() {
        return;
    }

    let Some(executable_dir) = std::env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(|parent| parent.to_path_buf()))
    else {
        return;
    };

    if let Err(error) = std::env::set_current_dir(&executable_dir) {
        eprintln!("failed to set startup working directory: {error}");
    }
}

#[cfg(not(windows))]
pub(crate) fn normalize_startup_working_directory() {}

#[cfg(windows)]
pub(crate) fn is_start_with_windows_enabled() -> bool {
    startup_shortcut_path().is_some_and(|path| path.exists()) || has_legacy_startup_registry_entry()
}

#[cfg(windows)]
fn has_legacy_startup_registry_entry() -> bool {
    Command::new("reg")
        .args([
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
            "/v",
            STARTUP_REGISTRY_VALUE,
        ])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub(crate) fn is_start_with_windows_enabled() -> bool {
    false
}

#[cfg(windows)]
pub(crate) fn set_start_with_windows(enabled: bool) -> std::io::Result<()> {
    remove_legacy_startup_registry_entry();

    if enabled {
        create_startup_shortcut()
    } else {
        remove_startup_shortcut()
    }
}

#[cfg(windows)]
fn startup_shortcut_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|app_data| {
        PathBuf::from(app_data)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup")
            .join(STARTUP_SHORTCUT_FILE)
    })
}

#[cfg(windows)]
fn create_startup_shortcut() -> std::io::Result<()> {
    let executable = std::env::current_exe()?;
    let executable_dir = executable
        .parent()
        .ok_or_else(|| {
            std::io::Error::other("failed to find executable directory for startup shortcut")
        })?
        .to_path_buf();
    let shortcut = startup_shortcut_path()
        .ok_or_else(|| std::io::Error::other("failed to find Windows Startup folder"))?;

    if let Some(parent) = shortcut.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let status = Command::new("powershell")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            r#"& {
param($ShortcutPath, $TargetPath, $Arguments, $WorkingDirectory, $IconPath)
$ErrorActionPreference = 'Stop'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($ShortcutPath)
$shortcut.TargetPath = $TargetPath
$shortcut.Arguments = $Arguments
$shortcut.WorkingDirectory = $WorkingDirectory
$shortcut.IconLocation = "$IconPath,0"
$shortcut.Save()
}
"#,
        ])
        .arg(shortcut)
        .arg(&executable)
        .arg(STARTUP_ARGUMENT)
        .arg(&executable_dir)
        .arg(&executable)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("failed to create startup shortcut"))
    }
}

#[cfg(windows)]
fn remove_startup_shortcut() -> std::io::Result<()> {
    let Some(shortcut) = startup_shortcut_path() else {
        return Ok(());
    };

    match std::fs::remove_file(shortcut) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
fn remove_legacy_startup_registry_entry() {
    let _ = Command::new("reg")
        .creation_flags(CREATE_NO_WINDOW)
        .arg("delete")
        .arg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run")
        .arg("/v")
        .arg(STARTUP_REGISTRY_VALUE)
        .arg("/f")
        .status();
}

#[cfg(not(windows))]
pub(crate) fn set_start_with_windows(_enabled: bool) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "start with Windows is only supported on Windows",
    ))
}
