use serde::Serialize;

use crate::TerminalGridColor;

pub const DEFAULT_TERMINAL_THEME_ID: &str = "shellow_dark";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalThemeId {
    ShellowDark,
    Midnight,
    Amber,
    PaperLight,
}

impl TerminalThemeId {
    pub const ALL: [Self; 4] = [
        Self::ShellowDark,
        Self::Midnight,
        Self::Amber,
        Self::PaperLight,
    ];

    pub const fn wire(self) -> &'static str {
        match self {
            Self::ShellowDark => "shellow_dark",
            Self::Midnight => "midnight",
            Self::Amber => "amber",
            Self::PaperLight => "paper_light",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::ShellowDark => "Shellow Dark",
            Self::Midnight => "Midnight",
            Self::Amber => "Amber",
            Self::PaperLight => "Paper Light",
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.wire() == value)
    }
}

impl Default for TerminalThemeId {
    fn default() -> Self {
        Self::ShellowDark
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalThemeRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalTheme {
    pub id: TerminalThemeId,
    pub is_dark: bool,
    pub foreground: TerminalGridColor,
    pub background: TerminalGridColor,
    pub cursor: TerminalGridColor,
    pub accent: TerminalGridColor,
    pub muted: TerminalGridColor,
    pub warning: TerminalGridColor,
    pub success: TerminalGridColor,
    pub selection: TerminalThemeRgba,
    pub search: TerminalThemeRgba,
    pub active_search: TerminalThemeRgba,
    pub palette: [TerminalGridColor; 256],
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TerminalThemeUpdate {
    pub accepted: bool,
    pub theme_id: String,
    pub theme_title: String,
    pub is_dark: bool,
    pub notes: Vec<String>,
}

impl TerminalThemeUpdate {
    pub(crate) fn accepted(theme: &TerminalTheme) -> Self {
        Self {
            accepted: true,
            theme_id: theme.id.wire().to_string(),
            theme_title: theme.id.title().to_string(),
            is_dark: theme.is_dark,
            notes: vec!["terminal theme applied; next renderer frame will redraw".to_string()],
        }
    }

    pub(crate) fn rejected(value: &str, current: &TerminalTheme) -> Self {
        Self {
            accepted: false,
            theme_id: current.id.wire().to_string(),
            theme_title: current.id.title().to_string(),
            is_dark: current.is_dark,
            notes: vec![format!("unknown terminal theme: {value}")],
        }
    }
}

pub fn built_in_theme(id: TerminalThemeId) -> TerminalTheme {
    match id {
        TerminalThemeId::ShellowDark => TerminalTheme {
            id,
            is_dark: true,
            foreground: rgb(0xE0, 0xE8, 0xDE),
            background: rgb(0x0D, 0x0F, 0x0E),
            cursor: rgb(0x75, 0xDB, 0xAB),
            accent: rgb(0x1C, 0x9F, 0x70),
            muted: rgb(0x95, 0xA3, 0x9B),
            warning: rgb(0xED, 0xAD, 0x38),
            success: rgb(0x45, 0xD1, 0x8C),
            selection: rgba(0x2E, 0x73, 0x61, 184),
            search: rgba(0x85, 0x70, 0x26, 112),
            active_search: rgba(0xC9, 0x9E, 0x2E, 184),
            palette: xterm_palette([
                rgb(0x15, 0x18, 0x17),
                rgb(0xE0, 0x6C, 0x75),
                rgb(0x45, 0xD1, 0x8C),
                rgb(0xED, 0xAD, 0x38),
                rgb(0x61, 0xAF, 0xEF),
                rgb(0xC6, 0x78, 0xDD),
                rgb(0x56, 0xB6, 0xC2),
                rgb(0xD7, 0xDD, 0xD9),
                rgb(0x5A, 0x65, 0x5F),
                rgb(0xF0, 0x7C, 0x85),
                rgb(0x75, 0xDB, 0xAB),
                rgb(0xF6, 0xC8, 0x5F),
                rgb(0x82, 0xC5, 0xF4),
                rgb(0xD8, 0x91, 0xE8),
                rgb(0x78, 0xCF, 0xD7),
                rgb(0xF3, 0xF6, 0xF4),
            ]),
        },
        TerminalThemeId::Midnight => TerminalTheme {
            id,
            is_dark: true,
            foreground: rgb(0xD7, 0xE3, 0xF4),
            background: rgb(0x0B, 0x12, 0x20),
            cursor: rgb(0x7D, 0xD3, 0xFC),
            accent: rgb(0x60, 0xA5, 0xFA),
            muted: rgb(0x7E, 0x8E, 0xA8),
            warning: rgb(0xF8, 0xC7, 0x60),
            success: rgb(0x69, 0xDB, 0xA5),
            selection: rgba(0x1D, 0x4E, 0x89, 184),
            search: rgba(0x78, 0x5A, 0x1F, 120),
            active_search: rgba(0xD0, 0x92, 0x2B, 190),
            palette: xterm_palette([
                rgb(0x11, 0x1A, 0x2B),
                rgb(0xFB, 0x71, 0x85),
                rgb(0x4A, 0xDE, 0x80),
                rgb(0xF6, 0xC7, 0x5D),
                rgb(0x60, 0xA5, 0xFA),
                rgb(0xC0, 0x84, 0xFC),
                rgb(0x22, 0xD3, 0xEE),
                rgb(0xC7, 0xD2, 0xE4),
                rgb(0x47, 0x58, 0x70),
                rgb(0xFD, 0x8A, 0x9A),
                rgb(0x69, 0xDB, 0xA5),
                rgb(0xFA, 0xD6, 0x7A),
                rgb(0x8B, 0xBD, 0xFF),
                rgb(0xD3, 0xA4, 0xFF),
                rgb(0x67, 0xE8, 0xF9),
                rgb(0xF1, 0xF5, 0xF9),
            ]),
        },
        TerminalThemeId::Amber => TerminalTheme {
            id,
            is_dark: true,
            foreground: rgb(0xF3, 0xE6, 0xC8),
            background: rgb(0x17, 0x13, 0x0D),
            cursor: rgb(0xF4, 0xB8, 0x60),
            accent: rgb(0xD9, 0x8D, 0x32),
            muted: rgb(0xA6, 0x94, 0x78),
            warning: rgb(0xFF, 0xC8, 0x57),
            success: rgb(0x9B, 0xC5, 0x66),
            selection: rgba(0x7A, 0x4D, 0x20, 184),
            search: rgba(0x78, 0x5A, 0x1F, 120),
            active_search: rgba(0xD9, 0x8D, 0x32, 190),
            palette: xterm_palette([
                rgb(0x21, 0x1A, 0x11),
                rgb(0xD8, 0x62, 0x4E),
                rgb(0x8F, 0xB9, 0x5A),
                rgb(0xDF, 0xA9, 0x3A),
                rgb(0x6D, 0x9D, 0xB3),
                rgb(0xB4, 0x78, 0xA5),
                rgb(0x68, 0xAA, 0xA2),
                rgb(0xDE, 0xCF, 0xAE),
                rgb(0x6D, 0x5D, 0x48),
                rgb(0xEC, 0x79, 0x61),
                rgb(0xA9, 0xCF, 0x72),
                rgb(0xF4, 0xC5, 0x63),
                rgb(0x83, 0xB4, 0xC9),
                rgb(0xC9, 0x90, 0xBA),
                rgb(0x82, 0xC3, 0xBA),
                rgb(0xFA, 0xF0, 0xD8),
            ]),
        },
        TerminalThemeId::PaperLight => TerminalTheme {
            id,
            is_dark: false,
            foreground: rgb(0x2B, 0x31, 0x2E),
            background: rgb(0xFA, 0xF8, 0xF2),
            cursor: rgb(0x14, 0x7A, 0x56),
            accent: rgb(0x14, 0x7A, 0x56),
            muted: rgb(0x6B, 0x75, 0x70),
            warning: rgb(0x9A, 0x63, 0x08),
            success: rgb(0x1E, 0x7A, 0x4C),
            selection: rgba(0x99, 0xC9, 0xB2, 184),
            search: rgba(0xF3, 0xD7, 0x72, 130),
            active_search: rgba(0xE0, 0xAC, 0x35, 190),
            palette: xterm_palette([
                rgb(0x2B, 0x31, 0x2E),
                rgb(0xB4, 0x23, 0x18),
                rgb(0x14, 0x7A, 0x56),
                rgb(0x9A, 0x63, 0x08),
                rgb(0x1F, 0x5F, 0xA6),
                rgb(0x7A, 0x3E, 0x9D),
                rgb(0x0F, 0x70, 0x78),
                rgb(0xD7, 0xD5, 0xCE),
                rgb(0x70, 0x78, 0x73),
                rgb(0xD0, 0x3A, 0x2E),
                rgb(0x1E, 0x8F, 0x59),
                rgb(0xB9, 0x78, 0x0B),
                rgb(0x2F, 0x78, 0xBF),
                rgb(0x94, 0x55, 0xB5),
                rgb(0x1B, 0x88, 0x91),
                rgb(0xFF, 0xFF, 0xFF),
            ]),
        },
    }
}

pub fn default_terminal_theme() -> TerminalTheme {
    built_in_theme(TerminalThemeId::default())
}

const fn rgb(r: u8, g: u8, b: u8) -> TerminalGridColor {
    TerminalGridColor { r, g, b }
}

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> TerminalThemeRgba {
    TerminalThemeRgba { r, g, b, a }
}

const fn xterm_palette(base16: [TerminalGridColor; 16]) -> [TerminalGridColor; 256] {
    let mut palette = [rgb(0, 0, 0); 256];
    let mut index = 0;
    while index < 16 {
        palette[index] = base16[index];
        index += 1;
    }

    let levels = [0, 95, 135, 175, 215, 255];
    let mut red = 0;
    while red < 6 {
        let mut green = 0;
        while green < 6 {
            let mut blue = 0;
            while blue < 6 {
                palette[16 + red * 36 + green * 6 + blue] =
                    rgb(levels[red], levels[green], levels[blue]);
                blue += 1;
            }
            green += 1;
        }
        red += 1;
    }

    let mut gray = 0;
    while gray < 24 {
        let value = 8 + gray as u8 * 10;
        palette[232 + gray] = rgb(value, value, value);
        gray += 1;
    }
    palette
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_theme_ids_round_trip() {
        for id in TerminalThemeId::ALL {
            assert_eq!(TerminalThemeId::from_wire(id.wire()), Some(id));
            assert_eq!(built_in_theme(id).id, id);
        }
        assert_eq!(TerminalThemeId::from_wire("unknown"), None);
    }

    #[test]
    fn extended_palette_uses_xterm_cube_and_grayscale() {
        let palette = built_in_theme(TerminalThemeId::Midnight).palette;
        assert_eq!(palette[16], rgb(0, 0, 0));
        assert_eq!(palette[21], rgb(0, 0, 255));
        assert_eq!(palette[231], rgb(255, 255, 255));
        assert_eq!(palette[232], rgb(8, 8, 8));
        assert_eq!(palette[255], rgb(238, 238, 238));
    }

    #[test]
    fn paper_is_the_only_light_built_in_theme() {
        let light_themes = TerminalThemeId::ALL
            .into_iter()
            .filter(|id| !built_in_theme(*id).is_dark)
            .collect::<Vec<_>>();
        assert_eq!(light_themes, vec![TerminalThemeId::PaperLight]);
    }
}
