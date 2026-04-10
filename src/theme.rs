#![allow(dead_code)]
use anyhow::{anyhow, Context, Result};
use iced::font::{Family, Style, Weight};
use iced::{Color, Font};
use serde::{Deserialize, Deserializer};
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

// --- Semantic color palette ---

/// Active/focused widget border and cursor text.
pub const ACTIVE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

/// Active color when rendered as a map-preview (not the live focus).
pub const ACTIVE_PREVIEW: Color = Color {
    r: 1.0,
    g: 0.647,
    b: 0.0,
    a: 1.0,
};

/// Item has a value / is selected / completed.
pub const SELECTED: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};

/// Filled-field border in the header widget (slightly darker green).
pub const SELECTED_DARK: Color = Color {
    r: 0.0,
    g: 0.627,
    b: 0.0,
    a: 1.0,
};

/// Navigation hint key labels.
pub const HINT: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

/// Modal, search bar, and help-overlay accents.
pub const MODAL: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

/// Inactive, unfocused, or disabled elements.
pub const MUTED: Color = Color {
    r: 0.333,
    g: 0.333,
    b: 0.333,
    a: 1.0,
};

/// Error status messages.
pub const ERROR: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// Previously-active return destination, now displaced by a more focused element.
pub const DISPLACED: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const TEXT: Color = Color {
    r: 0.9,
    g: 0.9,
    b: 0.9,
    a: 1.0,
};

pub const BACKGROUND: Color = Color {
    r: 0.06,
    g: 0.07,
    b: 0.09,
    a: 1.0,
};

pub const PANE_BACKGROUND: Color = Color {
    r: 0.08,
    g: 0.09,
    b: 0.12,
    a: 1.0,
};

pub const MODAL_BACKGROUND: Color = Color {
    r: 0.02,
    g: 0.04,
    b: 0.06,
    a: 1.0,
};

pub const STATUS_BACKGROUND: Color = Color {
    r: 0.04,
    g: 0.05,
    b: 0.07,
    a: 1.0,
};

pub const STICKY_DEFAULT_PREVIEW: Color = Color {
    r: 0.50,
    g: 0.75,
    b: 0.85,
    a: 1.0,
};

pub const CONFIRMED_MUTED_PREVIEW: Color = Color {
    r: 0.31,
    g: 0.69,
    b: 0.31,
    a: 1.0,
};

pub const PREVIEW_COPY_FLASH_BACKGROUND: Color = Color {
    r: 0.10,
    g: 0.35,
    b: 0.18,
    a: 1.0,
};

pub const TEXT_COLOR_FLASH: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const TRANSPARENT: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

pub const SCROLL_RAIL: Color = Color {
    r: 0.10,
    g: 0.16,
    b: 0.20,
    a: 1.0,
};

pub const SCROLL_SCROLLER: Color = Color {
    r: 0.22,
    g: 0.35,
    b: 0.44,
    a: 1.0,
};

