# TextOnlyBrowser — Project Management

## Overview
A speedy, text-only terminal browser for reading the news. Built in Rust with a full TUI.

---

## Implementation Status

### Done

- [x] **Cargo.toml** — all dependencies defined
- [x] **src/config.rs** — loads `config.toml` from platform config dir, overridden by `TEXTBROWSER_HOME` env var
- [x] **src/history.rs** — back/forward navigation stacks
- [x] **src/fetcher.rs** — async HTTP client (reqwest), Content-Type validation, redirect handling
- [x] **src/parser.rs** — walks HTML DOM via scraper, extracts text paragraphs and numbered hyperlinks
- [x] **src/renderer.rs** — word-wraps paragraphs into ratatui `Line`/`Span` structs with link styling
- [x] **src/input.rs** — maps crossterm key events to `Action` enum, mode-aware (Browse / UrlEntry)
- [x] **src/app.rs** — central `App` state: page, scroll, selection, mode, mpsc fetch channel
- [x] **src/browser.rs** — `navigate_to()`, `go_back()`, `go_forward()`, `handle_fetch_result()`
- [x] **src/ui.rs** — ratatui layout: URL bar / scrollable content / status bar
- [x] **src/main.rs** — tokio entry point, terminal setup, panic hook, `tokio::select!` event loop

---

## Todos

### Must-Have (before first use)
- [ ] **Install Rust** — not yet installed on this machine
  - `winget install Rustlang.Rustup` then open a new terminal
- [ ] **First build** — `cargo build --release`
- [ ] **Smoke test** — run `./target/release/tob.exe`, verify HN loads

### Bugs / Known Gaps
- [ ] Parser: `<a>` tags inside block elements may double-flush paragraph — needs testing on real pages
- [ ] Renderer: very long unbreakable words (URLs in body text) overflow the column width
- [ ] History: `go_back()` calls `history.back()` which pops current, then `navigate_to_no_push` — verify the stack doesn't drift after several back/forward cycles
- [ ] URL bar cursor is estimated with `url_input.len()` (bytes) — breaks for multi-byte input

### Nice-to-Have
- [ ] `Home` / `End` keys to jump to top / bottom of page
- [ ] Link number direct jump — type `[42]` to jump to link 42
- [ ] Page search — `/` to open a find bar, `n`/`N` to cycle matches
- [ ] Download non-HTML files (PDF, images) rather than showing an error
- [ ] Viewport scroll-to-selected-link when Tab cycles past the visible area
- [ ] Config: font/color theme option
- [ ] Config: `max_content_bytes` to cap huge pages
- [ ] Bookmarks file (`~/.config/textbrowser/bookmarks.toml`)
- [ ] Status bar: show page title in addition to URL

### Future / Stretch
- [ ] Cookie jar (reqwest cookie_store feature) for sites that require session
- [ ] Basic form support (GET forms only — search boxes)
- [ ] Mouse click support for links
- [ ] Gopher / Gemini protocol support

---

## Architecture Quick Reference

```
src/
  main.rs       tokio::main, terminal init, panic hook, tokio::select! loop
  app.rs        App struct — owns all state + mpsc channel (tx/rx)
  browser.rs    navigate_to / go_back / go_forward / handle_fetch_result
  config.rs     Config { home, timeout_secs, max_redirects } + load_config()
  fetcher.rs    build_client() / async fetch() → (final_url, html)
  parser.rs     parse(url, html) → RenderedPage { paragraphs, links, title }
  renderer.rs   render(page, width, selected) → Vec<Line<'static>>
  ui.rs         draw(frame, app) — URL bar / content / status bar
  input.rs      map_event(event, mode) → Action
  history.rs    History { back, forward, current }
```

### Data Flow
```
User types URL
  → input::map_event → Action::UrlInputSubmit
  → browser::navigate_to → tokio::spawn(fetch + parse)
  → FetchResult sent over mpsc channel
  → handle_fetch_result → renderer::render → app.rendered_lines
  → ui::draw reads app.rendered_lines + app.scroll → terminal
```

### Config File Location

| Platform | Path |
|----------|------|
| Windows  | `%APPDATA%\textbrowser\config.toml` |
| Linux    | `~/.config/textbrowser/config.toml` |
| macOS    | `~/Library/Application Support/textbrowser/config.toml` |

```toml
home         = "https://news.ycombinator.com"
timeout_secs = 15
max_redirects = 10
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `Page Up` | Scroll up one page |
| `Page Down` | Scroll down one page |
| `Tab` | Select next link |
| `Shift+Tab` | Select previous link |
| `Enter` | Follow selected link |
| `b` / `Backspace` | Go back |
| `f` | Go forward |
| `u` / `g` | Open URL bar |
| `Esc` | Cancel URL entry |
| `q` / `Ctrl+C` | Quit |

---

## Build & Run

```bash
# First time
cargo build --release

# Run (opens home page from config)
./target/release/tob.exe

# Override home page for one session
TEXTBROWSER_HOME=https://lobste.rs ./target/release/tob.exe
```
