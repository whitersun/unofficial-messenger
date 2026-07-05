#[cfg(windows)]
use std::process::Command;

#[cfg(windows)]
const STARTUP_REGISTRY_VALUE: &str = "UnofficialMessenger";

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
        let executable = format!("\"{}\"", executable.display());

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
