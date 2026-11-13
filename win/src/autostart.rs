use std::path::Path;

pub fn command(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

pub fn approved_disabled(data: Option<&[u8]>) -> bool {
    data.and_then(|d| d.first()).is_some_and(|b| b & 1 != 0)
}

pub fn in_temp(exe: &Path, temp: &Path) -> bool {
    let exe = exe.to_string_lossy().replace('/', "\\").to_lowercase();
    let mut temp = temp.to_string_lossy().replace('/', "\\").to_lowercase();
    if !temp.ends_with('\\') {
        temp.push('\\');
    }
    // temp_dir() can come back in 8.3 form (C:\Users\PRAPHA~1\...), so also match the usual folder by name.
    exe.starts_with(&temp) || exe.contains("\\appdata\\local\\temp\\")
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Skip,
    Disable,
    Keep,
    Record,
    Write,
}

pub fn decide(
    wanted: bool,
    stamp: Option<&str>,
    command: &str,
    run: Option<&str>,
    disabled: bool,
    in_temp: bool,
) -> Decision {
    if !wanted || in_temp {
        return Decision::Skip;
    }
    if disabled {
        return Decision::Disable;
    }
    if stamp == Some(command) {
        return if run == Some(command) {
            Decision::Keep
        } else {
            Decision::Disable
        };
    }
    if run == Some(command) {
        Decision::Record
    } else {
        Decision::Write
    }
}

#[cfg(windows)]
mod windows_registry {
    use super::*;
    use crate::store::Settings;
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows::{
        core::PCWSTR,
        Win32::{
            Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
            System::Registry::*,
        },
    };

    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APPROVED: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(Some(0)).collect()
    }
    fn read(key: &str, value: &str) -> Result<Option<Vec<u8>>, String> {
        unsafe {
            let mut h = HKEY::default();
            let path = wide(key);
            let status = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                Some(0),
                KEY_QUERY_VALUE,
                &mut h,
            );
            if status == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            if status != ERROR_SUCCESS {
                return Err(format!("registry open: {status:?}"));
            }
            let name = wide(value);
            let mut kind = REG_VALUE_TYPE(0);
            let mut len = 0u32;
            let status = RegQueryValueExW(
                h,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut kind),
                None,
                Some(&mut len),
            );
            if status == ERROR_FILE_NOT_FOUND {
                let _ = RegCloseKey(h);
                return Ok(None);
            }
            if status != ERROR_SUCCESS {
                let _ = RegCloseKey(h);
                return Err(format!("registry query: {status:?}"));
            }
            let mut data = vec![0u8; len as usize];
            let status = RegQueryValueExW(
                h,
                PCWSTR(name.as_ptr()),
                None,
                Some(&mut kind),
                Some(data.as_mut_ptr()),
                Some(&mut len),
            );
            let _ = RegCloseKey(h);
            if status != ERROR_SUCCESS {
                return Err(format!("registry read: {status:?}"));
            }
            data.truncate(len as usize);
            Ok(Some(data))
        }
    }
    fn write(key: &str, name: &str, data: &[u8], kind: REG_VALUE_TYPE) -> Result<(), String> {
        unsafe {
            let mut h = HKEY::default();
            let path = wide(key);
            let status = RegCreateKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                Some(0),
                None,
                REG_OPTION_NON_VOLATILE,
                KEY_SET_VALUE,
                None,
                &mut h,
                None,
            );
            if status != ERROR_SUCCESS {
                return Err(format!("registry create: {status:?}"));
            }
            let n = wide(name);
            let status = RegSetValueExW(h, PCWSTR(n.as_ptr()), Some(0), kind, Some(data));
            let _ = RegCloseKey(h);
            if status != ERROR_SUCCESS {
                return Err(format!("registry write: {status:?}"));
            }
            Ok(())
        }
    }
    fn delete(key: &str, name: &str) -> Result<(), String> {
        unsafe {
            let mut h = HKEY::default();
            let path = wide(key);
            let status = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(path.as_ptr()),
                Some(0),
                KEY_SET_VALUE,
                &mut h,
            );
            if status == ERROR_FILE_NOT_FOUND {
                return Ok(());
            }
            if status != ERROR_SUCCESS {
                return Err(format!("registry open: {status:?}"));
            }
            let n = wide(name);
            let status = RegDeleteValueW(h, PCWSTR(n.as_ptr()));
            let _ = RegCloseKey(h);
            if status != ERROR_SUCCESS && status != ERROR_FILE_NOT_FOUND {
                return Err(format!("registry delete: {status:?}"));
            }
            Ok(())
        }
    }
    const NAME: &str = "KeyFlip";
    fn reg_sz(text: &str) -> Vec<u8> {
        text.encode_utf16()
            .chain(Some(0))
            .flat_map(u16::to_le_bytes)
            .collect()
    }
    fn run_value() -> Result<Option<String>, String> {
        Ok(read(RUN, NAME)?.map(|data| {
            let units: Vec<u16> = data
                .as_chunks::<2>()
                .0
                .iter()
                .map(|c| u16::from_le_bytes(*c))
                .take_while(|&u| u != 0)
                .collect();
            String::from_utf16_lossy(&units)
        }))
    }
    fn disabled() -> Result<bool, String> {
        Ok(approved_disabled(read(APPROVED, NAME)?.as_deref()))
    }
    fn current() -> Result<(String, bool), String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        Ok((command(&exe), in_temp(&exe, &std::env::temp_dir())))
    }
    /// At startup: register this exe, follow a moved/updated folder, and respect the user turning it off.
    pub fn sync(settings: &mut Settings) -> bool {
        let (Ok((cmd, temp)), Ok(run), Ok(disabled)) = (current(), run_value(), disabled()) else {
            return false;
        };
        let stamp = settings.autostart_command.as_deref();
        match decide(
            settings.start_with_windows,
            stamp,
            &cmd,
            run.as_deref(),
            disabled,
            temp,
        ) {
            Decision::Skip | Decision::Keep => false,
            Decision::Disable => {
                settings.start_with_windows = false;
                settings.autostart_command = None;
                true
            }
            Decision::Record => {
                settings.autostart_command = Some(cmd);
                true
            }
            Decision::Write => {
                if write(RUN, NAME, &reg_sz(&cmd), REG_SZ).is_err() {
                    return false;
                }
                settings.autostart_command = Some(cmd);
                true
            }
        }
    }
    pub fn set(settings: &mut Settings, on: bool) -> Result<(), String> {
        let (cmd, _) = current()?;
        if on {
            write(RUN, NAME, &reg_sz(&cmd), REG_SZ)?;
            delete(APPROVED, NAME)?; // an old "Disabled" from Task Manager would keep it off
            settings.start_with_windows = true;
            settings.autostart_command = Some(cmd);
        } else {
            delete(RUN, NAME)?;
            settings.start_with_windows = false;
            settings.autostart_command = None;
        }
        Ok(())
    }
    /// The real state: the Run value points at this exe and Task Manager has not disabled it.
    pub fn active() -> bool {
        let (Ok((cmd, _)), Ok(run), Ok(disabled)) = (current(), run_value(), disabled()) else {
            return false;
        };
        run.as_deref() == Some(cmd.as_str()) && !disabled
    }
}
#[cfg(windows)]
pub use windows_registry::{set, sync};
#[cfg(windows)]
pub fn active() -> bool {
    windows_registry::active()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decisions() {
        let c = "\"C:\\KeyFlip\\keyflip.exe\"";
        assert_eq!(decide(false, None, c, None, false, false), Decision::Skip);
        assert_eq!(decide(true, None, c, None, false, true), Decision::Skip);
        assert_eq!(decide(true, None, c, None, true, false), Decision::Disable);
        assert_eq!(
            decide(true, Some(c), c, Some(c), false, false),
            Decision::Keep
        );
        assert_eq!(
            decide(true, Some(c), c, None, false, false),
            Decision::Disable
        );
        assert_eq!(
            decide(true, None, c, Some(c), false, false),
            Decision::Record
        );
        assert_eq!(decide(true, None, c, None, false, false), Decision::Write);
        assert_eq!(
            decide(true, Some("other"), c, Some("other"), false, false),
            Decision::Write
        );
        assert_eq!(
            decide(true, None, c, Some("other"), false, false),
            Decision::Write
        );
        assert_eq!(
            decide(true, Some("other"), c, None, false, false),
            Decision::Write
        );
        assert_eq!(
            decide(true, Some("other"), c, Some(c), false, false),
            Decision::Record
        );
    }
    #[test]
    fn approved_bits() {
        assert!(!approved_disabled(None));
        assert!(!approved_disabled(Some(&[2, 0])));
        assert!(approved_disabled(Some(&[3, 0])));
        assert!(!approved_disabled(Some(&[6, 0])));
        assert!(approved_disabled(Some(&[7, 0])));
        assert!(!approved_disabled(Some(&[])));
    }
    #[test]
    fn temp_prefix() {
        assert!(in_temp(Path::new("c:/TEMP/x.exe"), Path::new("C:/Temp")));
        assert!(!in_temp(Path::new("C:/Else/x.exe"), Path::new("C:/Temp")));
        assert!(!in_temp(Path::new("C:/Temp2/x.exe"), Path::new("C:/Temp")));
        assert!(in_temp(
            Path::new("C:/Users/Praphat Long/AppData/Local/Temp/Temp1_kf.zip/keyflip.exe"),
            Path::new("C:/Users/PRAPHA~1/AppData/Local/Temp")
        ));
    }
}
