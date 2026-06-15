use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub terminal: Terminal,
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
        let terminal = load_terminal_preference(&data_dir);
        Config { data_dir, terminal }
    }
}

fn default_data_dir() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().join(".lines"))
        .unwrap_or_else(|| PathBuf::from(".lines"))
}

fn load_terminal_preference(data_dir: &Path) -> Terminal {
    let config_path = data_dir.join("config.toml");
    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Terminal::default(),
    };

    match toml::from_str::<toml::Value>(&content) {
        Ok(value) => value
            .get("terminal")
            .and_then(|v| v.as_str())
            .and_then(Terminal::from_str)
            .unwrap_or_default(),
        Err(_) => Terminal::default(),
    }
}
