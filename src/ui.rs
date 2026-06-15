use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Focus};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(8),
            Constraint::Percentage(40),
            Constraint::Length(1),
        ])
        .split(f.area());

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
        ])
        .split(chunks[0]);

    draw_groups(f, app, top_chunks[0]);
    draw_files(f, app, top_chunks[1]);
    draw_sections(f, app, top_chunks[2]);
    draw_preview(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_groups(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .groups
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let style = if i == app.group_index {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(g.name.clone()).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Groups ")
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == Focus::Groups));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_files(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .current_files()
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let style = if i == app.file_index {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(file.name.clone()).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == Focus::Files));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_sections(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .current_sections()
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let label = if section.is_run_all() {
                format!("{} (all)", section.title)
            } else {
                section.title.clone()
            };
            let style = if i == app.section_index {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(label).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Commands ")
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == Focus::Sections));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_preview(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let text = match app.current_section() {
        Some(section) => {
            let title = if section.is_run_all() {
                format!("{} — runs all commands in this file", section.title)
            } else {
                section.title.clone()
            };

            let mut lines = vec![Line::from(Span::styled(
                title,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))];

            if section.is_run_all() {
                if let Some(file) = app.current_file() {
                    for sec in &file.sections {
                        if sec.is_run_all() {
                            continue;
                        }
                        lines.push(Line::from(Span::styled(
                            format!("# {}", sec.title),
                            Style::default().fg(Color::Cyan),
                        )));
                        for cmd in &sec.commands {
                            lines.push(Line::from(cmd.as_str()));
                        }
                        lines.push(Line::from(""));
                    }
                }
            } else {
                for cmd in &section.commands {
                    lines.push(Line::from(cmd.as_str()));
                }
            }

            Text::from(lines)
        }
        None => Text::from("No command to display."),
    };

    let block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_status(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let help = "↑/↓ or j/k: move | →/← or h/l: panel | Enter: run | o: open in new window | e: edit | r: reload | q: quit";
    let text = match &app.message {
        Some(msg) => Text::from(msg.as_str()),
        None => Text::from(help),
    };
    let paragraph = Paragraph::new(text).style(Style::default().fg(Color::Gray));
    f.render_widget(paragraph, area);
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
