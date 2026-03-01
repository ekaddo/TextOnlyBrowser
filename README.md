# Lightning Text Only Browser

A fast, distraction-free desktop browser for reading the news. Built in Rust with a native GUI — no images, no ads, no JavaScript. Just text and clickable links.

![Lightning Text Only Browser](https://img.shields.io/badge/built%20with-Rust-orange)

---

## Features

- **Text only** — strips all HTML down to readable paragraphs and numbered hyperlinks
- **Mouse-clickable links** — click any `[N] link` to navigate
- **Fast** — no rendering engine, no layout engine, just text
- **Native desktop window** — built with [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- **Back / Forward navigation** with toolbar buttons and `Alt+←` / `Alt+→`
- **Configurable home page** via config file or environment variable
- **HTTPS** with system certificate store (no configuration needed)
- **Compressed responses** — gzip, deflate, brotli all handled automatically

---

## Screenshots

```
┌─ Lightning Text Only Browser ──────────────────────────────────────┐
│ [←] [→]  https://text.npr.org                              [Go]    │
├────────────────────────────────────────────────────────────────────┤
│  NPR : National Public Radio                                       │
│  ────────────────────────────                                      │
│                                                                    │
│  [1] White House signs order directing agencies to cut staff       │
│  [2] Senate confirms new defense secretary in party-line vote      │
│  [3] Scientists find new clues about early solar system formation  │
│                                                                    │
│  3 links — click or use ← → to navigate                           │
└────────────────────────────────────────────────────────────────────┘
```

---

## Installation

### Prerequisites

- **Rust** (1.70 or later) — install from [rustup.rs](https://rustup.rs) or via:
  ```
  winget install Rustlang.Rustup
  ```
  After installing, open a **new terminal** so `cargo` is in your PATH.

### Build from source

```bash
git clone https://github.com/yourname/TextOnlyBrowser.git
cd TextOnlyBrowser
cargo build --release
```

The binary is at `target/release/tob.exe` (Windows) or `target/release/tob` (Linux/macOS).

### Run

```bash
./target/release/tob.exe
```

Or add the binary to your PATH and just type `tob`.

---

## Usage

| Action | How |
|--------|-----|
| Follow a link | Click it, or type its number (1–9) in the URL bar |
| Go back | Click `←` button or press `Alt+←` |
| Go forward | Click `→` button or press `Alt+→` |
| Navigate to URL | Click the address bar, type URL, press `Enter` or click `Go` |
| Reload | Press `Enter` in the address bar with the current URL |

---

## Configuration

### Config file

Create the file at the appropriate path for your OS:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\textbrowser\config.toml` |
| Linux | `~/.config/textbrowser/config.toml` |
| macOS | `~/Library/Application Support/textbrowser/config.toml` |

```toml
# Home page loaded on startup
home = "https://text.npr.org"

# Request timeout in seconds
timeout_secs = 15

# Maximum HTTP redirects to follow
max_redirects = 10
```

### Environment variable

Override the home page for a single session:

```bash
TEXTBROWSER_HOME=https://news.ycombinator.com ./target/release/tob.exe
```

---

## Good sites to read

These sites work especially well in a text-only browser:

- `https://text.npr.org` — NPR text edition (default)
- `https://lite.cnn.com` — CNN lite
- `https://news.ycombinator.com` — Hacker News
- `https://lobste.rs` — tech news
- `https://old.reddit.com` — Reddit classic view
- `https://en.m.wikipedia.org` — Wikipedia mobile (cleaner HTML)

---

## License

See [LICENSE](LICENSE).
