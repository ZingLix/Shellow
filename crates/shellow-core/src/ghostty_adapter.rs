use crate::{
    TerminalCursorShape, TerminalGridLine, TerminalGridRun, TerminalGridSnapshot,
    TerminalGridStyle, TerminalRow, TerminalScreenKind,
};

pub fn backend_name() -> &'static str {
    if is_ghostty_available() {
        "libghostty-vt"
    } else {
        "shellow-vt-adapter"
    }
}

pub fn target_backend_name() -> &'static str {
    "libghostty-vt"
}

pub fn is_ghostty_available() -> bool {
    cfg!(feature = "official-libghostty-vt-rs")
}

pub fn is_libghostty_vt_selected() -> bool {
    is_ghostty_available()
}

pub fn is_libghostty_vt_link_configured() -> bool {
    is_ghostty_available()
}

pub fn is_libghostty_vt_ready() -> bool {
    is_ghostty_available()
}

pub fn libghostty_vt_abi_contract() -> &'static str {
    "libghostty-vt-rs-0.2.0"
}

pub fn libghostty_vt_abi_status() -> String {
    if is_ghostty_available() {
        "linked crate=libghostty-vt version=0.2.0 sys=vendored-zig".to_string()
    } else {
        "not-linked crate=libghostty-vt version=0.2.0".to_string()
    }
}

pub fn migration_stage() -> &'static str {
    if is_ghostty_available() {
        "official-libghostty-vt-rs"
    } else {
        "adapter-boundary-awaiting-libghostty-vt"
    }
}

pub fn link_status() -> &'static str {
    if is_ghostty_available() {
        "rust-crate-linked"
    } else {
        "not-selected"
    }
}