#[derive(Debug, Clone)]
pub struct AppTheme {
    pub active: Color,
    pub active_preview: Color,
    pub selected: Color,
    pub selected_dark: Color,
    pub hint: Color,
    pub hint_prefix: Color,
    pub modal: Color,
    pub muted: Color,
    pub error: Color,
    pub displaced: Color,
    pub text: Color,
    pub background: Color,
    pub pane_background: Color,
    pub pane_active_background: Color,
    pub pane_inactive_background: Color,
    pub pane_active_border: Color,
    pub pane_inactive_border: Color,
    pub pane_border_width: f32,
    pub pane_buffer_width: f32,
    pub modal_background: Color,
    pub modal_panel_background: Color,
    pub modal_item_background: Color,
    pub modal_item_hovered_background: Color,
    pub modal_text: Color,
    pub modal_selected_text: Color,
    pub modal_muted_text: Color,
    pub modal_hint_text: Color,
    pub modal_input_background: Color,
    pub modal_input_text: Color,
    pub modal_input_placeholder: Color,
    pub modal_input_border: Color,
    pub sticky_default_preview: Color,
    pub confirmed_muted_preview: Color,
    pub preview_copy_flash_background: Color,
    pub text_color_flash: Color,
    pub status_background: Color,
    pub scroll_rail: Color,
    pub scroll_scroller: Color,
    pub scroll_rail_hovered: Color,
    pub scroll_scroller_hovered: Color,
    pub scroll_rail_dragged: Color,
    pub scroll_scroller_dragged: Color,
    pub scroll_gap: Color,
    pub scroll_border_width: f32,
    pub scroll_width: f32,
    pub scroll_spacing: f32,
    pub font_pane: Font,
    pub font_heading: Font,
    pub font_preview: Font,
    pub font_modal: Font,
    pub font_status: Font,
    pub preview_copy_flash_duration_ms: u64,
    pub text_color_flash_duration: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ThemeFile {
    custom_colors: Option<HashMap<String, String>>,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    active: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    active_preview: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    selected: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    selected_dark: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    hint: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    hint_prefix: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    muted: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    error: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    displaced: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    text: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    pane_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    pane_active_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    pane_inactive_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    pane_active_border: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    pane_inactive_border: ColorSetting,
    pane_border_width: Option<f32>,
    pane_buffer_width: Option<f32>,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_panel_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_item_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_item_hovered_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_text: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_selected_text: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_muted_text: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_hint_text: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_input_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_input_text: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_input_placeholder: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    modal_input_border: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    sticky_default_preview: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    confirmed_muted_preview: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    preview_copy_flash_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    text_color_flash: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    status_background: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    scroll_rail: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    scroll_scroller: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    scroll_rail_hovered: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    scroll_scroller_hovered: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    scroll_rail_dragged: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    scroll_scroller_dragged: ColorSetting,
    #[serde(default, deserialize_with = "deserialize_color_setting")]
    scroll_gap: ColorSetting,
    scroll_border_width: Option<f32>,
    scroll_width: Option<f32>,
    scroll_spacing: Option<f32>,
    font_pane: Option<String>,
    font_heading: Option<String>,
    font_preview: Option<String>,
    font_modal: Option<String>,
    font_status: Option<String>,
    preview_copy_flash_duration_ms: Option<u64>,
    text_color_flash_duration: Option<u64>,
}

#[derive(Debug, Clone, Default)]
enum ColorSetting {
    #[default]
    Missing,
    Transparent,
    Value(Value),
}

impl Default for AppTheme {
    fn default() -> Self {
        Self {
            active: ACTIVE,
            active_preview: ACTIVE_PREVIEW,
            selected: SELECTED,
            selected_dark: SELECTED_DARK,
            hint: HINT,
            hint_prefix: MODAL,
            modal: MODAL,
            muted: MUTED,
            error: ERROR,
            displaced: DISPLACED,
            text: TEXT,
            background: BACKGROUND,
            pane_background: PANE_BACKGROUND,
            pane_active_background: PANE_BACKGROUND,
            pane_inactive_background: BACKGROUND,
            pane_active_border: ACTIVE_PREVIEW,
            pane_inactive_border: MUTED,
            pane_border_width: 1.0,
            pane_buffer_width: 8.0,
            modal_background: MODAL_BACKGROUND,
            modal_panel_background: MODAL_BACKGROUND,
            modal_item_background: TRANSPARENT,
            modal_item_hovered_background: SCROLL_RAIL,
            modal_text: TEXT,
            modal_selected_text: SELECTED,
            modal_muted_text: MUTED,
            modal_hint_text: HINT,
            modal_input_background: BACKGROUND,
            modal_input_text: TEXT,
            modal_input_placeholder: MUTED,
            modal_input_border: MODAL,
            sticky_default_preview: STICKY_DEFAULT_PREVIEW,
            confirmed_muted_preview: CONFIRMED_MUTED_PREVIEW,
            preview_copy_flash_background: PREVIEW_COPY_FLASH_BACKGROUND,
            text_color_flash: TEXT_COLOR_FLASH,
            status_background: STATUS_BACKGROUND,
            scroll_rail: SCROLL_RAIL,
            scroll_scroller: SCROLL_SCROLLER,
            scroll_rail_hovered: SCROLL_SCROLLER,
            scroll_scroller_hovered: MODAL,
            scroll_rail_dragged: SCROLL_SCROLLER,
            scroll_scroller_dragged: ACTIVE_PREVIEW,
            scroll_gap: BACKGROUND,
            scroll_border_width: 1.0,
            scroll_width: 10.0,
            scroll_spacing: 6.0,
            font_pane: Font::MONOSPACE,
            font_heading: Font {
                weight: Weight::Bold,
                ..Font::MONOSPACE
            },
            font_preview: Font::MONOSPACE,
            font_modal: Font::MONOSPACE,
            font_status: Font::MONOSPACE,
            preview_copy_flash_duration_ms: 650,
            text_color_flash_duration: 650,
        }
    }
}

impl AppTheme {
    pub fn load(data_dir: &Path, theme_name: &str) -> Result<Self> {
        let path = theme_path(data_dir, theme_name);
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read theme file {}", path.display()))?;
        let file: ThemeFile = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse theme file {}", path.display()))?;
        Self::from_file(file)
    }

