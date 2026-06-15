use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub fn log_run(
    data_dir: &Path,
    group: &str,
    file: &str,
    section: &str,
    commands: &[String],
    in_new_window: bool,
) {
    let log_path = data_dir.join("history.log");

    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let window_note = if in_new_window { " [new-window]" } else { "" };
    let commands_str = commands.join("; ");

    let line = format!(
        "{} [{}/{}] {}{}: {}\n",
        timestamp, group, file, section, window_note, commands_str
    );

    if let Some(parent) = log_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(f) => f,
        Err(_) => return,
    };

    let _ = file.write_all(line.as_bytes());
}
