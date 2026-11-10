pub fn defaults() -> Vec<String> {
    [
        "WindowsTerminal.exe",
        "cmd.exe",
        "powershell.exe",
        "pwsh.exe",
        "Code.exe",
        "devenv.exe",
        "KeePass.exe",
        "KeePassXC.exe",
        "1Password.exe",
        "Bitwarden.exe",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}
pub fn excluded(path: &str, names: &[String]) -> bool {
    let exe = path.rsplit(['\\', '/']).next().unwrap_or(path);
    names.iter().any(|name| exe.eq_ignore_ascii_case(name))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_case_insensitive_filename_and_editable_list() {
        for name in defaults() {
            assert!(excluded(
                &format!("C:\\Apps\\{}", name.to_uppercase()),
                &defaults()
            ));
        }
        assert!(!excluded("C:\\Code.exe\\notepad.exe", &defaults()));
        assert!(!excluded("myCode.exe", &defaults()));
        assert!(!excluded("cmd.exe", &[]));
        assert!(excluded("/apps/Custom.EXE", &["custom.exe".into()]));
    }
}
