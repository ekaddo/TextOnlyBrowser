mod app;
mod browser;
mod config;
mod fetcher;
mod history;
mod parser;
mod ui;

fn main() {
    let cfg = config::load_config();
    let home_url = cfg.home.clone();

    let client = fetcher::build_client(cfg.timeout_secs, cfg.max_redirects)
        .expect("failed to build HTTP client");

    // Create a tokio runtime — eframe owns the event loop so we can't use #[tokio::main].
    // The runtime lives until main() returns (after the window closes).
    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
    let rt_handle = rt.handle().clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Lightning Text Only Browser")
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lightning Text Only Browser",
        options,
        Box::new(move |_cc| {
            let mut browser_app = app::BrowserApp::new(rt_handle, cfg);
            browser::navigate_to(&mut browser_app, home_url, client.clone());
            Ok(Box::new(ui::BrowserUi::new(browser_app, client)))
        }),
    )
    .expect("eframe failed");
}
