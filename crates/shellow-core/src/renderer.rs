#![allow(clippy::too_many_arguments)]

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "native-integrations")]
use std::ffi::c_void;
use std::hash::{Hash, Hasher};
#[cfg(feature = "native-integrations")]
use std::ptr::NonNull;
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use crate::{
    TerminalCursorShape, TerminalGridColor, TerminalGridSnapshot, TerminalGridStyle, TerminalRow,
    TerminalRowStyle, TerminalScreenKind,
    terminal_theme::{TerminalTheme, TerminalThemeId, TerminalThemeRgba, default_terminal_theme},
};

static NEXT_RENDERER_ID: AtomicU64 = AtomicU64::new(1);
const MAX_RENDERER_OVERLAY_RANGES: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalRendererInfo {
    pub renderer_id: String,
    pub backend: String,
    pub target_backend: String,
    pub pipeline_stage: String,
    pub renderer_ready: bool,
    pub native_surface_ready: bool,
    pub native_surface_attached: bool,
    pub native_surface_kind: Option<RendererSurfaceKind>,
    pub native_surface_generation: u64,
    pub native_surface_width_px: u32,
    pub native_surface_height_px: u32,
    pub native_surface_configured: bool,
    pub native_surface_presentation_ready: bool,
    pub native_surface_terminal_frame_ready: bool,
    pub native_surface_present_count: u64,
    pub native_surface_terminal_frame_count: u64,
    pub glyph_atlas_ready: bool,
    pub glyph_atlas_glyph_count: usize,
    pub glyph_atlas_revision: u64,
    pub glyph_atlas_backend: String,
    pub glyph_atlas_target_backend: String,
    pub glyph_atlas_real_font_ready: bool,
    pub glyph_layout_backend: String,
    pub glyph_layout_target_backend: String,
    pub glyph_layout_shaping_ready: bool,
    pub gpu_glyph_atlas_upload_count: u64,
    pub gpu_dirty_row_upload_count: u64,
    pub last_dirty_row_upload_bytes: usize,
    pub renderer_overlay_range_count: usize,
    pub persistent_device_ready: bool,
    pub frame_count: u64,
    pub width_px: u32,
    pub height_px: u32,
    pub cols: u32,
    pub rows: u32,
    pub gpu_backend: Option<String>,
    pub gpu_adapter: Option<String>,
    pub last_frame_signature: Option<String>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalRenderFrame {
    pub renderer_id: String,
    pub frame_index: u64,
    pub backend: String,
    pub target_backend: String,
    pub pipeline_stage: String,
    pub renderer_ready: bool,
    pub native_surface_ready: bool,
    pub native_surface_attached: bool,
    pub native_surface_kind: Option<RendererSurfaceKind>,
    pub native_surface_generation: u64,
    pub native_surface_configured: bool,
    pub native_surface_presented_this_frame: bool,
    pub native_surface_presentation_ready: bool,
    pub native_surface_terminal_frame_presented_this_frame: bool,
    pub native_surface_terminal_frame_ready: bool,
    pub native_surface_present_count: u64,
    pub native_surface_terminal_frame_count: u64,
    pub native_surface_terminal_cell_count: usize,
    pub native_surface_terminal_overlay_range_count: usize,
    pub native_surface_terminal_vertex_count: usize,
    pub glyph_atlas_ready: bool,
    pub glyph_atlas_glyph_count: usize,
    pub glyph_atlas_revision: u64,
    pub glyph_atlas_backend: String,
    pub glyph_atlas_target_backend: String,
    pub glyph_atlas_real_font_ready: bool,
    pub glyph_layout_backend: String,
    pub glyph_layout_target_backend: String,
    pub glyph_layout_shaping_ready: bool,
    pub glyph_layout_cluster_count: usize,
    pub glyph_layout_wide_cluster_count: usize,
    pub glyph_layout_zero_width_cluster_count: usize,
    pub glyph_layout_shaped_glyph_count: usize,
    pub glyph_atlas_uploaded: bool,
    pub dirty_row_upload_count: usize,
    pub dirty_row_upload_bytes: usize,
    pub gpu_dirty_row_upload_count: usize,
    pub gpu_dirty_row_upload_bytes: usize,
    pub persistent_device_ready: bool,
    pub reused_gpu_device: bool,
    pub viewport_changed: bool,
    pub content_changed: bool,
    pub offscreen_gpu_pass: bool,
    pub gpu_backend: Option<String>,
    pub gpu_adapter: Option<String>,
    pub width_px: u32,
    pub height_px: u32,
    pub cols: u32,
    pub rows: u32,
    pub cell_width_px: u32,
    pub cell_height_px: u32,
    pub active_screen: TerminalScreenKind,
    pub scrollback_len: usize,
    pub dirty_rows: Vec<usize>,
    pub visible_line_count: usize,
    pub styled_run_count: usize,
    pub text_cell_count: usize,
    pub cursor_column: u32,
    pub cursor_row: u32,
    pub cursor_visible: bool,
    pub frame_signature: String,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RendererSurfaceKind {
    CoreAnimationLayer,
    AndroidNativeWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RendererOverlayState {
    #[serde(default)]
    pub ranges: Vec<RendererOverlayRange>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RendererOverlayRange {
    pub kind: RendererOverlayKind,
    pub row: u32,
    pub start_col: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RendererOverlayKind {
    Selection,
    Search,
    ActiveSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RendererOverlayUpdate {
    pub accepted: bool,
    pub range_count: usize,
    pub max_range_count: usize,
    pub notes: Vec<String>,
}

impl RendererOverlayState {
    fn normalized(mut self) -> (Self, Vec<String>) {
        let original_range_count = self.ranges.len();
        let mut notes = Vec::new();
        self.ranges.retain(|range| range.end_col > range.start_col);
        if self.ranges.len() != original_range_count {
            notes.push("empty overlay ranges were discarded".to_string());
        }
        if self.ranges.len() > MAX_RENDERER_OVERLAY_RANGES {
            self.ranges.truncate(MAX_RENDERER_OVERLAY_RANGES);
            notes.push(format!(
                "overlay ranges were truncated to {MAX_RENDERER_OVERLAY_RANGES}"
            ));
        }
        (self, notes)
    }

    fn hash_into(&self, hasher: &mut DefaultHasher) {
        self.ranges.len().hash(hasher);
        for range in &self.ranges {
            range.hash(hasher);
        }
    }
}

impl RendererOverlayKind {
    fn surface_color(self, theme: &TerminalTheme) -> SurfaceColor {
        match self {
            Self::Selection => SurfaceColor::from_theme_rgba(theme.selection),
            Self::Search => SurfaceColor::from_theme_rgba(theme.search),
            Self::ActiveSearch => SurfaceColor::from_theme_rgba(theme.active_search),
        }
    }
}

impl RendererSurfaceKind {
    pub fn core_animation_layer() -> Self {
        Self::CoreAnimationLayer
    }

    pub fn android_native_window() -> Self {
        Self::AndroidNativeWindow
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::CoreAnimationLayer => "core-animation-layer",
            Self::AndroidNativeWindow => "android-native-window",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RendererSurfaceRequest {
    pub kind: RendererSurfaceKind,
    pub raw_handle: u64,
    pub width_px: u32,
    pub height_px: u32,
}

impl RendererSurfaceRequest {
    pub fn new(kind: RendererSurfaceKind, raw_handle: u64, width_px: u32, height_px: u32) -> Self {
        Self {
            kind,
            raw_handle,
            width_px,
            height_px,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RendererSurfaceAttachment {
    pub kind: RendererSurfaceKind,
    pub generation: u64,
    pub raw_handle_nonzero: bool,
    pub width_px: u32,
    pub height_px: u32,
    pub attach_api_ready: bool,
    pub platform_supported: bool,
    pub wgpu_surface_configured: bool,
    pub presentation_ready: bool,
    pub present_count: u64,
    pub status: String,
    pub notes: Vec<String>,
}

impl RendererSurfaceAttachment {
    fn detached(generation: u64) -> Self {
        Self {
            kind: RendererSurfaceKind::CoreAnimationLayer,
            generation,
            raw_handle_nonzero: false,
            width_px: 0,
            height_px: 0,
            attach_api_ready: false,
            platform_supported: false,
            wgpu_surface_configured: false,
            presentation_ready: false,
            present_count: 0,
            status: "detached".to_string(),
            notes: vec!["native renderer surface is detached".to_string()],
        }
    }

    fn summary(&self) -> String {
        if self.attach_api_ready {
            format!(
                "{}#{}:{}x{}",
                self.kind.label(),
                self.generation,
                self.width_px,
                self.height_px
            )
        } else {
            "pending".to_string()
        }
    }
}

impl TerminalRenderFrame {
    pub fn summary(&self) -> String {
        let gpu = match (&self.gpu_backend, &self.gpu_adapter) {
            (Some(backend), Some(adapter)) => format!("{adapter} via {backend}"),
            (Some(backend), None) => backend.clone(),
            _ => "gpu-pass-unavailable".to_string(),
        };
        let surface = self
            .native_surface_kind
            .map(RendererSurfaceKind::label)
            .unwrap_or("pending");

        format!(
            "{} stage={} target={} renderer={} frame#{} frame={}x{} cells={}x{} dirty={} runs={} gpu={} persistent={} reused={} atlas={} atlas-backend={} atlas-target={} layout={} layout-target={} clusters={} wide-clusters={} zero-width={} shaped={} atlas-glyphs={} row-upload={} row-bytes={} surface={} surface-config={} surface-present={} surface-terminal={} presents={} terminal-presents={} native-surface={}",
            self.backend,
            self.pipeline_stage,
            self.target_backend,
            self.renderer_id,
            self.frame_index,
            self.width_px,
            self.height_px,
            self.cols,
            self.rows,
            self.dirty_rows.len(),
            self.styled_run_count,
            gpu,
            ready_word(self.persistent_device_ready),
            self.reused_gpu_device,
            ready_word(self.glyph_atlas_ready),
            self.glyph_atlas_backend,
            self.glyph_atlas_target_backend,
            self.glyph_layout_backend,
            self.glyph_layout_target_backend,
            self.glyph_layout_cluster_count,
            self.glyph_layout_wide_cluster_count,
            self.glyph_layout_zero_width_cluster_count,
            self.glyph_layout_shaped_glyph_count,
            self.glyph_atlas_glyph_count,
            self.gpu_dirty_row_upload_count,
            self.gpu_dirty_row_upload_bytes,
            surface,
            ready_word(self.native_surface_configured),
            ready_word(self.native_surface_presentation_ready),
            ready_word(self.native_surface_terminal_frame_ready),
            self.native_surface_present_count,
            self.native_surface_terminal_frame_count,
            ready_word(self.native_surface_ready)
        )
    }
}

pub struct TerminalRenderer {
    renderer_id: String,
    frame_count: u64,
    last_frame_signature: Option<String>,
    dimensions: RendererDimensions,
    glyph_atlas: GlyphAtlas,
    layout_cache: GlyphLayoutCache,
    runtime: RendererRuntime,
    overlay_state: RendererOverlayState,
    theme: TerminalTheme,
    surface_attachment: Option<RendererSurfaceAttachment>,
    surface_generation: u64,
}

impl TerminalRenderer {
    pub fn new(cols: u32, rows: u32) -> Self {
        Self {
            renderer_id: next_renderer_id(),
            frame_count: 0,
            last_frame_signature: None,
            dimensions: RendererDimensions {
                width_px: 1,
                height_px: 1,
                cols: cols.max(1),
                rows: rows.max(1),
            },
            glyph_atlas: GlyphAtlas::new(),
            layout_cache: GlyphLayoutCache::new(),
            runtime: RendererRuntime::new(),
            overlay_state: RendererOverlayState::default(),
            theme: default_terminal_theme(),
            surface_attachment: None,
            surface_generation: 0,
        }
    }

    pub fn info(&self) -> TerminalRendererInfo {
        let mut notes = Vec::new();
        if is_wgpu_available() {
            notes.push(
                "persistent renderer runtime owns the wgpu device/queue after first frame"
                    .to_string(),
            );
        } else {
            notes.push("wgpu native integration is not compiled into this build".to_string());
        }
        if let Some(note) = self.runtime.failure_note() {
            notes.push(note);
        }
        notes.push(
            "native surface attach/detach is wired for iOS CoreAnimationLayer and Android ANativeWindow hosts"
                .to_string(),
        );
        notes.push(
            "glyph atlas metadata is owned by Rust; native builds upload it to wgpu".to_string(),
        );
        notes.push(format!(
            "glyph atlas backend is {}; target is {}",
            self.glyph_atlas.backend_name(),
            self.glyph_atlas.target_backend_name()
        ));
        notes.push(format!(
            "glyph layout backend is {}; target is {}",
            self.glyph_atlas.layout_backend_name(),
            self.glyph_atlas.layout_target_backend_name()
        ));
        if self.glyph_atlas.real_font_ready() {
            notes.push(
                "real font glyph rasterization is active; full shaping remains the target atlas work"
                    .to_string(),
            );
        } else {
            notes.push(
                "real font glyph rasterization is not available yet; procedural cell rasterizer is the current fallback"
                    .to_string(),
            );
        }
        notes.push(
            "dirty rows are packed for GPU upload before the native surface is attached"
                .to_string(),
        );
        if !self.overlay_state.ranges.is_empty() {
            notes.push(format!(
                "{} renderer overlay ranges are active",
                self.overlay_state.ranges.len()
            ));
        }
        if let Some(surface) = &self.surface_attachment {
            notes.push(format!(
                "native surface attach ABI accepted {}",
                surface.summary()
            ));
            notes.extend(surface.notes.clone());
        }

        TerminalRendererInfo {
            renderer_id: self.renderer_id.clone(),
            backend: backend_name().to_string(),
            target_backend: target_backend_name().to_string(),
            pipeline_stage: pipeline_stage().to_string(),
            renderer_ready: is_wgpu_available(),
            native_surface_ready: self.native_surface_ready(),
            native_surface_attached: self.native_surface_attached(),
            native_surface_kind: self.surface_attachment.as_ref().map(|surface| surface.kind),
            native_surface_generation: self.surface_generation,
            native_surface_width_px: self
                .surface_attachment
                .as_ref()
                .map_or(0, |surface| surface.width_px),
            native_surface_height_px: self
                .surface_attachment
                .as_ref()
                .map_or(0, |surface| surface.height_px),
            native_surface_configured: self.native_surface_configured(),
            native_surface_presentation_ready: self.native_surface_presentation_ready(),
            native_surface_terminal_frame_ready: self.native_surface_terminal_frame_ready(),
            native_surface_present_count: self.native_surface_present_count(),
            native_surface_terminal_frame_count: self.native_surface_terminal_frame_count(),
            glyph_atlas_ready: self.runtime.glyph_atlas_ready(),
            glyph_atlas_glyph_count: self.glyph_atlas.glyph_count(),
            glyph_atlas_revision: self.glyph_atlas.revision(),
            glyph_atlas_backend: self.glyph_atlas.backend_name().to_string(),
            glyph_atlas_target_backend: self.glyph_atlas.target_backend_name().to_string(),
            glyph_atlas_real_font_ready: self.glyph_atlas.real_font_ready(),
            glyph_layout_backend: self.glyph_atlas.layout_backend_name().to_string(),
            glyph_layout_target_backend: self.glyph_atlas.layout_target_backend_name().to_string(),
            glyph_layout_shaping_ready: self.glyph_atlas.shaping_ready(),
            gpu_glyph_atlas_upload_count: self.runtime.glyph_atlas_upload_count(),
            gpu_dirty_row_upload_count: self.runtime.dirty_row_upload_count(),
            last_dirty_row_upload_bytes: self.runtime.last_dirty_row_upload_bytes(),
            renderer_overlay_range_count: self.overlay_state.ranges.len(),
            persistent_device_ready: self.runtime.persistent_device_ready(),
            frame_count: self.frame_count,
            width_px: self.dimensions.width_px,
            height_px: self.dimensions.height_px,
            cols: self.dimensions.cols,
            rows: self.dimensions.rows,
            gpu_backend: self.runtime.gpu_backend(),
            gpu_adapter: self.runtime.gpu_adapter(),
            last_frame_signature: self.last_frame_signature.clone(),
            notes,
        }
    }

    pub fn attach_native_surface(
        &mut self,
        request: RendererSurfaceRequest,
    ) -> RendererSurfaceAttachment {
        self.surface_generation = self.surface_generation.saturating_add(1);
        let generation = self.surface_generation;
        let mut notes = Vec::new();
        let raw_handle_nonzero = request.raw_handle != 0;
        let has_dimensions = request.width_px > 0 && request.height_px > 0;
        let platform_supported = surface_platform_supported(request.kind);
        let attach_api_ready = raw_handle_nonzero && has_dimensions;
        let mut wgpu_surface_configured = false;
        let mut presentation_ready = false;
        let mut present_count = 0;

        if !raw_handle_nonzero {
            notes.push("native surface handle was zero".to_string());
        }
        if !has_dimensions {
            notes.push("native surface dimensions must be non-zero".to_string());
        }
        if platform_supported {
            notes.push(format!(
                "{} host is supported by Shellow's final wgpu surface ABI",
                request.kind.label()
            ));
        } else {
            notes.push(format!(
                "{} host is recorded for ABI compatibility, but this build cannot create that platform surface",
                request.kind.label()
            ));
        }
        if attach_api_ready {
            notes.push(
                "surface attach ABI is live; mounted platform views can now drive terminal content presentation through Rust render_frame"
                    .to_string(),
            );
            let surface_update = self.runtime.attach_native_surface(request, generation);
            wgpu_surface_configured = surface_update.wgpu_surface_configured;
            presentation_ready = surface_update.presentation_ready;
            present_count = surface_update.present_count;
            notes.extend(surface_update.notes);
        }

        let attachment = RendererSurfaceAttachment {
            kind: request.kind,
            generation,
            raw_handle_nonzero,
            width_px: request.width_px,
            height_px: request.height_px,
            attach_api_ready,
            platform_supported,
            wgpu_surface_configured,
            presentation_ready,
            present_count,
            status: if attach_api_ready {
                if presentation_ready {
                    "attached-wgpu-surface-presented".to_string()
                } else if wgpu_surface_configured {
                    "attached-wgpu-surface-configured".to_string()
                } else {
                    "attached-awaiting-wgpu-surface-presentation".to_string()
                }
            } else {
                "attach-rejected-invalid-descriptor".to_string()
            },
            notes,
        };

        if attach_api_ready {
            self.surface_attachment = Some(attachment.clone());
        }
        attachment
    }

    pub fn detach_native_surface(&mut self) -> RendererSurfaceAttachment {
        self.surface_generation = self.surface_generation.saturating_add(1);
        self.surface_attachment = None;
        self.runtime.detach_native_surface();
        RendererSurfaceAttachment::detached(self.surface_generation)
    }

    fn native_surface_attached(&self) -> bool {
        self.surface_attachment
            .as_ref()
            .is_some_and(|surface| surface.attach_api_ready)
    }

    fn native_surface_presentation_ready(&self) -> bool {
        self.surface_attachment
            .as_ref()
            .is_some_and(|surface| surface.presentation_ready)
    }

    fn native_surface_configured(&self) -> bool {
        self.surface_attachment
            .as_ref()
            .is_some_and(|surface| surface.wgpu_surface_configured)
    }

    fn native_surface_present_count(&self) -> u64 {
        self.surface_attachment
            .as_ref()
            .map_or(0, |surface| surface.present_count)
    }

    fn native_surface_terminal_frame_ready(&self) -> bool {
        self.runtime.surface_terminal_frame_ready()
    }

    fn native_surface_terminal_frame_count(&self) -> u64 {
        self.runtime.surface_terminal_frame_count()
    }

    fn native_surface_ready(&self) -> bool {
        self.native_surface_terminal_frame_ready()
    }

    pub fn resize_cells(&mut self, cols: u32, rows: u32) {
        if self.dimensions.cols != cols || self.dimensions.rows != rows {
            self.dimensions.cols = cols.max(1);
            self.dimensions.rows = rows.max(1);
            self.invalidate();
        }
    }

    pub fn invalidate(&mut self) {
        self.last_frame_signature = None;
    }

    pub fn set_theme(&mut self, theme: TerminalTheme) {
        if self.theme != theme {
            self.theme = theme;
            self.invalidate();
        }
    }

    pub fn theme_id(&self) -> TerminalThemeId {
        self.theme.id
    }

    pub fn set_overlay_state(&mut self, state: RendererOverlayState) -> RendererOverlayUpdate {
        let (state, mut notes) = state.normalized();
        let changed = self.overlay_state != state;
        self.overlay_state = state;
        if changed {
            self.invalidate();
            notes.push("renderer overlay state changed; next frame will redraw".to_string());
        } else {
            notes.push("renderer overlay state unchanged".to_string());
        }

        RendererOverlayUpdate {
            accepted: true,
            range_count: self.overlay_state.ranges.len(),
            max_range_count: MAX_RENDERER_OVERLAY_RANGES,
            notes,
        }
    }

    pub fn render_frame(
        &mut self,
        grid: Option<&TerminalGridSnapshot>,
        history_rows: &[TerminalRow],
        history_row_count: usize,
        terminal_cols: u32,
        terminal_rows: u32,
        width_px: u32,
        height_px: u32,
    ) -> TerminalRenderFrame {
        let width_px = width_px.max(1);
        let height_px = height_px.max(1);
        let cols = grid.map_or(terminal_cols, |grid| grid.cols).max(1);
        let rows = grid.map_or(terminal_rows, |grid| grid.rows).max(1);
        let viewport_changed = self.dimensions.update(width_px, height_px, cols, rows);
        if viewport_changed {
            self.invalidate();
        }

        let active_screen = grid.map_or(TerminalScreenKind::Primary, |grid| grid.active_screen);
        let scrollback_len = grid.map_or(0, |grid| grid.scrollback_len);
        let visible_line_count = grid.map_or(history_rows.len(), |grid| grid.lines.len());
        let styled_run_count = grid.map_or(0, |grid| {
            grid.styled_lines
                .iter()
                .map(|line| line.runs.len())
                .sum::<usize>()
        });
        let text_cell_count = match grid {
            Some(grid) => grid
                .lines
                .iter()
                .map(|line| terminal_text_cell_count(line))
                .sum::<usize>(),
            None => history_rows
                .iter()
                .map(|row| terminal_text_cell_count(&history_row_text(row)))
                .sum::<usize>(),
        };
        let cursor_column = grid.map_or(0, |grid| grid.cursor_column);
        let cursor_row = grid.map_or(0, |grid| grid.cursor_row);
        let cursor_visible = grid.is_some_and(|grid| grid.cursor_visible);

        let frame_signature = frame_signature(
            grid,
            history_rows,
            history_row_count,
            width_px,
            height_px,
            &self.overlay_state,
        );
        let content_changed = self
            .last_frame_signature
            .as_deref()
            .is_none_or(|previous| previous != frame_signature);
        let dirty_rows = dirty_rows_for(
            grid,
            history_row_count,
            rows,
            visible_line_count,
            content_changed,
        );

        let added_glyph_count =
            self.glyph_atlas
                .ensure_frame(grid, history_rows, &mut self.layout_cache);
        let dirty_row_upload = DirtyRowUpload::from_frame(
            grid,
            history_rows,
            &dirty_rows,
            &self.glyph_atlas,
            &mut self.layout_cache,
        );
        let surface_frame_upload = SurfaceFrameUpload::from_frame(
            grid,
            history_rows,
            width_px,
            height_px,
            &self.glyph_atlas,
            &mut self.layout_cache,
            &self.overlay_state,
            &self.theme,
        );
        let gpu_pass = self.runtime.render_frame(
            width_px,
            height_px,
            &self.glyph_atlas,
            &dirty_row_upload,
            &surface_frame_upload,
            SurfaceColor::from_grid(self.theme.background),
        );
        if let Some(surface) = self.surface_attachment.as_mut() {
            surface.wgpu_surface_configured = gpu_pass.native_surface_configured;
            surface.presentation_ready = gpu_pass.native_surface_presentation_ready;
            surface.present_count = gpu_pass.native_surface_present_count;
            if gpu_pass.native_surface_presentation_ready {
                surface.status = "attached-wgpu-surface-presented".to_string();
            } else if gpu_pass.native_surface_configured {
                surface.status = "attached-wgpu-surface-configured".to_string();
            }
        }
        let frame_index = self.frame_count.saturating_add(1);
        self.frame_count = frame_index;
        self.last_frame_signature = Some(frame_signature.clone());

        let mut notes = gpu_pass.notes;
        if content_changed {
            notes.push("content signature changed; dirty rows are scheduled".to_string());
        } else {
            notes
                .push("content signature unchanged; no synthetic dirty rows scheduled".to_string());
        }
        if added_glyph_count > 0 {
            notes.push(format!(
                "{added_glyph_count} glyphs added to the shared renderer atlas"
            ));
        }
        if dirty_row_upload.row_count > 0 {
            notes.push(format!(
                "{} dirty rows packed into {} bytes for GPU upload",
                dirty_row_upload.row_count,
                dirty_row_upload.byte_len()
            ));
        }
        if gpu_pass.native_surface_terminal_frame_presented_this_frame {
            notes.push(format!(
                "native wgpu surface drew {} terminal cells with {} vertices",
                gpu_pass.native_surface_terminal_cell_count,
                gpu_pass.native_surface_terminal_vertex_count
            ));
        }

        TerminalRenderFrame {
            renderer_id: self.renderer_id.clone(),
            frame_index,
            backend: backend_name().to_string(),
            target_backend: target_backend_name().to_string(),
            pipeline_stage: pipeline_stage().to_string(),
            renderer_ready: is_wgpu_available(),
            native_surface_ready: self.native_surface_ready(),
            native_surface_attached: self.native_surface_attached(),
            native_surface_kind: self.surface_attachment.as_ref().map(|surface| surface.kind),
            native_surface_generation: self.surface_generation,
            native_surface_configured: self.native_surface_configured(),
            native_surface_presented_this_frame: gpu_pass.native_surface_presented_this_frame,
            native_surface_presentation_ready: self.native_surface_presentation_ready(),
            native_surface_terminal_frame_presented_this_frame: gpu_pass
                .native_surface_terminal_frame_presented_this_frame,
            native_surface_terminal_frame_ready: self.native_surface_terminal_frame_ready(),
            native_surface_present_count: self.native_surface_present_count(),
            native_surface_terminal_frame_count: self.native_surface_terminal_frame_count(),
            native_surface_terminal_cell_count: gpu_pass.native_surface_terminal_cell_count,
            native_surface_terminal_overlay_range_count: surface_frame_upload.overlay_range_count,
            native_surface_terminal_vertex_count: gpu_pass.native_surface_terminal_vertex_count,
            glyph_atlas_ready: gpu_pass.glyph_atlas_ready,
            glyph_atlas_glyph_count: self.glyph_atlas.glyph_count(),
            glyph_atlas_revision: self.glyph_atlas.revision(),
            glyph_atlas_backend: self.glyph_atlas.backend_name().to_string(),
            glyph_atlas_target_backend: self.glyph_atlas.target_backend_name().to_string(),
            glyph_atlas_real_font_ready: self.glyph_atlas.real_font_ready(),
            glyph_layout_backend: self.glyph_atlas.layout_backend_name().to_string(),
            glyph_layout_target_backend: self.glyph_atlas.layout_target_backend_name().to_string(),
            glyph_layout_shaping_ready: self.glyph_atlas.shaping_ready(),
            glyph_layout_cluster_count: surface_frame_upload.glyph_layout_cluster_count,
            glyph_layout_wide_cluster_count: surface_frame_upload.glyph_layout_wide_cluster_count,
            glyph_layout_zero_width_cluster_count: surface_frame_upload
                .glyph_layout_zero_width_cluster_count,
            glyph_layout_shaped_glyph_count: surface_frame_upload.glyph_layout_shaped_glyph_count,
            glyph_atlas_uploaded: gpu_pass.glyph_atlas_uploaded,
            dirty_row_upload_count: dirty_row_upload.row_count,
            dirty_row_upload_bytes: dirty_row_upload.byte_len(),
            gpu_dirty_row_upload_count: gpu_pass.gpu_dirty_row_upload_count,
            gpu_dirty_row_upload_bytes: gpu_pass.gpu_dirty_row_upload_bytes,
            persistent_device_ready: self.runtime.persistent_device_ready(),
            reused_gpu_device: gpu_pass.reused_gpu_device,
            viewport_changed,
            content_changed,
            offscreen_gpu_pass: gpu_pass.offscreen_gpu_pass,
            gpu_backend: gpu_pass.gpu_backend,
            gpu_adapter: gpu_pass.gpu_adapter,
            width_px,
            height_px,
            cols,
            rows,
            cell_width_px: (width_px / cols).max(1),
            cell_height_px: (height_px / rows).max(1),
            active_screen,
            scrollback_len,
            dirty_rows,
            visible_line_count,
            styled_run_count,
            text_cell_count,
            cursor_column,
            cursor_row,
            cursor_visible,
            frame_signature,
            notes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RendererDimensions {
    width_px: u32,
    height_px: u32,
    cols: u32,
    rows: u32,
}

impl RendererDimensions {
    fn update(&mut self, width_px: u32, height_px: u32, cols: u32, rows: u32) -> bool {
        let next = Self {
            width_px,
            height_px,
            cols,
            rows,
        };
        let changed = *self != next;
        *self = next;
        changed
    }
}

#[derive(Debug, Clone)]
struct GpuPassResult {
    offscreen_gpu_pass: bool,
    gpu_backend: Option<String>,
    gpu_adapter: Option<String>,
    glyph_atlas_ready: bool,
    glyph_atlas_uploaded: bool,
    gpu_dirty_row_upload_count: usize,
    gpu_dirty_row_upload_bytes: usize,
    native_surface_configured: bool,
    native_surface_presented_this_frame: bool,
    native_surface_presentation_ready: bool,
    native_surface_terminal_frame_presented_this_frame: bool,
    native_surface_present_count: u64,
    native_surface_terminal_cell_count: usize,
    native_surface_terminal_vertex_count: usize,
    reused_gpu_device: bool,
    notes: Vec<String>,
}

#[derive(Debug, Clone)]
struct GpuSurfaceUpdate {
    wgpu_surface_configured: bool,
    presentation_ready: bool,
    present_count: u64,
    notes: Vec<String>,
}

struct DirtyRowUpload {
    row_count: usize,
    payload: Vec<u8>,
}

impl DirtyRowUpload {
    fn from_frame(
        grid: Option<&TerminalGridSnapshot>,
        history_rows: &[TerminalRow],
        dirty_rows: &[usize],
        glyph_atlas: &GlyphAtlas,
        layout_cache: &mut GlyphLayoutCache,
    ) -> Self {
        let mut payload = Vec::new();
        let mut row_count = 0usize;
        for row in dirty_rows {
            let line = match grid {
                Some(grid) => grid.lines.get(*row).cloned(),
                None => history_rows.get(*row).map(history_row_text),
            };
            let Some(line) = line else {
                continue;
            };

            row_count += 1;
            let layout = layout_cache.layout_for_text(&line, u32::MAX, glyph_atlas);

            push_u32(&mut payload, *row as u32);
            push_u32(&mut payload, layout.clusters.len() as u32);
            for cluster in layout.clusters {
                push_u32(&mut payload, cluster.cell_start);
                push_u32(&mut payload, cluster.cell_width);
                push_u32(&mut payload, glyph_atlas.glyph_index(cluster.glyph));
                push_u32(&mut payload, cluster.zero_width_count);
                push_u32(&mut payload, cluster.x_offset_cell_fraction as i32 as u32);
                push_u32(&mut payload, cluster.y_offset_cell_fraction as i32 as u32);
            }
        }

        Self { row_count, payload }
    }

    fn byte_len(&self) -> usize {
        self.payload.len()
    }
}

#[cfg_attr(not(feature = "native-integrations"), allow(dead_code))]
struct SurfaceFrameUpload {
    cell_count: usize,
    overlay_range_count: usize,
    vertex_count: usize,
    glyph_layout_cluster_count: usize,
    glyph_layout_wide_cluster_count: usize,
    glyph_layout_zero_width_cluster_count: usize,
    glyph_layout_shaped_glyph_count: usize,
    payload: Vec<u8>,
}

impl SurfaceFrameUpload {
    fn from_frame(
        grid: Option<&TerminalGridSnapshot>,
        history_rows: &[TerminalRow],
        width_px: u32,
        height_px: u32,
        glyph_atlas: &GlyphAtlas,
        layout_cache: &mut GlyphLayoutCache,
        overlay_state: &RendererOverlayState,
        theme: &TerminalTheme,
    ) -> Self {
        let width = width_px.max(1) as f32;
        let height = height_px.max(1) as f32;
        let mut builder = SurfaceFrameBuilder::new(width, height, glyph_atlas, layout_cache, theme);

        match grid {
            Some(grid) => builder.push_grid(grid, overlay_state),
            None => builder.push_history_rows(history_rows),
        }

        builder.finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GlyphLayoutPlan {
    clusters: Vec<GlyphLayoutCluster>,
    cell_count: u32,
    wide_cluster_count: usize,
    zero_width_cluster_count: usize,
    shaped_glyph_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlyphLayoutCluster {
    glyph: GlyphKey,
    cell_start: u32,
    cell_width: u32,
    zero_width_count: u32,
    x_offset_cell_fraction: i16,
    y_offset_cell_fraction: i16,
}

#[cfg_attr(not(feature = "native-integrations"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum GlyphKey {
    Codepoint(char),
    FontGlyph { font: u16, glyph: u16 },
}

struct GlyphLayoutCache {
    plans: BTreeMap<GlyphLayoutCacheKey, GlyphLayoutPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct GlyphLayoutCacheKey {
    text: String,
    max_cells: u32,
}

impl GlyphLayoutCache {
    const MAX_ENTRIES: usize = 4096;

    fn new() -> Self {
        Self {
            plans: BTreeMap::new(),
        }
    }

    fn layout_for_text(
        &mut self,
        text: &str,
        max_cells: u32,
        glyph_atlas: &GlyphAtlas,
    ) -> GlyphLayoutPlan {
        let key = GlyphLayoutCacheKey {
            text: text.to_string(),
            max_cells,
        };
        if let Some(plan) = self.plans.get(&key) {
            return plan.clone();
        }

        let plan = GlyphLayoutPlan::from_text(text, max_cells, glyph_atlas);
        if self.plans.len() >= Self::MAX_ENTRIES {
            self.plans.clear();
        }
        self.plans.insert(key, plan.clone());
        plan
    }
}

impl GlyphLayoutPlan {
    fn from_text(text: &str, max_cells: u32, glyph_atlas: &GlyphAtlas) -> Self {
        glyph_atlas
            .shape_text(text, max_cells)
            .unwrap_or_else(|| Self::from_terminal_cells(text, max_cells))
    }

    fn from_terminal_cells(text: &str, max_cells: u32) -> Self {
        let mut clusters: Vec<GlyphLayoutCluster> = Vec::new();
        let mut cell_count = 0u32;
        let mut wide_cluster_count = 0usize;
        let mut zero_width_cluster_count = 0usize;

        for glyph in text.chars() {
            let glyph_cells = terminal_cell_width(glyph);
            if glyph_cells == 0 {
                if let Some(cluster) = clusters.last_mut() {
                    cluster.zero_width_count = cluster.zero_width_count.saturating_add(1);
                    zero_width_cluster_count = zero_width_cluster_count.saturating_add(1);
                }
                continue;
            }

            if cell_count >= max_cells {
                break;
            }

            let cell_width = glyph_cells.min(max_cells.saturating_sub(cell_count));
            if cell_width == 0 {
                break;
            }
            if glyph_cells > 1 {
                wide_cluster_count = wide_cluster_count.saturating_add(1);
            }

            clusters.push(GlyphLayoutCluster {
                glyph: GlyphKey::Codepoint(glyph),
                cell_start: cell_count,
                cell_width,
                zero_width_count: 0,
                x_offset_cell_fraction: 0,
                y_offset_cell_fraction: 0,
            });
            cell_count = cell_count.saturating_add(cell_width);
        }

        Self {
            clusters,
            cell_count,
            wide_cluster_count,
            zero_width_cluster_count,
            shaped_glyph_count: 0,
        }
    }
}

struct SurfaceFrameBuilder<'a> {
    width: f32,
    height: f32,
    glyph_atlas: &'a GlyphAtlas,
    layout_cache: &'a mut GlyphLayoutCache,
    theme: &'a TerminalTheme,
    atlas_extent: GlyphAtlasExtent,
    vertices: Vec<SurfaceVertex>,
    cell_count: usize,
    overlay_range_count: usize,
    glyph_layout_cluster_count: usize,
    glyph_layout_wide_cluster_count: usize,
    glyph_layout_zero_width_cluster_count: usize,
    glyph_layout_shaped_glyph_count: usize,
}

impl<'a> SurfaceFrameBuilder<'a> {
    fn new(
        width: f32,
        height: f32,
        glyph_atlas: &'a GlyphAtlas,
        layout_cache: &'a mut GlyphLayoutCache,
        theme: &'a TerminalTheme,
    ) -> Self {
        Self {
            width,
            height,
            glyph_atlas,
            layout_cache,
            theme,
            atlas_extent: glyph_atlas.extent(),
            vertices: Vec::new(),
            cell_count: 0,
            overlay_range_count: 0,
            glyph_layout_cluster_count: 0,
            glyph_layout_wide_cluster_count: 0,
            glyph_layout_zero_width_cluster_count: 0,
            glyph_layout_shaped_glyph_count: 0,
        }
    }

    fn push_grid(&mut self, grid: &TerminalGridSnapshot, overlay_state: &RendererOverlayState) {
        let cols = grid.cols.max(1);
        let row_count = grid.lines.len().max(1) as u32;
        let cell_width = self.width / cols as f32;
        let row_slot_height = self.height / row_count as f32;
        let row_height = terminal_surface_row_height(cell_width, row_slot_height);

        for row in 0..grid.lines.len() {
            let y = terminal_surface_row_y(row as u32, row_slot_height, row_height);
            self.push_grid_row_overlays(row as u32, cols, y, cell_width, row_height, overlay_state);
            let line = &grid.lines[row];
            if let Some(styled_line) = grid
                .styled_lines
                .get(row)
                .filter(|line| !line.runs.is_empty())
            {
                let mut consumed_cells = 0u32;
                for run in &styled_line.runs {
                    consumed_cells = self.push_run(
                        &run.text,
                        run.style,
                        consumed_cells,
                        cols,
                        y,
                        cell_width,
                        row_height,
                    );
                    if consumed_cells >= cols {
                        break;
                    }
                }
            } else {
                self.push_run(
                    line,
                    TerminalGridStyle::default(),
                    0,
                    cols,
                    y,
                    cell_width,
                    row_height,
                );
            }
        }

        self.push_cursor(grid, cell_width, row_slot_height, row_height);
    }

    fn push_grid_row_overlays(
        &mut self,
        row: u32,
        cols: u32,
        row_y: f32,
        cell_width: f32,
        row_height: f32,
        overlay_state: &RendererOverlayState,
    ) {
        for range in overlay_state.ranges.iter().filter(|range| range.row == row) {
            let start_col = range.start_col.min(cols);
            let end_col = range.end_col.min(cols);
            if end_col <= start_col {
                continue;
            }
            let x = start_col as f32 * cell_width;
            let width = (end_col - start_col) as f32 * cell_width;
            self.push_solid_rect(
                x,
                row_y,
                width,
                row_height,
                range.kind.surface_color(self.theme),
            );
            self.overlay_range_count = self.overlay_range_count.saturating_add(1);
        }
    }

    fn push_history_rows(&mut self, rows: &[TerminalRow]) {
        let row_count = rows.len().max(1) as u32;
        let cols = rows
            .iter()
            .map(|row| {
                history_row_text(row)
                    .chars()
                    .map(terminal_cell_width)
                    .sum::<u32>()
            })
            .max()
            .unwrap_or(80)
            .max(1);
        let cell_width = self.width / cols as f32;
        let row_slot_height = self.height / row_count as f32;
        let row_height = terminal_surface_row_height(cell_width, row_slot_height);

        for (row_index, row) in rows.iter().enumerate() {
            let y = terminal_surface_row_y(row_index as u32, row_slot_height, row_height);
            self.push_run(
                &history_row_text(row),
                row_style_to_grid_style(row.style, self.theme),
                0,
                cols,
                y,
                cell_width,
                row_height,
            );
        }
    }

    fn push_run(
        &mut self,
        text: &str,
        style: TerminalGridStyle,
        mut consumed_cells: u32,
        cols: u32,
        row_y: f32,
        cell_width: f32,
        row_height: f32,
    ) -> u32 {
        if text.is_empty() || consumed_cells >= cols {
            return consumed_cells;
        }

        let (foreground, background) = style_colors(style, self.theme);
        let layout = self.layout_cache.layout_for_text(
            text,
            cols.saturating_sub(consumed_cells),
            self.glyph_atlas,
        );
        let run_cells = layout.cell_count;
        if run_cells == 0 {
            return consumed_cells;
        }

        let x = consumed_cells as f32 * cell_width;
        let run_width = run_cells as f32 * cell_width;
        if let Some(background) = background {
            self.push_solid_rect(x, row_y, run_width, row_height, background);
        }

        self.glyph_layout_cluster_count = self
            .glyph_layout_cluster_count
            .saturating_add(layout.clusters.len());
        self.glyph_layout_wide_cluster_count = self
            .glyph_layout_wide_cluster_count
            .saturating_add(layout.wide_cluster_count);
        self.glyph_layout_zero_width_cluster_count = self
            .glyph_layout_zero_width_cluster_count
            .saturating_add(layout.zero_width_cluster_count);
        self.glyph_layout_shaped_glyph_count = self
            .glyph_layout_shaped_glyph_count
            .saturating_add(layout.shaped_glyph_count);

        let fallback_glyph_height = terminal_glyph_quad_height(cell_width, row_height);
        let fallback_glyph_y_base = row_y + ((row_height - fallback_glyph_height) / 2.0).max(0.0);

        for cluster in layout.clusters {
            let glyph_x = (consumed_cells + cluster.cell_start) as f32 * cell_width
                + glyph_offset_fraction(cluster.x_offset_cell_fraction) * cell_width;
            let glyph_y_offset = glyph_offset_fraction(cluster.y_offset_cell_fraction) * row_height;
            self.push_glyph(
                cluster.glyph,
                glyph_x,
                cluster.cell_width as f32 * cell_width,
                row_y,
                row_height,
                glyph_y_offset,
                fallback_glyph_y_base,
                fallback_glyph_height,
                foreground,
            );
        }
        consumed_cells = consumed_cells.saturating_add(run_cells);

        if style.underline {
            let line_height = (row_height * 0.06).max(1.0);
            self.push_solid_rect(
                x,
                row_y + row_height * 0.82,
                run_width,
                line_height,
                foreground,
            );
        }
        if style.strikethrough {
            let line_height = (row_height * 0.05).max(1.0);
            self.push_solid_rect(
                x,
                row_y + row_height * 0.52,
                run_width,
                line_height,
                foreground,
            );
        }

        consumed_cells
    }

    fn push_cursor(
        &mut self,
        grid: &TerminalGridSnapshot,
        cell_width: f32,
        row_slot_height: f32,
        row_height: f32,
    ) {
        if !grid.cursor_visible {
            return;
        }
        let row = grid.cursor_row as usize;
        if row >= grid.lines.len() {
            return;
        }

        let column = grid.cursor_column.min(grid.cols.saturating_sub(1));
        let x = column as f32 * cell_width;
        let y = terminal_surface_row_y(row as u32, row_slot_height, row_height);
        let color = SurfaceColor::from_grid(self.theme.cursor).with_alpha(0.92);
        match grid.cursor_shape {
            TerminalCursorShape::Block => {
                self.push_solid_rect(x, y + 2.0, cell_width, (row_height - 4.0).max(1.0), color);
            }
            TerminalCursorShape::Underline => {
                self.push_solid_rect(x, y + (row_height - 4.0).max(0.0), cell_width, 2.0, color);
            }
            TerminalCursorShape::Bar => {
                self.push_solid_rect(x, y + 2.0, 2.0, (row_height - 4.0).max(1.0), color);
            }
        }
    }

    fn push_glyph(
        &mut self,
        glyph: GlyphKey,
        x: f32,
        target_cell_width: f32,
        row_y: f32,
        row_height: f32,
        glyph_y_offset: f32,
        fallback_y: f32,
        fallback_height: f32,
        color: SurfaceColor,
    ) {
        let glyph_index = self.glyph_atlas.glyph_index(glyph);
        let glyph_x = (glyph_index % GlyphAtlas::COLUMNS) * GlyphAtlas::CELL_WIDTH;
        let glyph_y = (glyph_index / GlyphAtlas::COLUMNS) * GlyphAtlas::CELL_HEIGHT;
        if let Some(metrics) = self.glyph_atlas.glyph_metrics(glyph) {
            let atlas_padding_x =
                ((GlyphAtlas::CELL_WIDTH as i32 - metrics.bitmap_width as i32) / 2).max(0) as u32;
            let atlas_padding_y =
                ((GlyphAtlas::CELL_HEIGHT as i32 - metrics.bitmap_height as i32) / 2).max(0) as u32;
            let sample_width = metrics
                .bitmap_width
                .min(GlyphAtlas::CELL_WIDTH.saturating_sub(atlas_padding_x));
            let sample_height = metrics
                .bitmap_height
                .min(GlyphAtlas::CELL_HEIGHT.saturating_sub(atlas_padding_y));
            if sample_width > 0 && sample_height > 0 {
                let line_height_px = (metrics.ascent - metrics.descent + metrics.line_gap)
                    .max(metrics.pixels_per_em);
                let advance_scale = target_cell_width / metrics.advance_width.max(1.0);
                let height_scale = (row_height * 0.92) / line_height_px.max(1.0);
                let scale = advance_scale.min(height_scale).max(0.01);
                let advance_width = metrics.advance_width * scale;
                let x_padding = ((target_cell_width - advance_width) / 2.0).max(0.0);
                let draw_x = x + x_padding + metrics.xmin as f32 * scale;
                let baseline = row_y
                    + ((row_height - line_height_px * scale) / 2.0).max(0.0)
                    + metrics.ascent * scale
                    + glyph_y_offset;
                let draw_y =
                    baseline - (metrics.ymin + metrics.bitmap_height as i32) as f32 * scale;
                let draw_width = sample_width as f32 * scale;
                let draw_height = sample_height as f32 * scale;
                let uv_left = (glyph_x + atlas_padding_x) as f32 / self.atlas_extent.width as f32;
                let uv_right = (glyph_x + atlas_padding_x + sample_width) as f32
                    / self.atlas_extent.width as f32;
                let uv_top = (glyph_y + atlas_padding_y) as f32 / self.atlas_extent.height as f32;
                let uv_bottom = (glyph_y + atlas_padding_y + sample_height) as f32
                    / self.atlas_extent.height as f32;
                self.push_glyph_rect(
                    draw_x,
                    draw_y,
                    draw_width,
                    draw_height,
                    color,
                    uv_left,
                    uv_right,
                    uv_top,
                    uv_bottom,
                );
                return;
            }
        }

        let uv_left = glyph_x as f32 / self.atlas_extent.width as f32;
        let uv_right = (glyph_x + GlyphAtlas::CELL_WIDTH) as f32 / self.atlas_extent.width as f32;
        let uv_top = glyph_y as f32 / self.atlas_extent.height as f32;
        let uv_bottom =
            (glyph_y + GlyphAtlas::CELL_HEIGHT) as f32 / self.atlas_extent.height as f32;
        self.push_glyph_rect(
            x,
            fallback_y,
            target_cell_width,
            fallback_height,
            color,
            uv_left,
            uv_right,
            uv_top,
            uv_bottom,
        );
    }

    fn push_glyph_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: SurfaceColor,
        uv_left: f32,
        uv_right: f32,
        uv_top: f32,
        uv_bottom: f32,
    ) {
        self.push_rect(
            x,
            y,
            width,
            height,
            color,
            [
                [uv_left, uv_bottom],
                [uv_right, uv_bottom],
                [uv_left, uv_top],
                [uv_right, uv_top],
            ],
            SurfaceVertexMode::Glyph,
        );
        self.cell_count = self.cell_count.saturating_add(1);
    }

    fn push_solid_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: SurfaceColor) {
        self.push_rect(
            x,
            y,
            width,
            height,
            color,
            [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
            SurfaceVertexMode::Solid,
        );
    }

    fn push_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: SurfaceColor,
        uv: [[f32; 2]; 4],
        mode: SurfaceVertexMode,
    ) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }

        let x0 = (x / self.width) * 2.0 - 1.0;
        let x1 = ((x + width) / self.width) * 2.0 - 1.0;
        let y0 = 1.0 - (y / self.height) * 2.0;
        let y1 = 1.0 - ((y + height) / self.height) * 2.0;
        let color = color.components();
        let mode = mode.value();

        self.vertices.extend_from_slice(&[
            SurfaceVertex::new([x0, y1], uv[0], color, mode),
            SurfaceVertex::new([x1, y1], uv[1], color, mode),
            SurfaceVertex::new([x0, y0], uv[2], color, mode),
            SurfaceVertex::new([x1, y1], uv[1], color, mode),
            SurfaceVertex::new([x1, y0], uv[3], color, mode),
            SurfaceVertex::new([x0, y0], uv[2], color, mode),
        ]);
    }

    fn finish(self) -> SurfaceFrameUpload {
        let mut payload = Vec::with_capacity(self.vertices.len() * SURFACE_VERTEX_STRIDE as usize);
        for vertex in &self.vertices {
            vertex.write_bytes(&mut payload);
        }

        SurfaceFrameUpload {
            cell_count: self.cell_count,
            overlay_range_count: self.overlay_range_count,
            vertex_count: self.vertices.len(),
            glyph_layout_cluster_count: self.glyph_layout_cluster_count,
            glyph_layout_wide_cluster_count: self.glyph_layout_wide_cluster_count,
            glyph_layout_zero_width_cluster_count: self.glyph_layout_zero_width_cluster_count,
            glyph_layout_shaped_glyph_count: self.glyph_layout_shaped_glyph_count,
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SurfaceVertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
    color: [f32; 4],
    mode: f32,
}

impl SurfaceVertex {
    fn new(position: [f32; 2], tex_coord: [f32; 2], color: [f32; 4], mode: f32) -> Self {
        Self {
            position,
            tex_coord,
            color,
            mode,
        }
    }

    fn write_bytes(&self, output: &mut Vec<u8>) {
        push_f32(output, self.position[0]);
        push_f32(output, self.position[1]);
        push_f32(output, self.tex_coord[0]);
        push_f32(output, self.tex_coord[1]);
        for component in self.color {
            push_f32(output, component);
        }
        push_f32(output, self.mode);
    }
}

#[derive(Debug, Clone, Copy)]
enum SurfaceVertexMode {
    Solid,
    Glyph,
}

impl SurfaceVertexMode {
    fn value(self) -> f32 {
        match self {
            Self::Solid => 0.0,
            Self::Glyph => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SurfaceColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl SurfaceColor {
    const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    fn from_grid(color: TerminalGridColor) -> Self {
        Self::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            1.0,
        )
    }

    fn from_theme_rgba(color: TerminalThemeRgba) -> Self {
        Self::new(
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        )
    }

    fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    fn components(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

const SURFACE_VERTEX_STRIDE: u64 = 36;
fn style_colors(
    style: TerminalGridStyle,
    theme: &TerminalTheme,
) -> (SurfaceColor, Option<SurfaceColor>) {
    let theme_foreground = SurfaceColor::from_grid(theme.foreground);
    let theme_background = SurfaceColor::from_grid(theme.background);
    let default_fg = if style.faint {
        theme_foreground.with_alpha(0.7)
    } else {
        theme_foreground
    };
    if style.inverse {
        let foreground = style
            .bg
            .map(SurfaceColor::from_grid)
            .unwrap_or(theme_background);
        let background = style
            .fg
            .map(SurfaceColor::from_grid)
            .unwrap_or(theme_foreground);
        return (foreground, Some(background));
    }

    (
        style.fg.map(SurfaceColor::from_grid).unwrap_or(default_fg),
        style.bg.map(SurfaceColor::from_grid),
    )
}

fn row_style_to_grid_style(style: TerminalRowStyle, theme: &TerminalTheme) -> TerminalGridStyle {
    let fg = match style {
        TerminalRowStyle::Command => Some(theme.accent),
        TerminalRowStyle::Muted => Some(theme.muted),
        TerminalRowStyle::Success => Some(theme.success),
        TerminalRowStyle::Prompt => Some(theme.accent),
        TerminalRowStyle::Warning => Some(theme.warning),
    };
    TerminalGridStyle {
        fg,
        ..Default::default()
    }
}

fn terminal_cell_width(glyph: char) -> u32 {
    if glyph == '\t' {
        return 4;
    }
    if glyph.is_control() {
        return 0;
    }

    let codepoint = glyph as u32;
    if (0x0300..=0x036f).contains(&codepoint)
        || (0x1ab0..=0x1aff).contains(&codepoint)
        || (0x1dc0..=0x1dff).contains(&codepoint)
        || (0x20d0..=0x20ff).contains(&codepoint)
        || (0xfe20..=0xfe2f).contains(&codepoint)
    {
        return 0;
    }
    if (0x1100..=0x115f).contains(&codepoint)
        || (0x2329..=0x232a).contains(&codepoint)
        || (0x2e80..=0xa4cf).contains(&codepoint)
        || (0xac00..=0xd7a3).contains(&codepoint)
        || (0xf900..=0xfaff).contains(&codepoint)
        || (0xfe10..=0xfe19).contains(&codepoint)
        || (0xfe30..=0xfe6f).contains(&codepoint)
        || (0xff00..=0xff60).contains(&codepoint)
        || (0xffe0..=0xffe6).contains(&codepoint)
        || (0x1f300..=0x1faff).contains(&codepoint)
    {
        return 2;
    }

    1
}

fn terminal_text_cell_count(text: &str) -> usize {
    text.chars().map(terminal_cell_width).sum::<u32>() as usize
}

fn terminal_surface_row_height(cell_width: f32, available_row_height: f32) -> f32 {
    let inferred_font_size = cell_width / 0.56;
    let desired = inferred_font_size * 1.25 + 9.0;
    desired.clamp(1.0, available_row_height.max(1.0))
}

fn terminal_surface_row_y(row: u32, row_slot_height: f32, row_height: f32) -> f32 {
    row as f32 * row_slot_height + ((row_slot_height - row_height) / 2.0).max(0.0)
}

fn terminal_glyph_quad_height(cell_width: f32, row_height: f32) -> f32 {
    let inferred_font_size = cell_width / 0.56;
    (inferred_font_size * 1.02).clamp(1.0, row_height.max(1.0))
}

#[cfg(feature = "native-integrations")]
#[derive(Debug, Clone, Copy)]
struct TerminalTextSegment {
    byte_start: u32,
    byte_end: u32,
    cell_start: u32,
    cell_width: u32,
    zero_width_count: u32,
}

#[cfg(feature = "native-integrations")]
struct TerminalTextSegments {
    segments: Vec<TerminalTextSegment>,
    cell_count: u32,
}

#[cfg(feature = "native-integrations")]
impl TerminalTextSegments {
    fn new(text: &str, max_cells: u32) -> Self {
        let mut segments: Vec<TerminalTextSegment> = Vec::new();
        let mut cell_count = 0u32;

        for (byte_start, glyph) in text.char_indices() {
            let byte_end = byte_start + glyph.len_utf8();
            let glyph_cells = terminal_cell_width(glyph);

            if glyph_cells == 0 {
                if let Some(segment) = segments.last_mut() {
                    segment.byte_end = byte_end as u32;
                    segment.zero_width_count = segment.zero_width_count.saturating_add(1);
                }
                continue;
            }

            if cell_count >= max_cells {
                break;
            }

            let cell_width = glyph_cells.min(max_cells.saturating_sub(cell_count));
            if cell_width == 0 {
                break;
            }

            segments.push(TerminalTextSegment {
                byte_start: byte_start as u32,
                byte_end: byte_end as u32,
                cell_start: cell_count,
                cell_width,
                zero_width_count: 0,
            });
            cell_count = cell_count.saturating_add(cell_width);
        }

        Self {
            segments,
            cell_count,
        }
    }

    fn segment_for_cluster(&self, cluster: u32) -> Option<TerminalTextSegment> {
        self.segments
            .iter()
            .copied()
            .find(|segment| segment.byte_start <= cluster && cluster < segment.byte_end)
            .or_else(|| {
                self.segments
                    .iter()
                    .copied()
                    .find(|segment| segment.byte_start >= cluster)
            })
    }
}

#[cfg(feature = "native-integrations")]
fn hb_position_to_cell_fraction(value: i32) -> i16 {
    (value / 64).clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

fn glyph_offset_fraction(value: i16) -> f32 {
    value as f32 / 1000.0
}

fn push_f32(output: &mut Vec<u8>, value: f32) {
    output.extend_from_slice(&value.to_le_bytes());
}

struct GlyphAtlas {
    glyphs: BTreeSet<GlyphKey>,
    glyph_indices: BTreeMap<GlyphKey, u32>,
    glyph_metrics: BTreeMap<GlyphKey, GlyphAtlasGlyphMetrics>,
    rasterizer: GlyphAtlasRasterizer,
    revision: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GlyphAtlasGlyphMetrics {
    bitmap_width: u32,
    bitmap_height: u32,
    xmin: i32,
    ymin: i32,
    advance_width: f32,
    ascent: f32,
    descent: f32,
    line_gap: f32,
    pixels_per_em: f32,
}

enum GlyphAtlasRasterizer {
    #[cfg(feature = "native-integrations")]
    FontdueSystemFont(Arc<SystemFontRasterizer>),
    ProceduralCell,
}

impl GlyphAtlas {
    const CELL_WIDTH: u32 = 36;
    const CELL_HEIGHT: u32 = 72;
    const COLUMNS: u32 = 16;

    fn new() -> Self {
        let mut glyphs = BTreeSet::new();
        for codepoint in 32u8..=126u8 {
            glyphs.insert(GlyphKey::Codepoint(codepoint as char));
        }
        glyphs.insert(GlyphKey::Codepoint('\u{fffd}'));

        let rasterizer = GlyphAtlasRasterizer::new();
        let mut atlas = Self {
            glyphs,
            glyph_indices: BTreeMap::new(),
            glyph_metrics: BTreeMap::new(),
            rasterizer,
            revision: 1,
        };
        atlas.refresh_glyph_indices();
        atlas.refresh_glyph_metrics();
        atlas
    }

    fn refresh_glyph_indices(&mut self) {
        self.glyph_indices.clear();
        for (index, glyph) in self.glyphs.iter().copied().enumerate() {
            self.glyph_indices.insert(glyph, index as u32);
        }
    }

    fn refresh_glyph_metrics(&mut self) {
        self.glyph_metrics.clear();
        for glyph in &self.glyphs {
            if let Some(metrics) = self.rasterizer.glyph_metrics(*glyph) {
                self.glyph_metrics.insert(*glyph, metrics);
            }
        }
    }

    fn ensure_frame(
        &mut self,
        grid: Option<&TerminalGridSnapshot>,
        history_rows: &[TerminalRow],
        layout_cache: &mut GlyphLayoutCache,
    ) -> usize {
        let mut added = 0usize;
        match grid {
            Some(grid) => {
                for line in &grid.lines {
                    added += self.ensure_text(line, layout_cache);
                }
            }
            None => {
                for row in history_rows {
                    added += self.ensure_text(&history_row_text(row), layout_cache);
                }
            }
        }

        if added > 0 {
            self.refresh_glyph_indices();
            self.revision = self.revision.saturating_add(1);
        }

        added
    }

    fn ensure_text(&mut self, text: &str, layout_cache: &mut GlyphLayoutCache) -> usize {
        let mut added = 0usize;
        let layout = layout_cache.layout_for_text(text, u32::MAX, self);
        for cluster in layout.clusters {
            if self.glyphs.insert(cluster.glyph) {
                if let Some(metrics) = self.rasterizer.glyph_metrics(cluster.glyph) {
                    self.glyph_metrics.insert(cluster.glyph, metrics);
                }
                added += 1;
            }
        }
        added
    }

    fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn backend_name(&self) -> &'static str {
        self.rasterizer.backend_name()
    }

    fn target_backend_name(&self) -> &'static str {
        self.rasterizer.target_backend_name()
    }

    fn real_font_ready(&self) -> bool {
        self.rasterizer.real_font_ready()
    }

    fn layout_backend_name(&self) -> &'static str {
        self.rasterizer.layout_backend_name()
    }

    fn layout_target_backend_name(&self) -> &'static str {
        "font-shaping-glyph-atlas"
    }

    fn shaping_ready(&self) -> bool {
        self.rasterizer.shaping_ready()
    }

    fn glyph_index(&self, glyph: GlyphKey) -> u32 {
        self.glyph_indices
            .get(&glyph)
            .copied()
            .or_else(|| {
                self.glyph_indices
                    .get(&GlyphKey::Codepoint('\u{fffd}'))
                    .copied()
            })
            .unwrap_or(0)
    }

    fn glyph_metrics(&self, glyph: GlyphKey) -> Option<GlyphAtlasGlyphMetrics> {
        self.glyph_metrics.get(&glyph).copied()
    }

    fn shape_text(&self, text: &str, max_cells: u32) -> Option<GlyphLayoutPlan> {
        self.rasterizer.shape_text(text, max_cells)
    }

    fn extent(&self) -> GlyphAtlasExtent {
        let rows = (self.glyphs.len() as u32).div_ceil(Self::COLUMNS).max(1);
        GlyphAtlasExtent {
            width: Self::COLUMNS * Self::CELL_WIDTH,
            height: rows * Self::CELL_HEIGHT,
        }
    }

    #[cfg(feature = "native-integrations")]
    fn rgba_pixels(&self) -> Vec<u8> {
        let extent = self.extent();
        let mut pixels = vec![0u8; (extent.width * extent.height * 4) as usize];
        for (index, glyph) in self.glyphs.iter().enumerate() {
            self.rasterizer.write_glyph_pixels(
                index as u32,
                *glyph,
                Self::CELL_WIDTH,
                Self::CELL_HEIGHT,
                Self::COLUMNS,
                extent.width,
                &mut pixels,
            );
        }
        pixels
    }
}

impl GlyphAtlasRasterizer {
    fn new() -> Self {
        #[cfg(feature = "native-integrations")]
        if let Some(rasterizer) = SystemFontRasterizer::cached() {
            return Self::FontdueSystemFont(rasterizer);
        }

        Self::ProceduralCell
    }

    fn backend_name(&self) -> &'static str {
        match self {
            #[cfg(feature = "native-integrations")]
            Self::FontdueSystemFont(_) => "fontdue-system-font-rasterizer",
            Self::ProceduralCell => "procedural-cell-rasterizer",
        }
    }

    fn target_backend_name(&self) -> &'static str {
        "font-shaping-glyph-atlas"
    }

    fn real_font_ready(&self) -> bool {
        match self {
            #[cfg(feature = "native-integrations")]
            Self::FontdueSystemFont(_) => true,
            Self::ProceduralCell => false,
        }
    }

    fn layout_backend_name(&self) -> &'static str {
        match self {
            #[cfg(feature = "native-integrations")]
            Self::FontdueSystemFont(_) => "rustybuzz-terminal-shaper",
            Self::ProceduralCell => "terminal-cell-cluster-layout",
        }
    }

    fn shaping_ready(&self) -> bool {
        match self {
            #[cfg(feature = "native-integrations")]
            Self::FontdueSystemFont(_) => true,
            Self::ProceduralCell => false,
        }
    }

    fn real_font_available() -> bool {
        #[cfg(feature = "native-integrations")]
        {
            SystemFontRasterizer::cached().is_some()
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            false
        }
    }

    fn shape_text(&self, text: &str, max_cells: u32) -> Option<GlyphLayoutPlan> {
        #[cfg(feature = "native-integrations")]
        if let Self::FontdueSystemFont(rasterizer) = self {
            return rasterizer.shape_text(text, max_cells);
        }

        let _ = (text, max_cells);
        None
    }

    fn glyph_metrics(&self, glyph: GlyphKey) -> Option<GlyphAtlasGlyphMetrics> {
        #[cfg(feature = "native-integrations")]
        if let Self::FontdueSystemFont(rasterizer) = self {
            return rasterizer.glyph_metrics(glyph);
        }

        let _ = glyph;
        None
    }

    #[cfg(feature = "native-integrations")]
    fn write_glyph_pixels(
        &self,
        index: u32,
        glyph: GlyphKey,
        cell_width: u32,
        cell_height: u32,
        columns: u32,
        atlas_width: u32,
        pixels: &mut [u8],
    ) {
        if let Self::FontdueSystemFont(rasterizer) = self {
            rasterizer.write_glyph_pixels(
                index,
                glyph,
                cell_width,
                cell_height,
                columns,
                atlas_width,
                pixels,
            );
            return;
        }

        write_procedural_glyph_pixels(
            index,
            glyph,
            cell_width,
            cell_height,
            columns,
            atlas_width,
            pixels,
        );
    }
}

#[cfg(feature = "native-integrations")]
struct SystemFontRasterizer {
    faces: Vec<SystemFontFace>,
}

#[cfg(feature = "native-integrations")]
struct SystemFontFace {
    font: fontdue::Font,
    data: Vec<u8>,
    collection_index: u32,
}

#[cfg(feature = "native-integrations")]
impl SystemFontRasterizer {
    const PIXELS_PER_EM: f32 = 48.0;
    const EMBEDDED_MONO_FONT: &'static [u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");
    const MAX_COLLECTION_FACES: u32 = 16;
    const SYSTEM_FALLBACK_PROBE_GLYPHS: &'static [char] = &['中', '漢'];

    fn cached() -> Option<Arc<Self>> {
        static SYSTEM_FONT_RASTERIZER: OnceLock<Option<Arc<SystemFontRasterizer>>> =
            OnceLock::new();
        SYSTEM_FONT_RASTERIZER
            .get_or_init(|| Self::load().map(Arc::new))
            .clone()
    }

    fn load() -> Option<Self> {
        let mut faces = Vec::new();
        if let Some(face) = load_system_font_face(Self::EMBEDDED_MONO_FONT.to_vec(), 0) {
            faces.push(face);
        }

        for candidate in system_font_candidates() {
            let Ok(bytes) = std::fs::read(&candidate) else {
                continue;
            };

            let Some(collection_index) = first_collection_face_supporting(
                &bytes,
                Self::SYSTEM_FALLBACK_PROBE_GLYPHS,
                Self::MAX_COLLECTION_FACES,
            ) else {
                continue;
            };
            if let Some(face) = load_system_font_face(bytes, collection_index) {
                faces.push(face);
                break;
            }
        }

        (!faces.is_empty()).then_some(Self { faces })
    }

    fn shape_text(&self, text: &str, max_cells: u32) -> Option<GlyphLayoutPlan> {
        if text.is_empty() || max_cells == 0 {
            return Some(GlyphLayoutPlan::from_terminal_cells(text, max_cells));
        }

        let segments = TerminalTextSegments::new(text, max_cells);
        let mut clusters = Vec::new();
        let mut wide_cluster_count = 0usize;
        let mut zero_width_cluster_count = 0usize;
        let mut shaped_glyph_count = 0usize;

        for run in self.font_runs(text) {
            let Some(face) = self.faces.get(run.font_index) else {
                continue;
            };
            let face = rustybuzz::Face::from_slice(&face.data, face.collection_index)?;
            let mut buffer = rustybuzz::UnicodeBuffer::new();
            buffer.set_direction(rustybuzz::Direction::LeftToRight);
            buffer.set_cluster_level(rustybuzz::BufferClusterLevel::MonotoneGraphemes);
            buffer.push_str(&text[run.byte_range.clone()]);
            let glyph_buffer = rustybuzz::shape(&face, &[], buffer);
            let infos = glyph_buffer.glyph_infos();
            let positions = glyph_buffer.glyph_positions();
            shaped_glyph_count = shaped_glyph_count.saturating_add(infos.len());

            for (index, info) in infos.iter().enumerate() {
                let cluster = run.byte_range.start as u32 + info.cluster;
                let segment = segments.segment_for_cluster(cluster)?;
                if segment.cell_width == 0 {
                    continue;
                }
                if segment.cell_start >= max_cells {
                    break;
                }

                let remaining_cells = max_cells.saturating_sub(segment.cell_start);
                let cell_width = segment.cell_width.min(remaining_cells);
                if cell_width == 0 {
                    break;
                }
                if cell_width > 1 {
                    wide_cluster_count = wide_cluster_count.saturating_add(1);
                }
                if segment.zero_width_count > 0 {
                    zero_width_cluster_count = zero_width_cluster_count.saturating_add(1);
                }

                let position = positions.get(index).copied().unwrap_or_default();
                clusters.push(GlyphLayoutCluster {
                    glyph: GlyphKey::FontGlyph {
                        font: run.font_index.min(u16::MAX as usize) as u16,
                        glyph: info.glyph_id.min(u16::MAX as u32) as u16,
                    },
                    cell_start: segment.cell_start,
                    cell_width,
                    zero_width_count: segment.zero_width_count,
                    x_offset_cell_fraction: hb_position_to_cell_fraction(position.x_offset),
                    y_offset_cell_fraction: hb_position_to_cell_fraction(-position.y_offset),
                });
            }
        }

        if clusters.is_empty() {
            return None;
        }

        Some(GlyphLayoutPlan {
            clusters,
            cell_count: segments.cell_count,
            wide_cluster_count,
            zero_width_cluster_count,
            shaped_glyph_count,
        })
    }

    fn glyph_metrics(&self, glyph: GlyphKey) -> Option<GlyphAtlasGlyphMetrics> {
        if matches!(glyph, GlyphKey::Codepoint(codepoint) if codepoint.is_whitespace() || codepoint.is_control())
        {
            return None;
        }

        let face = self.face_for_glyph(glyph)?;
        let metrics = match glyph {
            GlyphKey::Codepoint(glyph) => face.font.metrics(glyph, Self::PIXELS_PER_EM),
            GlyphKey::FontGlyph { glyph, .. } => {
                face.font.metrics_indexed(glyph, Self::PIXELS_PER_EM)
            }
        };
        if metrics.width == 0 || metrics.height == 0 || metrics.advance_width <= 0.0 {
            return None;
        }

        let (ascent, descent, line_gap) = face
            .font
            .horizontal_line_metrics(Self::PIXELS_PER_EM)
            .map(|metrics| (metrics.ascent, metrics.descent, metrics.line_gap))
            .unwrap_or((Self::PIXELS_PER_EM * 0.82, -Self::PIXELS_PER_EM * 0.18, 0.0));

        Some(GlyphAtlasGlyphMetrics {
            bitmap_width: metrics.width as u32,
            bitmap_height: metrics.height as u32,
            xmin: metrics.xmin,
            ymin: metrics.ymin,
            advance_width: metrics.advance_width,
            ascent,
            descent,
            line_gap,
            pixels_per_em: Self::PIXELS_PER_EM,
        })
    }

    fn write_glyph_pixels(
        &self,
        index: u32,
        glyph: GlyphKey,
        cell_width: u32,
        cell_height: u32,
        columns: u32,
        atlas_width: u32,
        pixels: &mut [u8],
    ) {
        if matches!(glyph, GlyphKey::Codepoint(' ')) {
            return;
        }

        let (metrics, bitmap) = match glyph {
            GlyphKey::Codepoint(glyph) if glyph.is_control() => return,
            GlyphKey::Codepoint(glyph) => self
                .face_for_codepoint(glyph)
                .map(|face| face.font.rasterize(glyph, Self::PIXELS_PER_EM))
                .unwrap_or_else(|| (fontdue::Metrics::default(), Vec::new())),
            GlyphKey::FontGlyph { font, glyph } => self
                .faces
                .get(font as usize)
                .map(|face| face.font.rasterize_indexed(glyph, Self::PIXELS_PER_EM))
                .unwrap_or_else(|| (fontdue::Metrics::default(), Vec::new())),
        };
        let should_fallback = should_write_visible_glyph_fallback(glyph, metrics);
        if bitmap.is_empty() || metrics.width == 0 || metrics.height == 0 {
            if should_fallback {
                write_procedural_glyph_pixels(
                    index,
                    glyph,
                    cell_width,
                    cell_height,
                    columns,
                    atlas_width,
                    pixels,
                );
            }
            return;
        }

        let origin_x = (index % columns) * cell_width;
        let origin_y = (index / columns) * cell_height;
        let x_offset = ((cell_width as i32 - metrics.width as i32) / 2).max(0);
        let y_offset = ((cell_height as i32 - metrics.height as i32) / 2).max(0);
        let mut wrote_pixel = false;

        for y in 0..metrics.height as u32 {
            let dest_y = y_offset + y as i32;
            if !(0..cell_height as i32).contains(&dest_y) {
                continue;
            }

            for x in 0..metrics.width as u32 {
                let dest_x = x_offset + x as i32;
                if !(0..cell_width as i32).contains(&dest_x) {
                    continue;
                }

                let alpha = bitmap[(y * metrics.width as u32 + x) as usize];
                if alpha == 0 {
                    continue;
                }
                wrote_pixel = true;

                let offset = (((origin_y + dest_y as u32) * atlas_width + origin_x + dest_x as u32)
                    * 4) as usize;
                pixels[offset] = 255;
                pixels[offset + 1] = 255;
                pixels[offset + 2] = 255;
                pixels[offset + 3] = alpha;
            }
        }

        if !wrote_pixel && should_fallback {
            write_procedural_glyph_pixels(
                index,
                glyph,
                cell_width,
                cell_height,
                columns,
                atlas_width,
                pixels,
            );
        }
    }

    fn font_runs(&self, text: &str) -> Vec<SystemFontRun> {
        let mut runs = Vec::new();
        let mut current_font_index = None::<usize>;
        let mut current_start = 0usize;
        let mut current_end = 0usize;

        for (byte_start, glyph) in text.char_indices() {
            let glyph_end = byte_start + glyph.len_utf8();
            let font_index = if terminal_cell_width(glyph) == 0 {
                current_font_index
                    .or_else(|| self.font_index_for_codepoint(glyph))
                    .unwrap_or(0)
            } else {
                self.font_index_for_codepoint(glyph).unwrap_or(0)
            };

            if current_font_index.is_some_and(|current| current != font_index) {
                runs.push(SystemFontRun {
                    font_index: current_font_index.unwrap_or(0),
                    byte_range: current_start..current_end,
                });
                current_start = byte_start;
            } else if current_font_index.is_none() {
                current_start = byte_start;
            }

            current_font_index = Some(font_index);
            current_end = glyph_end;
        }

        if let Some(font_index) = current_font_index {
            runs.push(SystemFontRun {
                font_index,
                byte_range: current_start..current_end,
            });
        }

        runs
    }

    fn font_index_for_codepoint(&self, glyph: char) -> Option<usize> {
        if glyph.is_whitespace() || glyph.is_control() {
            return Some(0);
        }

        self.faces
            .iter()
            .position(|face| face.font.lookup_glyph_index(glyph) != 0)
    }

    fn face_for_codepoint(&self, glyph: char) -> Option<&SystemFontFace> {
        let index = self.font_index_for_codepoint(glyph).unwrap_or(0);
        self.faces.get(index)
    }

    fn face_for_glyph(&self, glyph: GlyphKey) -> Option<&SystemFontFace> {
        match glyph {
            GlyphKey::Codepoint(glyph) => self.face_for_codepoint(glyph),
            GlyphKey::FontGlyph { font, .. } => self.faces.get(font as usize),
        }
    }
}

#[cfg(feature = "native-integrations")]
fn first_collection_face_supporting(
    bytes: &[u8],
    glyphs: &[char],
    max_collection_faces: u32,
) -> Option<u32> {
    let face_count = rustybuzz::ttf_parser::fonts_in_collection(bytes)
        .unwrap_or(1)
        .min(max_collection_faces);
    (0..face_count).find(|collection_index| {
        rustybuzz::Face::from_slice(bytes, *collection_index).is_some_and(|face| {
            glyphs
                .iter()
                .any(|glyph| face.glyph_index(*glyph).is_some())
        })
    })
}

#[cfg(feature = "native-integrations")]
fn should_write_visible_glyph_fallback(glyph: GlyphKey, metrics: fontdue::Metrics) -> bool {
    match glyph {
        GlyphKey::Codepoint(glyph) => !glyph.is_whitespace() && !glyph.is_control(),
        GlyphKey::FontGlyph { .. } => metrics.width > 0 && metrics.height > 0,
    }
}

#[cfg(feature = "native-integrations")]
struct SystemFontRun {
    font_index: usize,
    byte_range: std::ops::Range<usize>,
}

#[cfg(feature = "native-integrations")]
fn load_system_font_face(bytes: Vec<u8>, collection_index: u32) -> Option<SystemFontFace> {
    rustybuzz::Face::from_slice(&bytes, collection_index)?;
    let settings = fontdue::FontSettings {
        collection_index,
        ..fontdue::FontSettings::default()
    };
    let font = fontdue::Font::from_bytes(bytes.as_slice(), settings).ok()?;
    Some(SystemFontFace {
        font,
        data: bytes,
        collection_index,
    })
}

#[cfg(feature = "native-integrations")]
fn system_font_candidates() -> Vec<String> {
    let mut candidates = Vec::new();

    if let Ok(path) = std::env::var("SHELLOW_RENDERER_FONT_PATH")
        && !path.trim().is_empty()
    {
        candidates.push(path);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    candidates.extend(
        [
            "/System/Library/Fonts/Core/SFUIMono.ttf",
            "/System/Library/Fonts/Core/CourierNew.ttf",
            "/System/Library/Fonts/Core/Courier.ttf",
            "/System/Library/Fonts/SFNSMono.ttf",
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/Fonts/Supplemental/Courier New.ttf",
            "/System/Library/Fonts/Courier.ttc",
            "/System/Library/Fonts/Menlo.ttc",
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/LanguageSupport/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/System/Library/Fonts/Supplemental/Songti.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        ]
        .into_iter()
        .map(str::to_string),
    );

    #[cfg(target_os = "android")]
    candidates.extend(
        [
            "/system/fonts/RobotoMono-Regular.ttf",
            "/system/fonts/DroidSansMono.ttf",
            "/system/fonts/NotoSansMono-Regular.ttf",
            "/system/fonts/SysSans-Hans-Regular.ttf",
            "/system/fonts/SysFont-Hans-Regular.ttf",
            "/system/fonts/SysSans-Hant-Regular.ttf",
            "/system/fonts/SysFont-Hant-Regular.ttf",
            "/system/fonts/CarroisGothicSC-Regular.ttf",
            "/system/fonts/FZZWXBTOT_Uni.ttf",
            "/system/fonts/Roboto-Regular.ttf",
            "/system/fonts/NotoSansCJK-Regular.ttc",
            "/system/fonts/NotoSerifCJK-Regular.ttc",
            "/system/fonts/NotoSansSC-Regular.otf",
            "/system/fonts/NotoSansCJKsc-Regular.otf",
        ]
        .into_iter()
        .map(str::to_string),
    );

    #[cfg(target_os = "linux")]
    candidates.extend(
        [
            "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
            "/usr/share/fonts/truetype/liberation2/LiberationMono-Regular.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansMono-Regular.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJKsc-Regular.otf",
            "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
        ]
        .into_iter()
        .map(str::to_string),
    );

    candidates
}

#[cfg(feature = "native-integrations")]
fn write_procedural_glyph_pixels(
    index: u32,
    glyph: GlyphKey,
    cell_width: u32,
    cell_height: u32,
    columns: u32,
    atlas_width: u32,
    pixels: &mut [u8],
) {
    let origin_x = (index % columns) * cell_width;
    let origin_y = (index / columns) * cell_height;
    let seed = match glyph {
        GlyphKey::Codepoint(glyph) => glyph as u32,
        GlyphKey::FontGlyph { font, glyph } => 0x8000_0000 | ((font as u32) << 16) | glyph as u32,
    };

    for y in 0..cell_height {
        for x in 0..cell_width {
            let offset = (((origin_y + y) * atlas_width + origin_x + x) * 4) as usize;
            let edge = x == 0 || y == 0 || x + 1 == cell_width || y + 1 == cell_height;
            let alpha = if matches!(glyph, GlyphKey::Codepoint(' ')) {
                0
            } else if edge {
                48
            } else if ((seed.rotate_left(y) >> (x % 16)) & 1) == 1 {
                220
            } else {
                96
            };

            pixels[offset] = 255;
            pixels[offset + 1] = 255;
            pixels[offset + 2] = 255;
            pixels[offset + 3] = alpha;
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GlyphAtlasExtent {
    width: u32,
    height: u32,
}

fn push_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn history_row_text(row: &TerminalRow) -> String {
    match (row.prompt.is_empty(), row.text.is_empty()) {
        (true, true) => String::new(),
        (true, false) => row.text.clone(),
        (false, true) => row.prompt.clone(),
        (false, false) => format!("{} {}", row.prompt, row.text),
    }
}

pub fn backend_name() -> &'static str {
    if is_wgpu_available() {
        "wgpu-frame-api"
    } else {
        "snapshot-render-plan"
    }
}

pub fn target_backend_name() -> &'static str {
    "wgpu-native-surface"
}

pub fn is_wgpu_available() -> bool {
    cfg!(feature = "native-integrations")
}

pub fn is_native_surface_ready() -> bool {
    false
}

pub fn is_real_font_rasterizer_available() -> bool {
    GlyphAtlasRasterizer::real_font_available()
}

pub fn is_text_shaping_available() -> bool {
    GlyphAtlasRasterizer::real_font_available()
}

pub fn surface_platform_supported(kind: RendererSurfaceKind) -> bool {
    match kind {
        RendererSurfaceKind::CoreAnimationLayer => {
            cfg!(all(
                feature = "native-integrations",
                any(target_os = "ios", target_os = "macos")
            ))
        }
        RendererSurfaceKind::AndroidNativeWindow => {
            cfg!(all(feature = "native-integrations", target_os = "android"))
        }
    }
}

pub fn demo_frame_summary() -> String {
    render_terminal_frame(None, 0, 80, 24, 960, 480).summary()
}

pub fn render_terminal_frame(
    grid: Option<&TerminalGridSnapshot>,
    history_row_count: usize,
    terminal_cols: u32,
    terminal_rows: u32,
    width_px: u32,
    height_px: u32,
) -> TerminalRenderFrame {
    let mut renderer = TerminalRenderer::new(terminal_cols, terminal_rows);
    renderer.render_frame(
        grid,
        &[],
        history_row_count,
        terminal_cols,
        terminal_rows,
        width_px,
        height_px,
    )
}

fn pipeline_stage() -> &'static str {
    if is_wgpu_available() {
        "persistent-surface-frame-runtime"
    } else {
        "snapshot-frame-plan"
    }
}

fn dirty_rows_for(
    grid: Option<&TerminalGridSnapshot>,
    history_row_count: usize,
    rows: u32,
    visible_line_count: usize,
    content_changed: bool,
) -> Vec<usize> {
    match grid {
        Some(grid) if !grid.dirty_rows.is_empty() => grid.dirty_rows.clone(),
        Some(_) if content_changed => (0..visible_line_count.min(rows as usize)).collect(),
        Some(_) => Vec::new(),
        None if content_changed => (0..history_row_count.min(rows as usize)).collect(),
        None => Vec::new(),
    }
}

fn frame_signature(
    grid: Option<&TerminalGridSnapshot>,
    history_rows: &[TerminalRow],
    history_row_count: usize,
    width_px: u32,
    height_px: u32,
    overlay_state: &RendererOverlayState,
) -> String {
    let mut hasher = DefaultHasher::new();
    width_px.hash(&mut hasher);
    height_px.hash(&mut hasher);
    history_row_count.hash(&mut hasher);
    overlay_state.hash_into(&mut hasher);
    if let Some(grid) = grid {
        grid.cols.hash(&mut hasher);
        grid.rows.hash(&mut hasher);
        grid.cursor_column.hash(&mut hasher);
        grid.cursor_row.hash(&mut hasher);
        grid.cursor_visible.hash(&mut hasher);
        grid.active_screen.hash(&mut hasher);
        grid.scrollback_len.hash(&mut hasher);
        for line in &grid.lines {
            line.hash(&mut hasher);
        }
        for line in &grid.styled_lines {
            for run in &line.runs {
                run.text.hash(&mut hasher);
                hash_style(&run.style, &mut hasher);
            }
        }
    } else {
        for row in history_rows {
            row.prompt.hash(&mut hasher);
            row.text.hash(&mut hasher);
            format!("{:?}", row.style).hash(&mut hasher);
        }
    }
    format!("{:016x}", hasher.finish())
}

fn hash_style(style: &TerminalGridStyle, hasher: &mut DefaultHasher) {
    style.bold.hash(hasher);
    style.faint.hash(hasher);
    style.italic.hash(hasher);
    style.underline.hash(hasher);
    style.blink.hash(hasher);
    style.inverse.hash(hasher);
    style.strikethrough.hash(hasher);
    style
        .fg
        .map(|color| (color.r, color.g, color.b))
        .hash(hasher);
    style
        .bg
        .map(|color| (color.r, color.g, color.b))
        .hash(hasher);
}

fn ready_word(value: bool) -> &'static str {
    if value { "ready" } else { "pending" }
}

fn next_renderer_id() -> String {
    let id = NEXT_RENDERER_ID.fetch_add(1, Ordering::Relaxed);
    format!("shellow-renderer-{id}")
}

#[cfg(feature = "native-integrations")]
struct RendererRuntime {
    state: GpuRuntimeState,
}

#[cfg(feature = "native-integrations")]
#[allow(clippy::large_enum_variant)]
enum GpuRuntimeState {
    Pending,
    Ready(GpuRuntime),
    Failed(String),
}

#[cfg(feature = "native-integrations")]
struct GpuRuntime {
    instance: wgpu::Instance,
    adapter_handle: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    backend: String,
    adapter: String,
    format: wgpu::TextureFormat,
    glyph_atlas_texture: Option<wgpu::Texture>,
    glyph_atlas_revision: u64,
    glyph_atlas_upload_count: u64,
    dirty_rows_buffer: Option<wgpu::Buffer>,
    dirty_rows_buffer_capacity: u64,
    dirty_row_upload_count: u64,
    last_dirty_row_upload_bytes: usize,
    surface_pipeline: Option<GpuSurfacePipeline>,
    surface_vertex_buffer: Option<wgpu::Buffer>,
    surface_vertex_buffer_capacity: u64,
    surface_atlas_bind_group: Option<wgpu::BindGroup>,
    surface_atlas_bind_group_revision: u64,
    surface: Option<GpuSurfaceRuntime>,
}

#[cfg(feature = "native-integrations")]
struct GpuSurfaceRuntime {
    kind: RendererSurfaceKind,
    raw_handle: u64,
    width_px: u32,
    height_px: u32,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    present_count: u64,
    terminal_frame_present_count: u64,
}

#[cfg(feature = "native-integrations")]
struct GpuSurfacePipeline {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

#[cfg(feature = "native-integrations")]
#[derive(Debug, Clone, Copy)]
struct GpuSurfaceRenderResult {
    presented: bool,
    terminal_frame_presented: bool,
}

#[cfg(feature = "native-integrations")]
#[derive(Debug, Clone, Copy)]
struct GpuUploadStats {
    glyph_atlas_uploaded: bool,
    dirty_row_upload_count: usize,
    dirty_row_upload_bytes: usize,
}

#[cfg(feature = "native-integrations")]
impl RendererRuntime {
    fn new() -> Self {
        Self {
            state: GpuRuntimeState::Pending,
        }
    }

    fn persistent_device_ready(&self) -> bool {
        matches!(self.state, GpuRuntimeState::Ready(_))
    }

    fn glyph_atlas_ready(&self) -> bool {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.glyph_atlas_ready(),
            _ => false,
        }
    }

    fn glyph_atlas_upload_count(&self) -> u64 {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.glyph_atlas_upload_count,
            _ => 0,
        }
    }

    fn dirty_row_upload_count(&self) -> u64 {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.dirty_row_upload_count,
            _ => 0,
        }
    }

    fn last_dirty_row_upload_bytes(&self) -> usize {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.last_dirty_row_upload_bytes,
            _ => 0,
        }
    }

    fn gpu_backend(&self) -> Option<String> {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => Some(runtime.backend.clone()),
            _ => None,
        }
    }

    fn gpu_adapter(&self) -> Option<String> {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => Some(runtime.adapter.clone()),
            _ => None,
        }
    }

    fn failure_note(&self) -> Option<String> {
        match &self.state {
            GpuRuntimeState::Failed(error) => Some(format!("wgpu runtime init failed: {error}")),
            _ => None,
        }
    }

    fn render_frame(
        &mut self,
        width_px: u32,
        height_px: u32,
        glyph_atlas: &GlyphAtlas,
        dirty_rows: &DirtyRowUpload,
        surface_frame: &SurfaceFrameUpload,
        background: SurfaceColor,
    ) -> GpuPassResult {
        let reused_gpu_device = self.persistent_device_ready();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let runtime = self.ensure_ready()?;
            let upload_stats = runtime.upload_frame_inputs(glyph_atlas, dirty_rows)?;
            let native_surface_configured = runtime.surface_configured();
            let offscreen_gpu_pass = if native_surface_configured {
                false
            } else {
                runtime.render_offscreen(width_px, height_px, background)?;
                true
            };
            let surface_render = runtime.render_surface_frame(Some(surface_frame), background)?;
            Ok::<_, String>((
                runtime.backend.clone(),
                runtime.adapter.clone(),
                runtime.glyph_atlas_ready(),
                upload_stats,
                native_surface_configured,
                offscreen_gpu_pass,
                surface_render.presented,
                surface_render.terminal_frame_presented,
                runtime.surface_presentation_ready(),
                runtime.surface_present_count(),
                runtime.surface_terminal_frame_ready(),
            ))
        }));

        match result {
            Ok(Ok((
                backend,
                adapter,
                glyph_atlas_ready,
                upload_stats,
                native_surface_configured,
                offscreen_gpu_pass,
                native_surface_presented_this_frame,
                native_surface_terminal_frame_presented_this_frame,
                native_surface_presentation_ready,
                native_surface_present_count,
                _native_surface_terminal_frame_ready,
            ))) => GpuPassResult {
                offscreen_gpu_pass,
                gpu_backend: Some(backend),
                gpu_adapter: Some(adapter),
                glyph_atlas_ready,
                glyph_atlas_uploaded: upload_stats.glyph_atlas_uploaded,
                gpu_dirty_row_upload_count: upload_stats.dirty_row_upload_count,
                gpu_dirty_row_upload_bytes: upload_stats.dirty_row_upload_bytes,
                native_surface_configured,
                native_surface_presented_this_frame,
                native_surface_presentation_ready,
                native_surface_terminal_frame_presented_this_frame,
                native_surface_present_count,
                native_surface_terminal_cell_count:
                    if native_surface_terminal_frame_presented_this_frame {
                        surface_frame.cell_count
                    } else {
                        0
                    },
                native_surface_terminal_vertex_count:
                    if native_surface_terminal_frame_presented_this_frame {
                        surface_frame.vertex_count
                    } else {
                        0
                    },
                reused_gpu_device,
                notes: {
                    let mut notes = Vec::new();
                    if offscreen_gpu_pass {
                        notes.push(if reused_gpu_device {
                            "offscreen wgpu pass reused the persistent device/queue".to_string()
                        } else {
                            "offscreen wgpu pass created the persistent device/queue".to_string()
                        });
                    } else {
                        notes.push(if reused_gpu_device {
                            "native wgpu surface pass reused the persistent device/queue"
                                .to_string()
                        } else {
                            "native wgpu surface pass created the persistent device/queue"
                                .to_string()
                        });
                        notes.push(
                            "native surface is configured; offscreen wgpu pass was skipped"
                                .to_string(),
                        );
                    }
                    if native_surface_presented_this_frame {
                        notes.push(
                            "native wgpu surface frame was acquired, rendered, and presented"
                                .to_string(),
                        );
                    }
                    notes
                },
            },
            Ok(Err(error)) => GpuPassResult {
                offscreen_gpu_pass: false,
                gpu_backend: self.gpu_backend(),
                gpu_adapter: self.gpu_adapter(),
                glyph_atlas_ready: self.glyph_atlas_ready(),
                glyph_atlas_uploaded: false,
                gpu_dirty_row_upload_count: 0,
                gpu_dirty_row_upload_bytes: 0,
                native_surface_configured: self.surface_configured(),
                native_surface_presented_this_frame: false,
                native_surface_presentation_ready: self.surface_presentation_ready(),
                native_surface_terminal_frame_presented_this_frame: false,
                native_surface_present_count: self.surface_present_count(),
                native_surface_terminal_cell_count: 0,
                native_surface_terminal_vertex_count: 0,
                reused_gpu_device: false,
                notes: vec![error],
            },
            Err(_) => GpuPassResult {
                offscreen_gpu_pass: false,
                gpu_backend: self.gpu_backend(),
                gpu_adapter: self.gpu_adapter(),
                glyph_atlas_ready: self.glyph_atlas_ready(),
                glyph_atlas_uploaded: false,
                gpu_dirty_row_upload_count: 0,
                gpu_dirty_row_upload_bytes: 0,
                native_surface_configured: self.surface_configured(),
                native_surface_presented_this_frame: false,
                native_surface_presentation_ready: self.surface_presentation_ready(),
                native_surface_terminal_frame_presented_this_frame: false,
                native_surface_present_count: self.surface_present_count(),
                native_surface_terminal_cell_count: 0,
                native_surface_terminal_vertex_count: 0,
                reused_gpu_device: false,
                notes: vec!["wgpu panic while rendering offscreen terminal frame".to_string()],
            },
        }
    }

    fn attach_native_surface(
        &mut self,
        request: RendererSurfaceRequest,
        generation: u64,
    ) -> GpuSurfaceUpdate {
        if cfg!(test) {
            return GpuSurfaceUpdate {
                wgpu_surface_configured: false,
                presentation_ready: false,
                present_count: 0,
                notes: vec![
                    "wgpu surface creation is skipped in unit tests because raw platform handles are synthetic"
                        .to_string(),
                ],
            };
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let runtime = self.ensure_ready()?;
            runtime.attach_surface(request, generation)?;
            let presented = runtime
                .render_surface_frame(None, SurfaceColor::new(0.05, 0.06, 0.06, 1.0))?
                .presented;
            Ok::<_, String>((presented, runtime.surface_present_count()))
        }));

        match result {
            Ok(Ok((presented, present_count))) => GpuSurfaceUpdate {
                wgpu_surface_configured: self.surface_configured(),
                presentation_ready: self.surface_presentation_ready(),
                present_count,
                notes: vec![if presented {
                    "native wgpu surface was configured and a presentation probe succeeded"
                        .to_string()
                } else {
                    "native wgpu surface was configured; presentation probe did not acquire a frame"
                        .to_string()
                }],
            },
            Ok(Err(error)) => GpuSurfaceUpdate {
                wgpu_surface_configured: self.surface_configured(),
                presentation_ready: self.surface_presentation_ready(),
                present_count: self.surface_present_count(),
                notes: vec![format!("native wgpu surface attach failed: {error}")],
            },
            Err(_) => GpuSurfaceUpdate {
                wgpu_surface_configured: self.surface_configured(),
                presentation_ready: self.surface_presentation_ready(),
                present_count: self.surface_present_count(),
                notes: vec!["wgpu panic while attaching native renderer surface".to_string()],
            },
        }
    }

    fn detach_native_surface(&mut self) {
        if let GpuRuntimeState::Ready(runtime) = &mut self.state {
            runtime.detach_surface();
        }
    }

    fn surface_configured(&self) -> bool {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.surface_configured(),
            _ => false,
        }
    }

    fn surface_presentation_ready(&self) -> bool {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.surface_presentation_ready(),
            _ => false,
        }
    }

    fn surface_present_count(&self) -> u64 {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.surface_present_count(),
            _ => 0,
        }
    }

    fn surface_terminal_frame_ready(&self) -> bool {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.surface_terminal_frame_ready(),
            _ => false,
        }
    }

    fn surface_terminal_frame_count(&self) -> u64 {
        match &self.state {
            GpuRuntimeState::Ready(runtime) => runtime.surface_terminal_frame_count(),
            _ => 0,
        }
    }

    fn ensure_ready(&mut self) -> Result<&mut GpuRuntime, String> {
        if matches!(self.state, GpuRuntimeState::Pending) {
            self.state = match GpuRuntime::create() {
                Ok(runtime) => GpuRuntimeState::Ready(runtime),
                Err(error) => GpuRuntimeState::Failed(error),
            };
        }

        match &mut self.state {
            GpuRuntimeState::Ready(runtime) => Ok(runtime),
            GpuRuntimeState::Failed(error) => Err(error.clone()),
            GpuRuntimeState::Pending => unreachable!("renderer runtime pending after init"),
        }
    }
}

