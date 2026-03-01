mod app;
mod browser;
mod config;
mod fetcher;
mod history;
mod input;
mod parser;
mod renderer;
mod ui;

use std::io;

use anyhow::Result;
use crossterm::{
    event::EventStream,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};

use app::{App, FetchResult, Mode};
use input::{Action, map_event};

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load_config();
    let home_url = cfg.home.clone();

    let client = fetcher::build_client(cfg.timeout_secs, cfg.max_redirects)?;

    let mut app = App::new(cfg);

    // Install panic hook to restore terminal state
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initial page load
    browser::navigate_to(&mut app, home_url, client.clone());

    // Main event loop
    let mut event_stream = EventStream::new();

    loop {
        // Determine terminal width for renderer
        let term_width = terminal.size().map(|s| s.width).unwrap_or(80);

        terminal.draw(|f| ui::draw(f, &mut app))?;

        tokio::select! {
            // Keyboard / resize events
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        let action = map_event(event, &app.mode);
                        handle_action(&mut app, action, &client, term_width);
                        if app.should_quit {
                            break;
                        }
                    }
                    Some(Err(_)) => break,
                    None => break,
                }
            }

            // Fetch results from background task
            maybe_result = app.rx.recv() => {
                if let Some(result) = maybe_result {
                    browser::handle_fetch_result(&mut app, result, term_width);
                    // Rerender lines with current selection
                    if let Some(page) = &app.page {
                        app.rendered_lines =
                            renderer::render(page, term_width, app.selected_link);
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_action(app: &mut App, action: Action, client: &reqwest::Client, term_width: u16) {
    match action {
        Action::Quit => app.should_quit = true,

        Action::ScrollUp => {
            app.scroll_up(1);
        }
        Action::ScrollDown => {
            app.scroll_down(1);
        }
        Action::PageUp => {
            app.scroll_up(app.viewport_height.saturating_sub(1));
        }
        Action::PageDown => {
            app.scroll_down(app.viewport_height.saturating_sub(1));
        }

        Action::TabNext => {
            app.tab_next();
            rerender(app, term_width);
        }
        Action::TabPrev => {
            app.tab_prev();
            rerender(app, term_width);
        }

        Action::FollowSelected => {
            if let Some(url) = app.selected_url() {
                browser::navigate_to(app, url, client.clone());
            }
        }

        Action::GoToLink(n) => {
            if let Some(page) = &app.page {
                if let Some(link) = page.links.iter().find(|l| l.number == n) {
                    let url = link.url.clone();
                    browser::navigate_to(app, url, client.clone());
                }
            }
        }

        Action::GoBack => {
            browser::go_back(app, client.clone());
        }
        Action::GoForward => {
            browser::go_forward(app, client.clone());
        }

        Action::OpenUrlBar => {
            app.mode = Mode::UrlEntry;
            // Pre-populate with the current URL so the user can edit it
            app.url_input = app.current_url().to_string();
        }

        Action::UrlInputChar(c) => {
            app.url_input.push(c);
        }
        Action::UrlInputBackspace => {
            app.url_input.pop();
        }
        Action::UrlInputDelete => {
            app.url_input.pop();
        }
        Action::UrlInputSubmit => {
            let raw = app.url_input.trim().to_string();
            if !raw.is_empty() {
                // Prepend https:// if no scheme given
                let url = if raw.contains("://") {
                    raw
                } else {
                    format!("https://{raw}")
                };
                browser::navigate_to(app, url, client.clone());
            } else {
                app.mode = Mode::Browse;
            }
        }
        Action::UrlInputCancel => {
            app.mode = Mode::Browse;
            app.url_input.clear();
        }

        Action::Resize(w, _h) => {
            rerender(app, w);
        }

        Action::None => {}
    }
}

fn rerender(app: &mut App, term_width: u16) {
    if let Some(page) = &app.page {
        app.rendered_lines = renderer::render(page, term_width, app.selected_link);
        app.clamp_scroll();
    }
}
