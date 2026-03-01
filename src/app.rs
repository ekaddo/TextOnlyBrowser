use std::sync::mpsc;

use crate::{config::Config, history::History, parser::RenderedPage};

#[derive(Debug, Clone)]
pub enum LoadState {
    Idle,
    Loading(String),
    Error(String),
}

pub enum FetchResult {
    Success(RenderedPage),
    Failure { url: String, message: String },
}

pub struct BrowserApp {
    pub config: Config,
    pub history: History,
    pub page: Option<RenderedPage>,
    pub load_state: LoadState,
    pub url_input: String,
    pub scroll_to_top: bool,
    pub dark_mode: bool,
    pub fetch_handle: Option<tokio::task::JoinHandle<()>>,
    pub rx: mpsc::Receiver<FetchResult>,
    pub tx: mpsc::SyncSender<FetchResult>,
    pub rt: tokio::runtime::Handle,
}

impl BrowserApp {
    pub fn new(rt: tokio::runtime::Handle, config: Config) -> Self {
        let (tx, rx) = mpsc::sync_channel(8);
        Self {
            config,
            history: History::new(),
            page: None,
            load_state: LoadState::Idle,
            url_input: String::new(),
            scroll_to_top: false,
            dark_mode: true,
            fetch_handle: None,
            rx,
            tx,
            rt,
        }
    }

    pub fn current_url(&self) -> &str {
        if let LoadState::Loading(url) = &self.load_state {
            url.as_str()
        } else {
            self.history.current().unwrap_or("")
        }
    }

    /// Drain the fetch result channel and update state.
    pub fn poll_fetch_results(&mut self, ctx: &egui::Context) {
        while let Ok(result) = self.rx.try_recv() {
            match result {
                FetchResult::Success(page) => {
                    self.url_input = page.url.clone();
                    self.page = Some(page);
                    self.load_state = LoadState::Idle;
                    self.scroll_to_top = true;
                }
                FetchResult::Failure { message, .. } => {
                    self.load_state = LoadState::Error(message);
                }
            }
            ctx.request_repaint();
        }
    }
}
