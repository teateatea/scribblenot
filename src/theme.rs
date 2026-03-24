#![allow(dead_code)]
use ratatui::style::{Color, Modifier, Style};

// --- Semantic color palette ---

/// Active/focused widget border and cursor text.
pub const ACTIVE: Color = Color::Yellow;

/// Active color when rendered as a map-preview (not the live focus).
pub const ACTIVE_PREVIEW: Color = Color::Rgb(255, 165, 0);

/// Item has a value / is selected / completed.
pub const SELECTED: Color = Color::Green;

/// Filled-field border in the header widget (slightly darker green).
pub const SELECTED_DARK: Color = Color::Rgb(0, 160, 0);

/// Navigation hint key labels.
pub const HINT: Color = Color::Magenta;

/// Modal, search bar, and help-overlay accents.
pub const MODAL: Color = Color::Cyan;

/// Inactive, unfocused, or disabled elements.
pub const MUTED: Color = Color::DarkGray;

/// Error status messages.
pub const ERROR: Color = Color::Red;

/// Previously-active return destination, now displaced by a more focused element.
pub const DISPLACED: Color = Color::Red;

// --- Composed style helpers ---

pub fn active() -> Style {
    Style::default().fg(ACTIVE)
}

pub fn active_bold() -> Style {
    Style::default().fg(ACTIVE).add_modifier(Modifier::BOLD)
}

pub fn active_preview() -> Style {
    Style::default().fg(ACTIVE_PREVIEW)
}

pub fn active_preview_bold() -> Style {
    Style::default().fg(ACTIVE_PREVIEW).add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::default().fg(SELECTED)
}

pub fn selected_dark() -> Style {
    Style::default().fg(SELECTED_DARK)
}

pub fn hint() -> Style {
    Style::default().fg(HINT)
}

pub fn modal() -> Style {
    Style::default().fg(MODAL)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

pub fn muted_bold() -> Style {
    Style::default().fg(MUTED).add_modifier(Modifier::BOLD)
}

pub fn error() -> Style {
    Style::default().fg(ERROR)
}

pub fn displaced() -> Style {
    Style::default().fg(DISPLACED)
}

pub fn displaced_bold() -> Style {
    Style::default().fg(DISPLACED).add_modifier(Modifier::BOLD)
}

pub fn dim() -> Style {
    Style::default().add_modifier(Modifier::DIM)
}

pub fn bold() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}