pub fn demo_terminal_summary() -> String {
    #[cfg(feature = "official-libghostty-vt-rs")]
    {
        return libghostty_vt_backend::demo_terminal_summary();
    }

    #[cfg(not(feature = "official-libghostty-vt-rs"))]
    {
        "Ghostty VT core is not compiled; Shellow adapter boundary is active".to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveVtBackend {
    #[cfg(feature = "official-libghostty-vt-rs")]
    LibGhosttyVt,
    #[cfg(not(feature = "official-libghostty-vt-rs"))]
    AdapterFallback,
}

fn active_backend() -> ActiveVtBackend {
    #[cfg(feature = "official-libghostty-vt-rs")]
    {
        return ActiveVtBackend::LibGhosttyVt;
    }

    #[cfg(not(feature = "official-libghostty-vt-rs"))]
    {
        ActiveVtBackend::AdapterFallback
    }
}

impl ActiveVtBackend {
    fn terminal_rows_from_vt_output(self, output: &[u8]) -> Vec<TerminalRow> {
        let text = String::from_utf8_lossy(output);
        self.render_vt_plain_text(&text)
            .lines()
            .map(str::trim_end)
            .filter(|line| !line.is_empty())
            .map(TerminalRow::muted)
            .collect()
    }

    fn terminal_title_from_vt_bytes(self, output: &[u8]) -> Option<String> {
        match self {
            #[cfg(feature = "official-libghostty-vt-rs")]
            ActiveVtBackend::LibGhosttyVt => {
                libghostty_vt_backend::terminal_title_from_vt_bytes(output)
            }
            #[cfg(not(feature = "official-libghostty-vt-rs"))]
            ActiveVtBackend::AdapterFallback => terminal_title_from_osc(output),
        }
    }

    fn terminal_clipboard_from_vt_bytes(self, output: &[u8]) -> Option<String> {
        match self {
            #[cfg(feature = "official-libghostty-vt-rs")]
            ActiveVtBackend::LibGhosttyVt => {
                libghostty_vt_backend::terminal_clipboard_from_vt_bytes(output)
            }
            #[cfg(not(feature = "official-libghostty-vt-rs"))]
            ActiveVtBackend::AdapterFallback => terminal_clipboard_from_osc52(output),
        }
    }

    fn terminal_bell_count_from_vt_bytes(self, output: &[u8]) -> usize {
        match self {
            #[cfg(feature = "official-libghostty-vt-rs")]
            ActiveVtBackend::LibGhosttyVt => {
                libghostty_vt_backend::terminal_bell_count_from_vt_bytes(output)
            }
            #[cfg(not(feature = "official-libghostty-vt-rs"))]
            ActiveVtBackend::AdapterFallback => terminal_bell_count_from_raw_bytes(output),
        }
    }

    fn terminal_grid_from_vt_bytes(
        self,
        bytes: &[u8],
        cols: u32,
        rows: u32,
    ) -> TerminalGridSnapshot {
        match self {
            #[cfg(feature = "official-libghostty-vt-rs")]
            ActiveVtBackend::LibGhosttyVt => {
                libghostty_vt_backend::terminal_grid_from_vt_bytes(bytes, cols, rows)
            }
            #[cfg(not(feature = "official-libghostty-vt-rs"))]
            ActiveVtBackend::AdapterFallback => {
                adapter_terminal_grid_from_vt_bytes(bytes, cols, rows)
            }
        }
    }

    fn render_vt_plain_text(self, output: &str) -> String {
        match self {
            #[cfg(feature = "official-libghostty-vt-rs")]
            ActiveVtBackend::LibGhosttyVt => libghostty_vt_backend::render_vt_plain_text(output),
            #[cfg(not(feature = "official-libghostty-vt-rs"))]
            ActiveVtBackend::AdapterFallback => strip_vt_sequences(output.as_bytes()),
        }
    }
}

#[cfg(feature = "official-libghostty-vt-rs")]
mod libghostty_vt_backend {
    use super::{
        TerminalCursorShape, TerminalGridLine, TerminalGridRun, TerminalGridSnapshot,
        TerminalGridStyle, TerminalScreenKind,
    };
    use crate::TerminalGridColor;
    use libghostty_vt::{
        Error, RenderState, Terminal, TerminalOptions,
        render::{CursorVisualStyle, Dirty, RowIterator},
        screen::{Cell, CellContentTag, CellWide, GridRef, Screen},
        style::{PaletteIndex, RgbColor, Style, StyleColor, Underline},
        terminal::{Mode, Point, PointCoordinate},
    };
    use std::cell::Cell as CounterCell;

    const DEFAULT_COLS: u16 = 80;
    const DEFAULT_ROWS: u16 = 24;
    const MAX_SCROLLBACK: usize = 10_000;

    pub(super) fn demo_terminal_summary() -> String {
        let mut terminal = match new_terminal(DEFAULT_COLS as u32, 5) {
            Some(terminal) => terminal,
            None => return "libghostty-vt failed to initialize".to_string(),
        };
        terminal.vt_write(b"\x1b[32mlibghostty-vt terminal core\x1b[0m\r\n");
        terminal.vt_write("wide cells: Shellow \u{5B57}\u{7B26}\r\n".as_bytes());
        format!(
            "libghostty-vt compiled; cursor=({}, {}) dump={:?}",
            terminal.cursor_x().unwrap_or(0),
            terminal.cursor_y().unwrap_or(0),
            render_vt_plain_text_from_terminal(&terminal)
        )
    }

    pub(super) fn terminal_title_from_vt_bytes(output: &[u8]) -> Option<String> {
        let mut terminal = new_terminal(DEFAULT_COLS as u32, DEFAULT_ROWS as u32)?;
        terminal.vt_write(output);
        terminal
            .title()
            .ok()
            .and_then(super::sanitize_terminal_title)
    }

    pub(super) fn terminal_clipboard_from_vt_bytes(output: &[u8]) -> Option<String> {
        super::terminal_clipboard_from_osc52(output)
    }

    pub(super) fn terminal_bell_count_from_vt_bytes(output: &[u8]) -> usize {
        let bell_count = CounterCell::new(0usize);
        let Some(mut terminal) = Terminal::new(TerminalOptions {
            cols: DEFAULT_COLS,
            rows: DEFAULT_ROWS,
            max_scrollback: MAX_SCROLLBACK,
        })
        .ok() else {
            return 0;
        };
        if terminal
            .on_bell(|_| bell_count.set(bell_count.get().saturating_add(1)))
            .is_err()
        {
            return 0;
        }
        terminal.vt_write(output);
        bell_count.get()
    }

    pub(super) fn terminal_grid_from_vt_bytes(
        bytes: &[u8],
        cols: u32,
        rows: u32,
    ) -> TerminalGridSnapshot {
        terminal_grid_from_vt_bytes_inner(bytes, cols, rows)
            .unwrap_or_else(|| super::adapter_terminal_grid_from_vt_bytes(bytes, cols, rows))
    }

    pub(super) fn render_vt_plain_text(output: &str) -> String {
        let Some(mut terminal) = new_terminal(DEFAULT_COLS as u32, DEFAULT_ROWS as u32) else {
            return super::strip_vt_sequences(output.as_bytes());
        };
        terminal.vt_write(output.as_bytes());
        render_vt_plain_text_from_terminal(&terminal)
    }

    fn new_terminal(cols: u32, rows: u32) -> Option<Terminal<'static, 'static>> {
        Terminal::new(TerminalOptions {
            cols: clamp_cols(cols),
            rows: clamp_rows(rows),
            max_scrollback: MAX_SCROLLBACK,
        })
        .ok()
    }

    fn terminal_grid_from_vt_bytes_inner(
        bytes: &[u8],
        cols: u32,
        rows: u32,
    ) -> Option<TerminalGridSnapshot> {
        let cols_u16 = clamp_cols(cols);
        let rows_u16 = clamp_rows(rows);
        let mut terminal = Terminal::new(TerminalOptions {
            cols: cols_u16,
            rows: rows_u16,
            max_scrollback: MAX_SCROLLBACK,
        })
        .ok()?;
        terminal.vt_write(bytes);

        let active_screen = match terminal.active_screen().ok()? {
            Screen::Primary => TerminalScreenKind::Primary,
            Screen::Alternate => TerminalScreenKind::Alternate,
        };
        let scrollback_len = terminal.scrollback_rows().unwrap_or(0);
        let total_rows = terminal
            .total_rows()
            .unwrap_or(rows_u16 as usize)
            .max(rows_u16 as usize);
        let cursor_row_offset = if active_screen == TerminalScreenKind::Primary {
            scrollback_len as u32
        } else {
            0
        };
        let metadata = render_metadata(&terminal, cursor_row_offset as usize, rows_u16 as usize);
        let palette = terminal.color_palette().ok();

        let styled_lines = (0..total_rows)
            .map(|row| {
                styled_line_from_terminal_row(&terminal, row as u32, cols_u16, palette.as_ref())
            })
            .collect::<Vec<_>>();
        let lines = styled_lines
            .iter()
            .map(|line| line.runs.iter().map(|run| run.text.as_str()).collect())
            .collect();

        Some(TerminalGridSnapshot {
            cols: cols_u16 as u32,
            rows: rows_u16 as u32,
            cursor_column: terminal.cursor_x().unwrap_or(0) as u32,
            cursor_row: terminal.cursor_y().unwrap_or(0) as u32 + cursor_row_offset,
            cursor_visible: metadata.cursor_visible,
            cursor_shape: metadata.cursor_shape,
            active_screen,
            scrollback_len,
            bracketed_paste: terminal.mode(Mode::BRACKETED_PASTE).unwrap_or(false),
            application_cursor_keys: terminal.mode(Mode::DECCKM).unwrap_or(false),
            mouse_reporting: terminal.is_mouse_tracking().unwrap_or(false),
            mouse_drag_reporting: terminal.mode(Mode::BUTTON_MOUSE).unwrap_or(false)
                || terminal.mode(Mode::ANY_MOUSE).unwrap_or(false),
            sgr_mouse: terminal.mode(Mode::SGR_MOUSE).unwrap_or(false),
            lines,
            styled_lines,
            dirty_rows: metadata.dirty_rows,
        })
    }

    fn render_metadata(
        terminal: &Terminal<'_, '_>,
        row_offset: usize,
        viewport_rows: usize,
    ) -> RenderMetadata {
        let fallback = RenderMetadata {
            cursor_visible: terminal.is_cursor_visible().unwrap_or(true),
            cursor_shape: TerminalCursorShape::Block,
            dirty_rows: Vec::new(),
        };
        let Ok(mut render_state) = RenderState::new() else {
            return fallback;
        };
        let Ok(snapshot) = render_state.update(terminal) else {
            return fallback;
        };
        let cursor_visible = snapshot
            .cursor_visible()
            .unwrap_or_else(|_| terminal.is_cursor_visible().unwrap_or(true));
        let cursor_shape = snapshot
            .cursor_visual_style()
            .map(cursor_visual_style)
            .unwrap_or(TerminalCursorShape::Block);
        let dirty_rows = match snapshot.dirty().unwrap_or(Dirty::Full) {
            Dirty::Clean => Vec::new(),
            Dirty::Full => (row_offset..row_offset + viewport_rows).collect(),
            Dirty::Partial => {
                let mut dirty = Vec::new();
                if let Ok(mut rows) = RowIterator::new() {
                    if let Ok(mut row_iter) = rows.update(&snapshot) {
                        let mut row_index = 0usize;
                        while let Some(row) = row_iter.next() {
                            if row.dirty().unwrap_or(false) {
                                dirty.push(row_offset + row_index);
                            }
                            row_index += 1;
                        }
                    }
                }
                dirty
            }
        };

        RenderMetadata {
            cursor_visible,
            cursor_shape,
            dirty_rows,
        }
    }

    struct RenderMetadata {
        cursor_visible: bool,
        cursor_shape: TerminalCursorShape,
        dirty_rows: Vec<usize>,
    }

    fn styled_line_from_terminal_row(
        terminal: &Terminal<'_, '_>,
        row: u32,
        cols: u16,
        palette: Option<&[RgbColor; 256]>,
    ) -> TerminalGridLine {
        let mut cells = Vec::with_capacity(cols as usize);
        for col in 0..cols {
            if let Some((text, style)) = styled_cell_at(terminal, col, row, palette) {
                cells.push((text, style));
            }
        }

        while cells
            .last()
            .is_some_and(|(text, style)| text == " " && *style == TerminalGridStyle::default())
        {
            cells.pop();
        }

        let mut runs: Vec<TerminalGridRun> = Vec::new();
        for (text, style) in cells {
            if let Some(last) = runs.last_mut() {
                if last.style == style {
                    last.text.push_str(&text);
                    continue;
                }
            }
            runs.push(TerminalGridRun { text, style });
        }

        TerminalGridLine { runs }
    }

    fn styled_cell_at(
        terminal: &Terminal<'_, '_>,
        col: u16,
        row: u32,
        palette: Option<&[RgbColor; 256]>,
    ) -> Option<(String, TerminalGridStyle)> {
        let point = Point::Screen(PointCoordinate { x: col, y: row });
        let grid_ref = terminal.grid_ref(point).ok()?;
        let cell = grid_ref.cell().ok()?;
        if matches!(
            cell.wide().ok()?,
            CellWide::SpacerTail | CellWide::SpacerHead
        ) {
            return None;
        }

        let text = if cell.has_text().unwrap_or(false) {
            grapheme_string(&grid_ref).unwrap_or_else(|| {
                cell.codepoint()
                    .ok()
                    .and_then(char::from_u32)
                    .map(|ch| ch.to_string())
                    .filter(|text| !text.is_empty())
                    .unwrap_or_else(|| " ".to_string())
            })
        } else {
            " ".to_string()
        };

        let mut style = grid_ref
            .style()
            .map(|style| terminal_style(style, palette))
            .unwrap_or_default();
        if style.bg.is_none() {
            style.bg = cell_background_color(cell, palette);
        }
        Some((text, style))
    }

    fn grapheme_string(grid_ref: &GridRef<'_>) -> Option<String> {
        let mut buf = ['\0'; 8];
        match grid_ref.graphemes(&mut buf) {
            Ok(0) => None,
            Ok(len) => Some(buf[..len].iter().filter(|ch| **ch != '\0').collect()),
            Err(Error::OutOfSpace { required }) => {
                let mut dynamic = vec!['\0'; required];
                grid_ref
                    .graphemes(&mut dynamic)
                    .ok()
                    .filter(|len| *len > 0)
                    .map(|len| dynamic[..len].iter().filter(|ch| **ch != '\0').collect())
            }
            Err(_) => None,
        }
    }

    fn terminal_style(style: Style, palette: Option<&[RgbColor; 256]>) -> TerminalGridStyle {
        TerminalGridStyle {
            bold: style.bold,
            faint: style.faint,
            italic: style.italic,
            underline: style.underline != Underline::None,
            blink: style.blink,
            inverse: style.inverse,
            strikethrough: style.strikethrough,
            fg: style_color(style.fg_color, palette),
            bg: style_color(style.bg_color, palette),
        }
    }

    fn style_color(
        color: StyleColor,
        palette: Option<&[RgbColor; 256]>,
    ) -> Option<TerminalGridColor> {
        match color {
            StyleColor::None => None,
            StyleColor::Rgb(color) => Some(terminal_color(color)),
            StyleColor::Palette(PaletteIndex(index)) => palette
                .and_then(|palette| palette.get(index as usize))
                .copied()
                .map(terminal_color),
        }
    }

    fn cell_background_color(
        cell: Cell,
        palette: Option<&[RgbColor; 256]>,
    ) -> Option<TerminalGridColor> {
        match cell.content_tag().ok()? {
            CellContentTag::BgColorRgb => cell.bg_color_rgb().ok().map(terminal_color),
            CellContentTag::BgColorPalette => {
                let index = cell.bg_color_palette().ok()?.0 as usize;
                palette
                    .and_then(|palette| palette.get(index))
                    .copied()
                    .map(terminal_color)
            }
            CellContentTag::Codepoint | CellContentTag::CodepointGrapheme => None,
        }
    }

    fn terminal_color(color: RgbColor) -> TerminalGridColor {
        TerminalGridColor {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }

    fn cursor_visual_style(style: CursorVisualStyle) -> TerminalCursorShape {
        match style {
            CursorVisualStyle::Bar => TerminalCursorShape::Bar,
            CursorVisualStyle::Underline => TerminalCursorShape::Underline,
            CursorVisualStyle::Block | CursorVisualStyle::BlockHollow => TerminalCursorShape::Block,
            _ => TerminalCursorShape::Block,
        }
    }

    fn render_vt_plain_text_from_terminal(terminal: &Terminal<'_, '_>) -> String {
        let total_rows = terminal
            .total_rows()
            .unwrap_or(DEFAULT_ROWS as usize)
            .max(DEFAULT_ROWS as usize);
        let palette = terminal.color_palette().ok();
        let mut lines = Vec::with_capacity(total_rows);
        for row in 0..total_rows {
            let line = styled_line_from_terminal_row(
                terminal,
                row as u32,
                terminal.cols().unwrap_or(DEFAULT_COLS),
                palette.as_ref(),
            );
            let text = line
                .runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>()
                .trim_end()
                .to_string();
            if !text.is_empty() {
                lines.push(text);
            }
        }
        lines.join("\n")
    }

    fn clamp_cols(cols: u32) -> u16 {
        cols.clamp(20, 300) as u16
    }

    fn clamp_rows(rows: u32) -> u16 {
        rows.clamp(8, 120) as u16
    }
}

pub(crate) fn terminal_rows_from_vt_output(output: &[u8]) -> Vec<TerminalRow> {
    active_backend().terminal_rows_from_vt_output(output)
}

pub(crate) fn terminal_title_from_vt_bytes(output: &[u8]) -> Option<String> {
    active_backend().terminal_title_from_vt_bytes(output)
}

#[cfg(not(feature = "official-libghostty-vt-rs"))]
fn terminal_title_from_osc(output: &[u8]) -> Option<String> {
    let mut cursor = 0;

    while cursor + 2 <= output.len() {
        let Some(offset) = output[cursor..]
            .windows(2)
            .position(|window| window == b"\x1b]")
        else {
            return None;
        };
        let payload_start = cursor + offset + 2;
        let mut index = payload_start;

        while index < output.len() {
            if output[index] == b'\x07' {
                if let Some(title) = osc_title_payload(&output[payload_start..index]) {
                    return sanitize_terminal_title(title);
                }
                cursor = index + 1;
                break;
            }

            if index + 1 < output.len() && output[index] == b'\x1b' && output[index + 1] == b'\\' {
                if let Some(title) = osc_title_payload(&output[payload_start..index]) {
                    return sanitize_terminal_title(title);
                }
                cursor = index + 2;
                break;
            }

            index += 1;
        }

        if index >= output.len() {
            return None;
        }
    }

    None
}

#[cfg(not(feature = "official-libghostty-vt-rs"))]
fn osc_title_payload(payload: &[u8]) -> Option<&str> {
    let semi = payload.iter().position(|byte| *byte == b';')?;
    let selector = &payload[..semi];
    if selector != b"0" && selector != b"2" {
        return None;
    }

    std::str::from_utf8(&payload[semi + 1..]).ok()
}

fn sanitize_terminal_title(title: &str) -> Option<String> {
    let cleaned: String = title
        .chars()
        .filter(|character| !character.is_control())
        .take(80)
        .collect::<String>()
        .trim()
        .to_string();

    (!cleaned.is_empty()).then_some(cleaned)
}

pub(crate) fn terminal_clipboard_from_vt_bytes(output: &[u8]) -> Option<String> {
    active_backend().terminal_clipboard_from_vt_bytes(output)
}

fn terminal_clipboard_from_osc52(output: &[u8]) -> Option<String> {
    let mut cursor = 0;
    let mut clipboard = None;

    while cursor + 2 <= output.len() {
        let Some(offset) = output[cursor..]
            .windows(2)
            .position(|window| window == b"\x1b]")
        else {
            break;
        };
        let payload_start = cursor + offset + 2;
        let mut index = payload_start;

        while index < output.len() {
            if output[index] == b'\x07' {
                if let Some(text) = osc52_clipboard_payload(&output[payload_start..index]) {
                    clipboard = Some(text);
                }
                cursor = index + 1;
                break;
            }

            if index + 1 < output.len() && output[index] == b'\x1b' && output[index + 1] == b'\\' {
                if let Some(text) = osc52_clipboard_payload(&output[payload_start..index]) {
                    clipboard = Some(text);
                }
                cursor = index + 2;
                break;
            }

            index += 1;
        }

        if index >= output.len() {
            break;
        }
    }

    clipboard
}

fn osc52_clipboard_payload(payload: &[u8]) -> Option<String> {
    let mut parts = payload.splitn(3, |byte| *byte == b';');
    let selector = parts.next()?;
    if selector != b"52" {
        return None;
    }

    let target = parts.next()?;
    if !target.is_empty()
        && !target
            .iter()
            .all(|byte| matches!(*byte, b'c' | b'p' | b's' | b'q'))
    {
        return None;
    }

    let encoded = parts.next()?;
    if encoded == b"?" {
        return None;
    }

    let decoded = decode_base64_standard(encoded)?;
    let text = String::from_utf8(decoded).ok()?;
    sanitize_clipboard_text(&text)
}

fn sanitize_clipboard_text(text: &str) -> Option<String> {
    let cleaned: String = text
        .chars()
        .filter(|character| *character != '\0')
        .take(8_192)
        .collect();

    (!cleaned.is_empty()).then_some(cleaned)
}

fn decode_base64_standard(input: &[u8]) -> Option<Vec<u8>> {
    let mut buffer: u32 = 0;
    let mut bits: u8 = 0;
    let mut output = Vec::with_capacity(input.len() * 3 / 4);
    let mut saw_padding = false;

    for byte in input
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
    {
        if byte == b'=' {
            saw_padding = true;
            continue;
        }

        if saw_padding {
            return None;
        }

        let value = base64_standard_value(byte)? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;

        while bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xff) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Some(output)
}

fn base64_standard_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

pub(crate) fn terminal_bell_count_from_vt_bytes(output: &[u8]) -> usize {
    active_backend().terminal_bell_count_from_vt_bytes(output)
}

#[cfg(not(feature = "official-libghostty-vt-rs"))]
fn terminal_bell_count_from_raw_bytes(output: &[u8]) -> usize {
    let mut count = 0;
    let mut index = 0;

    while index < output.len() {
        if output[index] == b'\x07' {
            count += 1;
            index += 1;
            continue;
        }

        if index + 1 < output.len() && output[index] == b'\x1b' && output[index + 1] == b']' {
            index += 2;
            while index < output.len() {
                if output[index] == b'\x07' {
                    index += 1;
                    break;
                }
                if index + 1 < output.len()
                    && output[index] == b'\x1b'
                    && output[index + 1] == b'\\'
                {
                    index += 2;
                    break;
                }
                index += 1;
            }
            continue;
        }

        index += 1;
    }

    count
}

pub(crate) fn terminal_grid_from_vt_bytes(
    bytes: &[u8],
    cols: u32,
    rows: u32,
) -> TerminalGridSnapshot {
    active_backend().terminal_grid_from_vt_bytes(bytes, cols, rows)
}

fn adapter_terminal_grid_from_vt_bytes(bytes: &[u8], cols: u32, rows: u32) -> TerminalGridSnapshot {
    let text = strip_vt_sequences(bytes);
    let all_lines = text
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>();
    let scrollback_len = all_lines.len().saturating_sub(rows as usize);
    let cursor_row = all_lines.len().saturating_sub(1) as u32;
    let lines = all_lines;
    let styled_lines = lines
        .iter()
        .map(|line| TerminalGridLine {
            runs: vec![TerminalGridRun {
                text: line.clone(),
                style: TerminalGridStyle::default(),
            }],
        })
        .collect();

    TerminalGridSnapshot {
        cols,
        rows,
        cursor_column: 0,
        cursor_row,
        cursor_visible: true,
        cursor_shape: terminal_cursor_shape_from_bytes(bytes),
        active_screen: terminal_screen_kind_from_private_modes(bytes)
            .unwrap_or(TerminalScreenKind::Primary),
        scrollback_len,
        bracketed_paste: false,
        application_cursor_keys: false,
        mouse_reporting: terminal_private_mode_enabled(bytes, &[1000, 1002, 1003]),
        mouse_drag_reporting: terminal_private_mode_enabled(bytes, &[1002, 1003]),
        sgr_mouse: terminal_private_mode_enabled(bytes, &[1006]),
        lines,
        styled_lines,
        dirty_rows: Vec::new(),
    }
}

fn terminal_screen_kind_from_private_modes(bytes: &[u8]) -> Option<TerminalScreenKind> {
    let mut active_screen = None;
    let mut index = 0;

    while index + 3 < bytes.len() {
        if bytes[index] != b'\x1b' || bytes[index + 1] != b'[' || bytes[index + 2] != b'?' {
            index += 1;
            continue;
        }

        index += 3;
        let parameters_start = index;
        while index < bytes.len() {
            let byte = bytes[index];
            if byte == b'h' || byte == b'l' {
                let params = String::from_utf8_lossy(&bytes[parameters_start..index]);
                if params
                    .split(';')
                    .filter_map(|part| part.parse::<u32>().ok())
                    .any(|mode| matches!(mode, 47 | 1047 | 1049))
                {
                    active_screen = Some(if byte == b'h' {
                        TerminalScreenKind::Alternate
                    } else {
                        TerminalScreenKind::Primary
                    });
                }
                index += 1;
                break;
            }
            if !(byte.is_ascii_digit() || byte == b';') {
                index += 1;
                break;
            }
            index += 1;
        }
    }

    active_screen
}

fn terminal_private_mode_enabled(bytes: &[u8], modes: &[u32]) -> bool {
    let mut enabled = false;
    let mut index = 0;

    while index + 3 < bytes.len() {
        if bytes[index] != b'\x1b' || bytes[index + 1] != b'[' || bytes[index + 2] != b'?' {
            index += 1;
            continue;
        }

        index += 3;
        let parameters_start = index;
        while index < bytes.len() {
            let byte = bytes[index];
            if byte == b'h' || byte == b'l' {
                let final_byte = byte;
                let params = String::from_utf8_lossy(&bytes[parameters_start..index]);
                if params
                    .split(';')
                    .filter_map(|part| part.parse::<u32>().ok())
                    .any(|mode| modes.contains(&mode))
                {
                    enabled = final_byte == b'h';
                }
                index += 1;
                break;
            }
            if !(byte.is_ascii_digit() || byte == b';') {
                index += 1;
                break;
            }
            index += 1;
        }
    }

    enabled
}

fn terminal_cursor_shape_from_bytes(bytes: &[u8]) -> TerminalCursorShape {
    let mut shape = TerminalCursorShape::Block;
    let mut index = 0;

    while index + 2 < bytes.len() {
        if bytes[index] != b'\x1b' || bytes[index + 1] != b'[' {
            index += 1;
            continue;
        }

        let mut cursor = index + 2;
        let mut params = Vec::new();
        while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
            params.push(bytes[cursor]);
            cursor += 1;
        }

        if cursor + 1 < bytes.len() && bytes[cursor] == b' ' && bytes[cursor + 1] == b'q' {
            let param = std::str::from_utf8(&params)
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            shape = match param {
                3 | 4 => TerminalCursorShape::Underline,
                5 | 6 => TerminalCursorShape::Bar,
                _ => TerminalCursorShape::Block,
            };
            index = cursor + 2;
            continue;
        }

        index += 1;
    }

    shape
}

#[cfg(test)]
pub(crate) fn render_vt_plain_text(output: &str) -> String {
    active_backend().render_vt_plain_text(output)
}

fn strip_vt_sequences(output: &[u8]) -> String {
    let mut plain = Vec::with_capacity(output.len());
    let mut index = 0;

    while index < output.len() {
        if output[index] != b'\x1b' {
            let byte = output[index];
            if !byte.is_ascii_control() || matches!(byte, b'\n' | b'\r' | b'\t') {
                plain.push(byte);
            }
            index += 1;
            continue;
        }

        index += 1;
        match output.get(index).copied() {
            Some(b']') => {
                index += 1;
                while index < output.len() {
                    if output[index] == b'\x07' {
                        index += 1;
                        break;
                    }
                    if index + 1 < output.len()
                        && output[index] == b'\x1b'
                        && output[index + 1] == b'\\'
                    {
                        index += 2;
                        break;
                    }
                    index += 1;
                }
            }
            Some(b'[') => {
                index += 1;
                while index < output.len() {
                    let byte = output[index];
                    index += 1;
                    if (0x40..=0x7E).contains(&byte) {
                        break;
                    }
                }
            }
            Some(_) | None => {
                index += 1;
            }
        }
    }

    String::from_utf8_lossy(&plain).trim().to_string()
}
