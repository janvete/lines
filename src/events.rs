use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;
use std::time::Duration;

use crate::app::{App, PendingCommand, Screen};
use crate::commands::{edit_file, run_section};
use crate::custom::build_commands;

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

            match app.screen {
                Screen::Main => handle_main_event(app, key),
                Screen::Custom => handle_custom_event(app, key),
            }
        }
    }
}

fn handle_main_event(app: &mut App, key: event::KeyEvent) -> bool {
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
        KeyCode::Char('c') => {
            app.enter_custom();
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

fn handle_custom_event(app: &mut App, key: event::KeyEvent) -> bool {
    let Some(state) = app.custom_state.as_mut() else {
        app.exit_custom();
        return true;
    };

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.exit_custom();
        }
        KeyCode::Char('j') | KeyCode::Down => state.next(),
        KeyCode::Char('k') | KeyCode::Up => state.previous(),
        KeyCode::Char(' ') => state.toggle(),
        KeyCode::Backspace => {
            state.command.pop();
        }
        KeyCode::Enter => {
            let commands = build_commands(state);
            if !commands.is_empty()
                && let (Some(file), Some(group)) = (app.current_file(), app.current_group())
            {
                app.pending_command = Some(PendingCommand {
                    group: group.name.clone(),
                    file: file.name.clone(),
                    section: "custom".to_string(),
                    commands,
                });
                return false;
            }
        }
        KeyCode::Char(c) => {
            state.command.push(c);
        }
        _ => {}
    }
    true
}
