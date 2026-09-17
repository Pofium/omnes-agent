//! Windows Run registry autostart configuration (`HKCU\...\Run\Handy`).

use super::HandyError;
use std::path::Path;

const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "Handy";

/// Check if Handy autostart is currently enabled in the registry.
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(run_key) = hkcu.open_subkey(RUN_KEY_PATH) {
            let val: Result<String, _> = run_key.get_value(VALUE_NAME);
            return val.is_ok();
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Enable or update Handy autostart with Windows.
pub fn set_autostart_enabled(exe_path: &Path, enabled: bool) -> Result<(), HandyError> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu
            .create_subkey(RUN_KEY_PATH)
            .map_err(|e| HandyError::Other(format!("Failed to open Run key: {e}")))?
            .0;

        if enabled {
            let cmd_str = format!("\"{}\" --start-hidden", exe_path.display());
            run_key
                .set_value(VALUE_NAME, &cmd_str)
                .map_err(|e| HandyError::Other(format!("Failed to set autostart registry value: {e}")))?;
        } else {
            let _ = run_key.delete_value(VALUE_NAME);
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (exe_path, enabled);
        Ok(())
    }
}
