use std::path::PathBuf;

pub struct Config {
    pub data_dir: PathBuf,
}

impl Config {
    pub fn new(data_dir: Option<PathBuf>) -> Self {
        let data_dir = data_dir.unwrap_or_else(default_data_dir);
        Config { data_dir }
    }
}

fn default_data_dir() -> PathBuf {
    directories::UserDirs::new()
        .map(|d| d.home_dir().join(".lines"))
        .unwrap_or_else(|| PathBuf::from(".lines"))
}