#[cfg(feature = "native-integrations")]
impl GpuRuntime {
    fn create() -> Result<Self, String> {
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        descriptor.backends = native_backends();
        let instance = wgpu::Instance::new(descriptor);

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("tokio runtime failed: {error}"))?;

        let adapters = runtime.block_on(instance.enumerate_adapters(native_backends()));
        let adapter = adapters.first().ok_or_else(|| {
            format!(
                "wgpu runtime active; no {:?} adapters enumerated yet",
                native_backends()
            )
        })?;
        let info = adapter.get_info();
        let (device, queue) = runtime
            .block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("shellow-terminal-frame-device"),
                required_limits: adapter.limits(),
                ..Default::default()
            }))
            .map_err(|error| format!("wgpu device request failed: {error}"))?;

        Ok(Self {
            instance,
            adapter_handle: adapter.clone(),
            device,
            queue,
            backend: format!("{:?}", info.backend),
            adapter: info.name,
            format: wgpu::TextureFormat::Rgba8Unorm,
            glyph_atlas_texture: None,
            glyph_atlas_revision: 0,
            glyph_atlas_upload_count: 0,
            dirty_rows_buffer: None,
            dirty_rows_buffer_capacity: 0,
            dirty_row_upload_count: 0,
            last_dirty_row_upload_bytes: 0,
            surface_pipeline: None,
            surface_vertex_buffer: None,
            surface_vertex_buffer_capacity: 0,
            surface_atlas_bind_group: None,
            surface_atlas_bind_group_revision: 0,
            surface: None,
        })
    }

    fn glyph_atlas_ready(&self) -> bool {
        self.glyph_atlas_texture.is_some()
    }

    fn upload_frame_inputs(
        &mut self,
        glyph_atlas: &GlyphAtlas,
        dirty_rows: &DirtyRowUpload,
    ) -> Result<GpuUploadStats, String> {
        let glyph_atlas_uploaded = self.upload_glyph_atlas(glyph_atlas)?;
        let (dirty_row_upload_count, dirty_row_upload_bytes) =
            self.upload_dirty_rows(dirty_rows)?;
        Ok(GpuUploadStats {
            glyph_atlas_uploaded,
            dirty_row_upload_count,
            dirty_row_upload_bytes,
        })
    }

    fn upload_glyph_atlas(&mut self, glyph_atlas: &GlyphAtlas) -> Result<bool, String> {
        if self.glyph_atlas_texture.is_some() && self.glyph_atlas_revision == glyph_atlas.revision()
        {
            return Ok(false);
        }

        let extent = glyph_atlas.extent();
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shellow-terminal-glyph-atlas"),
            size: wgpu::Extent3d {
                width: extent.width,
                height: extent.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let pixels = glyph_atlas.rgba_pixels();
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(extent.width * 4),
                rows_per_image: Some(extent.height),
            },
            wgpu::Extent3d {
                width: extent.width,
                height: extent.height,
                depth_or_array_layers: 1,
            },
        );

        self.glyph_atlas_texture = Some(texture);
        self.glyph_atlas_revision = glyph_atlas.revision();
        self.glyph_atlas_upload_count = self.glyph_atlas_upload_count.saturating_add(1);
        self.surface_atlas_bind_group = None;
        Ok(true)
    }

    fn upload_dirty_rows(&mut self, dirty_rows: &DirtyRowUpload) -> Result<(usize, usize), String> {
        if dirty_rows.payload.is_empty() {
            self.last_dirty_row_upload_bytes = 0;
            return Ok((0, 0));
        }

        let required_capacity = dirty_rows.payload.len() as u64;
        if self.dirty_rows_buffer_capacity < required_capacity {
            self.dirty_rows_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("shellow-terminal-dirty-rows"),
                size: required_capacity.max(1),
                usage: wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::STORAGE,
                mapped_at_creation: false,
            }));
            self.dirty_rows_buffer_capacity = required_capacity;
        }

        let buffer = self
            .dirty_rows_buffer
            .as_ref()
            .ok_or_else(|| "dirty row GPU buffer was not created".to_string())?;
        self.queue.write_buffer(buffer, 0, &dirty_rows.payload);
        self.dirty_row_upload_count = self
            .dirty_row_upload_count
            .saturating_add(dirty_rows.row_count as u64);
        self.last_dirty_row_upload_bytes = dirty_rows.byte_len();
        Ok((dirty_rows.row_count, dirty_rows.byte_len()))
    }

    fn attach_surface(
        &mut self,
        request: RendererSurfaceRequest,
        _generation: u64,
    ) -> Result<(), String> {
        if self.surface.as_ref().is_some_and(|surface| {
            surface.kind == request.kind
                && surface.raw_handle == request.raw_handle
                && surface.width_px == request.width_px
                && surface.height_px == request.height_px
        }) {
            return Ok(());
        }

        let surface = self.create_surface(request)?;
        let config = surface
            .get_default_config(
                &self.adapter_handle,
                request.width_px.max(1),
                request.height_px.max(1),
            )
            .ok_or_else(|| {
                format!(
                    "wgpu adapter {} cannot present to {}",
                    self.adapter,
                    request.kind.label()
                )
            })?;
        surface.configure(&self.device, &config);
        self.surface = Some(GpuSurfaceRuntime {
            kind: request.kind,
            raw_handle: request.raw_handle,
            width_px: request.width_px.max(1),
            height_px: request.height_px.max(1),
            surface,
            config,
            present_count: 0,
            terminal_frame_present_count: 0,
        });
        Ok(())
    }

    fn create_surface(
        &self,
        request: RendererSurfaceRequest,
    ) -> Result<wgpu::Surface<'static>, String> {
        match request.kind {
            RendererSurfaceKind::CoreAnimationLayer => {
                #[cfg(any(target_os = "ios", target_os = "macos"))]
                {
                    if !surface_platform_supported(request.kind) {
                        return Err(format!(
                            "{} is not supported by this build target",
                            request.kind.label()
                        ));
                    }
                    let layer = request.raw_handle as *mut c_void;
                    if layer.is_null() {
                        return Err("CoreAnimationLayer handle was null".to_string());
                    }
                    unsafe {
                        self.instance
                            .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(
                                layer,
                            ))
                            .map_err(|error| {
                                format!("wgpu CoreAnimationLayer surface failed: {error}")
                            })
                    }
                }
                #[cfg(not(any(target_os = "ios", target_os = "macos")))]
                {
                    Err(format!(
                        "{} is not supported by this build target",
                        request.kind.label()
                    ))
                }
            }
            RendererSurfaceKind::AndroidNativeWindow => {
                if !surface_platform_supported(request.kind) {
                    return Err(format!(
                        "{} is not supported by this build target",
                        request.kind.label()
                    ));
                }
                let window = NonNull::new(request.raw_handle as *mut c_void)
                    .ok_or_else(|| "Android native window handle was null".to_string())?;
                let raw_window_handle = wgpu::rwh::AndroidNdkWindowHandle::new(window).into();
                let raw_display_handle = wgpu::rwh::AndroidDisplayHandle::new().into();
                unsafe {
                    self.instance
                        .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                            raw_display_handle: Some(raw_display_handle),
                            raw_window_handle,
                        })
                        .map_err(|error| {
                            format!("wgpu Android native window surface failed: {error}")
                        })
                }
            }
        }
    }

    fn detach_surface(&mut self) {
        self.surface = None;
    }

    fn surface_configured(&self) -> bool {
        self.surface.is_some()
    }

    fn surface_presentation_ready(&self) -> bool {
        self.surface_present_count() > 0
    }

    fn surface_present_count(&self) -> u64 {
        self.surface
            .as_ref()
            .map_or(0, |surface| surface.present_count)
    }

    fn surface_terminal_frame_ready(&self) -> bool {
        self.surface_terminal_frame_count() > 0
    }

    fn surface_terminal_frame_count(&self) -> u64 {
        self.surface
            .as_ref()
            .map_or(0, |surface| surface.terminal_frame_present_count)
    }

    fn ensure_surface_pipeline(&mut self, format: wgpu::TextureFormat) -> Result<(), String> {
        if self
            .surface_pipeline
            .as_ref()
            .is_some_and(|pipeline| pipeline.format == format)
        {
            return Ok(());
        }

        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shellow-terminal-surface-shader"),
                source: wgpu::ShaderSource::Wgsl(SURFACE_SHADER.into()),
            });
        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("shellow-terminal-surface-bind-group-layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shellow-terminal-surface-pipeline-layout"),
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0,
            });
        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shellow-terminal-surface-pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: SURFACE_VERTEX_STRIDE,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 16,
                                shader_location: 2,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32,
                                offset: 32,
                                shader_location: 3,
                            },
                        ],
                    })],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shellow-terminal-surface-atlas-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        self.surface_atlas_bind_group = None;
        self.surface_pipeline = Some(GpuSurfacePipeline {
            format,
            pipeline,
            bind_group_layout,
            sampler,
        });
        Ok(())
    }

    fn ensure_surface_atlas_bind_group(&mut self) -> Result<(), String> {
        if self.surface_atlas_bind_group.is_some()
            && self.surface_atlas_bind_group_revision == self.glyph_atlas_revision
        {
            return Ok(());
        }

        let pipeline = self
            .surface_pipeline
            .as_ref()
            .ok_or_else(|| "native surface pipeline was not created".to_string())?;
        let texture = self
            .glyph_atlas_texture
            .as_ref()
            .ok_or_else(|| "glyph atlas texture is not available".to_string())?;
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.surface_atlas_bind_group =
            Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("shellow-terminal-surface-atlas-bind-group"),
                layout: &pipeline.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                    },
                ],
            }));
        self.surface_atlas_bind_group_revision = self.glyph_atlas_revision;
        Ok(())
    }

    fn upload_surface_vertices(&mut self, frame: &SurfaceFrameUpload) -> Result<(), String> {
        if frame.payload.is_empty() {
            return Ok(());
        }

        let required_capacity = frame.payload.len() as u64;
        if self.surface_vertex_buffer_capacity < required_capacity {
            self.surface_vertex_buffer = Some(self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("shellow-terminal-surface-vertices"),
                size: required_capacity.max(1),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            }));
            self.surface_vertex_buffer_capacity = required_capacity;
        }

        let buffer = self
            .surface_vertex_buffer
            .as_ref()
            .ok_or_else(|| "native surface vertex buffer was not created".to_string())?;
        self.queue.write_buffer(buffer, 0, &frame.payload);
        Ok(())
    }

    fn render_surface_frame(
        &mut self,
        surface_frame: Option<&SurfaceFrameUpload>,
        background: SurfaceColor,
    ) -> Result<GpuSurfaceRenderResult, String> {
        let Some(surface_runtime) = self.surface.as_ref() else {
            return Ok(GpuSurfaceRenderResult {
                presented: false,
                terminal_frame_presented: false,
            });
        };
        let surface_format = surface_runtime.config.format;
        let draw_terminal_frame =
            surface_frame.is_some_and(|frame| frame.vertex_count > 0 && self.glyph_atlas_ready());

        if draw_terminal_frame {
            let frame = surface_frame.expect("checked terminal surface frame");
            self.ensure_surface_pipeline(surface_format)?;
            self.ensure_surface_atlas_bind_group()?;
            self.upload_surface_vertices(frame)?;
        }

        let frame = {
            let surface_runtime = self
                .surface
                .as_mut()
                .ok_or_else(|| "native renderer surface disappeared before present".to_string())?;
            match surface_runtime.surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame)
                | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
                wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                    return Ok(GpuSurfaceRenderResult {
                        presented: false,
                        terminal_frame_presented: false,
                    });
                }
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    surface_runtime
                        .surface
                        .configure(&self.device, &surface_runtime.config);
                    return Ok(GpuSurfaceRenderResult {
                        presented: false,
                        terminal_frame_presented: false,
                    });
                }
                wgpu::CurrentSurfaceTexture::Validation => {
                    return Err("wgpu surface texture validation failed".to_string());
                }
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("shellow-terminal-surface-encoder"),
            });

        {
            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: background.r as f64,
                        g: background.g as f64,
                        b: background.b as f64,
                        a: background.a as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shellow-terminal-surface-pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if draw_terminal_frame {
                let pipeline = self
                    .surface_pipeline
                    .as_ref()
                    .ok_or_else(|| "native surface pipeline was not created".to_string())?;
                let bind_group = self
                    .surface_atlas_bind_group
                    .as_ref()
                    .ok_or_else(|| "native surface atlas bind group was not created".to_string())?;
                let vertex_buffer = self
                    .surface_vertex_buffer
                    .as_ref()
                    .ok_or_else(|| "native surface vertex buffer was not created".to_string())?;
                let vertex_count = surface_frame
                    .map_or(0, |frame| frame.vertex_count.min(u32::MAX as usize) as u32);
                pass.set_pipeline(&pipeline.pipeline);
                pass.set_bind_group(0, bind_group, &[]);
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                pass.draw(0..vertex_count, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        if let Some(surface_runtime) = self.surface.as_mut() {
            surface_runtime.present_count = surface_runtime.present_count.saturating_add(1);
            if draw_terminal_frame {
                surface_runtime.terminal_frame_present_count = surface_runtime
                    .terminal_frame_present_count
                    .saturating_add(1);
            }
        }
        Ok(GpuSurfaceRenderResult {
            presented: true,
            terminal_frame_presented: draw_terminal_frame,
        })
    }

    fn render_offscreen(
        &self,
        width_px: u32,
        height_px: u32,
        background: SurfaceColor,
    ) -> Result<(), String> {
        let size = wgpu::Extent3d {
            width: width_px.max(1),
            height: height_px.max(1),
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shellow-terminal-frame"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("shellow-terminal-frame-encoder"),
            });

        {
            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: background.r as f64,
                        g: background.g as f64,
                        b: background.b as f64,
                        a: background.a as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shellow-terminal-clear-pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        self.queue.submit(Some(encoder.finish()));
        Ok(())
    }
}

#[cfg(not(feature = "native-integrations"))]
struct RendererRuntime;

#[cfg(feature = "native-integrations")]
const SURFACE_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) mode: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) mode: f32,
};

