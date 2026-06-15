use std::path::PathBuf;

use crate::config::Terminal;
use crate::parser::{load_groups, CommandFile, Group, Section};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Groups,
    Files,
    Sections,
}

pub struct App {
    pub data_dir: PathBuf,
    pub terminal: Terminal,
    pub shell: String,
    pub groups: Vec<Group>,
    pub group_index: usize,
    pub file_index: usize,
    pub section_index: usize,
    pub focus: Focus,
    pub pending_command: Option<Vec<String>>,
    pub message: Option<String>,
}

impl App {
    pub fn new(data_dir: PathBuf, terminal: Terminal, shell: String) -> Self {
        let groups = load_groups(&data_dir);
        App {
            data_dir,
            terminal,
            shell,
            groups,
            group_index: 0,
            file_index: 0,
            section_index: 0,
            focus: Focus::Groups,
            pending_command: None,
            message: None,
        }
    }

    pub fn reload(&mut self) {
        self.groups = load_groups(&self.data_dir);
        self.group_index = self.group_index.min(self.groups.len().saturating_sub(1));
        self.file_index = self.file_index.min(self.current_files().len().saturating_sub(1));
        self.section_index = self.section_index.min(self.current_sections().len().saturating_sub(1));
    }

    pub fn current_group(&self) -> Option<&Group> {
        self.groups.get(self.group_index)
    }

    pub fn current_file(&self) -> Option<&CommandFile> {
        self.current_group()?.files.get(self.file_index)
    }

    pub fn current_section(&self) -> Option<&Section> {
        self.current_file()?.sections.get(self.section_index)
    }

    pub fn current_files(&self) -> &[CommandFile] {
        match self.current_group() {
            Some(g) => &g.files,
            None => &[],
        }
    }

    pub fn current_sections(&self) -> &[Section] {
        match self.current_file() {
            Some(f) => &f.sections,
            None => &[],
        }
    }

    pub fn next(&mut self) {
        match self.focus {
            Focus::Groups => {
                if !self.groups.is_empty() {
                    self.group_index = (self.group_index + 1) % self.groups.len();
                    self.file_index = 0;
                    self.section_index = 0;
                }
            }
            Focus::Files => {
                let len = self.current_files().len();
                if len > 0 {
                    self.file_index = (self.file_index + 1) % len;
                    self.section_index = 0;
                }
            }
            Focus::Sections => {
                let len = self.current_sections().len();
                if len > 0 {
                    self.section_index = (self.section_index + 1) % len;
                }
            }
        }
    }

    pub fn previous(&mut self) {
        match self.focus {
            Focus::Groups => {
                if !self.groups.is_empty() {
                    self.group_index = self.group_index.checked_sub(1).unwrap_or(self.groups.len() - 1);
                    self.file_index = 0;
                    self.section_index = 0;
                }
            }
            Focus::Files => {
                let len = self.current_files().len();
                if len > 0 {
                    self.file_index = self.file_index.checked_sub(1).unwrap_or(len - 1);
                    self.section_index = 0;
                }
            }
            Focus::Sections => {
                let len = self.current_sections().len();
                if len > 0 {
                    self.section_index = self.section_index.checked_sub(1).unwrap_or(len - 1);
                }
            }
        }
    }

    pub fn focus_right(&mut self) {
        self.focus = match self.focus {
            Focus::Groups => Focus::Files,
            Focus::Files => Focus::Sections,
            Focus::Sections => Focus::Sections,
        };
    }

    pub fn focus_left(&mut self) {
        self.focus = match self.focus {
            Focus::Groups => Focus::Groups,
            Focus::Files => Focus::Groups,
            Focus::Sections => Focus::Files,
        };
    }
}
