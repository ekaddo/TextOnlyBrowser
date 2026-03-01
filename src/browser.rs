use reqwest::Client;

use crate::{
    app::{App, FetchResult, LoadState, Mode},
    fetcher,
    parser,
    renderer,
};

pub fn navigate_to(app: &mut App, url: String, client: Client) {
    // Abort any in-flight fetch
    if let Some(handle) = app.fetch_handle.take() {
        handle.abort();
    }

    app.history.push(url.clone());
    app.load_state = LoadState::Loading(url.clone());
    app.scroll = 0;
    app.selected_link = None;
    app.rendered_lines.clear();
    app.mode = Mode::Browse;

    let tx = app.tx.clone();
    let handle = tokio::spawn(async move {
        match fetcher::fetch(&client, &url).await {
            Ok((final_url, html)) => {
                let page = parser::parse(&final_url, &html);
                let _ = tx.send(FetchResult::Success(page));
            }
            Err(e) => {
                let _ = tx.send(FetchResult::Failure {
                    url,
                    message: e.to_string(),
                });
            }
        }
    });
    app.fetch_handle = Some(handle);
}

pub fn go_back(app: &mut App, client: Client) {
    // Remove current from history so back() returns the previous
    if let Some(url) = app.history.back() {
        // history.back() already updated current; navigate there
        navigate_to_no_push(app, url, client);
    }
}

pub fn go_forward(app: &mut App, client: Client) {
    if let Some(url) = app.history.forward() {
        navigate_to_no_push(app, url, client);
    }
}

/// Navigate without pushing to history (used after history.back/forward already updated state).
fn navigate_to_no_push(app: &mut App, url: String, client: Client) {
    if let Some(handle) = app.fetch_handle.take() {
        handle.abort();
    }

    app.load_state = LoadState::Loading(url.clone());
    app.scroll = 0;
    app.selected_link = None;
    app.rendered_lines.clear();
    app.mode = Mode::Browse;

    let tx = app.tx.clone();
    let handle = tokio::spawn(async move {
        match fetcher::fetch(&client, &url).await {
            Ok((final_url, html)) => {
                let page = parser::parse(&final_url, &html);
                let _ = tx.send(FetchResult::Success(page));
            }
            Err(e) => {
                let _ = tx.send(FetchResult::Failure {
                    url,
                    message: e.to_string(),
                });
            }
        }
    });
    app.fetch_handle = Some(handle);
}

pub fn handle_fetch_result(app: &mut App, result: FetchResult, terminal_width: u16) {
    match result {
        FetchResult::Success(page) => {
            let lines = renderer::render(&page, terminal_width, app.selected_link);
            app.rendered_lines = lines;
            app.page = Some(page);
            app.load_state = LoadState::Idle;
        }
        FetchResult::Failure { url, message } => {
            app.load_state = LoadState::Error(message.clone());
            // Show error as page content
            app.rendered_lines = vec![
                ratatui::text::Line::from(format!("Error loading: {url}")),
                ratatui::text::Line::from(""),
                ratatui::text::Line::from(message),
            ];
        }
    }
}
