use std::path::PathBuf;

use crate::config::Terminal;
use crate::parser::{load_groups, CommandFile, Group, Section};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Main,
    Custom,
    Search,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub group: String,
    pub file: String,
    pub section: String,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct SearchState {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub cursor: usize,
}

#[derive(Debug, Clone)]
pub struct PendingCommand {
    pub group: String,
    pub file: String,
    pub section: String,
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomFocus {
    Lines,
    Input,
}

#[derive(Debug, Clone)]
pub struct CustomLine {
    pub section: String,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct CustomState {
    pub lines: Vec<CustomLine>,
    pub selected: Vec<bool>,
    pub cursor: usize,
    pub focus: CustomFocus,
    pub command: String,
}

impl CustomState {
    pub fn from_file(file: &CommandFile) -> Self {
        let mut lines = Vec::new();
        for section in &file.sections {
            if section.is_run_all() {
                continue;
            }
            for cmd in &section.commands {
                lines.push(CustomLine {
                    section: section.title.clone(),
                    command: cmd.clone(),
                });
            }
        }
        let selected = vec![false; lines.len()];
        CustomState {
            lines,
            selected,
            cursor: 0,
            focus: CustomFocus::Lines,
            command: String::new(),
        }
    }

    pub fn toggle(&mut self) {
        if self.focus != CustomFocus::Lines {
            return;
        }
        if let Some(s) = self.selected.get_mut(self.cursor) {
            *s = !*s;
        }
    }

    pub fn next(&mut self) {
        if !self.lines.is_empty() && self.focus == CustomFocus::Lines {
            self.cursor = (self.cursor + 1) % self.lines.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.lines.is_empty() && self.focus == CustomFocus::Lines {
            self.cursor = self.cursor.checked_sub(1).unwrap_or(self.lines.len() - 1);
        }
    }

    pub fn selected_lines(&self) -> Vec<String> {
        self.lines
            .iter()
            .enumerate()
            .filter(|(i, _)| self.selected.get(*i).copied().unwrap_or(false))
            .map(|(_, line)| line.command.clone())
            .collect()
    }

    pub fn focus_input(&mut self) {
        self.focus = CustomFocus::Input;
    }

    pub fn focus_lines(&mut self) {
        self.focus = CustomFocus::Lines;
    }

    pub fn toggle_all(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        let all_selected = self.selected.iter().all(|s| *s);
        for s in &mut self.selected {
            *s = !all_selected;
        }
    }
}

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
    pub screen: Screen,
    pub custom_state: Option<CustomState>,
    pub search_state: Option<SearchState>,
    pub pending_command: Option<PendingCommand>,
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
            screen: Screen::Main,
            custom_state: None,
            search_state: None,
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

    pub fn enter_custom(&mut self) {
        if let Some(file) = self.current_file() {
            self.custom_state = Some(CustomState::from_file(file));
            self.screen = Screen::Custom;
        }
    }

    pub fn exit_custom(&mut self) {
        self.screen = Screen::Main;
        self.custom_state = None;
    }

    pub fn enter_search(&mut self) {
        let results = build_search_index(&self.groups);
        self.search_state = Some(SearchState {
            query: String::new(),
            results,
            cursor: 0,
        });
        self.screen = Screen::Search;
    }

    pub fn exit_search(&mut self) {
        self.screen = Screen::Main;
        self.search_state = None;
    }

    pub fn update_search(&mut self) {
        if let Some(state) = self.search_state.as_mut() {
            let query = state.query.to_lowercase();
            state.results = build_search_index(&self.groups)
                .into_iter()
                .filter(|r| {
                    r.group.to_lowercase().contains(&query)
                        || r.file.to_lowercase().contains(&query)
                        || r.section.to_lowercase().contains(&query)
                        || r.command.to_lowercase().contains(&query)
                })
                .collect();
            state.cursor = state.cursor.min(state.results.len().saturating_sub(1));
        }
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

fn build_search_index(groups: &[Group]) -> Vec<SearchResult> {
    let mut results = Vec::new();
    for group in groups {
        for file in &group.files {
            for section in &file.sections {
                if section.is_run_all() {
                    continue;
                }
                for command in &section.commands {
                    results.push(SearchResult {
                        group: group.name.clone(),
                        file: file.name.clone(),
                        section: section.title.clone(),
                        command: command.clone(),
                    });
                }
            }
        }
    }
    results
}
