use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

use crate::parser::{RenderedPage, Segment};

pub fn render(
    page: &RenderedPage,
    width: u16,
    selected_link: Option<usize>,
) -> Vec<Line<'static>> {
    let max_width = width.saturating_sub(2) as usize; // leave a small margin
    let max_width = max_width.max(20);

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Page title as a styled header
    if !page.title.is_empty() {
        let title_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(Span::styled(page.title.clone(), title_style)));
        lines.push(Line::from(""));
    }

    for para in &page.paragraphs {
        // Check if paragraph starts with a heading prefix
        let is_heading = matches!(para.first(), Some(Segment::Text(t)) if t.starts_with('#'));
        let is_hr = matches!(
            para.first(),
            Some(Segment::Text(t)) if t.starts_with('─')
        );

        if is_hr {
            // Render as a separator spanning the width
            let sep: String = "─".repeat(max_width);
            lines.push(Line::from(Span::styled(
                sep,
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            continue;
        }

        let heading_style = if is_heading {
            Some(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            None
        };

        // Word-wrap the paragraph into lines of Spans
        let para_lines = wrap_paragraph(para, max_width, selected_link, heading_style);
        lines.extend(para_lines);
        lines.push(Line::from("")); // blank line between paragraphs
    }

    lines
}

fn wrap_paragraph(
    para: &[Segment],
    max_width: usize,
    selected_link: Option<usize>,
    heading_style: Option<Style>,
) -> Vec<Line<'static>> {
    let mut output_lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_width: usize = 0;

    let flush = |spans: &mut Vec<Span<'static>>, out: &mut Vec<Line<'static>>| {
        if !spans.is_empty() {
            out.push(Line::from(std::mem::take(spans)));
        }
    };

    for segment in para {
        match segment {
            Segment::Text(text) => {
                let style = heading_style.unwrap_or_default();
                let words: Vec<&str> = text.split_whitespace().collect();
                for word in words {
                    let word_w = UnicodeWidthStr::width(word);
                    let space_needed = if current_width > 0 { 1 } else { 0 };

                    if current_width + space_needed + word_w > max_width && current_width > 0 {
                        flush(&mut current_spans, &mut output_lines);
                        current_width = 0;
                    }

                    if current_width > 0 {
                        current_spans.push(Span::raw(" "));
                        current_width += 1;
                    }

                    current_spans.push(Span::styled(word.to_string(), style));
                    current_width += word_w;
                }
            }
            Segment::Link { number, text } => {
                let is_selected = selected_link == Some(*number);
                let text_style = if is_selected {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::UNDERLINED)
                };
                let marker_style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::DarkGray)
                };

                // Render as "[N] link text" — marker first, like w3m/lynx
                let marker = format!("[{}] ", number);
                let full_width = UnicodeWidthStr::width(marker.as_str())
                    + UnicodeWidthStr::width(text.as_str());

                let space_needed = if current_width > 0 { 1 } else { 0 };

                if current_width + space_needed + full_width > max_width && current_width > 0 {
                    flush(&mut current_spans, &mut output_lines);
                    current_width = 0;
                }

                if current_width > 0 {
                    current_spans.push(Span::raw(" "));
                    current_width += 1;
                }

                current_spans.push(Span::styled(marker.clone(), marker_style));
                current_spans.push(Span::styled(text.clone(), text_style));
                current_width += UnicodeWidthStr::width(marker.as_str())
                    + UnicodeWidthStr::width(text.as_str());
            }
        }
    }

    if !current_spans.is_empty() {
        output_lines.push(Line::from(current_spans));
    }

    output_lines
}
