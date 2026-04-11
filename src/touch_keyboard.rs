#[cfg(target_os = "windows")]
pub fn show_touch_keyboard() {
    use std::path::PathBuf;
    use std::process::Command;

    let mut candidates = Vec::new();

    if let Ok(common_program_files) = std::env::var("CommonProgramFiles") {
        candidates.push(PathBuf::from(&common_program_files).join("microsoft shared/ink/TabTip.exe"));
    }
    if let Ok(common_program_files_x86) = std::env::var("CommonProgramFiles(x86)") {
        candidates
            .push(PathBuf::from(&common_program_files_x86).join("microsoft shared/ink/TabTip.exe"));
    }

    candidates.push(PathBuf::from(
        r"C:\Program Files\Common Files\microsoft shared\ink\TabTip.exe",
    ));
    candidates.push(PathBuf::from(
        r"C:\Program Files (x86)\Common Files\microsoft shared\ink\TabTip.exe",
    ));

    for candidate in candidates {
        if candidate.exists() {
            let _ = Command::new(candidate).spawn();
            break;
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn show_touch_keyboard() {}
