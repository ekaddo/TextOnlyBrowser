use ratatui::text::Line;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{config::Config, history::History, parser::RenderedPage};

#[derive(Debug, Clone)]
pub enum LoadState {
    Idle,
    Loading(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Browse,
    UrlEntry,
}

pub enum FetchResult {
    Success(RenderedPage),
    Failure { url: String, message: String },
}

pub struct App {
    pub config: Config,
    pub history: History,
    pub page: Option<RenderedPage>,
    pub load_state: LoadState,
    pub mode: Mode,
    pub scroll: u16,
    pub selected_link: Option<usize>,
    pub url_input: String,
    pub rendered_lines: Vec<Line<'static>>,
    pub viewport_height: u16,
    pub should_quit: bool,
    pub fetch_handle: Option<tokio::task::JoinHandle<()>>,
    pub rx: UnboundedReceiver<FetchResult>,
    pub tx: UnboundedSender<FetchResult>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            config,
            history: History::new(),
            page: None,
            load_state: LoadState::Idle,
            mode: Mode::Browse,
            scroll: 0,
            selected_link: None,
            url_input: String::new(),
            rendered_lines: Vec::new(),
            viewport_height: 24,
            should_quit: false,
            fetch_handle: None,
            rx,
            tx,
        }
    }

    pub fn current_url(&self) -> &str {
        if let LoadState::Loading(url) = &self.load_state {
            url.as_str()
        } else {
            self.history.current().unwrap_or("")
        }
    }

    pub fn clamp_scroll(&mut self) {
        let max = (self.rendered_lines.len() as u16).saturating_sub(self.viewport_height);
        if self.scroll > max {
            self.scroll = max;
        }
    }

    pub fn scroll_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_add(n);
        self.clamp_scroll();
    }

    pub fn tab_next(&mut self) {
        if let Some(page) = &self.page {
            if page.links.is_empty() {
                return;
            }
            let next = match self.selected_link {
                None => 1,
                Some(n) => {
                    if n >= page.links.len() {
                        1
                    } else {
                        n + 1
                    }
                }
            };
            self.selected_link = Some(next);
        }
    }

    pub fn tab_prev(&mut self) {
        if let Some(page) = &self.page {
            if page.links.is_empty() {
                return;
            }
            let prev = match self.selected_link {
                None => page.links.len(),
                Some(n) => {
                    if n <= 1 {
                        page.links.len()
                    } else {
                        n - 1
                    }
                }
            };
            self.selected_link = Some(prev);
        }
    }

    pub fn selected_url(&self) -> Option<String> {
        let page = self.page.as_ref()?;
        let num = self.selected_link?;
        page.links
            .iter()
            .find(|l| l.number == num)
            .map(|l| l.url.clone())
    }
}
