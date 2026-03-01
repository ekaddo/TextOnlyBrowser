use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, LoadState, Mode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Update viewport height (subtract URL bar + status bar)
    app.viewport_height = area.height.saturating_sub(2);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // URL bar
            Constraint::Min(0),    // content
            Constraint::Length(1), // status bar
        ])
        .split(area);

    // ── URL bar ─────────────────────────────────────────────────────────────
    let url_bar_content = match &app.mode {
        Mode::UrlEntry => {
            let input = app.url_input.clone();
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    input,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(
                    "█",
                    Style::default().fg(Color::White),
                ),
            ])
        }
        Mode::Browse => {
            let url = app.current_url().to_string();
            let loading = matches!(app.load_state, LoadState::Loading(_));
            let indicator = if loading { " ⟳" } else { "" };
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{url}{indicator}"),
                    Style::default().fg(Color::Cyan),
                ),
            ])
        }
    };

    let url_bar = Paragraph::new(url_bar_content)
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(url_bar, chunks[0]);

    // ── Content area ─────────────────────────────────────────────────────────
    let content = if app.rendered_lines.is_empty() {
        match &app.load_state {
            LoadState::Loading(url) => vec![Line::from(format!("Loading {}…", url))],
            LoadState::Idle => vec![Line::from("Press 'u' to enter a URL")],
            LoadState::Error(e) => vec![Line::from(e.clone())],
        }
    } else {
        app.rendered_lines.clone()
    };

    let content_widget = Paragraph::new(content)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));
    f.render_widget(content_widget, chunks[1]);

    // ── Status bar ────────────────────────────────────────────────────────────
    let status = build_status(app);
    let status_bar = Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(status_bar, chunks[2]);

    // Position cursor when in URL entry mode
    if app.mode == Mode::UrlEntry {
        let x = chunks[0].x + 5 + app.url_input.len() as u16; // "URL: " = 5 chars
        let y = chunks[0].y;
        f.set_cursor_position((x.min(chunks[0].right().saturating_sub(1)), y));
    }
}

fn build_status(app: &App) -> Line<'static> {
    let key_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let dim_style = Style::default().fg(Color::DarkGray);

    match &app.mode {
        Mode::UrlEntry => Line::from(vec![
            Span::styled("  Enter", key_style),
            Span::styled(":go  ", dim_style),
            Span::styled("Esc", key_style),
            Span::styled(":cancel", dim_style),
        ]),
        Mode::Browse => {
            let link_info = if let Some(page) = &app.page {
                if !page.links.is_empty() {
                    let sel = app
                        .selected_link
                        .map(|n| format!(" link {}/{}", n, page.links.len()))
                        .unwrap_or_default();
                    format!("{sel}  ")
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let back_fwd = format!(
                "{}{}",
                if app.history.can_go_back() { "← " } else { "" },
                if app.history.can_go_forward() { "→ " } else { "" },
            );

            Line::from(vec![
                Span::styled("  ↑↓", key_style),
                Span::styled(":scroll  ", dim_style),
                Span::styled("Tab", key_style),
                Span::styled(":link  ", dim_style),
                Span::styled("Enter", key_style),
                Span::styled(":follow  ", dim_style),
                Span::styled("b", key_style),
                Span::styled(":back  ", dim_style),
                Span::styled("u", key_style),
                Span::styled(":url  ", dim_style),
                Span::styled("q", key_style),
                Span::styled(":quit", dim_style),
                Span::raw(format!("  {back_fwd}{link_info}")),
            ])
        }
    }
}