    fn from_file(file: ThemeFile) -> Result<Self> {
        let default = Self::default();
        let custom_colors = custom_colors(file.custom_colors)?;
        let pane_background = optional_color(
            file.pane_background.clone(),
            default.pane_background,
            &custom_colors,
        )?;
        Ok(Self {
            active: optional_color(file.active, default.active, &custom_colors)?,
            active_preview: optional_color(
                file.active_preview,
                default.active_preview,
                &custom_colors,
            )?,
            selected: optional_color(file.selected, default.selected, &custom_colors)?,
            selected_dark: optional_color(
                file.selected_dark,
                default.selected_dark,
                &custom_colors,
            )?,
            hint: optional_color(file.hint, default.hint, &custom_colors)?,
            hint_prefix: optional_color(file.hint_prefix, default.hint_prefix, &custom_colors)?,
            modal: optional_color(file.modal, default.modal, &custom_colors)?,
            muted: optional_color(file.muted, default.muted, &custom_colors)?,
            error: optional_color(file.error, default.error, &custom_colors)?,
            displaced: optional_color(file.displaced, default.displaced, &custom_colors)?,
            text: optional_color(file.text, default.text, &custom_colors)?,
            background: optional_color(file.background, default.background, &custom_colors)?,
            pane_background,
            pane_active_background: optional_color(
                file.pane_active_background,
                pane_background,
                &custom_colors,
            )?,
            pane_inactive_background: optional_color(
                file.pane_inactive_background,
                pane_background,
                &custom_colors,
            )?,
            pane_active_border: optional_color(
                file.pane_active_border,
                default.pane_active_border,
                &custom_colors,
            )?,
            pane_inactive_border: optional_color(
                file.pane_inactive_border,
                default.pane_inactive_border,
                &custom_colors,
            )?,
            pane_border_width: nonnegative_number(
                file.pane_border_width,
                default.pane_border_width,
                "pane_border_width",
            )?,
            pane_buffer_width: nonnegative_number(
                file.pane_buffer_width,
                default.pane_buffer_width,
                "pane_buffer_width",
            )?,
            modal_background: optional_color(
                file.modal_background,
                default.modal_background,
                &custom_colors,
            )?,
            modal_panel_background: optional_color(
                file.modal_panel_background,
                default.modal_panel_background,
                &custom_colors,
            )?,
            modal_item_background: optional_color(
                file.modal_item_background,
                default.modal_item_background,
                &custom_colors,
            )?,
            modal_item_hovered_background: optional_color(
                file.modal_item_hovered_background,
                default.modal_item_hovered_background,
                &custom_colors,
            )?,
            modal_text: optional_color(file.modal_text, default.modal_text, &custom_colors)?,
            modal_selected_text: optional_color(
                file.modal_selected_text,
                default.modal_selected_text,
                &custom_colors,
            )?,
            modal_muted_text: optional_color(
                file.modal_muted_text,
                default.modal_muted_text,
                &custom_colors,
            )?,
            modal_hint_text: optional_color(
                file.modal_hint_text,
                default.modal_hint_text,
                &custom_colors,
            )?,
            modal_input_background: optional_color(
                file.modal_input_background,
                default.modal_input_background,
                &custom_colors,
            )?,
            modal_input_text: optional_color(
                file.modal_input_text,
                default.modal_input_text,
                &custom_colors,
            )?,
            modal_input_placeholder: optional_color(
                file.modal_input_placeholder,
                default.modal_input_placeholder,
                &custom_colors,
            )?,
            modal_input_border: optional_color(
                file.modal_input_border,
                default.modal_input_border,
                &custom_colors,
            )?,
            sticky_default_preview: optional_color(
                file.sticky_default_preview,
                default.sticky_default_preview,
                &custom_colors,
            )?,
            confirmed_muted_preview: optional_color(
                file.confirmed_muted_preview,
                default.confirmed_muted_preview,
                &custom_colors,
            )?,
            preview_copy_flash_background: optional_color(
                file.preview_copy_flash_background,
                default.preview_copy_flash_background,
                &custom_colors,
            )?,
            text_color_flash: optional_color(
                file.text_color_flash,
                default.text_color_flash,
                &custom_colors,
            )?,
            status_background: optional_color(
                file.status_background,
                default.status_background,
                &custom_colors,
            )?,
            scroll_rail: optional_color(file.scroll_rail, default.scroll_rail, &custom_colors)?,
            scroll_scroller: optional_color(
                file.scroll_scroller,
                default.scroll_scroller,
                &custom_colors,
            )?,
            scroll_rail_hovered: optional_color(
                file.scroll_rail_hovered,
                default.scroll_rail_hovered,
                &custom_colors,
            )?,
            scroll_scroller_hovered: optional_color(
                file.scroll_scroller_hovered,
                default.scroll_scroller_hovered,
                &custom_colors,
            )?,
            scroll_rail_dragged: optional_color(
                file.scroll_rail_dragged,
                default.scroll_rail_dragged,
                &custom_colors,
            )?,
            scroll_scroller_dragged: optional_color(
                file.scroll_scroller_dragged,
                default.scroll_scroller_dragged,
                &custom_colors,
            )?,
            scroll_gap: optional_color(file.scroll_gap, default.scroll_gap, &custom_colors)?,
            scroll_border_width: nonnegative_number(
                file.scroll_border_width,
                default.scroll_border_width,
                "scroll_border_width",
            )?,
            scroll_width: nonnegative_number(
                file.scroll_width,
                default.scroll_width,
                "scroll_width",
            )?,
            scroll_spacing: nonnegative_number(
                file.scroll_spacing,
                default.scroll_spacing,
                "scroll_spacing",
            )?,
            font_pane: optional_font(file.font_pane, default.font_pane),
            font_heading: optional_font(file.font_heading, default.font_heading),
            font_preview: optional_font(file.font_preview, default.font_preview),
            font_modal: optional_font(file.font_modal, default.font_modal),
            font_status: optional_font(file.font_status, default.font_status),
            preview_copy_flash_duration_ms: file
                .preview_copy_flash_duration_ms
                .unwrap_or(default.preview_copy_flash_duration_ms),
            text_color_flash_duration: file
                .text_color_flash_duration
                .unwrap_or(default.text_color_flash_duration),
        })
    }
}

fn optional_font(value: Option<String>, fallback: Font) -> Font {
    value
        .as_deref()
        .map(parse_font_name)
        .filter(|font| font.family != Family::Name(""))
        .unwrap_or(fallback)
}

fn parse_font_name(value: &str) -> Font {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Font::DEFAULT;
    }

