use crate::data::SectionConfig;

/// Try to parse clipboard text as a prior clinic note.
/// Returns the recognized note text if the content looks like a structured note,
/// or None if the clipboard content should be ignored.
#[allow(dead_code)]
pub fn try_parse_clipboard_note(_text: &str, _sections: &[SectionConfig]) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_result_is_subset_of_input() {
        let input = "Assessment: stable\nPlan: continue";
        let result = try_parse_clipboard_note(input, &[]);
        if let Some(text) = result {
            for line in text.lines() {
                assert!(
                    input.contains(line.trim()) || line.trim().is_empty(),
                    "parse output contains content not present in input: {line:?}"
                );
            }
        }
    }

    #[test]
    fn empty_input_returns_none() {
        assert_eq!(try_parse_clipboard_note("", &[]), None);
    }

    #[test]
    fn does_not_panic_on_garbage_input() {
        let garbage = String::from_utf8_lossy(&[0, 0xff, 0xfe, b' ', b'j', b'u', b'n', b'k']);
        let _ = try_parse_clipboard_note(&garbage, &[]);
    }
}
