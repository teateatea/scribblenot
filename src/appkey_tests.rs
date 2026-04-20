// Tests for AppKey enum and its conversion from iced keyboard events.
// These tests FAIL at compile time until AppKey and appkey_from_iced are implemented in app.rs.
//
// Sub-task #2 TDD anchor: verifies that:
//   1. AppKey variants map correctly from iced keyboard Key + Modifiers
//   2. The conversion function covers all binding types used in match_binding_str

#[cfg(test)]
mod appkey_conversion_tests {
    use crate::app::{appkey_from_iced, AppKey};
    use iced::keyboard::key::Named;
    use iced::keyboard::{Key, Modifiers};

    // Helper: convert a Named key with no modifiers
    fn named(n: Named) -> AppKey {
        appkey_from_iced(Key::Named(n), Modifiers::empty())
    }

    // Helper: convert a character key with no modifiers
    fn ch(c: char) -> AppKey {
        appkey_from_iced(Key::Character(c.to_string().into()), Modifiers::empty())
    }

    #[test]
    fn enter_without_modifiers_maps_to_enter() {
        assert_eq!(
            named(Named::Enter),
            AppKey::Enter,
            "Named::Enter with no modifiers should map to AppKey::Enter"
        );
    }

    #[test]
    fn shift_enter_maps_to_shift_enter() {
        let result = appkey_from_iced(Key::Named(Named::Enter), Modifiers::SHIFT);
        assert_eq!(
            result,
            AppKey::ShiftEnter,
            "Named::Enter with SHIFT modifier should map to AppKey::ShiftEnter"
        );
    }

    #[test]
    fn escape_maps_to_esc() {
        assert_eq!(
            named(Named::Escape),
            AppKey::Esc,
            "Named::Escape should map to AppKey::Esc"
        );
    }

    #[test]
    fn backspace_maps_to_backspace() {
        assert_eq!(
            named(Named::Backspace),
            AppKey::Backspace,
            "Named::Backspace should map to AppKey::Backspace"
        );
    }

    #[test]
    fn arrow_keys_map_correctly() {
        assert_eq!(named(Named::ArrowDown), AppKey::Down, "ArrowDown -> Down");
        assert_eq!(named(Named::ArrowUp), AppKey::Up, "ArrowUp -> Up");
        assert_eq!(named(Named::ArrowLeft), AppKey::Left, "ArrowLeft -> Left");
        assert_eq!(
            named(Named::ArrowRight),
            AppKey::Right,
            "ArrowRight -> Right"
        );
    }

    #[test]
    fn space_character_maps_to_space() {
        assert_eq!(
            ch(' '),
            AppKey::Space,
            "Key::Character(' ') should map to AppKey::Space"
        );
    }

    #[test]
    fn regular_char_maps_to_char_variant() {
        assert_eq!(
            ch('q'),
            AppKey::Char('q'),
            "Key::Character('q') should map to AppKey::Char('q')"
        );
    }

    #[test]
    fn ctrl_c_maps_to_ctrl_c() {
        let result = appkey_from_iced(Key::Character("c".into()), Modifiers::CTRL);
        assert_eq!(
            result,
            AppKey::CtrlC,
            "Key::Character('c') with CTRL should map to AppKey::CtrlC"
        );
    }

    #[test]
    fn shift_letter_maps_to_uppercase_char() {
        let result = appkey_from_iced(Key::Character("a".into()), Modifiers::SHIFT);
        assert_eq!(
            result,
            AppKey::Char('A'),
            "shifted letter input should preserve capitalization"
        );
    }

    #[test]
    fn shift_number_maps_to_shifted_symbol() {
        let result = appkey_from_iced(Key::Character("1".into()), Modifiers::SHIFT);
        assert_eq!(
            result,
            AppKey::Char('!'),
            "shifted number input should preserve symbol text entry"
        );
    }

    #[test]
    fn ctrl_uppercase_letter_normalizes_to_lowercase_ctrl_char() {
        let result =
            appkey_from_iced(Key::Character("R".into()), Modifiers::CTRL | Modifiers::SHIFT);
        assert_eq!(
            result,
            AppKey::CtrlChar('r'),
            "ctrl-modified letters should normalize to lowercase command bindings"
        );
    }
}