    let mut words: Vec<&str> = trimmed.split_whitespace().collect();
    let mut weight = Weight::Normal;
    let mut style = Style::Normal;

    loop {
        let Some(last) = words.last().copied() else {
            break;
        };
        let normalized_last = last.to_ascii_lowercase();
        let consumed = match normalized_last.as_str() {
            "thin" => {
                weight = Weight::Thin;
                true
            }
            "extralight" | "extra-light" => {
                weight = Weight::ExtraLight;
                true
            }
            "light" => {
                weight = Weight::Light;
                true
            }
            "regular" | "normal" => {
                weight = Weight::Normal;
                true
            }
            "medium" => {
                weight = Weight::Medium;
                true
            }
            "semibold" | "semi-bold" => {
                weight = Weight::Semibold;
                true
            }
            "bold" => {
                weight = Weight::Bold;
                true
            }
            "extrabold" | "extra-bold" => {
                weight = Weight::ExtraBold;
                true
            }
            "black" => {
                weight = Weight::Black;
                true
            }
            "italic" => {
                style = Style::Italic;
                true
            }
            "oblique" => {
                style = Style::Oblique;
                true
            }
            _ => false,
        };
        if consumed {
            words.pop();
        } else {
            break;
        }
    }

    let family_name = if words.is_empty() {
        trimmed.to_string()
    } else {
        words.join(" ")
    };
    let normalized = family_name.to_ascii_lowercase();
    let family = match normalized.as_str() {
        "default" | "sans" | "sans-serif" | "sans serif" => Family::SansSerif,
        "serif" => Family::Serif,
        "mono" | "monospace" | "mono-space" => Family::Monospace,
        _ => Family::Name(Box::leak(family_name.into_boxed_str())),
    };

