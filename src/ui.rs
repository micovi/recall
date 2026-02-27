use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, Paragraph, Tabs, Wrap};
use ratatui::Frame;

use crate::app::{App, Mode};

pub fn render(f: &mut Frame, app: &mut App) {
    let [tabs_area, list_area, preview_area, status_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(10),
        Constraint::Length(3),
    ])
    .areas(f.area());

    render_tabs(f, app, tabs_area);
    render_list(f, app, list_area);
    render_preview(f, app, preview_area);
    render_status(f, app, status_area);
}

fn render_tabs(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let mut titles: Vec<String> = app.categories.clone();
    titles.push("All".into());

    let tabs = Tabs::new(titles)
        .block(Block::bordered().title(" recall "))
        .select(app.selected_tab)
        .style(Style::new().fg(Color::DarkGray))
        .highlight_style(
            Style::new()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider(" | ");

    f.render_widget(tabs, area);
}

fn render_list(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let items: Vec<ListItem> = app
        .visible
        .iter()
        .map(|&i| {
            let cmd = &app.commands[i];
            let line = Line::from(vec![
                Span::styled(format!("{:<14} ", cmd.name), Style::new().fg(Color::Cyan)),
                Span::raw(&cmd.description),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::bordered())
        .highlight_style(
            Style::new()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_preview(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let content = selected_command(app).map_or_else(
        || vec![Line::from("No command selected")],
        |cmd| {
            let mut text = Vec::new();

            if !cmd.definition.is_empty() {
                text.push(Line::from(vec![
                    Span::styled(
                        "Expands to: ",
                        Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&cmd.definition, Style::new().fg(Color::White)),
                ]));
            }

            if let Some(ref example) = cmd.example {
                text.push(Line::from(""));
                text.push(Line::from(Span::styled(
                    "Example:",
                    Style::new().fg(Color::Green).add_modifier(Modifier::BOLD),
                )));
                for line in example.lines() {
                    text.push(Line::from(Span::styled(
                        format!("  {line}"),
                        Style::new().fg(Color::Gray),
                    )));
                }
            }

            text
        },
    );

    let preview = Paragraph::new(content)
        .block(Block::bordered().title(" Preview "))
        .wrap(Wrap { trim: false });

    f.render_widget(preview, area);
}

fn render_status(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let line = match app.mode {
        Mode::Search => Line::from(vec![
            Span::styled(
                format!(" / {}_", app.search_query),
                Style::new().fg(Color::Yellow),
            ),
            Span::raw("  "),
            Span::styled("ESC cancel  Enter copy  ", Style::new().fg(Color::DarkGray)),
            Span::styled(
                format!("{} matches", app.visible.len()),
                Style::new().fg(Color::DarkGray),
            ),
        ]),
        Mode::Normal => Line::from(vec![
            Span::styled(
                " / search  j/k navigate  Tab category  ",
                Style::new().fg(Color::DarkGray),
            ),
            Span::styled("Enter copy  q quit", Style::new().fg(Color::DarkGray)),
        ]),
    };

    let status = Paragraph::new(line).block(Block::bordered());
    f.render_widget(status, area);
}

fn selected_command(app: &App) -> Option<&crate::config::Command> {
    app.visible
        .get(app.list_state.selected().unwrap_or(0))
        .map(|&i| &app.commands[i])
}
