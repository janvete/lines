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
        .title(" Skupiny ")
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
        .title(" Soubory ")
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
                format!("{} (vše)", section.title)
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
        .title(" Příkazy ")
        .borders(Borders::ALL)
        .border_style(border_style(app.focus == Focus::Sections));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn draw_preview(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let text = match app.current_section() {
        Some(section) => {
            let title = if section.is_run_all() {
                format!("{} — spustí všechny příkazy v souboru", section.title)
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
        None => Text::from("Žádný příkaz k zobrazení."),
    };

    let block = Block::default()
        .title(" Náhled ")
        .borders(Borders::ALL);

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn draw_status(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let help = "↑/↓ nebo j/k: pohyb | →/←: panel | Enter: spustit | e: editovat | r: reload | q: konec";
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