    Font {
        family,
        weight,
        style,
        ..Font::DEFAULT
    }
}

fn theme_path(data_dir: &Path, theme_name: &str) -> PathBuf {
    let mut name = theme_name.trim().to_string();
    if name.is_empty() {
        name = "default-theme".to_string();
    }
    if !name.ends_with(".yml") && !name.ends_with(".yaml") {
        name.push_str(".yml");
    }
    data_dir.join(name)
}

fn optional_color(
    value: ColorSetting,
    fallback: Color,
    custom_colors: &HashMap<String, Color>,
) -> Result<Color> {
    match value {
        ColorSetting::Value(value) => parse_color_value(value, custom_colors),
        ColorSetting::Transparent => Ok(TRANSPARENT),
        ColorSetting::Missing => Ok(fallback),
    }
}

fn deserialize_color_setting<'de, D>(deserializer: D) -> std::result::Result<ColorSetting, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        Some(Value::Null) | None => Ok(ColorSetting::Transparent),
        Some(value) => Ok(ColorSetting::Value(value)),
    }
}

fn parse_color_value(value: Value, custom_colors: &HashMap<String, Color>) -> Result<Color> {
    match value {
        Value::Null => Ok(TRANSPARENT),
        Value::String(value) => parse_color(&value, custom_colors),
        value => Err(anyhow!(
            "unsupported color value {value:?}. Use a color string, blank, or null"
        )),
    }
}

fn parse_color(value: &str, custom_colors: &HashMap<String, Color>) -> Result<Color> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Ok(TRANSPARENT);
    }
    if let Some(hex) = normalized.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    if is_hex_color(normalized) {
        return parse_hex_color(normalized);
    }

    let name = normalized.to_ascii_lowercase();
    if let Some(color) = custom_colors.get(&name) {
        return Ok(*color);
    }

    match name.as_str() {
        "none" | "null" | "transparent" | "no-color" | "no-colour" | "no color" | "no colour" => {
            Ok(TRANSPARENT)
        }
        "black" => Ok(Color::BLACK),
        "white" => Ok(Color::WHITE),
        "red" => Ok(Color::from_rgb(1.0, 0.0, 0.0)),
        "green" => Ok(Color::from_rgb(0.0, 1.0, 0.0)),
        "blue" => Ok(Color::from_rgb(0.0, 0.0, 1.0)),
        "yellow" => Ok(Color::from_rgb(1.0, 1.0, 0.0)),
        "cyan" | "teal" => Ok(Color::from_rgb(0.0, 1.0, 1.0)),
        "magenta" => Ok(Color::from_rgb(1.0, 0.0, 1.0)),
        "gray" | "grey" => Ok(Color::from_rgb(0.5, 0.5, 0.5)),
        "darkgray" | "darkgrey" => Ok(Color::from_rgb(0.333, 0.333, 0.333)),
        _ => Err(anyhow!(
            "unsupported color '{normalized}'. Use RRGGBB, RRGGBBAA, #RRGGBB, #RRGGBBAA, none, transparent, a built-in color name, or a custom color name"
        )),
    }
}

fn custom_colors(value: Option<HashMap<String, String>>) -> Result<HashMap<String, Color>> {
    let Some(value) = value else {
        return Ok(HashMap::new());
    };

    let raw: HashMap<String, String> = value
        .into_iter()
        .map(|(key, value)| (key.trim().to_ascii_lowercase(), value))
        .collect();
    let mut resolved = HashMap::new();

    for name in raw.keys() {
        resolve_custom_color(name, &raw, &mut resolved, &mut HashSet::new())?;
    }

    Ok(resolved)
}

fn resolve_custom_color(
    name: &str,
    raw: &HashMap<String, String>,
    resolved: &mut HashMap<String, Color>,
    resolving: &mut HashSet<String>,
) -> Result<Color> {
    if let Some(color) = resolved.get(name) {
        return Ok(*color);
    }
    if !resolving.insert(name.to_string()) {
        return Err(anyhow!("custom color '{name}' references itself"));
    }

    let value = raw
        .get(name)
        .ok_or_else(|| anyhow!("unknown custom color '{name}'"))?;
    let reference = value.trim().to_ascii_lowercase();
    let color = if raw.contains_key(&reference) {
        resolve_custom_color(&reference, raw, resolved, resolving)?
    } else {
        parse_color(value, resolved)?
    };

    resolving.remove(name);
    resolved.insert(name.to_string(), color);
    Ok(color)
}

