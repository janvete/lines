use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::config::Terminal;
use crate::logger;
use crate::parser::{CommandFile, Section};

#[allow(clippy::too_many_arguments)]
pub fn run_section(
    section: &Section,
    file: &CommandFile,
    terminal: &Terminal,
    shell: &str,
    data_dir: &Path,
    group_name: &str,
    file_name: &str,
    section_name: &str,
) {
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

    logger::log_run(
        data_dir,
        group_name,
        file_name,
        section_name,
        &commands,
        true,
    );

    match terminal {
        Terminal::Ghostty => run_in_ghostty(&commands, shell),
        _ => run_via_script(&commands, terminal, shell, data_dir),
    }
}

fn run_via_script(commands: &[String], terminal: &Terminal, shell: &str, data_dir: &Path) {
    let script = commands.join("\n");
    let run_dir = data_dir.join(".run");
    if fs::create_dir_all(&run_dir).is_err() {
        return;
    }

    let script_path = run_dir.join(format!("lines_run_{}.sh", random_suffix()));

    let mut file = match fs::File::create(&script_path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let script_with_cleanup = format!(
        "{}\nrm -f {}",
        script,
        escape_shell(&script_path.to_string_lossy())
    );

    if file.write_all(script_with_cleanup.as_bytes()).is_err() {
        return;
    }

    let _ = fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755));

    match terminal {
        Terminal::Iterm => run_in_iterm(&script_path, shell),
        Terminal::Builtin => run_in_terminal_app(&script_path, shell),
        Terminal::Ghostty => unreachable!(),
    }
}

fn run_in_ghostty(commands: &[String], shell: &str) {
    let script = commands.join("\n");
    let encoded = STANDARD.encode(script);
    let shell_command = format!("echo {} | base64 -D | {}", encoded, shell);

    let _ = Command::new("open")
        .arg("-na")
        .arg("Ghostty")
        .arg("--args")
        .arg("-e")
        .arg(shell)
        .arg("-c")
        .arg(shell_command)
        .spawn();
}

fn run_in_terminal_app(script_path: &Path, shell: &str) {
    let path_str = script_path.to_string_lossy();
    let applescript = format!(
        r#"tell application "Terminal"
    do script "clear; {} {}"
    activate
end tell"#,
        shell,
        escape_applescript(&path_str)
    );

    let _ = Command::new("osascript")
        .arg("-e")
        .arg(applescript)
        .spawn();
}

fn run_in_iterm(script_path: &Path, shell: &str) {
    let path_str = script_path.to_string_lossy();
    let applescript = format!(
        r#"tell application "iTerm"
    create window with default profile
    tell current session of current window
        write text "clear; {} {}"
    end tell
    activate
end tell"#,
        shell,
        escape_applescript(&path_str)
    );

    let _ = Command::new("osascript")
        .arg("-e")
        .arg(applescript)
        .spawn();
}

pub fn edit_file(path: &Path, terminal: &Terminal, shell: &str) {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

    match terminal {
        Terminal::Ghostty => {
            let _ = Command::new("open")
                .arg("-na")
                .arg("Ghostty")
                .arg("--args")
                .arg("-e")
                .arg(shell)
                .arg("-c")
                .arg(format!(
                    "cd {} && {} {}; exec {}",
                    escape_shell(&std::env::current_dir().unwrap_or_default().to_string_lossy()),
                    escape_shell(&editor),
                    escape_shell(&path.to_string_lossy()),
                    shell
                ))
                .spawn();
        }
        _ => {
            let app = match terminal {
                Terminal::Iterm => "iTerm",
                _ => "Terminal",
            };
            let _ = Command::new("osascript")
                .arg("-e")
                .arg(format!(
                    r#"tell application "{}"
    do script "cd {}; {} {}; exit"
    activate
end tell"#,
                    app,
                    escape_applescript(&std::env::current_dir().unwrap_or_default().to_string_lossy()),
                    escape_applescript(&editor),
                    escape_applescript(&path.to_string_lossy())
                ))
                .spawn();
        }
    }
}

fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_shell(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\'', "'\"'\"'")
}

fn random_suffix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
