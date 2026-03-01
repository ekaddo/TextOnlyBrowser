use scraper::{ElementRef, Html, Node};
use url::Url;

/// A link extracted from the page.
#[derive(Debug, Clone)]
pub struct PageLink {
    pub number: usize,
    pub url: String,
    pub label: String,
}

/// An inline segment within a paragraph.
#[derive(Debug, Clone)]
pub enum Segment {
    Text(String),
    Link { number: usize, text: String },
}

/// Complete parsed representation of a page.
#[derive(Debug, Clone)]
pub struct RenderedPage {
    pub title: String,
    pub url: String,
    pub paragraphs: Vec<Vec<Segment>>,
    pub links: Vec<PageLink>,
}

const BLOCK_TAGS: &[&str] = &[
    "div", "p", "h1", "h2", "h3", "h4", "h5", "h6", "article", "section", "main",
    "header", "footer", "nav", "ul", "ol", "li", "blockquote", "pre", "table", "tr",
    "td", "th", "figure", "figcaption", "details", "summary", "aside",
];

const DISCARD_TAGS: &[&str] = &[
    "script", "style", "noscript", "svg", "head", "meta", "link", "button", "input",
    "select", "textarea", "form", "iframe", "embed", "object",
];

struct Parser<'a> {
    base_url: Option<Url>,
    links: Vec<PageLink>,
    paragraphs: Vec<Vec<Segment>>,
    current: Vec<Segment>,
    in_pre: bool,
    title: String,
    html: &'a Html,
}

impl<'a> Parser<'a> {
    fn new(url: &str, html: &'a Html) -> Self {
        let base_url = Url::parse(url).ok();
        Self {
            base_url,
            links: Vec::new(),
            paragraphs: Vec::new(),
            current: Vec::new(),
            in_pre: false,
            title: String::new(),
            html,
        }
    }

    fn flush_paragraph(&mut self) {
        let para = std::mem::take(&mut self.current);
        // Trim leading/trailing whitespace-only Text segments
        let para: Vec<Segment> = para
            .into_iter()
            .filter(|s| match s {
                Segment::Text(t) => !t.trim().is_empty(),
                Segment::Link { text, .. } => !text.trim().is_empty(),
            })
            .collect();
        if !para.is_empty() {
            self.paragraphs.push(para);
        }
    }

    fn push_text(&mut self, text: &str) {
        let text = if self.in_pre {
            text.to_string()
        } else {
            // Collapse runs of whitespace to a single space
            let collapsed: String = text
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            collapsed
        };
        if text.is_empty() {
            return;
        }
        if let Some(Segment::Text(last)) = self.current.last_mut() {
            if !last.ends_with(' ') && !text.starts_with(' ') {
                last.push(' ');
            }
            last.push_str(&text);
        } else {
            self.current.push(Segment::Text(text));
        }
    }

    fn resolve_url(&self, href: &str) -> Option<String> {
        if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
            return None;
        }
        if let Some(base) = &self.base_url {
            base.join(href).ok().map(|u| u.to_string())
        } else {
            Some(href.to_string())
        }
    }

    fn walk(&mut self, node: ElementRef<'a>) {
        let tag = node.value().name().to_ascii_lowercase();

        if DISCARD_TAGS.contains(&tag.as_str()) {
            return;
        }

        // Capture title text
        if tag == "title" {
            self.title = node.text().collect::<String>().trim().to_string();
            return;
        }

        let is_pre = tag == "pre" || tag == "code";
        let was_pre = self.in_pre;
        if is_pre {
            self.in_pre = true;
        }

        let is_block = BLOCK_TAGS.contains(&tag.as_str());
        let heading_level = match tag.as_str() {
            "h1" => Some(1usize),
            "h2" => Some(2),
            "h3" => Some(3),
            "h4" => Some(4),
            "h5" => Some(5),
            "h6" => Some(6),
            _ => None,
        };

        if is_block {
            self.flush_paragraph();
        }

        // For headings, add a prefix marker
        if let Some(level) = heading_level {
            let prefix = "#".repeat(level) + " ";
            self.current.push(Segment::Text(prefix));
        }

        // For list items, add bullet
        if tag == "li" {
            self.current.push(Segment::Text("• ".to_string()));
        }

        // For anchor tags, collect children as a link
        if tag == "a" {
            let href = node.value().attr("href").unwrap_or("");
            if let Some(resolved) = self.resolve_url(href) {
                let link_text: String = node
                    .text()
                    .collect::<String>()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                if !link_text.is_empty() {
                    let number = self.links.len() + 1;
                    self.links.push(PageLink {
                        number,
                        url: resolved,
                        label: link_text.clone(),
                    });
                    self.current.push(Segment::Link {
                        number,
                        text: link_text,
                    });
                }
                // Don't recurse into <a> children — already captured text above
                if is_block {
                    self.flush_paragraph();
                }
                if is_pre {
                    self.in_pre = was_pre;
                }
                return;
            }
        }

        // Handle <hr>
        if tag == "hr" {
            self.flush_paragraph();
            self.paragraphs.push(vec![Segment::Text(
                "────────────────────────────────────────".to_string(),
            )]);
            return;
        }

        // Walk children
        for child in node.children() {
            match child.value() {
                Node::Text(text) => {
                    self.push_text(text);
                }
                Node::Element(_) => {
                    if let Some(elem) = ElementRef::wrap(child) {
                        self.walk(elem);
                    }
                }
                _ => {}
            }
        }

        if is_block {
            self.flush_paragraph();
        }

        if is_pre {
            self.in_pre = was_pre;
        }
    }
}

pub fn parse(url: &str, html: &str) -> RenderedPage {
    let document = Html::parse_document(html);
    let mut parser = Parser::new(url, &document);

    // Find body element, fall back to document root
    let body_sel = scraper::Selector::parse("body").unwrap();
    let start_nodes: Vec<ElementRef> = document.select(&body_sel).collect();

    if start_nodes.is_empty() {
        // No body — walk entire document
        let root_sel = scraper::Selector::parse("html").unwrap();
        for elem in document.select(&root_sel) {
            parser.walk(elem);
        }
    } else {
        for body in start_nodes {
            parser.walk(body);
        }
    }

    parser.flush_paragraph();

    // Extract title if not found via <title> tag
    if parser.title.is_empty() {
        let title_sel = scraper::Selector::parse("title").unwrap();
        if let Some(elem) = document.select(&title_sel).next() {
            parser.title = elem.text().collect::<String>().trim().to_string();
        }
    }

    RenderedPage {
        title: parser.title,
        url: url.to_string(),
        paragraphs: parser.paragraphs,
        links: parser.links,
    }
}