@group(0) @binding(0) var glyph_atlas: texture_2d<f32>;
@group(0) @binding(1) var glyph_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.tex_coord = input.tex_coord;
    output.color = input.color;
    output.mode = input.mode;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if (input.mode < 0.5) {
        return input.color;
    }

    let sample = textureSample(glyph_atlas, glyph_sampler, input.tex_coord);
    return vec4<f32>(input.color.rgb, input.color.a * sample.a);
}
"#;

#[cfg(not(feature = "native-integrations"))]
impl RendererRuntime {
    fn new() -> Self {
        Self
    }

    fn persistent_device_ready(&self) -> bool {
        false
    }

    fn glyph_atlas_ready(&self) -> bool {
        false
    }

    fn glyph_atlas_upload_count(&self) -> u64 {
        0
    }

    fn dirty_row_upload_count(&self) -> u64 {
        0
    }

    fn last_dirty_row_upload_bytes(&self) -> usize {
        0
    }

    fn surface_terminal_frame_ready(&self) -> bool {
        false
    }

    fn surface_terminal_frame_count(&self) -> u64 {
        0
    }

    fn gpu_backend(&self) -> Option<String> {
        None
    }

    fn gpu_adapter(&self) -> Option<String> {
        None
    }

    fn failure_note(&self) -> Option<String> {
        None
    }