fn nonnegative_number(value: Option<f32>, fallback: f32, label: &str) -> Result<f32> {
    match value {
        Some(value) if value >= 0.0 => Ok(value),
        Some(value) => Err(anyhow!("{label} must be zero or greater, got {value}")),
        None => Ok(fallback),
    }
}

fn is_hex_color(value: &str) -> bool {
    matches!(value.len(), 6 | 8) && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn parse_hex_color(hex: &str) -> Result<Color> {
    let parse_channel = |range: std::ops::Range<usize>| -> Result<f32> {
        let value = u8::from_str_radix(&hex[range], 16)?;
        Ok(f32::from(value) / 255.0)
    };

    match hex.len() {
        6 => Ok(Color {
            r: parse_channel(0..2)?,
            g: parse_channel(2..4)?,
            b: parse_channel(4..6)?,
            a: 1.0,
        }),
        8 => Ok(Color {
            r: parse_channel(0..2)?,
            g: parse_channel(2..4)?,
            b: parse_channel(4..6)?,
            a: parse_channel(6..8)?,
        }),
        _ => Err(anyhow!(
            "hex colors must be RRGGBB, RRGGBBAA, #RRGGBB, or #RRGGBBAA"
        )),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn bare_hex_colors_are_supported() {
        assert_eq!(
            super::parse_color("1F3B4D", &Default::default()).unwrap(),
            super::parse_hex_color("1F3B4D").unwrap()
        );
    }

    #[test]
    fn custom_color_names_resolve_theme_fields() {
        let file: super::ThemeFile = serde_yaml::from_str(
            r#"
custom_colors:
  shadowy-blue: 1F3B4D
pane_active_background: shadowy-blue
scroll_scroller_hovered: shadowy-blue
"#,
        )
        .unwrap();
        let theme = super::AppTheme::from_file(file).unwrap();
        let expected = super::parse_hex_color("1F3B4D").unwrap();

        assert_eq!(theme.pane_active_background, expected);
        assert_eq!(theme.scroll_scroller_hovered, expected);
    }

    #[test]
    fn custom_color_names_can_reference_other_custom_colors() {
        let file: super::ThemeFile = serde_yaml::from_str(
            r#"
custom_colors:
  shadowy-blue: 1F3B4D
  pane-border: shadowy-blue
pane_inactive_border: pane-border
"#,
        )
        .unwrap();
        let theme = super::AppTheme::from_file(file).unwrap();

        assert_eq!(
            theme.pane_inactive_border,
            super::parse_hex_color("1F3B4D").unwrap()
        );
    }

    #[test]
    fn pane_background_falls_back_into_active_and_inactive_roles() {
        let file: super::ThemeFile = serde_yaml::from_str(
            r#"
pane_background: 1F3B4D
"#,
        )
        .unwrap();
        let theme = super::AppTheme::from_file(file).unwrap();
        let expected = super::parse_hex_color("1F3B4D").unwrap();

        assert_eq!(theme.pane_background, expected);
        assert_eq!(theme.pane_active_background, expected);
        assert_eq!(theme.pane_inactive_background, expected);
    }

    #[test]
    fn blank_yaml_color_values_are_transparent() {
        let file: super::ThemeFile = serde_yaml::from_str(
            r#"
pane_inactive_border:
scroll_rail:
"#,
        )
        .unwrap();
        let theme = super::AppTheme::from_file(file).unwrap();

        assert_eq!(theme.pane_inactive_border, super::TRANSPARENT);
        assert_eq!(theme.scroll_rail, super::TRANSPARENT);
    }

    #[test]
    fn transparent_color_names_are_supported() {
        assert_eq!(
            super::parse_color("transparent", &Default::default()).unwrap(),
            super::TRANSPARENT
        );
        assert_eq!(
            super::parse_color("none", &Default::default()).unwrap(),
            super::TRANSPARENT
        );
        assert_eq!(
            super::parse_color("", &Default::default()).unwrap(),
            super::TRANSPARENT
        );
    }
}
