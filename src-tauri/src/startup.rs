#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const STARTUP_REGISTRY_VALUE: &str = "UnofficialMessenger";
#[cfg(windows)]
const STARTUP_ARGUMENT: &str = "--tauri-startup";

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
    let status = if enabled {
        let executable = std::env::current_exe()?;
        let executable = format!("\"{}\" {STARTUP_ARGUMENT}", executable.display());

        Command::new("reg")
            .arg("add")
            .arg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run")
            .arg("/v")
            .arg(STARTUP_REGISTRY_VALUE)
            .arg("/t")
            .arg("REG_SZ")
            .arg("/d")
            .arg(executable)
            .arg("/f")
            .status()?
    } else {
        Command::new("reg")
            .arg("delete")
            .arg(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run")
            .arg("/v")
            .arg(STARTUP_REGISTRY_VALUE)
            .arg("/f")
            .status()?
    };

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(
            "failed to update Windows startup registry entry",
        ))
    }
}

#[cfg(not(windows))]
pub(crate) fn set_start_with_windows(_enabled: bool) -> std::io::Result<()> {
    Err(std::io::Error::other(
        "start with Windows is only supported on Windows",
    ))
}