    fn attach_native_surface(
        &mut self,
        _request: RendererSurfaceRequest,
        _generation: u64,
    ) -> GpuSurfaceUpdate {
        GpuSurfaceUpdate {
            wgpu_surface_configured: false,
            presentation_ready: false,
            present_count: 0,
            notes: vec!["wgpu native integration is not compiled into this build".to_string()],
        }
    }

    fn detach_native_surface(&mut self) {}

    fn render_frame(
        &mut self,
        _width_px: u32,
        _height_px: u32,
        _glyph_atlas: &GlyphAtlas,
        _dirty_rows: &DirtyRowUpload,
        _surface_frame: &SurfaceFrameUpload,
        _background: SurfaceColor,
    ) -> GpuPassResult {
        GpuPassResult {
            offscreen_gpu_pass: false,
            gpu_backend: None,
            gpu_adapter: None,
            glyph_atlas_ready: false,
            glyph_atlas_uploaded: false,
            gpu_dirty_row_upload_count: 0,
            gpu_dirty_row_upload_bytes: 0,
            native_surface_configured: false,
            native_surface_presented_this_frame: false,
            native_surface_presentation_ready: false,
            native_surface_terminal_frame_presented_this_frame: false,
            native_surface_present_count: 0,
            native_surface_terminal_cell_count: 0,
            native_surface_terminal_vertex_count: 0,
            reused_gpu_device: false,
            notes: vec!["wgpu native integration is not compiled into this build".to_string()],
        }
    }
}

