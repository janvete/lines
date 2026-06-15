use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

use crate::app::{App, PendingCommand};
use crate::commands::{edit_file, run_section};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    Tick,
    Key(event::KeyEvent),
}

pub fn poll_event(timeout: Duration) -> io::Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(Some(AppEvent::Key(key))),
            _ => Ok(Some(AppEvent::Tick)),
        }
    } else {
        Ok(Some(AppEvent::Tick))
    }
}

pub fn handle_event(app: &mut App, event: AppEvent) -> bool {
    match event {
        AppEvent::Tick => true,
        AppEvent::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return true;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return false,
                KeyCode::Char('j') | KeyCode::Down => app.next(),
                KeyCode::Char('k') | KeyCode::Up => app.previous(),
                KeyCode::Char('h') | KeyCode::Left => app.focus_left(),
                KeyCode::Char('l') | KeyCode::Right => app.focus_right(),
                KeyCode::Char('r') => app.reload(),
                KeyCode::Char('e') => {
                    if let Some(file) = app.current_file() {
                        edit_file(&file.path, &app.terminal, &app.shell);
                    }
                }
                KeyCode::Char('o') => {
                    if let (Some(section), Some(file), Some(group)) =
                        (app.current_section(), app.current_file(), app.current_group())
                    {
                        run_section(
                            section,
                            file,
                            &app.terminal,
                            &app.shell,
                            &app.data_dir,
                            &group.name,
                            &file.name,
                            &section.title,
                        );
                    }
                }
                KeyCode::Enter => {
                    if let (Some(section), Some(file), Some(group)) =
                        (app.current_section(), app.current_file(), app.current_group())
                    {
                        let commands = if section.is_run_all() {
                            file.sections
                                .iter()
                                .filter(|s| !s.is_run_all())
                                .flat_map(|s| s.commands.clone())
                                .collect()
                        } else {
                            section.commands.clone()
                        };
                        if !commands.is_empty() {
                            app.pending_command = Some(PendingCommand {
                                group: group.name.clone(),
                                file: file.name.clone(),
                                section: section.title.clone(),
                                commands,
                            });
                            return false;
                        }
                    }
                }
                _ => {}
            }
            true
        }
    }
}
