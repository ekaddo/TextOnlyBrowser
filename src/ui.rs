use egui::{Color32, Context, Key, RichText, Sense, TopBottomPanel, Ui, Vec2};

use crate::{
    app::{BrowserApp, LoadState},
    browser,
    parser::Segment,
};

/// Wrapper that implements eframe::App, owning the BrowserApp state and HTTP client.
pub struct BrowserUi {
    inner: BrowserApp,
    client: reqwest::Client,
}

impl BrowserUi {
    pub fn new(inner: BrowserApp, client: reqwest::Client) -> Self {
        Self { inner, client }
    }
}

impl eframe::App for BrowserUi {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Apply theme every frame so toggling takes effect immediately
        if self.inner.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Pick up any completed fetch results
        self.inner.poll_fetch_results(ctx);

        // While loading, keep repainting so we notice when the result arrives
        if matches!(self.inner.load_state, LoadState::Loading(_)) {
            ctx.request_repaint();
        }

        // ── Top panel: URL bar ───────────────────────────────────────────────
        TopBottomPanel::top("url_bar").show(ctx, |ui| {
            self.draw_url_bar(ui, ctx);
        });

        // ── Bottom panel: status bar ─────────────────────────────────────────
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.draw_status_bar(ui);
        });

        // ── Central panel: page content ──────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_content(ui);
        });
    }
}

impl BrowserUi {
    fn draw_url_bar(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.horizontal(|ui| {
            // Back button
            let back_btn = ui.add_enabled(
                self.inner.history.can_go_back(),
                egui::Button::new("←"),
            );
            if back_btn.clicked() {
                browser::go_back(&mut self.inner, self.client.clone());
            }

            // Forward button
            let fwd_btn = ui.add_enabled(
                self.inner.history.can_go_forward(),
                egui::Button::new("→"),
            );
            if fwd_btn.clicked() {
                browser::go_forward(&mut self.inner, self.client.clone());
            }

            // URL text input — takes remaining width minus the Go button and theme toggle
            let go_width = 32.0;
            let toggle_width = 32.0;
            let available = ui.available_width() - go_width - toggle_width
                - 2.0 * ui.spacing().item_spacing.x;
            let url_edit = ui.add_sized(
                [available, ui.spacing().interact_size.y],
                egui::TextEdit::singleline(&mut self.inner.url_input)
                    .hint_text("Enter URL…"),
            );

            // Submit on Enter
            if url_edit.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter)) {
                self.submit_url();
            }

            // Go button
            if ui.button("Go").clicked() {
                self.submit_url();
            }

            // Dark / light mode toggle
            let theme_icon = if self.inner.dark_mode { "☀" } else { "🌙" };
            if ui.button(theme_icon).on_hover_text("Toggle dark/light mode").clicked() {
                self.inner.dark_mode = !self.inner.dark_mode;
            }

            // Alt+← / Alt+→ keyboard shortcuts (when URL bar is not focused)
            if !url_edit.has_focus() {
                if ctx.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowLeft)) {
                    browser::go_back(&mut self.inner, self.client.clone());
                }
                if ctx.input(|i| i.modifiers.alt && i.key_pressed(Key::ArrowRight)) {
                    browser::go_forward(&mut self.inner, self.client.clone());
                }
            }
        });
    }

    fn submit_url(&mut self) {
        let raw = self.inner.url_input.trim().to_string();
        if raw.is_empty() {
            return;
        }
        let url = if raw.contains("://") {
            raw
        } else {
            format!("https://{raw}")
        };
        browser::navigate_to(&mut self.inner, url, self.client.clone());
    }

    fn draw_status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            match &self.inner.load_state {
                LoadState::Loading(url) => {
                    ui.spinner();
                    ui.label(format!("Loading {url}…"));
                }
                LoadState::Error(msg) => {
                    ui.colored_label(Color32::RED, format!("Error: {msg}"));
                }
                LoadState::Idle => {
                    if let Some(page) = &self.inner.page {
                        let links = page.links.len();
                        if links > 0 {
                            ui.label(
                                RichText::new(format!("{links} links — click or use ← → to navigate"))
                                    .color(Color32::GRAY)
                                    .small(),
                            );
                        }
                    }
                }
            }
        });
    }

    fn link_color(&self) -> Color32 {
        if self.inner.dark_mode {
            Color32::from_rgb(100, 180, 255)
        } else {
            Color32::from_rgb(0, 90, 200)
        }
    }

    fn draw_content(&mut self, ui: &mut Ui) {
        // Collect any link URL the user clicks — resolved after the scroll area
        // borrow ends to avoid conflicting borrows on self.
        let mut navigate_to: Option<String> = None;
        let link_color = self.link_color();

        let mut scroll = egui::ScrollArea::vertical();
        if self.inner.scroll_to_top {
            self.inner.scroll_to_top = false;
            scroll = scroll.scroll_offset(Vec2::ZERO);
        }

        scroll.show(ui, |ui| {
            match &self.inner.load_state {
                LoadState::Loading(_) => {
                    ui.centered_and_justified(|ui| {
                        ui.spinner();
                    });
                    return;
                }
                LoadState::Error(msg) => {
                    ui.colored_label(Color32::RED, msg.clone());
                    return;
                }
                LoadState::Idle => {}
            }

            if let Some(page) = &self.inner.page {
                // Title
                ui.heading(RichText::new(&page.title).strong());
                ui.separator();
                ui.add_space(4.0);

                for para in &page.paragraphs {
                    // Check for a horizontal rule
                    if matches!(para.first(), Some(Segment::Text(t)) if t.starts_with('─')) {
                        ui.separator();
                        continue;
                    }

                    ui.horizontal_wrapped(|ui| {
                        // Tighten spacing so words flow naturally
                        ui.spacing_mut().item_spacing.x = 4.0;

                        for segment in para {
                            match segment {
                                Segment::Text(text) => {
                                    // Detect heading prefix (# ## ###)
                                    let trimmed = text.trim_start_matches('#');
                                    let hashes = text.len() - trimmed.len();
                                    if hashes > 0 && text.starts_with('#') {
                                        let size = match hashes {
                                            1 => 32.0,
                                            2 => 28.0,
                                            _ => 24.0,
                                        };
                                        let label_text = trimmed.trim();
                                        if !label_text.is_empty() {
                                            ui.label(
                                                RichText::new(label_text)
                                                    .strong()
                                                    .size(size),
                                            );
                                        }
                                    } else {
                                        // Emit word by word so horizontal_wrapped can wrap
                                        for word in text.split_whitespace() {
                                            ui.label(RichText::new(word).size(24.0));
                                        }
                                    }
                                }
                                Segment::Link { number, text } => {
                                    let label = format!("[{number}] {text}");
                                    let resp = ui.add(
                                        egui::Label::new(
                                            RichText::new(label)
                                                .color(link_color)
                                                .underline()
                                                .size(24.0),
                                        )
                                        .sense(Sense::click()),
                                    );
                                    if let Some(link) =
                                        page.links.iter().find(|l| l.number == *number)
                                    {
                                        let resp = resp.on_hover_text(link.url.clone());
                                        if resp.clicked() {
                                            navigate_to = Some(link.url.clone());
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Enter a URL above to start browsing.");
                });
            }
        });

        // Navigate after the scroll area borrow ends
        if let Some(url) = navigate_to {
            browser::navigate_to(&mut self.inner, url, self.client.clone());
        }
    }
}