#[cfg(all(feature = "native-integrations", target_os = "android"))]
fn native_backends() -> wgpu::Backends {
    wgpu::Backends::VULKAN
}

#[cfg(all(feature = "native-integrations", not(target_os = "android")))]
fn native_backends() -> wgpu::Backends {
    wgpu::Backends::METAL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_theme_controls_default_inverse_and_overlay_colors() {
        let theme = crate::terminal_theme::built_in_theme(TerminalThemeId::PaperLight);
        let (foreground, background) = style_colors(TerminalGridStyle::default(), &theme);
        assert_eq!(
            foreground.components(),
            SurfaceColor::from_grid(theme.foreground).components()
        );
        assert!(background.is_none());

        let inverse = TerminalGridStyle {
            inverse: true,
            ..Default::default()
        };
        let (foreground, background) = style_colors(inverse, &theme);
        assert_eq!(
            foreground.components(),
            SurfaceColor::from_grid(theme.background).components()
        );
        assert_eq!(
            background.expect("inverse background").components(),
            SurfaceColor::from_grid(theme.foreground).components()
        );
        assert_eq!(
            RendererOverlayKind::Selection
                .surface_color(&theme)
                .components(),
            SurfaceColor::from_theme_rgba(theme.selection).components(),
        );
    }

    #[test]
    fn terminal_surface_row_height_does_not_fill_tall_surface_rows() {
        let cell_width = 37.75;
        let available_row_height = 133.0;

        let row_height = terminal_surface_row_height(cell_width, available_row_height);

        assert!(row_height < available_row_height * 0.75);
        assert!(row_height > cell_width);
    }

    #[test]
    fn terminal_surface_rows_are_centered_in_available_slots() {
        let row_slot_height = 100.0;
        let row_height = 76.0;

        assert_eq!(terminal_surface_row_y(0, row_slot_height, row_height), 12.0);
        assert_eq!(
            terminal_surface_row_y(1, row_slot_height, row_height),
            112.0
        );
    }

    #[test]
    fn terminal_glyph_quad_height_leaves_breathing_room() {
        let cell_width = 37.75;
        let row_height = terminal_surface_row_height(cell_width, 133.0);

        let glyph_height = terminal_glyph_quad_height(cell_width, row_height);

        assert!(glyph_height < row_height * 0.8);
        assert!(glyph_height > cell_width);
    }

    #[test]
    fn terminal_glyph_quad_height_clamps_to_compact_rows() {
        let row_height = 18.0;

        let glyph_height = terminal_glyph_quad_height(14.0, row_height);

        assert_eq!(glyph_height, row_height);
    }

    #[cfg(feature = "native-integrations")]
    #[test]
    fn system_font_rasterizer_cache_is_shared() {
        let first = SystemFontRasterizer::cached().expect("embedded mono font should load");
        let second = SystemFontRasterizer::cached().expect("embedded mono font should stay cached");

        assert!(Arc::ptr_eq(&first, &second));
        assert!(first.faces.len() <= 2);
    }

    #[cfg(feature = "native-integrations")]
    #[test]
    fn embedded_font_collection_probe_does_not_claim_cjk_coverage() {
        assert_eq!(
            first_collection_face_supporting(
                SystemFontRasterizer::EMBEDDED_MONO_FONT,
                SystemFontRasterizer::SYSTEM_FALLBACK_PROBE_GLYPHS,
                SystemFontRasterizer::MAX_COLLECTION_FACES,
            ),
            None,
        );
    }

    #[cfg(feature = "native-integrations")]
    #[test]
    fn shaped_space_glyph_rasterizes_transparent() {
        let rasterizer = SystemFontRasterizer::cached().expect("embedded mono font should load");
        let layout = rasterizer
            .shape_text(" ", 1)
            .expect("embedded mono font should shape a space");
        let glyph = layout
            .clusters
            .first()
            .expect("space should produce a shaped cluster")
            .glyph;
        let mut pixels = vec![0u8; (GlyphAtlas::CELL_WIDTH * GlyphAtlas::CELL_HEIGHT * 4) as usize];

        rasterizer.write_glyph_pixels(
            0,
            glyph,
            GlyphAtlas::CELL_WIDTH,
            GlyphAtlas::CELL_HEIGHT,
            1,
            GlyphAtlas::CELL_WIDTH,
            &mut pixels,
        );

        assert!(pixels.iter().all(|component| *component == 0));
    }

    #[cfg(feature = "native-integrations")]
    #[test]
    fn visible_glyph_fallback_skips_spaces_but_keeps_shaped_glyphs_visible() {
        let visible_metrics = fontdue::Metrics {
            width: 12,
            height: 18,
            ..Default::default()
        };

        assert!(!should_write_visible_glyph_fallback(
            GlyphKey::Codepoint(' '),
            visible_metrics
        ));
        assert!(should_write_visible_glyph_fallback(
            GlyphKey::Codepoint('中'),
            fontdue::Metrics::default()
        ));
        assert!(should_write_visible_glyph_fallback(
            GlyphKey::FontGlyph { font: 1, glyph: 42 },
            visible_metrics
        ));
        assert!(!should_write_visible_glyph_fallback(
            GlyphKey::FontGlyph { font: 1, glyph: 3 },
            fontdue::Metrics::default()
        ));
    }
}
