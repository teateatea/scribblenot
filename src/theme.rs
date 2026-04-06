#![allow(dead_code)]
use iced::Color;

// --- Semantic color palette ---

/// Active/focused widget border and cursor text.
pub const ACTIVE: Color = Color { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };

/// Active color when rendered as a map-preview (not the live focus).
pub const ACTIVE_PREVIEW: Color = Color { r: 1.0, g: 0.647, b: 0.0, a: 1.0 };

/// Item has a value / is selected / completed.
pub const SELECTED: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };

/// Filled-field border in the header widget (slightly darker green).
pub const SELECTED_DARK: Color = Color { r: 0.0, g: 0.627, b: 0.0, a: 1.0 };

/// Navigation hint key labels.
pub const HINT: Color = Color { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };

/// Modal, search bar, and help-overlay accents.
pub const MODAL: Color = Color { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };

/// Inactive, unfocused, or disabled elements.
pub const MUTED: Color = Color { r: 0.333, g: 0.333, b: 0.333, a: 1.0 };

/// Error status messages.
pub const ERROR: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };

/// Previously-active return destination, now displaced by a more focused element.
pub const DISPLACED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };

#[cfg(test)]
mod tests {
    use iced::Color;

    // Verify that each semantic color constant exists and is an iced::Color.
    // These tests compile-fail until theme.rs defines all constants as iced::Color.

    #[test]
    fn active_is_iced_color() {
        let _c: Color = super::ACTIVE;
    }

    #[test]
    fn selected_is_iced_color() {
        let _c: Color = super::SELECTED;
    }

    #[test]
    fn hint_is_iced_color() {
        let _c: Color = super::HINT;
    }

    #[test]
    fn modal_is_iced_color() {
        let _c: Color = super::MODAL;
    }

    #[test]
    fn muted_is_iced_color() {
        let _c: Color = super::MUTED;
    }

    #[test]
    fn error_is_iced_color() {
        let _c: Color = super::ERROR;
    }

    #[test]
    fn displaced_is_iced_color() {
        let _c: Color = super::DISPLACED;
    }
}
