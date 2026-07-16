use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseEventKind};
use std::io;
use std::time::Duration;

use crate::app::{App, CustomFocus, CustomState, PendingCommand, Screen};
use crate::commands::{edit_file, run_section};
use crate::custom::build_commands;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Tick,
    Key(event::KeyEvent),
    Mouse(event::MouseEvent),
    Paste(String),
}

pub fn poll_event(timeout: Duration) -> io::Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(Some(AppEvent::Key(key))),
            Event::Mouse(mouse) => Ok(Some(AppEvent::Mouse(mouse))),
            Event::Paste(text) => Ok(Some(AppEvent::Paste(text))),
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
                Screen::Search => handle_search_event(app, key),
            }
        }
        AppEvent::Mouse(mouse) => match mouse.kind {
            MouseEventKind::ScrollDown => {
                scroll_down(app);
                true
            }
            MouseEventKind::ScrollUp => {
                scroll_up(app);
                true
            }
            _ => true,
        },
        AppEvent::Paste(text) => handle_paste_event(app, &text),
    }
}

fn scroll_down(app: &mut App) {
    match app.screen {
        Screen::Main => app.next(),
        Screen::Custom => {
            if let Some(state) = app.custom_state.as_mut() {
                state.next();
            }
        }
        Screen::Search => {
            if let Some(state) = app.search_state.as_mut() && !state.results.is_empty() {
                state.cursor = (state.cursor + 1) % state.results.len();
            }
        }
    }
}

fn scroll_up(app: &mut App) {
    match app.screen {
        Screen::Main => app.previous(),
        Screen::Custom => {
            if let Some(state) = app.custom_state.as_mut() {
                state.previous();
            }
        }
        Screen::Search => {
            if let Some(state) = app.search_state.as_mut() && !state.results.is_empty() {
                state.cursor = state.cursor.checked_sub(1).unwrap_or(state.results.len() - 1);
            }
        }
    }
}

fn handle_main_event(app: &mut App, key: event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return false,
        KeyCode::Down => app.next(),
        KeyCode::Up => app.previous(),
        KeyCode::Left => app.focus_left(),
        KeyCode::Right => app.focus_right(),
        KeyCode::Char('r') => app.reload(),
        KeyCode::Char('e') => {
            if let Some(file) = app.current_file() {
                edit_file(&file.path, &app.terminal, &app.shell);
            }
        }
        KeyCode::Char('c') => {
            app.enter_custom();
        }
        KeyCode::Char('/') => {
            app.enter_search();
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
        KeyCode::Tab => {
            state.cycle_focus();
        }
        KeyCode::Down => {
            if state.focus == CustomFocus::Lines {
                state.next();
            }
        }
        KeyCode::Up => {
            if state.focus == CustomFocus::Lines {
                state.previous();
            }
        }
        KeyCode::Char(' ') => {
            if state.focus == CustomFocus::Lines {
                state.toggle();
            } else {
                push_to_active_input(state, ' ');
            }
        }
        KeyCode::Char('a') => {
            if state.focus == CustomFocus::Lines {
                state.toggle_all();
            } else {
                push_to_active_input(state, 'a');
            }
        }
        KeyCode::Backspace => {
            pop_from_active_input(state);
        }
        KeyCode::Enter => {
            if state.focus == CustomFocus::Input || state.focus == CustomFocus::PreCommand {
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
            } else {
                state.focus = CustomFocus::Input;
            }
        }
        KeyCode::Char(c) => {
            if state.focus == CustomFocus::Lines {
                state.command.push(c);
            } else {
                push_to_active_input(state, c);
            }
        }
        _ => {}
    }
    true
}

fn push_to_active_input(state: &mut CustomState, c: char) {
    match state.focus {
        CustomFocus::Input => state.command.push(c),
        CustomFocus::PreCommand => state.pre_command.push(c),
        CustomFocus::Lines => {}
    }
}

fn pop_from_active_input(state: &mut CustomState) {
    match state.focus {
        CustomFocus::Input => {
            state.command.pop();
        }
        CustomFocus::PreCommand => {
            state.pre_command.pop();
        }
        CustomFocus::Lines => {}
    }
}

fn handle_paste_event(app: &mut App, text: &str) -> bool {
    match app.screen {
        Screen::Main => true,
        Screen::Custom => {
            let Some(state) = app.custom_state.as_mut() else {
                app.exit_custom();
                return true;
            };
            match state.focus {
                CustomFocus::Input => state.command.push_str(text),
                CustomFocus::PreCommand => state.pre_command.push_str(text),
                CustomFocus::Lines => {}
            }
            true
        }
        Screen::Search => {
            let Some(state) = app.search_state.as_mut() else {
                app.exit_search();
                return true;
            };
            // Search queries are single-line; keep only the first pasted line.
            let query = text.lines().next().unwrap_or(text);
            state.query.push_str(query);
            app.update_search();
            true
        }
    }
}

fn handle_search_event(app: &mut App, key: event::KeyEvent) -> bool {
    let Some(state) = app.search_state.as_mut() else {
        app.exit_search();
        return true;
    };

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.exit_search();
        }
        KeyCode::Down => {
            if !state.results.is_empty() {
                state.cursor = (state.cursor + 1) % state.results.len();
            }
        }
        KeyCode::Up => {
            if !state.results.is_empty() {
                state.cursor = state.cursor.checked_sub(1).unwrap_or(state.results.len() - 1);
            }
        }
        KeyCode::Backspace => {
            state.query.pop();
            app.update_search();
        }
        KeyCode::Enter => {
            if let Some(result) = state.results.get(state.cursor) {
                app.pending_command = Some(PendingCommand {
                    group: result.group.clone(),
                    file: result.file.clone(),
                    section: result.section.clone(),
                    commands: vec![result.command.clone()],
                });
                return false;
            }
        }
        KeyCode::Char(c) => {
            state.query.push(c);
            app.update_search();
        }
        _ => {}
    }
    true
}
