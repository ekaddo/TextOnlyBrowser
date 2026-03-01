use crossterm::event::{Event, KeyCode, KeyModifiers};

use crate::app::Mode;

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    TabNext,
    TabPrev,
    FollowSelected,
    GoBack,
    GoForward,
    OpenUrlBar,
    UrlInputChar(char),
    UrlInputBackspace,
    UrlInputSubmit,
    UrlInputCancel,
    Resize(u16, u16),
    None,
}

pub fn map_event(event: Event, mode: &Mode) -> Action {
    match event {
        Event::Resize(w, h) => Action::Resize(w, h),
        Event::Key(key) => match mode {
            Mode::Browse => match key.code {
                KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    Action::Quit
                }
                KeyCode::Up | KeyCode::Char('k') => Action::ScrollUp,
                KeyCode::Down | KeyCode::Char('j') => Action::ScrollDown,
                KeyCode::PageUp => Action::PageUp,
                KeyCode::PageDown => Action::PageDown,
                KeyCode::Tab => Action::TabNext,
                KeyCode::BackTab => Action::TabPrev,
                KeyCode::Enter => Action::FollowSelected,
                KeyCode::Char('b') | KeyCode::Backspace => Action::GoBack,
                KeyCode::Char('f') => Action::GoForward,
                KeyCode::Char('u') | KeyCode::Char('g') => Action::OpenUrlBar,
                _ => Action::None,
            },
            Mode::UrlEntry => match key.code {
                KeyCode::Esc => Action::UrlInputCancel,
                KeyCode::Enter => Action::UrlInputSubmit,
                KeyCode::Backspace => Action::UrlInputBackspace,
                KeyCode::Char(c) => Action::UrlInputChar(c),
                _ => Action::None,
            },
        },
        _ => Action::None,
    }
}
