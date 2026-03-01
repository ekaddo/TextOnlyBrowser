use reqwest::Client;

use crate::{
    app::{BrowserApp, FetchResult, LoadState},
    fetcher,
    parser,
};

pub fn navigate_to(app: &mut BrowserApp, url: String, client: Client) {
    if let Some(handle) = app.fetch_handle.take() {
        handle.abort();
    }

    app.history.push(url.clone());
    app.load_state = LoadState::Loading(url.clone());
    app.url_input = url.clone();

    let tx = app.tx.clone();
    let handle = app.rt.spawn(async move {
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

pub fn go_back(app: &mut BrowserApp, client: Client) {
    if let Some(url) = app.history.back() {
        navigate_no_push(app, url, client);
    }
}

pub fn go_forward(app: &mut BrowserApp, client: Client) {
    if let Some(url) = app.history.forward() {
        navigate_no_push(app, url, client);
    }
}

fn navigate_no_push(app: &mut BrowserApp, url: String, client: Client) {
    if let Some(handle) = app.fetch_handle.take() {
        handle.abort();
    }

    app.load_state = LoadState::Loading(url.clone());
    app.url_input = url.clone();

    let tx = app.tx.clone();
    let handle = app.rt.spawn(async move {
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
