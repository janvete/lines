use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use crate::parser::{CommandFile, Section};

pub fn run_section(section: &Section, file: &CommandFile) {
    let commands = if section.is_run_all() {
        file.sections
            .iter()
            .filter(|s| !s.is_run_all())
            .flat_map(|s| s.commands.clone())
            .collect()
    } else {
        section.commands.clone()
    };

    if commands.is_empty() {
        return;
    }

    let script = build_script(&commands);
    execute_in_terminal(&script);
}

fn build_script(commands: &[String]) -> String {
    commands.join("\n")
}

fn execute_in_terminal(script: &str) {
    let tmp_dir = std::env::temp_dir();
    let script_path = tmp_dir.join(format!("lines_run_{}.sh", random_suffix()));

    let mut file = match std::fs::File::create(&script_path) {
        Ok(f) => f,
        Err(_) => return,
    };

    if file.write_all(script.as_bytes()).is_err() {
        return;
    }

    // Best-effort attempt to make executable; Terminal.app can run `bash script.sh` anyway.
    let _ = std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755));

    let path_str = script_path.to_string_lossy();

    // Open a new Terminal.app window and run the script.
    let applescript = format!(
        r#"tell application "Terminal"
    do script "clear; bash {}; rm {}"
    activate
end tell"#,
        escape_applescript(&path_str),
        escape_applescript(&path_str)
    );

    let _ = Command::new("osascript")
        .arg("-e")
        .arg(applescript)
        .spawn();
}

fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn random_suffix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn edit_file(path: &Path) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(format!(
            r#"tell application "Terminal"
    do script "cd {}; {} {}; exit"
    activate
end tell"#,
            escape_applescript(&std::env::current_dir().unwrap_or_default().to_string_lossy()),
            escape_applescript(&editor),
            escape_applescript(&path.to_string_lossy())
        ))
        .spawn();
}
