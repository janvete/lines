use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub terminal: Terminal,
    pub shell: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Terminal {
    #[default]
    Builtin,
    Ghostty,
    Iterm,
}

impl Terminal {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "terminal" | "terminal.app" => Some(Terminal::Builtin),
            "ghostty" => Some(Terminal::Ghostty),
            "iterm" | "iterm2" => Some(Terminal::Iterm),
            _ => None,
        }
    }
}

impl Config {
    pub fn new(data_dir: Option<PathBuf>) -> Self {
        let data_dir = data_dir.unwrap_or_else(default_data_dir);
        let (terminal, shell) = load_preferences(&data_dir);
        Config {
            data_dir,
            terminal,
            shell,
        }
    }
}

fn default_data_dir() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().join(".lines"))
        .unwrap_or_else(|| PathBuf::from(".lines"))
}

fn default_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| {
            Path::new(&s)
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "zsh".to_string())
}

fn load_preferences(data_dir: &Path) -> (Terminal, String) {
    let config_path = data_dir.join("config.toml");
    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return (Terminal::default(), default_shell()),
    };

    match toml::from_str::<toml::Value>(&content) {
        Ok(value) => {
            let terminal = value
                .get("terminal")
                .and_then(|v| v.as_str())
                .and_then(Terminal::from_str)
                .unwrap_or_default();
            let shell = value
                .get("shell")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(default_shell);
            (terminal, shell)
        }
        Err(_) => (Terminal::default(), default_shell()),
    }
}
