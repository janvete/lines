use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub commands: Vec<String>,
}

impl Section {
    pub fn is_run_all(&self) -> bool {
        self.commands.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct CommandFile {
    pub name: String,
    pub path: PathBuf,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub name: String,
    pub files: Vec<CommandFile>,
}

pub fn load_groups(data_dir: &Path) -> Vec<Group> {
    let mut groups = Vec::new();

    if !data_dir.exists() {
        return groups;
    }

    let entries = match fs::read_dir(data_dir) {
        Ok(e) => e,
        Err(_) => return groups,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let files = load_files_in_group(&path);
        if !files.is_empty() {
            groups.push(Group { name, files });
        }
    }

    groups.sort_by_key(|a| a.name.to_lowercase());
    groups
}

fn load_files_in_group(group_path: &Path) -> Vec<CommandFile> {
    let mut files = Vec::new();

    let entries = match fs::read_dir(group_path) {
        Ok(e) => e,
        Err(_) => return files,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let sections = parse_file(&path);
        if !sections.is_empty() {
            files.push(CommandFile {
                name,
                path,
                sections,
            });
        }
    }

    files.sort_by_key(|a| a.name.to_lowercase());
    files
}

pub fn parse_file(path: &Path) -> Vec<Section> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    parse_markdown(&content)
}

fn parse_markdown(content: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current_title = String::new();
    let mut current_commands = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(title) = trimmed.strip_prefix("# ") {
            if !current_title.is_empty() {
                sections.push(Section {
                    title: current_title,
                    commands: current_commands,
                });
            }
            current_title = title.trim().to_string();
            current_commands = Vec::new();
        } else if trimmed.starts_with('#') || trimmed.is_empty() {
            // lower-level markdown headings, commented command lines and empty lines are ignored
            continue;
        } else {
            current_commands.push(trimmed.to_string());
        }
    }

    if !current_title.is_empty() {
        sections.push(Section {
            title: current_title,
            commands: current_commands,
        });
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_basic() {
        let input = "# Sysel DSM\ndsmview ssh root@192.168.40.15\n\n# Johy DSM\ndsmview ssh root@192.168.51.3\n";
        let sections = parse_markdown(input);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "Sysel DSM");
        assert_eq!(sections[0].commands, vec!["dsmview ssh root@192.168.40.15"]);
        assert_eq!(sections[1].title, "Johy DSM");
        assert_eq!(sections[1].commands, vec!["dsmview ssh root@192.168.51.3"]);
    }

    #[test]
    fn test_parse_markdown_run_all() {
        let input = "# Run all\n\n# Docker update\nssh admin@host1.example.com /opt/app/update.sh\nssh admin@host2.example.com /opt/app/update.sh\n";
        let sections = parse_markdown(input);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].title, "Run all");
        assert!(sections[0].commands.is_empty());
        assert_eq!(sections[1].commands.len(), 2);
    }

    #[test]
    fn test_parse_markdown_ignores_comments_and_empty() {
        let input = "# Section\n\n## Subsection\ncommand1\n\ncommand2\n";
        let sections = parse_markdown(input);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].commands, vec!["command1", "command2"]);
    }
}
