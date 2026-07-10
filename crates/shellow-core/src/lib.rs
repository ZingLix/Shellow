use std::cell::RefCell;

use serde::{Deserialize, Serialize};

#[cfg(not(feature = "native-integrations"))]
compile_error!(
    "shellow-core requires the native-integrations feature; fallback builds are unsupported."
);

pub mod codex;
pub mod ghostty_adapter;
pub mod integrations;
pub mod renderer;
pub mod ssh;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostProfile {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub authentication: AuthenticationKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trusted_host_key_sha256: Option<String>,
}

impl HostProfile {
    pub fn endpoint(&self) -> String {
        format!("{}@{}:{}", self.username, self.host, self.port)
    }

    pub fn host_key_status(&self) -> String {
        match ssh::normalize_sha256_fingerprint_option(self.trusted_host_key_sha256.as_deref()) {
            Some(fingerprint) => format!("host-key=pinned {fingerprint}"),
            None => "host-key=unverified (no SHA256 pin)".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationKind {
    Password,
    PrivateKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalSnapshot {
    pub title: String,
    pub host: String,
    pub state: ConnectionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_host_key_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_clipboard_text: Option<String>,
    pub clipboard_sequence: u64,
    pub bell_count: usize,
    pub rows: Vec<TerminalRow>,
    pub grid: Option<TerminalGridSnapshot>,
    pub cursor_column: usize,
    pub terminal_cols: u32,
    pub terminal_rows: u32,
    pub integration: IntegrationReport,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalRow {
    pub prompt: String,
    pub text: String,
    pub style: TerminalRowStyle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminalRowStyle {
    Command,
    Muted,
    Success,
    Prompt,
    Warning,
}

impl TerminalRow {
    fn command(text: impl Into<String>) -> Self {
        Self {
            prompt: "$".to_string(),
            text: text.into(),
            style: TerminalRowStyle::Command,
        }
    }

    fn muted(text: impl Into<String>) -> Self {
        Self {
            prompt: String::new(),
            text: text.into(),
            style: TerminalRowStyle::Muted,
        }
    }

    fn success(text: impl Into<String>) -> Self {
        Self {
            prompt: String::new(),
            text: text.into(),
            style: TerminalRowStyle::Success,
        }
    }

    fn warning(text: impl Into<String>) -> Self {
        Self {
            prompt: String::new(),
            text: text.into(),
            style: TerminalRowStyle::Warning,
        }
    }

    fn prompt() -> Self {
        Self {
            prompt: "$".to_string(),
            text: String::new(),
            style: TerminalRowStyle::Prompt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalGridSnapshot {
    pub cols: u32,
    pub rows: u32,
    pub cursor_column: u32,
    pub cursor_row: u32,
    pub cursor_visible: bool,
    pub cursor_shape: TerminalCursorShape,
    pub active_screen: TerminalScreenKind,
    pub scrollback_len: usize,
    pub bracketed_paste: bool,
    pub application_cursor_keys: bool,
    pub mouse_reporting: bool,
    pub mouse_drag_reporting: bool,
    pub sgr_mouse: bool,
    pub lines: Vec<String>,
    pub styled_lines: Vec<TerminalGridLine>,
    pub dirty_rows: Vec<usize>,
}

impl TerminalGridSnapshot {
    #[cfg(feature = "native-integrations")]
    fn has_visible_content(&self) -> bool {
        self.lines.iter().any(|line| !line.trim_end().is_empty())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminalCursorShape {
    Block,
    Underline,
    Bar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalGridLine {
    pub runs: Vec<TerminalGridRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalGridRun {
    pub text: String,
    pub style: TerminalGridStyle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TerminalGridStyle {
    pub bold: bool,
    pub faint: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub inverse: bool,
    pub strikethrough: bool,
    pub fg: Option<TerminalGridColor>,
    pub bg: Option<TerminalGridColor>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminalGridColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TerminalScreenKind {
    Primary,
    Alternate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationReport {
    pub terminal_backend: String,
    pub terminal_target_backend: String,
    pub terminal_backend_migration: String,
    pub ssh_backend: String,
    pub renderer_backend: String,
    pub renderer_target_backend: String,
    pub ghostty_ready: bool,
    pub libghostty_vt_link_configured: bool,
    pub libghostty_vt_ready: bool,
    pub libghostty_vt_abi_contract: String,
    pub libghostty_vt_abi_status: String,
    pub russh_ready: bool,
    pub wgpu_ready: bool,
    pub renderer_surface_ready: bool,
}

pub struct ShellowEngine {
    title: String,
    host: String,
    state: ConnectionState,
    bell_count: usize,
    rows: Vec<TerminalRow>,
    cursor_column: usize,
    terminal_cols: u32,
    terminal_rows: u32,
    observed_host_key_sha256: Option<String>,
    pending_clipboard_text: Option<String>,
    clipboard_sequence: u64,
    local_input: String,
    local_cursor: usize,
    command_history: Vec<String>,
    history_cursor: Option<usize>,
    history_draft: String,
    reverse_search_active: bool,
    reverse_search_query: String,
    reverse_search_match_index: Option<usize>,
    reverse_search_draft: String,
    demo_editor_active: bool,
    demo_editor_text: String,
    demo_editor_status: String,
    demo_pager_active: bool,
    demo_pager_offset: usize,
    demo_pager_status: String,
    demo_mouse_active: bool,
    demo_mouse_status: String,
    demo_tui: Option<LocalTuiDemo>,
    demo_tui_status: String,
    demo_tui_prefix_armed: bool,
    demo_grid_bytes: Option<Vec<u8>>,
    last_grid_signature: RefCell<Option<GridRenderSignature>>,
    renderer: RefCell<renderer::TerminalRenderer>,
    integration: IntegrationReport,
    #[cfg(feature = "native-integrations")]
    live_shell: Option<ssh::LiveShellHandle>,
    #[cfg(feature = "native-integrations")]
    live_terminal: Option<ghostty_adapter::LiveTerminalState>,
    codex_session: Option<codex::CodexSession>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GridRenderSignature {
    cols: u32,
    rows: u32,
    active_screen: TerminalScreenKind,
    scrollback_len: usize,
    lines: Vec<String>,
    styled_lines: Vec<TerminalGridLine>,
    cursor_column: u32,
    cursor_row: u32,
    cursor_visible: bool,
    cursor_shape: TerminalCursorShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalTuiDemo {
    Nano,
    Top,
    Tmux,
}

impl LocalTuiDemo {
    fn label(self) -> &'static str {
        match self {
            Self::Nano => "NANO",
            Self::Top => "TOP",
            Self::Tmux => "TMUX",
        }
    }

    fn default_status(self) -> &'static str {
        match self {
            Self::Nano => "Nano demo - type text, Ctrl-O writes, Ctrl-X exits",
            Self::Top => "Top demo - q or Ctrl-C exits, arrows change selection",
            Self::Tmux => "Tmux demo - Ctrl-B arms prefix, Esc exits",
        }
    }
}

impl GridRenderSignature {
    fn from_snapshot(snapshot: &TerminalGridSnapshot) -> Self {
        Self {
            cols: snapshot.cols,
            rows: snapshot.rows,
            active_screen: snapshot.active_screen,
            scrollback_len: snapshot.scrollback_len,
            lines: snapshot.lines.clone(),
            styled_lines: snapshot.styled_lines.clone(),
            cursor_column: snapshot.cursor_column,
            cursor_row: snapshot.cursor_row,
            cursor_visible: snapshot.cursor_visible,
            cursor_shape: snapshot.cursor_shape,
        }
    }

    fn cursor_dirty_row(&self) -> Option<usize> {
        self.cursor_visible
            .then_some(self.cursor_row as usize)
            .filter(|row| *row < self.lines.len())
    }
}

fn viewport_grid_snapshot(
    mut snapshot: TerminalGridSnapshot,
    requested_first_row: usize,
    requested_row_count: usize,
) -> TerminalGridSnapshot {
    let total_rows = snapshot.lines.len();
    if total_rows == 0 {
        return snapshot;
    }

    let row_count = requested_row_count
        .max(1)
        .min(total_rows)
        .max(snapshot.rows as usize)
        .min(total_rows);
    let first_row = if snapshot.active_screen == TerminalScreenKind::Alternate {
        0
    } else {
        requested_first_row.min(total_rows.saturating_sub(row_count))
    };
    let end_row = first_row.saturating_add(row_count).min(total_rows);

    snapshot.lines = snapshot.lines[first_row..end_row].to_vec();
    snapshot.styled_lines = snapshot
        .styled_lines
        .get(first_row..end_row)
        .map_or_else(Vec::new, <[TerminalGridLine]>::to_vec);
    snapshot.dirty_rows = snapshot
        .dirty_rows
        .iter()
        .filter_map(|row| {
            if *row >= first_row && *row < end_row {
                Some(row - first_row)
            } else {
                None
            }
        })
        .collect();

    if snapshot.cursor_visible {
        let cursor_row = snapshot.cursor_row as usize;
        if cursor_row >= first_row && cursor_row < end_row {
            snapshot.cursor_row = cursor_row.saturating_sub(first_row) as u32;
        } else {
            snapshot.cursor_visible = false;
            snapshot.cursor_row = 0;
        }
    }

    snapshot
}

impl Default for ShellowEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellowEngine {
    pub fn new() -> Self {
        let integration = IntegrationReport {
            terminal_backend: ghostty_adapter::backend_name().to_string(),
            terminal_target_backend: ghostty_adapter::target_backend_name().to_string(),
            terminal_backend_migration: ghostty_adapter::migration_stage().to_string(),
            ssh_backend: ssh::backend_name().to_string(),
            renderer_backend: renderer::backend_name().to_string(),
            renderer_target_backend: renderer::target_backend_name().to_string(),
            ghostty_ready: ghostty_adapter::is_ghostty_available(),
            libghostty_vt_link_configured: ghostty_adapter::is_libghostty_vt_link_configured(),
            libghostty_vt_ready: ghostty_adapter::is_libghostty_vt_ready(),
            libghostty_vt_abi_contract: ghostty_adapter::libghostty_vt_abi_contract().to_string(),
            libghostty_vt_abi_status: ghostty_adapter::libghostty_vt_abi_status(),
            russh_ready: ssh::is_russh_available(),
            wgpu_ready: renderer::is_wgpu_available(),
            renderer_surface_ready: renderer::is_native_surface_ready(),
        };

        Self {
            title: "Shellow".to_string(),
            host: "rust-demo.local".to_string(),
            state: ConnectionState::Connected,
            bell_count: 0,
            rows: vec![
                TerminalRow::success("Shellow Rust core online"),
                TerminalRow::muted(format!(
                    "terminal={}  ssh={}  renderer={}",
                    integration.terminal_backend,
                    integration.ssh_backend,
                    integration.renderer_backend
                )),
                TerminalRow::command("shellow integrations"),
                TerminalRow::muted(integrations::summary_line(&integration)),
                TerminalRow::prompt(),
            ],
            cursor_column: 0,
            terminal_cols: 80,
            terminal_rows: 24,
            observed_host_key_sha256: None,
            pending_clipboard_text: None,
            clipboard_sequence: 0,
            local_input: String::new(),
            local_cursor: 0,
            command_history: Vec::new(),
            history_cursor: None,
            history_draft: String::new(),
            reverse_search_active: false,
            reverse_search_query: String::new(),
            reverse_search_match_index: None,
            reverse_search_draft: String::new(),
            demo_editor_active: false,
            demo_editor_text: String::new(),
            demo_editor_status: String::new(),
            demo_pager_active: false,
            demo_pager_offset: 0,
            demo_pager_status: String::new(),
            demo_mouse_active: false,
            demo_mouse_status: String::new(),
            demo_tui: None,
            demo_tui_status: String::new(),
            demo_tui_prefix_armed: false,
            demo_grid_bytes: None,
            last_grid_signature: RefCell::new(None),
            renderer: RefCell::new(renderer::TerminalRenderer::new(80, 24)),
            integration,
            #[cfg(feature = "native-integrations")]
            live_shell: None,
            #[cfg(feature = "native-integrations")]
            live_terminal: None,
            codex_session: None,
        }
    }

    pub fn snapshot(&self) -> TerminalSnapshot {
        TerminalSnapshot {
            title: self.title.clone(),
            host: self.host.clone(),
            state: self.state,
            observed_host_key_sha256: self.observed_host_key_sha256.clone(),
            pending_clipboard_text: self.pending_clipboard_text.clone(),
            clipboard_sequence: self.clipboard_sequence,
            bell_count: self.bell_count,
            rows: self.rows.clone(),
            grid: self.grid_snapshot(),
            cursor_column: self.cursor_column,
            terminal_cols: self.terminal_cols,
            terminal_rows: self.terminal_rows,
            integration: self.integration.clone(),
        }
    }

    pub fn render_frame(&self, width_px: u32, height_px: u32) -> renderer::TerminalRenderFrame {
        let grid = self.grid_snapshot();
        self.renderer.borrow_mut().render_frame(
            grid.as_ref(),
            &self.rows,
            self.rows.len(),
            self.terminal_cols,
            self.terminal_rows,
            width_px,
            height_px,
        )
    }

    pub fn render_frame_viewport(
        &self,
        width_px: u32,
        height_px: u32,
        first_row: u32,
        row_count: u32,
    ) -> renderer::TerminalRenderFrame {
        let grid = self.grid_snapshot_viewport(first_row as usize, row_count as usize);
        self.renderer.borrow_mut().render_frame(
            grid.as_ref(),
            &self.rows,
            self.rows.len(),
            self.terminal_cols,
            self.terminal_rows,
            width_px,
            height_px,
        )
    }

    pub fn render_surface_frame_presented(
        &self,
        width_px: u32,
        height_px: u32,
        first_row: u32,
        row_count: u32,
    ) -> bool {
        self.render_frame_viewport(width_px, height_px, first_row, row_count)
            .native_surface_terminal_frame_presented_this_frame
    }

    pub fn renderer_info(&self) -> renderer::TerminalRendererInfo {
        self.renderer.borrow().info()
    }

    pub fn live_shell_event_revision(&self) -> u64 {
        #[cfg(feature = "native-integrations")]
        {
            self.live_shell
                .as_ref()
                .map_or(0, ssh::LiveShellHandle::event_revision)
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            0
        }
    }

    pub fn set_renderer_overlay(
        &self,
        state: renderer::RendererOverlayState,
    ) -> renderer::RendererOverlayUpdate {
        self.renderer.borrow_mut().set_overlay_state(state)
    }

    pub fn attach_core_animation_layer_renderer_surface(
        &mut self,
        raw_handle: u64,
        width_px: u32,
        height_px: u32,
    ) -> renderer::RendererSurfaceAttachment {
        self.attach_renderer_surface(renderer::RendererSurfaceRequest::new(
            renderer::RendererSurfaceKind::core_animation_layer(),
            raw_handle,
            width_px,
            height_px,
        ))
    }

    pub fn attach_android_native_window_renderer_surface(
        &mut self,
        raw_handle: u64,
        width_px: u32,
        height_px: u32,
    ) -> renderer::RendererSurfaceAttachment {
        self.attach_renderer_surface(renderer::RendererSurfaceRequest::new(
            renderer::RendererSurfaceKind::android_native_window(),
            raw_handle,
            width_px,
            height_px,
        ))
    }

    pub fn detach_renderer_surface(&mut self) -> renderer::RendererSurfaceAttachment {
        let attachment = self.renderer.borrow_mut().detach_native_surface();
        self.integration.renderer_surface_ready = attachment.presentation_ready;
        attachment
    }

    fn attach_renderer_surface(
        &mut self,
        request: renderer::RendererSurfaceRequest,
    ) -> renderer::RendererSurfaceAttachment {
        let attachment = self.renderer.borrow_mut().attach_native_surface(request);
        self.integration.renderer_surface_ready = attachment.presentation_ready;
        attachment
    }

    pub fn connect_preview(&mut self, profile: HostProfile) -> TerminalSnapshot {
        self.disconnect_live_shell_handle();
        self.title = profile.name.clone();
        self.host = profile.endpoint();
        self.state = ConnectionState::Connecting;
        self.bell_count = 0;
        self.observed_host_key_sha256 = None;
        self.pending_clipboard_text = None;
        self.rows.clear();
        self.local_input.clear();
        self.clear_reverse_search();
        self.clear_demo_editor();
        self.clear_demo_pager();
        self.clear_demo_mouse();
        self.clear_demo_tui();
        self.clear_demo_grid();
        self.rows
            .push(TerminalRow::command(format!("ssh {}", self.host)));
        self.rows
            .push(TerminalRow::success("Preview terminal ready"));
        self.rows.push(TerminalRow::prompt());
        self.state = ConnectionState::Connected;
        self.cursor_column = 0;
        self.snapshot()
    }

    pub fn connect_password_exec(
        &mut self,
        profile: HostProfile,
        password: String,
        command: String,
    ) -> TerminalSnapshot {
        #[cfg(not(feature = "native-integrations"))]
        let _ = &password;

        self.disconnect_live_shell_handle();
        self.title = profile.name.clone();
        self.host = profile.endpoint();
        self.state = ConnectionState::Connecting;
        self.bell_count = 0;
        self.observed_host_key_sha256 = None;
        self.pending_clipboard_text = None;
        self.rows.clear();
        self.local_input.clear();
        self.clear_reverse_search();
        self.clear_demo_editor();
        self.clear_demo_pager();
        self.clear_demo_mouse();
        self.clear_demo_tui();
        self.clear_demo_grid();
        self.rows
            .push(TerminalRow::command(format!("ssh {}", self.host)));
        self.rows.push(TerminalRow::muted("Connecting..."));
        self.rows
            .push(TerminalRow::command(format!("exec {}", command.trim())));

        #[cfg(feature = "native-integrations")]
        let result = ssh::exec_password_blocking(
            ssh::RusshConnectOptions {
                host: profile.host,
                port: profile.port,
                username: profile.username,
                auth: ssh::RusshAuthMethod::Password(password),
                expected_host_key_sha256: profile.trusted_host_key_sha256,
                keepalive_interval_secs: None,
                keepalive_max: ssh::DEFAULT_KEEPALIVE_MAX,
                cols: self.terminal_cols,
                rows: self.terminal_rows,
                inactivity_timeout_secs: 12,
            },
            command.trim(),
        );

        #[cfg(not(feature = "native-integrations"))]
        let result: Result<String, String> =
            Err("russh native integration is not compiled into this build".to_string());

        match result {
            Ok(output) => {
                self.rows.push(TerminalRow::success("Command completed"));
                let output_rows = self.terminal_rows_from_remote_output(&output);
                self.rows.extend(output_rows);
                self.state = ConnectionState::Connected;
            }
            Err(error) => {
                self.rows.push(TerminalRow::warning("Command failed"));
                self.rows.push(TerminalRow::muted(error));
                self.state = ConnectionState::Disconnected;
            }
        }

        self.rows.push(TerminalRow::prompt());
        self.cursor_column = 0;
        self.snapshot()
    }

    pub fn connect_private_key_exec(
        &mut self,
        profile: HostProfile,
        private_key_pem: String,
        passphrase: Option<String>,
        command: String,
    ) -> TerminalSnapshot {
        #[cfg(not(feature = "native-integrations"))]
        let _ = (&private_key_pem, &passphrase);

        self.disconnect_live_shell_handle();
        self.title = profile.name.clone();
        self.host = profile.endpoint();
        self.state = ConnectionState::Connecting;
        self.bell_count = 0;
        self.observed_host_key_sha256 = None;
        self.pending_clipboard_text = None;
        self.rows.clear();
        self.local_input.clear();
        self.clear_reverse_search();
        self.clear_demo_editor();
        self.clear_demo_pager();
        self.clear_demo_mouse();
        self.clear_demo_tui();
        self.clear_demo_grid();
        self.rows
            .push(TerminalRow::command(format!("ssh {}", self.host)));
        self.rows.push(TerminalRow::muted("Connecting..."));
        self.rows
            .push(TerminalRow::command(format!("exec {}", command.trim())));

        #[cfg(feature = "native-integrations")]
        let result = ssh::exec_private_key_blocking(
            ssh::RusshConnectOptions {
                host: profile.host,
                port: profile.port,
                username: profile.username,
                auth: ssh::RusshAuthMethod::PrivateKey {
                    private_key_pem,
                    passphrase,
                },
                expected_host_key_sha256: profile.trusted_host_key_sha256,
                keepalive_interval_secs: None,
                keepalive_max: ssh::DEFAULT_KEEPALIVE_MAX,
                cols: self.terminal_cols,
                rows: self.terminal_rows,
                inactivity_timeout_secs: 12,
            },
            command.trim(),
        );

        #[cfg(not(feature = "native-integrations"))]
        let result: Result<String, String> =
            Err("russh native integration is not compiled into this build".to_string());

        match result {
            Ok(output) => {
                self.rows.push(TerminalRow::success("Command completed"));
                let output_rows = self.terminal_rows_from_remote_output(&output);
                self.rows.extend(output_rows);
                self.state = ConnectionState::Connected;
            }
            Err(error) => {
                self.rows.push(TerminalRow::warning("Command failed"));
                self.rows.push(TerminalRow::muted(error));
                self.state = ConnectionState::Disconnected;
            }
        }

        self.rows.push(TerminalRow::prompt());
        self.cursor_column = 0;
        self.snapshot()
    }

    pub fn start_password_shell(
        &mut self,
        profile: HostProfile,
        password: String,
    ) -> TerminalSnapshot {
        #[cfg(not(feature = "native-integrations"))]
        let _ = &password;

        self.disconnect_live_shell_handle();
        self.title = profile.name.clone();
        self.host = profile.endpoint();
        self.state = ConnectionState::Connecting;
        self.bell_count = 0;
        self.observed_host_key_sha256 = None;
        self.pending_clipboard_text = None;
        self.rows.clear();
        self.local_input.clear();
        self.clear_reverse_search();
        self.clear_demo_editor();
        self.clear_demo_pager();
        self.clear_demo_mouse();
        self.clear_demo_tui();
        self.clear_demo_grid();
        self.rows
            .push(TerminalRow::command(format!("ssh {}", self.host)));
        self.rows.push(TerminalRow::muted("Connecting..."));

        #[cfg(feature = "native-integrations")]
        {
            self.live_terminal =
                ghostty_adapter::LiveTerminalState::new(self.terminal_cols, self.terminal_rows);
            if self.live_terminal.is_none() {
                self.rows
                    .push(TerminalRow::warning("libghostty-vt live terminal failed"));
                self.state = ConnectionState::Disconnected;
            } else {
                match ssh::LiveShellHandle::spawn_password(ssh::RusshConnectOptions {
                    host: profile.host,
                    port: profile.port,
                    username: profile.username,
                    auth: ssh::RusshAuthMethod::Password(password),
                    expected_host_key_sha256: profile.trusted_host_key_sha256,
                    keepalive_interval_secs: Some(ssh::DEFAULT_LIVE_KEEPALIVE_INTERVAL_SECS),
                    keepalive_max: ssh::DEFAULT_KEEPALIVE_MAX,
                    cols: self.terminal_cols,
                    rows: self.terminal_rows,
                    inactivity_timeout_secs: 3_600,
                }) {
                    Ok(handle) => {
                        self.live_shell = Some(handle);
                        self.rows.push(TerminalRow::success("Connected"));
                    }
                    Err(error) => {
                        self.live_terminal = None;
                        self.rows.push(TerminalRow::warning("Connection failed"));
                        self.rows.push(TerminalRow::muted(error));
                        self.state = ConnectionState::Disconnected;
                    }
                }
            };
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            self.rows.push(TerminalRow::warning(
                "russh native integration is not compiled into this build",
            ));
            self.state = ConnectionState::Disconnected;
        }

        self.rows.push(TerminalRow::prompt());
        self.cursor_column = 0;
        self.snapshot()
    }

    pub fn start_private_key_shell(
        &mut self,
        profile: HostProfile,
        private_key_pem: String,
        passphrase: Option<String>,
    ) -> TerminalSnapshot {
        #[cfg(not(feature = "native-integrations"))]
        let _ = (&private_key_pem, &passphrase);

        self.disconnect_live_shell_handle();
        self.title = profile.name.clone();
        self.host = profile.endpoint();
        self.state = ConnectionState::Connecting;
        self.bell_count = 0;
        self.observed_host_key_sha256 = None;
        self.pending_clipboard_text = None;
        self.rows.clear();
        self.local_input.clear();
        self.clear_reverse_search();
        self.clear_demo_editor();
        self.clear_demo_pager();
        self.clear_demo_mouse();
        self.clear_demo_tui();
        self.clear_demo_grid();
        self.rows
            .push(TerminalRow::command(format!("ssh {}", self.host)));
        self.rows.push(TerminalRow::muted("Connecting..."));

        #[cfg(feature = "native-integrations")]
        {
            self.live_terminal =
                ghostty_adapter::LiveTerminalState::new(self.terminal_cols, self.terminal_rows);
            let private_key_result =
                ssh::validate_private_key_auth(&private_key_pem, passphrase.as_deref());

            if self.live_terminal.is_none() {
                self.rows
                    .push(TerminalRow::warning("libghostty-vt live terminal failed"));
                self.state = ConnectionState::Disconnected;
            } else {
                match private_key_result.and_then(|_| {
                    ssh::LiveShellHandle::spawn(ssh::RusshConnectOptions {
                        host: profile.host,
                        port: profile.port,
                        username: profile.username,
                        auth: ssh::RusshAuthMethod::PrivateKey {
                            private_key_pem,
                            passphrase,
                        },
                        expected_host_key_sha256: profile.trusted_host_key_sha256,
                        keepalive_interval_secs: Some(ssh::DEFAULT_LIVE_KEEPALIVE_INTERVAL_SECS),
                        keepalive_max: ssh::DEFAULT_KEEPALIVE_MAX,
                        cols: self.terminal_cols,
                        rows: self.terminal_rows,
                        inactivity_timeout_secs: 3_600,
                    })
                }) {
                    Ok(handle) => {
                        self.live_shell = Some(handle);
                        self.rows.push(TerminalRow::success("Connected"));
                    }
                    Err(error) => {
                        self.live_terminal = None;
                        self.rows.push(TerminalRow::warning("Connection failed"));
                        self.rows.push(TerminalRow::muted(error));
                        self.state = ConnectionState::Disconnected;
                    }
                }
            };
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            self.rows.push(TerminalRow::warning(
                "russh native integration is not compiled into this build",
            ));
            self.state = ConnectionState::Disconnected;
        }

        self.rows.push(TerminalRow::prompt());
        self.cursor_column = 0;
        self.snapshot()
    }

    pub fn poll_live_shell(&mut self) -> TerminalSnapshot {
        #[cfg(feature = "native-integrations")]
        {
            let poll = self.live_shell.as_ref().map(ssh::LiveShellHandle::poll);
            if let Some(poll) = poll {
                if let Some(live_terminal) = self.live_terminal.as_mut() {
                    live_terminal.write(&poll.output);
                }
                self.apply_live_vt_side_effects(&poll.output);
                self.rebuild_live_shell_rows(&poll.status);
            }
        }

        self.snapshot()
    }

    pub fn disconnect_live_shell(&mut self) -> TerminalSnapshot {
        self.disconnect_live_shell_handle();
        self.state = ConnectionState::Disconnected;
        self.observed_host_key_sha256 = None;
        self.pending_clipboard_text = None;
        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt && row.text.is_empty()));
        self.rows
            .push(TerminalRow::warning("live shell disconnected"));
        self.rows.push(TerminalRow::prompt());
        self.cursor_column = 0;
        self.local_input.clear();
        self.clear_reverse_search();
        self.clear_demo_editor();
        self.clear_demo_pager();
        self.clear_demo_mouse();
        self.clear_demo_tui();
        self.clear_demo_grid();
        self.snapshot()
    }

    pub fn codex_snapshot(&self) -> codex::CodexSnapshot {
        self.codex_session
            .as_ref()
            .map(codex::CodexSession::snapshot)
            .unwrap_or_else(codex::CodexSnapshot::disconnected)
    }

    pub fn codex_event_revision(&self) -> u64 {
        self.codex_session
            .as_ref()
            .map(codex::CodexSession::event_revision)
            .unwrap_or(0)
    }

    pub fn start_codex_password(
        &mut self,
        profile: HostProfile,
        password: String,
        cwd: Option<String>,
    ) -> codex::CodexSnapshot {
        #[cfg(not(feature = "native-integrations"))]
        let _ = (&profile, &password, &cwd);

        #[cfg(feature = "native-integrations")]
        {
            self.codex_session = None;
            return match codex::CodexSession::start_password(profile, password, cwd) {
                Ok(mut session) => {
                    let snapshot = session.poll();
                    self.codex_session = Some(session);
                    snapshot
                }
                Err(error) => codex::CodexSnapshot::failure(error),
            };
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            codex::CodexSnapshot::failure(
                "russh native integration is not compiled into this build",
            )
        }
    }

    pub fn start_codex_private_key(
        &mut self,
        profile: HostProfile,
        private_key_pem: String,
        passphrase: Option<String>,
        cwd: Option<String>,
    ) -> codex::CodexSnapshot {
        #[cfg(not(feature = "native-integrations"))]
        let _ = (&profile, &private_key_pem, &passphrase, &cwd);

        #[cfg(feature = "native-integrations")]
        {
            self.codex_session = None;
            return match codex::CodexSession::start_private_key(
                profile,
                private_key_pem,
                passphrase,
                cwd,
            ) {
                Ok(mut session) => {
                    let snapshot = session.poll();
                    self.codex_session = Some(session);
                    snapshot
                }
                Err(error) => codex::CodexSnapshot::failure(error),
            };
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            codex::CodexSnapshot::failure(
                "russh native integration is not compiled into this build",
            )
        }
    }

    pub fn poll_codex(&mut self) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session.poll(),
            None => codex::CodexSnapshot::disconnected(),
        }
    }

    pub fn send_codex_message(&mut self, text: &str) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .send_user_message(text)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn update_codex_settings(
        &mut self,
        model: Option<&str>,
        approval_policy: Option<&str>,
        sandbox: Option<&str>,
    ) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .update_settings(model, approval_policy, sandbox)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn browse_codex_directory(&mut self, path: &str) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .browse_directory(path)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn list_codex_threads(
        &mut self,
        cwd: Option<&str>,
        search_term: Option<&str>,
    ) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .list_threads(cwd, search_term)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn list_codex_threads_page(
        &mut self,
        cwd: Option<&str>,
        search_term: Option<&str>,
        cursor: Option<&str>,
        archived: bool,
        append: bool,
    ) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .list_threads_page(cwd, search_term, cursor, archived, append)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn start_codex_thread(&mut self, cwd: Option<&str>) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .start_thread(cwd)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn resume_codex_thread(&mut self, thread_id: &str) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .resume_thread(thread_id)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn read_codex_thread(&mut self, thread_id: &str) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .read_thread(thread_id)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn load_more_codex_thread_turns(
        &mut self,
        thread_id: &str,
        cursor: Option<&str>,
    ) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .load_more_thread_turns(thread_id, cursor)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn rename_codex_thread(&mut self, thread_id: &str, name: &str) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .rename_thread(thread_id, name)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn archive_codex_thread(&mut self, thread_id: &str) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .archive_thread(thread_id)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn unarchive_codex_thread(&mut self, thread_id: &str) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .unarchive_thread(thread_id)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn delete_codex_thread(&mut self, thread_id: &str) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .delete_thread(thread_id)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn fork_codex_thread(
        &mut self,
        thread_id: &str,
        cwd: Option<&str>,
    ) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .fork_thread(thread_id, cwd)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn interrupt_codex_turn(&mut self) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .interrupt_turn()
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn answer_codex_approval(
        &mut self,
        request_id: &str,
        decision: &str,
    ) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => session
                .answer_approval(request_id, decision)
                .unwrap_or_else(codex::CodexSnapshot::failure),
            None => codex::CodexSnapshot::failure("Codex is not connected"),
        }
    }

    pub fn disconnect_codex(&mut self) -> codex::CodexSnapshot {
        match self.codex_session.as_mut() {
            Some(session) => {
                session.disconnect();
                let snapshot = session.snapshot();
                self.codex_session = None;
                snapshot
            }
            None => codex::CodexSnapshot::disconnected(),
        }
    }

    pub fn send_command(&mut self, input: &str) -> TerminalSnapshot {
        let command = input.trim();
        if command.is_empty() {
            return self.snapshot();
        }

        #[cfg(feature = "native-integrations")]
        if let Some(handle) = &self.live_shell {
            let mut input = input.to_string();
            if !input.ends_with('\n') {
                input.push('\n');
            }

            if let Err(error) = handle.send_input(&input) {
                self.rows
                    .retain(|row| !(row.style == TerminalRowStyle::Prompt && row.text.is_empty()));
                self.rows
                    .push(TerminalRow::warning("live shell input failed"));
                self.rows.push(TerminalRow::muted(error));
                self.rows.push(TerminalRow::prompt());
                self.cursor_column = 0;
                return self.snapshot();
            }

            return self.poll_live_shell();
        }

        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt && row.text.is_empty()));
        self.rows.push(TerminalRow::command(command));
        self.record_command_history(command);
        if self.run_local_full_screen_command(command) {
            self.local_input.clear();
            self.clear_history_navigation();
            return self.snapshot();
        }
        let output_rows = self.command_output(command);
        self.rows.extend(output_rows);
        self.local_input.clear();
        self.clear_history_navigation();
        self.push_local_prompt();
        self.snapshot()
    }

    pub fn send_terminal_input(&mut self, input: &str) -> TerminalSnapshot {
        if input.is_empty() {
            return self.snapshot();
        }

        #[cfg(feature = "native-integrations")]
        if let Some(handle) = &self.live_shell {
            if let Err(error) = handle.send_input(input) {
                self.rows
                    .retain(|row| !(row.style == TerminalRowStyle::Prompt && row.text.is_empty()));
                self.rows
                    .push(TerminalRow::warning("live shell input failed"));
                self.rows.push(TerminalRow::muted(error));
                self.rows.push(TerminalRow::prompt());
                self.cursor_column = 0;
                return self.snapshot();
            }

            return self.poll_live_shell();
        }

        if self.demo_editor_active {
            self.apply_demo_editor_input(input);
            return self.snapshot();
        }

        if self.demo_pager_active {
            self.apply_demo_pager_input(input);
            return self.snapshot();
        }

        if self.demo_mouse_active {
            self.apply_demo_mouse_input(input);
            return self.snapshot();
        }

        if self.demo_tui.is_some() {
            self.apply_demo_tui_input(input);
            return self.snapshot();
        }

        self.clear_demo_grid();
        self.apply_local_terminal_input(input);
        self.snapshot()
    }

    pub fn resize_terminal(&mut self, cols: u32, rows: u32) -> TerminalSnapshot {
        let cols = cols.clamp(20, 300);
        let rows = rows.clamp(8, 120);
        if self.terminal_cols == cols && self.terminal_rows == rows {
            return self.snapshot();
        }

        self.terminal_cols = cols;
        self.terminal_rows = rows;
        self.renderer.borrow_mut().resize_cells(cols, rows);
        if self.demo_editor_active {
            self.refresh_demo_editor_status();
        }
        if self.demo_pager_active {
            self.demo_pager_offset = self.demo_pager_offset.min(self.demo_pager_max_offset());
        }

        #[cfg(feature = "native-integrations")]
        if let Some(handle) = &self.live_shell {
            if let Err(error) = handle.resize(cols, rows) {
                self.rows
                    .retain(|row| !(row.style == TerminalRowStyle::Prompt && row.text.is_empty()));
                self.rows.push(TerminalRow::warning("pty resize failed"));
                self.rows.push(TerminalRow::muted(error));
                self.rows.push(TerminalRow::prompt());
                self.cursor_column = 0;
                return self.snapshot();
            }
            if let Some(live_terminal) = self.live_terminal.as_mut() {
                if !live_terminal.resize(cols, rows) {
                    self.live_terminal = ghostty_adapter::LiveTerminalState::new(cols, rows);
                }
            }
            return self.poll_live_shell();
        }

        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        self.push_local_prompt();
        self.snapshot()
    }

    pub fn clear_terminal(&mut self) -> TerminalSnapshot {
        self.clear_local_screen();
        if !self.demo_editor_active
            && !self.demo_pager_active
            && !self.demo_mouse_active
            && self.demo_tui.is_none()
        {
            self.clear_demo_grid();
        }
        #[cfg(feature = "native-integrations")]
        {
            if !self.demo_editor_active
                && !self.demo_pager_active
                && !self.demo_mouse_active
                && self.demo_tui.is_none()
            {
                if self.live_shell.is_some() {
                    self.live_terminal = ghostty_adapter::LiveTerminalState::new(
                        self.terminal_cols,
                        self.terminal_rows,
                    );
                }
            }
        }
        self.last_grid_signature.replace(None);
        self.renderer.borrow_mut().invalidate();
        self.snapshot()
    }

    pub fn reset_terminal(&mut self) -> TerminalSnapshot {
        self.rows.clear();
        self.local_input.clear();
        self.clear_history_navigation();
        self.clear_reverse_search();
        self.clear_demo_editor();
        self.clear_demo_pager();
        self.clear_demo_mouse();
        self.clear_demo_tui();
        self.clear_demo_grid();
        self.pending_clipboard_text = None;
        self.cursor_column = 0;
        #[cfg(feature = "native-integrations")]
        {
            if self.live_shell.is_some() {
                self.live_terminal =
                    ghostty_adapter::LiveTerminalState::new(self.terminal_cols, self.terminal_rows);
            } else {
                self.live_terminal = None;
            }
        }
        self.last_grid_signature.replace(None);
        self.renderer.borrow_mut().invalidate();
        self.push_local_prompt();
        self.snapshot()
    }

    fn command_output(&mut self, command: &str) -> Vec<TerminalRow> {
        match command {
            "pwd" => vec![TerminalRow::muted("/home/shellow")],
            "ls" => vec![TerminalRow::muted("api  deploy  logs  tmux  shellow")],
            "whoami" => vec![TerminalRow::muted("shellow")],
            "shellow integrations" => vec![TerminalRow::muted(integrations::summary_line(
                &self.integration,
            ))],
            "shellow renderer" => vec![TerminalRow::muted(self.render_frame(960, 480).summary())],
            "shellow ssh" => vec![TerminalRow::muted(ssh::demo_transport_summary())],
            "shellow ghostty" => vec![TerminalRow::muted(ghostty_adapter::demo_terminal_summary())],
            "shellow links" => vec![
                TerminalRow::muted("Shellow link detection demo"),
                TerminalRow::muted("Open build log: https://example.com/shellow/build/42"),
                TerminalRow::muted("Docs mirror: https://docs.example.com/shellow?view=terminal"),
            ],
            "shellow size" => vec![TerminalRow::muted(format!(
                "terminal size={}x{}",
                self.terminal_cols, self.terminal_rows
            ))],
            "shellow ansi" => Vec::new(),
            "shellow wide" => Vec::new(),
            "shellow scrollback" => Vec::new(),
            "shellow cursor" => Vec::new(),
            "shellow mouse" => Vec::new(),
            "shellow osc52" => {
                let bytes = osc52_demo_bytes();
                self.apply_vt_side_effects(&bytes);
                ghostty_adapter::terminal_rows_from_vt_output(&bytes)
            }
            "shellow bell" => {
                let bytes = bell_demo_bytes();
                self.apply_vt_side_effects(&bytes);
                ghostty_adapter::terminal_rows_from_vt_output(&bytes)
            }
            "shellow title" => {
                let bytes = title_demo_bytes();
                self.apply_vt_side_effects(&bytes);
                ghostty_adapter::terminal_rows_from_vt_output(&bytes)
            }
            "clear" => Vec::new(),
            "vim" | "vi" | "nvim" | "less" | "more" | "shellow less" | "nano" | "pico" | "top"
            | "htop" | "tmux" | "shellow top" | "shellow tmux" => Vec::new(),
            _ => vec![TerminalRow::muted(format!(
                "queued for future SSH channel: {command}"
            ))],
        }
    }

    fn apply_local_terminal_input(&mut self, input: &str) {
        if self.reverse_search_active {
            self.apply_reverse_search_input(input);
            return;
        }

        let mut chars = input.chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                '\r' | '\n' => self.commit_local_input(),
                '\u{7f}' | '\u{8}' => {
                    self.clear_history_navigation();
                    self.delete_before_local_cursor();
                    self.replace_local_prompt();
                }
                '\u{3}' => self.cancel_local_input("^C"),
                '\u{4}' => self.cancel_local_input("^D"),
                '\u{c}' => self.clear_local_screen(),
                '\u{1a}' => self.cancel_local_input("^Z"),
                '\u{1}' => {
                    self.move_local_cursor_home();
                    self.replace_local_prompt();
                }
                '\u{5}' => {
                    self.move_local_cursor_end();
                    self.replace_local_prompt();
                }
                '\u{b}' => {
                    self.kill_local_input_after_cursor();
                    self.replace_local_prompt();
                }
                '\u{15}' => {
                    self.kill_local_input_before_cursor();
                    self.replace_local_prompt();
                }
                '\u{17}' => {
                    self.delete_word_before_local_cursor();
                    self.replace_local_prompt();
                }
                '\u{12}' => self.start_reverse_search(),
                '\t' => self.record_control_input("Tab"),
                '\u{1b}' => match chars.peek().copied() {
                    Some('[') => {
                        chars.next();
                        self.apply_local_escape_key(decode_csi_key(&mut chars));
                    }
                    Some('O') => {
                        chars.next();
                        self.apply_local_escape_key(decode_ss3_key(&mut chars));
                    }
                    Some(next) if !next.is_control() => {
                        chars.next();
                        self.clear_history_navigation();
                        self.record_control_input(format!("Alt-{}", key_name(next)));
                    }
                    _ => {
                        self.clear_history_navigation();
                        self.record_control_input("Esc");
                    }
                },
                character if character.is_control() => {
                    self.clear_history_navigation();
                    self.record_control_input(format!("Ctrl-{}", control_name(character)));
                }
                character => {
                    self.clear_history_navigation();
                    self.insert_local_character(character);
                    self.replace_local_prompt();
                }
            }
        }
    }

    fn local_input_len(&self) -> usize {
        self.local_input.chars().count()
    }

    fn local_byte_index(&self, char_index: usize) -> usize {
        if char_index == 0 {
            return 0;
        }

        self.local_input
            .char_indices()
            .nth(char_index)
            .map(|(index, _)| index)
            .unwrap_or(self.local_input.len())
    }

    fn clamp_local_cursor(&mut self) {
        self.local_cursor = self.local_cursor.min(self.local_input_len());
    }

    fn set_local_input_to_end(&mut self, input: String) {
        self.local_input = input;
        self.local_cursor = self.local_input_len();
    }

    fn insert_local_character(&mut self, character: char) {
        self.clamp_local_cursor();
        let index = self.local_byte_index(self.local_cursor);
        self.local_input.insert(index, character);
        self.local_cursor += 1;
    }

    fn delete_before_local_cursor(&mut self) {
        self.clamp_local_cursor();
        if self.local_cursor == 0 {
            return;
        }

        let start = self.local_byte_index(self.local_cursor - 1);
        let end = self.local_byte_index(self.local_cursor);
        self.local_input.replace_range(start..end, "");
        self.local_cursor -= 1;
    }

    fn delete_at_local_cursor(&mut self) {
        self.clamp_local_cursor();
        if self.local_cursor >= self.local_input_len() {
            return;
        }

        let start = self.local_byte_index(self.local_cursor);
        let end = self.local_byte_index(self.local_cursor + 1);
        self.local_input.replace_range(start..end, "");
    }

    fn move_local_cursor_left(&mut self) {
        self.local_cursor = self.local_cursor.saturating_sub(1);
    }

    fn move_local_cursor_right(&mut self) {
        self.local_cursor = (self.local_cursor + 1).min(self.local_input_len());
    }

    fn move_local_cursor_home(&mut self) {
        self.local_cursor = 0;
    }

    fn move_local_cursor_end(&mut self) {
        self.local_cursor = self.local_input_len();
    }

    fn kill_local_input_after_cursor(&mut self) {
        self.clamp_local_cursor();
        let index = self.local_byte_index(self.local_cursor);
        self.local_input.truncate(index);
    }

    fn kill_local_input_before_cursor(&mut self) {
        self.clamp_local_cursor();
        let index = self.local_byte_index(self.local_cursor);
        self.local_input.replace_range(0..index, "");
        self.local_cursor = 0;
    }

    fn delete_word_before_local_cursor(&mut self) {
        self.clamp_local_cursor();
        if self.local_cursor == 0 {
            return;
        }

        let chars: Vec<char> = self.local_input.chars().collect();
        let mut start = self.local_cursor;
        while start > 0 && chars[start - 1].is_whitespace() {
            start -= 1;
        }
        while start > 0 && !chars[start - 1].is_whitespace() {
            start -= 1;
        }

        let start_byte = self.local_byte_index(start);
        let end_byte = self.local_byte_index(self.local_cursor);
        self.local_input.replace_range(start_byte..end_byte, "");
        self.local_cursor = start;
    }

    fn commit_local_input(&mut self) {
        let command = self.local_input.trim().to_string();
        if command.is_empty() {
            self.replace_local_prompt();
            return;
        }

        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        self.rows.push(TerminalRow::command(&command));
        self.record_command_history(&command);
        if self.run_local_full_screen_command(&command) {
            self.local_input.clear();
            self.local_cursor = 0;
            self.clear_history_navigation();
            return;
        }
        let output_rows = self.command_output(&command);
        self.rows.extend(output_rows);
        self.local_input.clear();
        self.local_cursor = 0;
        self.clear_history_navigation();
        self.push_local_prompt();
    }

    fn cancel_local_input(&mut self, marker: &str) {
        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        let interrupted = if self.local_input.is_empty() {
            marker.to_string()
        } else {
            format!("{} {}", self.local_input, marker)
        };
        self.rows.push(TerminalRow::command(interrupted));
        self.local_input.clear();
        self.local_cursor = 0;
        self.clear_history_navigation();
        self.push_local_prompt();
    }

    fn clear_local_screen(&mut self) {
        self.rows.clear();
        self.local_input.clear();
        self.local_cursor = 0;
        self.clear_history_navigation();
        self.clear_reverse_search();
        self.push_local_prompt();
    }

    fn record_control_input(&mut self, key: impl Into<String>) {
        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        self.rows.push(TerminalRow::muted(format!(
            "{} sent to terminal input stream",
            key.into()
        )));
        self.push_local_prompt();
    }

    fn apply_local_escape_key(&mut self, key: &'static str) {
        match key {
            "Up" => self.navigate_local_history(-1),
            "Down" => self.navigate_local_history(1),
            "Left" => {
                self.move_local_cursor_left();
                self.replace_local_prompt();
            }
            "Right" => {
                self.move_local_cursor_right();
                self.replace_local_prompt();
            }
            "Home" => {
                self.move_local_cursor_home();
                self.replace_local_prompt();
            }
            "End" => {
                self.move_local_cursor_end();
                self.replace_local_prompt();
            }
            "Delete" => {
                self.delete_at_local_cursor();
                self.replace_local_prompt();
            }
            _ => {
                self.clear_history_navigation();
                self.record_control_input(key);
            }
        }
    }

    fn navigate_local_history(&mut self, direction: isize) {
        if self.command_history.is_empty() {
            self.replace_local_prompt();
            return;
        }

        let next_cursor = match (self.history_cursor, direction) {
            (None, -1) => {
                self.history_draft = self.local_input.clone();
                Some(self.command_history.len() - 1)
            }
            (Some(cursor), -1) => Some(cursor.saturating_sub(1)),
            (Some(cursor), 1) if cursor + 1 < self.command_history.len() => Some(cursor + 1),
            (Some(_), 1) => None,
            (cursor, _) => cursor,
        };

        self.history_cursor = next_cursor;
        let recalled = match next_cursor {
            Some(cursor) => self.command_history[cursor].clone(),
            None => std::mem::take(&mut self.history_draft),
        };
        self.set_local_input_to_end(recalled);
        self.replace_local_prompt();
    }

    fn record_command_history(&mut self, command: &str) {
        let command = command.trim();
        if command.is_empty() {
            return;
        }

        if self
            .command_history
            .last()
            .is_none_or(|previous| previous != command)
        {
            self.command_history.push(command.to_string());
        }
        self.clear_history_navigation();
    }

    fn clear_history_navigation(&mut self) {
        self.history_cursor = None;
        self.history_draft.clear();
    }

    fn start_reverse_search(&mut self) {
        self.clear_history_navigation();
        self.reverse_search_active = true;
        self.reverse_search_query.clear();
        self.reverse_search_draft = self.local_input.clone();
        self.update_reverse_search_match(false);
        self.replace_local_prompt();
    }

    fn apply_reverse_search_input(&mut self, input: &str) {
        let mut chars = input.chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                '\r' | '\n' => self.accept_reverse_search(true),
                '\u{7f}' | '\u{8}' => {
                    self.reverse_search_query.pop();
                    self.update_reverse_search_match(false);
                    self.replace_local_prompt();
                }
                '\u{12}' => {
                    self.update_reverse_search_match(true);
                    self.replace_local_prompt();
                }
                '\u{3}' => self.cancel_reverse_search_with_marker("^C"),
                '\u{7}' | '\u{1b}' => {
                    if character == '\u{1b}' {
                        match chars.peek().copied() {
                            Some('[') => {
                                chars.next();
                                self.apply_reverse_search_key(decode_csi_key(&mut chars));
                                continue;
                            }
                            Some('O') => {
                                chars.next();
                                self.apply_reverse_search_key(decode_ss3_key(&mut chars));
                                continue;
                            }
                            _ => {}
                        }
                    }
                    self.cancel_reverse_search();
                }
                character if character.is_control() => {}
                character => {
                    self.reverse_search_query.push(character);
                    self.update_reverse_search_match(false);
                    self.replace_local_prompt();
                }
            }
        }
    }

    fn apply_reverse_search_key(&mut self, key: &'static str) {
        match key {
            "Up" | "PageUp" => self.update_reverse_search_match(true),
            "Down" | "PageDown" => self.update_reverse_search_match(false),
            _ => {}
        }
        self.replace_local_prompt();
    }

    fn update_reverse_search_match(&mut self, cycle_older: bool) {
        let before = if cycle_older {
            self.reverse_search_match_index
        } else {
            None
        };
        let mut match_index = self.find_reverse_history_match(before);
        if match_index.is_none() && cycle_older {
            match_index = self.find_reverse_history_match(None);
        }

        self.reverse_search_match_index = match_index;
        let matched = match match_index {
            Some(index) => self.command_history[index].clone(),
            None => self.reverse_search_query.clone(),
        };
        self.set_local_input_to_end(matched);
    }

    fn find_reverse_history_match(&self, before: Option<usize>) -> Option<usize> {
        let upper_bound = before.unwrap_or(self.command_history.len());
        (0..upper_bound)
            .rev()
            .find(|&index| self.command_history[index].contains(&self.reverse_search_query))
    }

    fn accept_reverse_search(&mut self, execute: bool) {
        let accepted = self.local_input.clone();
        self.clear_reverse_search();
        self.set_local_input_to_end(accepted);
        if execute {
            self.commit_local_input();
        } else {
            self.replace_local_prompt();
        }
    }

    fn cancel_reverse_search(&mut self) {
        let draft = self.reverse_search_draft.clone();
        self.clear_reverse_search();
        self.set_local_input_to_end(draft);
        self.replace_local_prompt();
    }

    fn cancel_reverse_search_with_marker(&mut self, marker: &str) {
        let draft = self.reverse_search_draft.clone();
        self.clear_reverse_search();
        self.set_local_input_to_end(draft);
        self.cancel_local_input(marker);
    }

    fn clear_reverse_search(&mut self) {
        self.reverse_search_active = false;
        self.reverse_search_query.clear();
        self.reverse_search_match_index = None;
        self.reverse_search_draft.clear();
    }

    fn replace_local_prompt(&mut self) {
        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        self.push_local_prompt();
    }

    fn push_local_prompt(&mut self) {
        self.clamp_local_cursor();
        self.cursor_column = self.local_cursor;
        if self.reverse_search_active {
            let prompt = if self.reverse_search_match_index.is_some()
                || self.reverse_search_query.is_empty()
            {
                format!("(reverse-i-search)`{}':", self.reverse_search_query)
            } else {
                format!("(failed reverse-i-search)`{}':", self.reverse_search_query)
            };
            self.rows.push(TerminalRow {
                prompt,
                text: self.local_input.clone(),
                style: TerminalRowStyle::Prompt,
            });
            return;
        }

        self.rows.push(TerminalRow {
            prompt: "$".to_string(),
            text: self.local_input.clone(),
            style: TerminalRowStyle::Prompt,
        });
    }

    fn terminal_rows_from_remote_output(&mut self, output: &str) -> Vec<TerminalRow> {
        self.apply_vt_side_effects(output.as_bytes());
        let mut rows = ghostty_adapter::terminal_rows_from_vt_output(output.as_bytes());

        if rows.is_empty() {
            rows.push(TerminalRow::muted("remote command produced no output"));
        }

        rows
    }

    fn disconnect_live_shell_handle(&mut self) {
        #[cfg(feature = "native-integrations")]
        {
            if let Some(handle) = self.live_shell.take() {
                handle.disconnect();
            }
            self.live_terminal = None;
        }
    }

    #[cfg(feature = "native-integrations")]
    fn rebuild_live_shell_rows(&mut self, status: &ssh::LiveShellStatus) {
        self.rows.clear();
        self.rows
            .push(TerminalRow::command(format!("ssh {}", self.host)));

        match status {
            ssh::LiveShellStatus::Connecting => {
                self.state = ConnectionState::Connecting;
                self.rows.push(TerminalRow::muted("Connecting..."));
            }
            ssh::LiveShellStatus::Connected {
                observed_host_key_sha256,
            } => {
                self.state = ConnectionState::Connected;
                self.observed_host_key_sha256 = observed_host_key_sha256.clone();
                self.rows
                    .push(TerminalRow::success("interactive russh PTY connected"));
                if let Some(fingerprint) = observed_host_key_sha256 {
                    self.rows.push(TerminalRow::muted(format!(
                        "host-key=observed {fingerprint}"
                    )));
                }
            }
            ssh::LiveShellStatus::Closed => {
                self.state = ConnectionState::Disconnected;
                self.rows.push(TerminalRow::warning("remote shell closed"));
            }
            ssh::LiveShellStatus::Failed(error) => {
                self.state = ConnectionState::Disconnected;
                self.rows.push(TerminalRow::warning("remote shell failed"));
                self.rows.push(TerminalRow::muted(error));
            }
        }

        self.rows.push(TerminalRow::prompt());
        self.cursor_column = 0;
    }

    #[cfg(feature = "native-integrations")]
    fn apply_live_vt_side_effects(&mut self, output: &[u8]) {
        if let Some(live_terminal) = &self.live_terminal {
            self.bell_count += live_terminal.take_bell_count();
            if let Some(title) = live_terminal.title() {
                self.title = title;
            }
        } else {
            self.bell_count += ghostty_adapter::terminal_bell_count_from_vt_bytes(output);
            if let Some(title) = ghostty_adapter::terminal_title_from_vt_bytes(output) {
                self.title = title;
            }
        }
        if let Some(clipboard_text) = ghostty_adapter::terminal_clipboard_from_vt_bytes(output) {
            self.clipboard_sequence = self.clipboard_sequence.saturating_add(1);
            self.pending_clipboard_text = Some(clipboard_text);
        }
    }

    fn apply_vt_side_effects(&mut self, output: &[u8]) {
        self.bell_count += ghostty_adapter::terminal_bell_count_from_vt_bytes(output);
        if let Some(title) = ghostty_adapter::terminal_title_from_vt_bytes(output) {
            self.title = title;
        }
        if let Some(clipboard_text) = ghostty_adapter::terminal_clipboard_from_vt_bytes(output) {
            self.clipboard_sequence = self.clipboard_sequence.saturating_add(1);
            self.pending_clipboard_text = Some(clipboard_text);
        }
    }

    fn run_local_full_screen_command(&mut self, command: &str) -> bool {
        match command {
            "vim" | "vi" | "nvim" => {
                self.demo_editor_active = true;
                self.demo_editor_text.clear();
                self.demo_editor_status =
                    "INSERT demo - type text, arrows update status, Esc exits".to_string();
                self.cursor_column = 0;
                true
            }
            "less" | "more" | "shellow less" => {
                self.demo_pager_active = true;
                self.demo_pager_offset = 0;
                self.demo_pager_status =
                    "LESS demo - Up/Down, PgUp/PgDn, Home/End, q exits".to_string();
                self.cursor_column = 0;
                true
            }
            "shellow ansi" => {
                self.demo_grid_bytes =
                    Some(ansi_demo_bytes(self.terminal_cols, self.terminal_rows));
                self.cursor_column = 0;
                true
            }
            "shellow wide" => {
                self.demo_grid_bytes =
                    Some(wide_demo_bytes(self.terminal_cols, self.terminal_rows));
                self.cursor_column = 0;
                true
            }
            "shellow scrollback" => {
                self.demo_grid_bytes = Some(scrollback_demo_bytes(
                    self.terminal_cols,
                    self.terminal_rows,
                ));
                self.cursor_column = 0;
                true
            }
            "shellow cursor" => {
                self.demo_grid_bytes = Some(cursor_demo_bytes(
                    self.terminal_cols,
                    self.terminal_rows,
                    TerminalCursorShape::Bar,
                ));
                self.cursor_column = 0;
                true
            }
            "shellow mouse" => {
                self.demo_mouse_active = true;
                self.demo_mouse_status =
                    "Mouse demo ready - tap or drag terminal rows to send SGR mouse events"
                        .to_string();
                self.cursor_column = 0;
                true
            }
            "nano" | "pico" => {
                self.start_demo_tui(LocalTuiDemo::Nano);
                true
            }
            "top" | "htop" | "shellow top" => {
                self.start_demo_tui(LocalTuiDemo::Top);
                true
            }
            "tmux" | "shellow tmux" => {
                self.start_demo_tui(LocalTuiDemo::Tmux);
                true
            }
            _ => false,
        }
    }

    fn start_demo_tui(&mut self, kind: LocalTuiDemo) {
        self.demo_tui = Some(kind);
        self.demo_tui_status = kind.default_status().to_string();
        self.demo_tui_prefix_armed = false;
        self.cursor_column = 0;
    }

    fn apply_demo_editor_input(&mut self, input: &str) {
        let mut chars = input.chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                '\u{1b}' if chars.peek() == Some(&'[') || chars.peek() == Some(&'O') => {
                    let introducer = chars.next().unwrap_or('[');
                    let key = if introducer == 'O' {
                        decode_ss3_key(&mut chars)
                    } else {
                        decode_csi_key(&mut chars)
                    };
                    self.demo_editor_status = match key {
                        "BracketedPasteStart" => "Bracketed paste started".to_string(),
                        "BracketedPasteEnd" => format!(
                            "Bracketed paste wrapper received; buffer has {} chars",
                            self.demo_editor_text.chars().count()
                        ),
                        _ if introducer == 'O' && is_application_cursor_key(key) => {
                            format!("Application cursor {key} key handled inside alternate screen")
                        }
                        _ => format!("{key} key handled inside alternate screen"),
                    };
                }
                '\u{1b}' if chars.peek().is_some_and(|next| !next.is_control()) => {
                    let key = chars.next().unwrap_or_default();
                    self.demo_editor_status =
                        format!("Alt-{} sent to alternate-screen app", key_name(key));
                }
                '\u{1b}' => {
                    self.exit_demo_editor("demo editor closed with Esc");
                    return;
                }
                '\u{3}' => {
                    self.exit_demo_editor("demo editor interrupted with Ctrl-C");
                    return;
                }
                '\u{7f}' | '\u{8}' => {
                    self.demo_editor_text.pop();
                    self.demo_editor_status =
                        "Backspace edited alternate-screen buffer".to_string();
                }
                '\r' | '\n' => {
                    self.demo_editor_text.push('\n');
                    self.demo_editor_status = "Enter inserted a new editor line".to_string();
                }
                '\t' => {
                    self.demo_editor_text.push_str("    ");
                    self.demo_editor_status = "Tab inserted spaces in editor buffer".to_string();
                }
                '\u{c}' => {
                    self.demo_editor_status = "Ctrl-L redraw requested".to_string();
                }
                '\u{1a}' => {
                    self.demo_editor_status = "Ctrl-Z sent to foreground app".to_string();
                }
                character if character.is_control() => {
                    self.demo_editor_status = format!(
                        "Ctrl-{} sent to alternate-screen app",
                        control_name(character)
                    );
                }
                character => {
                    self.demo_editor_text.push(character);
                    self.demo_editor_status = format!(
                        "typed {} chars in alternate-screen buffer",
                        self.demo_editor_text.chars().count()
                    );
                }
            }
        }

        self.refresh_demo_editor_status();
    }

    fn apply_demo_pager_input(&mut self, input: &str) {
        let mut chars = input.chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                '\u{1b}' if chars.peek() == Some(&'[') || chars.peek() == Some(&'O') => {
                    let introducer = chars.next().unwrap_or('[');
                    let key = if introducer == 'O' {
                        decode_ss3_key(&mut chars)
                    } else {
                        decode_csi_key(&mut chars)
                    };
                    self.apply_demo_pager_key(key);
                }
                '\u{1b}' => {
                    self.exit_demo_pager("pager closed with Esc");
                    return;
                }
                '\u{3}' => {
                    self.exit_demo_pager("pager interrupted with Ctrl-C");
                    return;
                }
                'q' => {
                    self.exit_demo_pager("pager closed with q");
                    return;
                }
                ' ' | 'f' => {
                    self.scroll_demo_pager_by(self.demo_pager_visible_rows() as isize, "PageDown")
                }
                'b' => {
                    self.scroll_demo_pager_by(-(self.demo_pager_visible_rows() as isize), "PageUp")
                }
                'j' => self.scroll_demo_pager_by(1, "Down"),
                'k' => self.scroll_demo_pager_by(-1, "Up"),
                'g' => self.set_demo_pager_offset(0, "Home"),
                'G' => self.set_demo_pager_offset(self.demo_pager_max_offset(), "End"),
                character if character.is_control() => {
                    self.demo_pager_status =
                        format!("Ctrl-{} sent to pager", control_name(character));
                }
                character => {
                    self.demo_pager_status = format!("Pager ignored key {}", key_name(character));
                }
            }
        }
    }

    fn apply_demo_pager_key(&mut self, key: &str) {
        match key {
            "Up" => self.scroll_demo_pager_by(-1, "Up"),
            "Down" => self.scroll_demo_pager_by(1, "Down"),
            "PageUp" => {
                self.scroll_demo_pager_by(-(self.demo_pager_visible_rows() as isize), "PageUp")
            }
            "PageDown" => {
                self.scroll_demo_pager_by(self.demo_pager_visible_rows() as isize, "PageDown")
            }
            "Home" => self.set_demo_pager_offset(0, "Home"),
            "End" => self.set_demo_pager_offset(self.demo_pager_max_offset(), "End"),
            other => {
                self.demo_pager_status = format!("{other} key sent to pager");
            }
        }
    }

    fn scroll_demo_pager_by(&mut self, delta: isize, action: &str) {
        let current = self.demo_pager_offset as isize;
        let target = current.saturating_add(delta).max(0) as usize;
        self.set_demo_pager_offset(target, action);
    }

    fn set_demo_pager_offset(&mut self, target: usize, action: &str) {
        let max_offset = self.demo_pager_max_offset();
        let clamped = target.min(max_offset);
        self.demo_pager_offset = clamped;

        let first = clamped + 1;
        let last = (clamped + self.demo_pager_visible_rows()).min(DEMO_PAGER_LINES.len());
        self.demo_pager_status = if target > max_offset {
            format!("Bottom of file - lines {first}-{last}")
        } else if target == 0 && clamped == 0 && action == "Up" {
            format!("Top of file - lines {first}-{last}")
        } else {
            format!("{action}: showing lines {first}-{last}")
        };
    }

    fn demo_pager_visible_rows(&self) -> usize {
        pager_visible_rows(self.terminal_rows)
    }

    fn demo_pager_max_offset(&self) -> usize {
        DEMO_PAGER_LINES
            .len()
            .saturating_sub(self.demo_pager_visible_rows())
    }

    fn apply_demo_mouse_input(&mut self, input: &str) {
        let mut chars = input.chars().peekable();

        while let Some(character) = chars.next() {
            match character {
                '\u{1b}' if chars.peek() == Some(&'[') => {
                    chars.next();
                    if let Some(event) = decode_sgr_mouse_event(&mut chars) {
                        self.demo_mouse_status = format!(
                            "Mouse {} at col {} row {} ({})",
                            event.action, event.col, event.row, event.encoding
                        );
                    } else {
                        self.demo_mouse_status =
                            "Mouse demo received non-mouse CSI input".to_string();
                    }
                }
                '\u{1b}' => {
                    self.exit_demo_mouse("mouse demo closed with Esc");
                    return;
                }
                _ => {
                    self.demo_mouse_status =
                        "Mouse demo ignores keyboard input except Esc".to_string();
                }
            }
        }
    }

    fn apply_demo_tui_input(&mut self, input: &str) {
        let Some(kind) = self.demo_tui else { return };
        let mut chars = input.chars().peekable();

        while let Some(character) = chars.next() {
            if kind == LocalTuiDemo::Tmux && self.demo_tui_prefix_armed {
                self.demo_tui_prefix_armed = false;
                match character {
                    'd' => {
                        self.exit_demo_tui("tmux detached with Ctrl-B d");
                        return;
                    }
                    'c' => {
                        self.demo_tui_status = "tmux prefix + c created a new window".to_string();
                    }
                    'n' => {
                        self.demo_tui_status = "tmux prefix + n selected next window".to_string();
                    }
                    'p' => {
                        self.demo_tui_status =
                            "tmux prefix + p selected previous window".to_string();
                    }
                    '%' => {
                        self.demo_tui_status =
                            "tmux prefix + % split the pane vertically".to_string();
                    }
                    '"' => {
                        self.demo_tui_status =
                            "tmux prefix + \" split the pane horizontally".to_string();
                    }
                    other if other.is_control() => {
                        self.demo_tui_status =
                            format!("tmux prefix consumed Ctrl-{}", control_name(other));
                    }
                    other => {
                        self.demo_tui_status =
                            format!("tmux prefix + {} sent to session", key_name(other));
                    }
                }
                continue;
            }

            match character {
                '\u{1b}' if chars.peek() == Some(&'[') || chars.peek() == Some(&'O') => {
                    let introducer = chars.next().unwrap_or('[');
                    let key = if introducer == 'O' {
                        decode_ss3_key(&mut chars)
                    } else {
                        decode_csi_key(&mut chars)
                    };
                    self.apply_demo_tui_key(kind, key);
                }
                '\u{1b}' => {
                    self.exit_demo_tui(match kind {
                        LocalTuiDemo::Nano => "nano closed with Esc",
                        LocalTuiDemo::Top => "top closed with Esc",
                        LocalTuiDemo::Tmux => "tmux closed with Esc",
                    });
                    return;
                }
                '\u{2}' if kind == LocalTuiDemo::Tmux => {
                    self.demo_tui_prefix_armed = true;
                    self.demo_tui_status = "tmux prefix Ctrl-B armed".to_string();
                }
                '\u{18}' if kind == LocalTuiDemo::Nano => {
                    self.exit_demo_tui("nano closed with Ctrl-X");
                    return;
                }
                '\u{f}' if kind == LocalTuiDemo::Nano => {
                    self.demo_tui_status = "nano Ctrl-O writeout requested".to_string();
                }
                '\u{b}' if kind == LocalTuiDemo::Nano => {
                    self.demo_tui_status = "nano Ctrl-K cut current line".to_string();
                }
                '\u{15}' if kind == LocalTuiDemo::Nano => {
                    self.demo_tui_status = "nano Ctrl-U uncut text".to_string();
                }
                '\u{3}' if kind == LocalTuiDemo::Top => {
                    self.exit_demo_tui("top interrupted with Ctrl-C");
                    return;
                }
                'q' if kind == LocalTuiDemo::Top => {
                    self.exit_demo_tui("top closed with q");
                    return;
                }
                'q' if kind == LocalTuiDemo::Tmux => {
                    self.demo_tui_status = "q sent to tmux pane".to_string();
                }
                '\u{3}' if kind == LocalTuiDemo::Tmux => {
                    self.demo_tui_status = "Ctrl-C sent to tmux pane".to_string();
                }
                '\u{3}' => {
                    self.exit_demo_tui(match kind {
                        LocalTuiDemo::Nano => "nano interrupted with Ctrl-C",
                        LocalTuiDemo::Top => "top interrupted with Ctrl-C",
                        LocalTuiDemo::Tmux => "tmux interrupted with Ctrl-C",
                    });
                    return;
                }
                '\r' | '\n' if kind == LocalTuiDemo::Nano => {
                    self.demo_tui_status = "nano inserted a new line".to_string();
                }
                '\t' if kind == LocalTuiDemo::Nano => {
                    self.demo_tui_status = "nano inserted a tab".to_string();
                }
                character if character.is_control() => {
                    self.demo_tui_status =
                        format!("Ctrl-{} sent to {}", control_name(character), kind.label());
                }
                character => {
                    self.demo_tui_status =
                        format!("{} received key {}", kind.label(), key_name(character));
                }
            }
        }
    }

    fn apply_demo_tui_key(&mut self, kind: LocalTuiDemo, key: &str) {
        self.demo_tui_status = match kind {
            LocalTuiDemo::Nano => match key {
                "Up" | "Down" | "Left" | "Right" => {
                    format!("nano cursor moved with {key}")
                }
                "Home" => "nano jumped to line start".to_string(),
                "End" => "nano jumped to line end".to_string(),
                "PageUp" => "nano paged up".to_string(),
                "PageDown" => "nano paged down".to_string(),
                other => format!("{other} sent to nano"),
            },
            LocalTuiDemo::Top => match key {
                "Up" => "top selected previous process".to_string(),
                "Down" => "top selected next process".to_string(),
                "Left" | "Right" => format!("top changed sort column with {key}"),
                "PageUp" => "top scrolled process list up".to_string(),
                "PageDown" => "top scrolled process list down".to_string(),
                other => format!("{other} sent to top"),
            },
            LocalTuiDemo::Tmux => match key {
                "Up" | "Down" | "Left" | "Right" => {
                    format!("tmux pane received {key}")
                }
                "PageUp" => "tmux copy-mode PageUp".to_string(),
                "PageDown" => "tmux copy-mode PageDown".to_string(),
                other => format!("{other} sent to tmux"),
            },
        };
    }

    fn refresh_demo_editor_status(&mut self) {
        if self.demo_editor_active && self.demo_editor_status.is_empty() {
            self.demo_editor_status =
                "INSERT demo - type text, arrows update status, Esc exits".to_string();
        }
    }

    fn exit_demo_editor(&mut self, message: &str) {
        self.clear_demo_editor();
        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        self.rows.push(TerminalRow::muted(message));
        self.push_local_prompt();
    }

    fn clear_demo_editor(&mut self) {
        self.demo_editor_active = false;
        self.demo_editor_text.clear();
        self.demo_editor_status.clear();
    }

    fn exit_demo_pager(&mut self, message: &str) {
        self.clear_demo_pager();
        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        self.rows.push(TerminalRow::muted(message));
        self.push_local_prompt();
    }

    fn clear_demo_pager(&mut self) {
        self.demo_pager_active = false;
        self.demo_pager_offset = 0;
        self.demo_pager_status.clear();
    }

    fn exit_demo_mouse(&mut self, message: &str) {
        self.clear_demo_mouse();
        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        self.rows.push(TerminalRow::muted(message));
        self.push_local_prompt();
    }

    fn clear_demo_mouse(&mut self) {
        self.demo_mouse_active = false;
        self.demo_mouse_status.clear();
    }

    fn exit_demo_tui(&mut self, message: &str) {
        self.clear_demo_tui();
        self.rows
            .retain(|row| !(row.style == TerminalRowStyle::Prompt));
        self.rows.push(TerminalRow::muted(message));
        self.push_local_prompt();
    }

    fn clear_demo_tui(&mut self) {
        self.demo_tui = None;
        self.demo_tui_status.clear();
        self.demo_tui_prefix_armed = false;
    }

    fn grid_snapshot(&self) -> Option<TerminalGridSnapshot> {
        let snapshot = if self.demo_editor_active {
            self.demo_editor_grid()
        } else if self.demo_pager_active {
            Some(self.demo_pager_grid())
        } else if self.demo_mouse_active {
            Some(self.demo_mouse_grid())
        } else if self.demo_tui.is_some() {
            Some(self.demo_tui_grid())
        } else if let Some(bytes) = &self.demo_grid_bytes {
            Some(ghostty_adapter::terminal_grid_from_vt_bytes(
                bytes,
                self.terminal_cols,
                self.terminal_rows,
            ))
        } else {
            #[cfg(feature = "native-integrations")]
            if let Some(live_terminal) = &self.live_terminal {
                if let Some(snapshot) = live_terminal.snapshot() {
                    if snapshot.has_visible_content()
                        || snapshot.active_screen == TerminalScreenKind::Alternate
                    {
                        return Some(self.annotate_dirty_rows(snapshot));
                    }
                }
            }

            None
        };

        match snapshot {
            Some(snapshot) => Some(self.annotate_dirty_rows(snapshot)),
            None => {
                self.last_grid_signature.replace(None);
                None
            }
        }
    }

    fn grid_snapshot_viewport(
        &self,
        requested_first_row: usize,
        requested_row_count: usize,
    ) -> Option<TerminalGridSnapshot> {
        #[cfg(feature = "native-integrations")]
        {
            let live_terminal_is_primary_source = !self.demo_editor_active
                && !self.demo_pager_active
                && !self.demo_mouse_active
                && self.demo_tui.is_none()
                && self.demo_grid_bytes.is_none();
            if live_terminal_is_primary_source {
                if let Some(live_terminal) = &self.live_terminal {
                    if let Some(snapshot) =
                        live_terminal.snapshot_viewport(requested_first_row, requested_row_count)
                    {
                        if snapshot.has_visible_content()
                            || snapshot.active_screen == TerminalScreenKind::Alternate
                        {
                            return Some(snapshot);
                        }
                    }
                }
            }
        }

        self.grid_snapshot()
            .map(|grid| viewport_grid_snapshot(grid, requested_first_row, requested_row_count))
    }

    fn annotate_dirty_rows(&self, mut snapshot: TerminalGridSnapshot) -> TerminalGridSnapshot {
        let signature = GridRenderSignature::from_snapshot(&snapshot);
        let dirty_rows = {
            let mut previous = self.last_grid_signature.borrow_mut();
            let rows = previous
                .as_ref()
                .map(|previous| dirty_rows_between(previous, &signature))
                .unwrap_or_else(|| all_dirty_rows(&signature));
            *previous = Some(signature);
            rows
        };
        snapshot.dirty_rows = dirty_rows;
        snapshot
    }

    fn demo_editor_grid(&self) -> Option<TerminalGridSnapshot> {
        let cols = self.terminal_cols.max(20);
        let rows = self.terminal_rows.max(8);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"\x1b[?1049h\x1b[?1h\x1b[?2004h\x1b[2J\x1b[H");
        bytes.extend_from_slice(b"Shellow demo editor - alternate screen");
        bytes.extend_from_slice(b"\r\n\r\n");

        if self.demo_editor_text.is_empty() {
            bytes.extend_from_slice(b"Type here. Use arrows, Tab, Backspace, Ctrl-L, then Esc.");
        } else {
            bytes.extend_from_slice(self.demo_editor_text.as_bytes());
        }

        let status_row = rows.saturating_sub(1).max(1);
        let status = format!(
            "\x1b[{};1H\x1b[7m-- SHELLOW VIM DEMO -- {} -- {}x{} --\x1b[0m",
            status_row, self.demo_editor_status, cols, rows
        );
        bytes.extend_from_slice(status.as_bytes());

        let (cursor_row, cursor_col) = demo_editor_cursor(&self.demo_editor_text, cols, rows);
        let cursor = format!("\x1b[{};{}H", cursor_row + 1, cursor_col + 1);
        bytes.extend_from_slice(cursor.as_bytes());

        Some(ghostty_adapter::terminal_grid_from_vt_bytes(
            &bytes, cols, rows,
        ))
    }

    fn demo_pager_grid(&self) -> TerminalGridSnapshot {
        let cols = self.terminal_cols.max(20);
        let rows = self.terminal_rows.max(8);
        ghostty_adapter::terminal_grid_from_vt_bytes(
            &pager_demo_bytes(cols, rows, self.demo_pager_offset, &self.demo_pager_status),
            cols,
            rows,
        )
    }

    fn demo_mouse_grid(&self) -> TerminalGridSnapshot {
        let cols = self.terminal_cols.max(20);
        let rows = self.terminal_rows.max(8);
        ghostty_adapter::terminal_grid_from_vt_bytes(
            &mouse_demo_bytes(cols, rows, &self.demo_mouse_status),
            cols,
            rows,
        )
    }

    fn demo_tui_grid(&self) -> TerminalGridSnapshot {
        let cols = self.terminal_cols.max(20);
        let rows = self.terminal_rows.max(8);
        let kind = self.demo_tui.unwrap_or(LocalTuiDemo::Top);
        ghostty_adapter::terminal_grid_from_vt_bytes(
            &tui_demo_bytes(
                cols,
                rows,
                kind,
                &self.demo_tui_status,
                self.demo_tui_prefix_armed,
            ),
            cols,
            rows,
        )
    }

    fn clear_demo_grid(&mut self) {
        self.demo_grid_bytes = None;
    }
}

fn demo_editor_cursor(text: &str, cols: u32, rows: u32) -> (u32, u32) {
    let max_editor_row = rows.saturating_sub(3).max(2);
    let mut row = 2_u32;
    let mut col = 0_u32;

    for character in text.chars() {
        if character == '\n' {
            row = (row + 1).min(max_editor_row);
            col = 0;
            continue;
        }

        col += 1;
        if col >= cols {
            row = (row + 1).min(max_editor_row);
            col = 0;
        }
    }

    (row, col.min(cols.saturating_sub(1)))
}

fn all_dirty_rows(signature: &GridRenderSignature) -> Vec<usize> {
    (0..signature.lines.len()).collect()
}

fn dirty_rows_between(previous: &GridRenderSignature, current: &GridRenderSignature) -> Vec<usize> {
    if previous.cols != current.cols
        || previous.rows != current.rows
        || previous.active_screen != current.active_screen
        || previous.scrollback_len != current.scrollback_len
        || previous.lines.len() != current.lines.len()
    {
        return all_dirty_rows(current);
    }

    let mut dirty = vec![false; current.lines.len()];
    for row in 0..current.lines.len() {
        if previous.lines.get(row) != current.lines.get(row)
            || previous.styled_lines.get(row) != current.styled_lines.get(row)
        {
            dirty[row] = true;
        }
    }

    let cursor_changed = previous.cursor_column != current.cursor_column
        || previous.cursor_row != current.cursor_row
        || previous.cursor_visible != current.cursor_visible
        || previous.cursor_shape != current.cursor_shape;
    if cursor_changed {
        if let Some(row) = previous.cursor_dirty_row() {
            dirty[row] = true;
        }
        if let Some(row) = current.cursor_dirty_row() {
            dirty[row] = true;
        }
    }

    dirty
        .into_iter()
        .enumerate()
        .filter_map(|(row, is_dirty)| is_dirty.then_some(row))
        .collect()
}

struct MouseEvent {
    action: &'static str,
    col: u32,
    row: u32,
    encoding: &'static str,
}

fn decode_sgr_mouse_event<I>(chars: &mut std::iter::Peekable<I>) -> Option<MouseEvent>
where
    I: Iterator<Item = char>,
{
    if chars.next()? != '<' {
        return None;
    }

    let mut parameters = String::new();
    let mut final_byte = None;

    while let Some(character) = chars.next() {
        match character {
            'M' | 'm' => {
                final_byte = Some(character);
                break;
            }
            '0'..='9' | ';' => parameters.push(character),
            _ => return None,
        }
    }

    let final_byte = final_byte?;
    let mut parts = parameters.split(';');
    let button = parts.next()?.parse::<u32>().ok()?;
    let col = parts.next()?.parse::<u32>().ok()?;
    let row = parts.next()?.parse::<u32>().ok()?;

    Some(MouseEvent {
        action: if final_byte == 'm' {
            "release"
        } else if button & 32 != 0 {
            "drag"
        } else {
            "press"
        },
        col,
        row,
        encoding: match (button & 32 != 0, button & 3) {
            (true, 0) => "SGR 1006 left drag",
            (false, 0) => "SGR 1006 left",
            _ => "SGR 1006",
        },
    })
}

const DEMO_PAGER_LINES: &[&str] = &[
    "001 Shellow pager demo document",
    "002 A useful terminal must handle pagers, not only editors.",
    "003 less keeps the application in the alternate screen.",
    "004 Arrow keys move one line at a time.",
    "005 PageDown moves by a viewport.",
    "006 PageUp moves back by a viewport.",
    "007 Home jumps to the start of the file.",
    "008 End jumps to the final page.",
    "009 Space follows common less PageDown behavior.",
    "010 b follows common less PageUp behavior.",
    "011 j and k mirror vi-style pager navigation.",
    "012 q exits back to the normal shell prompt.",
    "013 Esc also exits for mobile toolbar parity.",
    "014 The status line reports the active line range.",
    "015 This demo exercises full-screen key routing.",
    "016 The renderer keeps monospace alignment stable.",
    "017 The selection layer still sits above the grid.",
    "018 Copy Terminal should contain only visible text.",
    "019 The pager is intentionally deterministic.",
    "020 Determinism makes simulator verification cheap.",
    "021 Remote less, man, git log, and journalctl behave similarly.",
    "022 tmux and top use the same alternate-screen path.",
    "023 Terminal size changes affect viewport paging.",
    "024 Cursor visibility is hidden inside pager mode.",
    "025 Dirty rows update after every scroll action.",
    "026 Page movement should not mutate shell history.",
    "027 The normal prompt returns when the pager exits.",
    "028 This line helps prove bottom navigation.",
    "029 Another line near the end of the document.",
    "030 Final page marker for Shellow pager QA.",
];

fn pager_visible_rows(rows: u32) -> usize {
    rows.saturating_sub(4).max(1) as usize
}

fn pager_demo_bytes(cols: u32, rows: u32, offset: usize, status: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let visible_rows = pager_visible_rows(rows);
    let max_offset = DEMO_PAGER_LINES.len().saturating_sub(visible_rows);
    let offset = offset.min(max_offset);
    let last = (offset + visible_rows).min(DEMO_PAGER_LINES.len());

    bytes.extend_from_slice(b"\x1b[?1049h\x1b[?1h\x1b[?25l\x1b[2J\x1b[H");
    bytes.extend_from_slice(b"\x1b[1m");
    push_terminal_line(&mut bytes, "Shellow pager demo", cols);
    bytes.extend_from_slice(b"\x1b[0m");
    push_terminal_line(&mut bytes, "Keys: PgUp/PgDn Home/End q", cols);
    push_terminal_line(&mut bytes, "", cols);

    for line in &DEMO_PAGER_LINES[offset..last] {
        push_terminal_line(&mut bytes, line, cols);
    }

    let first_line = offset + 1;
    let status = if status.is_empty() {
        "LESS demo ready"
    } else {
        status
    };
    let status = format!(
        "-- LESS {first_line}-{last}/{} {status} --",
        DEMO_PAGER_LINES.len()
    );
    let status = terminal_line_prefix(&status, cols);
    let status = format!("\x1b[{};1H\x1b[7m{status}\x1b[0m", rows);
    bytes.extend_from_slice(status.as_bytes());

    let cursor = format!("\x1b[{};{}H", rows, cols);
    bytes.extend_from_slice(cursor.as_bytes());

    bytes
}

fn push_terminal_line(bytes: &mut Vec<u8>, line: &str, cols: u32) {
    bytes.extend_from_slice(terminal_line_prefix(line, cols).as_bytes());
    bytes.extend_from_slice(b"\r\n");
}

fn terminal_line_prefix(line: &str, cols: u32) -> String {
    let max = cols.saturating_sub(1).max(1) as usize;
    line.chars().take(max).collect()
}

fn mouse_demo_bytes(cols: u32, rows: u32, status: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x1b[?1049h\x1b[?1000h\x1b[?1002h\x1b[?1006h\x1b[2J\x1b[H");
    bytes.extend_from_slice(b"Shellow mouse reporting demo - alternate screen\r\n\r\n");
    bytes.extend_from_slice(
        b"Tap or drag terminal grid rows. Shellow sends SGR mouse events to the app.\r\n",
    );
    bytes.extend_from_slice(b"Rows use xterm mouse modes 1000/1002 with SGR coordinates.\r\n\r\n");
    bytes.extend_from_slice(status.as_bytes());

    let status_row = rows.saturating_sub(1).max(1);
    let footer = format!(
        "\x1b[{};1H\x1b[7m-- MOUSE DEMO -- mouse=1000/1002 sgr=1006 -- {}x{} --\x1b[0m",
        status_row, cols, rows
    );
    bytes.extend_from_slice(footer.as_bytes());
    bytes.extend_from_slice(b"\x1b[6;1H");
    bytes
}

fn tui_demo_bytes(
    cols: u32,
    rows: u32,
    kind: LocalTuiDemo,
    status: &str,
    prefix_armed: bool,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x1b[?1049h\x1b[?1h\x1b[2J\x1b[H");
    if kind == LocalTuiDemo::Nano {
        bytes.extend_from_slice(b"\x1b[?2004h");
    }
    if kind == LocalTuiDemo::Top {
        bytes.extend_from_slice(b"\x1b[?25l");
    }

    match kind {
        LocalTuiDemo::Nano => {
            bytes.extend_from_slice(b"\x1b[1;37;44m");
            push_terminal_line(&mut bytes, "Shellow GNU nano demo", cols);
            bytes.extend_from_slice(b"\x1b[0m");
            push_terminal_line(&mut bytes, "", cols);
            push_terminal_line(&mut bytes, "File: ~/notes/shellow-terminal.md", cols);
            push_terminal_line(
                &mut bytes,
                "This alternate-screen editor accepts text,",
                cols,
            );
            push_terminal_line(
                &mut bytes,
                "arrows, paste wrappers, and control keys.",
                cols,
            );
            push_terminal_line(&mut bytes, "", cols);
            push_terminal_line(&mut bytes, "^O Write Out   ^K Cut Text   ^U Uncut", cols);
            push_terminal_line(&mut bytes, "^X Exit        arrows move cursor", cols);
            bytes.extend_from_slice(b"\x1b[4;1H");
        }
        LocalTuiDemo::Top => {
            bytes.extend_from_slice(b"\x1b[1m");
            push_terminal_line(&mut bytes, "Shellow top demo - alternate screen", cols);
            bytes.extend_from_slice(b"\x1b[0m");
            push_terminal_line(&mut bytes, "Tasks: 42 total, 1 running, 41 sleeping", cols);
            push_terminal_line(&mut bytes, "%Cpu(s): 7.0 us, 2.0 sy, 91.0 id", cols);
            push_terminal_line(
                &mut bytes,
                "MiB Mem : 8192 total, 2410 used, 3900 free",
                cols,
            );
            push_terminal_line(&mut bytes, "", cols);
            push_terminal_line(&mut bytes, "  PID USER      %CPU %MEM COMMAND", cols);
            push_terminal_line(&mut bytes, " 101 shellow    12.0  1.4 renderer", cols);
            push_terminal_line(&mut bytes, " 118 shellow     4.0  0.7 russh-pty", cols);
            push_terminal_line(&mut bytes, " 131 shellow     1.0  0.3 vt-parser", cols);
            push_terminal_line(&mut bytes, "", cols);
            push_terminal_line(
                &mut bytes,
                "q quits, Ctrl-C interrupts, arrows select",
                cols,
            );
        }
        LocalTuiDemo::Tmux => {
            push_terminal_line(&mut bytes, "Shellow tmux demo - alternate screen", cols);
            push_terminal_line(&mut bytes, "", cols);
            push_terminal_line(&mut bytes, "$ cargo test -p shellow-core", cols);
            push_terminal_line(&mut bytes, "running terminal interaction checks...", cols);
            push_terminal_line(&mut bytes, "", cols);
            push_terminal_line(&mut bytes, "Pane 0: zsh  |  Pane 1: logs", cols);
            push_terminal_line(&mut bytes, "", cols);
            push_terminal_line(
                &mut bytes,
                "Ctrl-B then c/n/p/%/\" exercises tmux prefix.",
                cols,
            );
            push_terminal_line(&mut bytes, "Ctrl-B then d detaches. Esc exits demo.", cols);
        }
    }

    let status_row = rows.saturating_sub(1).max(1);
    let status = if status.is_empty() {
        kind.default_status()
    } else {
        status
    };
    let prefix = if prefix_armed { " prefix" } else { "" };
    let footer = format!(
        "-- {} DEMO{} -- {} -- {}x{} --",
        kind.label(),
        prefix,
        status,
        cols,
        rows
    );
    let footer = terminal_line_prefix(&footer, cols);
    let footer = format!("\x1b[{};1H\x1b[7m{footer}\x1b[0m", status_row);
    bytes.extend_from_slice(footer.as_bytes());

    if kind != LocalTuiDemo::Top {
        let cursor = format!("\x1b[{};{}H", 4_u32.min(rows), 1);
        bytes.extend_from_slice(cursor.as_bytes());
    }

    bytes
}

fn ansi_demo_bytes(cols: u32, rows: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x1b[2J\x1b[H");
    bytes.extend_from_slice(b"Shellow ANSI style grid demo\r\n\r\n");
    bytes.extend_from_slice(b"\x1b[31mred\x1b[0m ");
    bytes.extend_from_slice(b"\x1b[32mgreen\x1b[0m ");
    bytes.extend_from_slice(b"\x1b[34mblue\x1b[0m ");
    bytes.extend_from_slice(b"\x1b[1mbold\x1b[0m ");
    bytes.extend_from_slice(b"\x1b[4munderline\x1b[0m\r\n");
    bytes.extend_from_slice(b"\x1b[38;5;208m256-color orange\x1b[0m ");
    bytes.extend_from_slice(b"\x1b[38;2;125;207;255mtruecolor sky\x1b[0m\r\n");
    bytes.extend_from_slice(b"\x1b[48;5;23mbackground cell\x1b[0m ");
    bytes.extend_from_slice(b"\x1b[7minverse video\x1b[0m");

    let status_row = rows.clamp(8, 120);
    let status = format!(
        "\x1b[{};1H\x1b[2mstyles from libghostty-vt cells | {}x{}\x1b[0m",
        status_row, cols, rows
    );
    bytes.extend_from_slice(status.as_bytes());
    bytes
}

fn wide_demo_bytes(cols: u32, rows: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x1b[2J\x1b[H");
    bytes.extend_from_slice("Shellow UTF-8 width demo\r\n\r\n".as_bytes());
    bytes.extend_from_slice("CJK double-width: 你好 世界 终端\r\n".as_bytes());
    bytes.extend_from_slice("Mixed prompt: deploy@主机:~/项目 ❯ ls\r\n".as_bytes());
    bytes.extend_from_slice("Box drawing: ┌─ Shellow ─┐ │ OK │\r\n".as_bytes());
    bytes.extend_from_slice("Combining/emoji: cafe\u{301} λ 🚀\r\n".as_bytes());

    let status_row = rows.clamp(8, 120);
    let status = format!(
        "\x1b[{};1H\x1b[2mwide cells from libghostty-vt | {}x{}\x1b[0m",
        status_row, cols, rows
    );
    bytes.extend_from_slice(status.as_bytes());
    bytes.extend_from_slice(b"\x1b[3;19H");
    bytes
}

fn scrollback_demo_bytes(cols: u32, rows: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x1b[2J\x1b[H");
    bytes.extend_from_slice(b"Shellow scrollback demo\r\n");

    let total_lines = rows.clamp(8, 120) + 12;
    for line in 1..=total_lines {
        let row = format!("scrollback demo line {:03}\r\n", line);
        bytes.extend_from_slice(row.as_bytes());
    }

    let footer = format!("scrollback footer | {}x{}", cols, rows);
    bytes.extend_from_slice(footer.as_bytes());
    bytes
}

fn cursor_demo_bytes(cols: u32, rows: u32, shape: TerminalCursorShape) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x1b[2J\x1b[H");
    bytes.extend_from_slice(b"Shellow cursor shape demo\r\n\r\n");
    bytes.extend_from_slice(b"DECSCUSR supported: block, underline, bar\r\n");
    bytes.extend_from_slice(b"Active cursor style: ");
    let shape_name = match shape {
        TerminalCursorShape::Block => "block",
        TerminalCursorShape::Underline => "underline",
        TerminalCursorShape::Bar => "bar",
    };
    bytes.extend_from_slice(shape_name.as_bytes());
    bytes.extend_from_slice(b"\r\n\r\n");
    bytes.extend_from_slice(b"The cursor marker below is driven by VT state.\r\n");

    let param = match shape {
        TerminalCursorShape::Block => 1,
        TerminalCursorShape::Underline => 3,
        TerminalCursorShape::Bar => 5,
    };
    let footer = format!(
        "\x1b[{};1H\x1b[2mDECSCUSR cursor shape from libghostty-vt | {}x{}\x1b[0m",
        rows.clamp(8, 120),
        cols,
        rows
    );
    bytes.extend_from_slice(footer.as_bytes());

    let cursor_row = rows.clamp(8, 120).saturating_sub(2).max(6);
    let cursor_line = format!("\x1b[{};1H\x1b[{} qCursor sample: ", cursor_row, param);
    bytes.extend_from_slice(cursor_line.as_bytes());
    bytes
}

fn title_demo_bytes() -> Vec<u8> {
    let title = "Shellow OSC Title";
    format!("\x1b]2;{title}\x07OSC 2 title set by terminal output\r\nCurrent title: {title}")
        .into_bytes()
}

fn bell_demo_bytes() -> Vec<u8> {
    b"\x07Shellow terminal bell demo\r\nBEL received from terminal output".to_vec()
}

fn osc52_demo_bytes() -> Vec<u8> {
    b"\x1b]52;c;Y29waWVkIGZyb20gcmVtb3RlIHZpYSBPU0MgNTI=\x07Shellow OSC 52 clipboard demo\r\nRemote app requested clipboard copy"
        .to_vec()
}

fn decode_csi_key<I>(chars: &mut std::iter::Peekable<I>) -> &'static str
where
    I: Iterator<Item = char>,
{
    let mut parameters = String::new();

    while let Some(character) = chars.next() {
        match character {
            'A' => return "Up",
            'B' => return "Down",
            'C' => return "Right",
            'D' => return "Left",
            'H' => return "Home",
            'F' => return "End",
            '~' => {
                return match parameters.as_str() {
                    "1" | "7" => "Home",
                    "3" => "Delete",
                    "4" | "8" => "End",
                    "5" => "PageUp",
                    "6" => "PageDown",
                    "11" => "F1",
                    "12" => "F2",
                    "13" => "F3",
                    "14" => "F4",
                    "15" => "F5",
                    "17" => "F6",
                    "18" => "F7",
                    "19" => "F8",
                    "20" => "F9",
                    "21" => "F10",
                    "23" => "F11",
                    "24" => "F12",
                    "200" => "BracketedPasteStart",
                    "201" => "BracketedPasteEnd",
                    _ => "Escape sequence",
                };
            }
            '0'..='9' | ';' => parameters.push(character),
            _ => return "Escape sequence",
        }
    }

    "Escape sequence"
}

fn decode_ss3_key<I>(chars: &mut std::iter::Peekable<I>) -> &'static str
where
    I: Iterator<Item = char>,
{
    match chars.next() {
        Some('A') => "Up",
        Some('B') => "Down",
        Some('C') => "Right",
        Some('D') => "Left",
        Some('H') => "Home",
        Some('F') => "End",
        Some('P') => "F1",
        Some('Q') => "F2",
        Some('R') => "F3",
        Some('S') => "F4",
        _ => "Escape sequence",
    }
}

fn is_application_cursor_key(key: &str) -> bool {
    matches!(key, "Up" | "Down" | "Right" | "Left")
}

fn key_name(character: char) -> String {
    match character {
        ' ' => "Space".to_string(),
        '\t' => "Tab".to_string(),
        _ => character.to_string(),
    }
}

fn control_name(character: char) -> char {
    let value = character as u32;
    if (1..=26).contains(&value) {
        char::from_u32(('A' as u32) + value - 1).unwrap_or('?')
    } else {
        '?'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_renderer_glyph_atlas_backend(
        backend: &str,
        target_backend: &str,
        real_font_ready: bool,
    ) {
        assert_eq!(target_backend, "font-shaping-glyph-atlas");
        if renderer::is_real_font_rasterizer_available() {
            assert_eq!(backend, "fontdue-system-font-rasterizer");
            assert!(real_font_ready);
        } else {
            assert_eq!(backend, "procedural-cell-rasterizer");
            assert!(!real_font_ready);
        }
    }

    fn assert_renderer_glyph_layout_backend(
        backend: &str,
        target_backend: &str,
        shaping_ready: bool,
    ) {
        assert_eq!(target_backend, "font-shaping-glyph-atlas");
        if renderer::is_text_shaping_available() {
            assert_eq!(backend, "rustybuzz-terminal-shaper");
            assert!(shaping_ready);
        } else {
            assert_eq!(backend, "terminal-cell-cluster-layout");
            assert!(!shaping_ready);
        }
    }

    #[test]
    fn demo_engine_returns_snapshot_json() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.send_command("pwd");

        assert_eq!(snapshot.state, ConnectionState::Connected);
        assert!(snapshot.rows.iter().any(|row| row.text == "/home/shellow"));
    }

    #[test]
    fn clear_terminal_removes_visible_history_without_dropping_session_state() {
        let mut engine = ShellowEngine::new();
        engine.resize_terminal(120, 36);
        engine.send_terminal_input("pwd");
        engine.send_terminal_input("\r");

        let snapshot = engine.clear_terminal();

        assert_eq!(snapshot.state, ConnectionState::Connected);
        assert_eq!(snapshot.terminal_cols, 120);
        assert_eq!(snapshot.terminal_rows, 36);
        assert_eq!(snapshot.rows.len(), 1);
        assert_eq!(snapshot.rows[0].style, TerminalRowStyle::Prompt);
        assert_eq!(snapshot.rows[0].text, "");
        assert!(snapshot.grid.is_none());
        assert!(!snapshot.rows.iter().any(|row| row.text == "/home/shellow"));
    }

    #[test]
    fn reset_terminal_exits_alternate_screen_and_clears_pending_input() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("vim");
        let vim_snapshot = engine.send_terminal_input("\r");
        assert_eq!(
            vim_snapshot
                .grid
                .expect("vim demo should enter grid")
                .active_screen,
            TerminalScreenKind::Alternate
        );

        engine.send_terminal_input("abc");
        let snapshot = engine.reset_terminal();

        assert_eq!(snapshot.state, ConnectionState::Connected);
        assert!(snapshot.grid.is_none());
        assert_eq!(snapshot.rows.len(), 1);
        assert_eq!(snapshot.rows[0].style, TerminalRowStyle::Prompt);
        assert_eq!(snapshot.rows[0].text, "");
        assert_eq!(snapshot.cursor_column, 0);
    }

    #[test]
    fn host_key_fingerprint_normalization_accepts_common_forms() {
        assert_eq!(
            ssh::normalize_sha256_fingerprint(" SHA256:abcDEF012+/= comment "),
            Some("SHA256:abcDEF012+/=".to_string())
        );
        assert_eq!(
            ssh::normalize_sha256_fingerprint("abcDEF012+/="),
            Some("SHA256:abcDEF012+/=".to_string())
        );
        assert!(ssh::normalize_sha256_fingerprint("   ").is_none());
        assert!(ssh::sha256_fingerprints_match(
            "SHA256:abcDEF012+/=",
            "abcDEF012+/= comment"
        ));
    }

    #[test]
    fn preview_uses_compact_terminal_output() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.connect_preview(HostProfile {
            name: "Pinned".to_string(),
            host: "10.0.0.18".to_string(),
            port: 22,
            username: "deploy".to_string(),
            authentication: AuthenticationKind::PrivateKey,
            trusted_host_key_sha256: Some("sha256:abcDEF012+/= comment".to_string()),
        });

        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| { row.text == "Preview terminal ready" })
        );
        assert!(snapshot.rows.iter().all(|row| {
            !row.text.contains("host-key=")
                && !row.text.contains("keepalive=")
                && !row.text.contains("auth=")
                && !row.text.contains("russh")
        }));
    }

    #[test]
    fn snapshot_reports_observed_host_key_only_when_available() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.snapshot();
        assert_eq!(snapshot.observed_host_key_sha256, None);

        let json = serde_json::to_string(&snapshot).expect("snapshot should encode");
        assert!(!json.contains("observed_host_key_sha256"));

        engine.observed_host_key_sha256 = Some("SHA256:observed-test-key".to_string());
        let snapshot = engine.snapshot();
        assert_eq!(
            snapshot.observed_host_key_sha256,
            Some("SHA256:observed-test-key".to_string())
        );

        let json = serde_json::to_string(&snapshot).expect("snapshot should encode");
        assert!(json.contains("\"observed_host_key_sha256\":\"SHA256:observed-test-key\""));
    }

    #[cfg(feature = "native-integrations")]
    #[test]
    fn private_key_shell_reports_parse_error_before_network() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.start_private_key_shell(
            HostProfile {
                name: "Bad Key".to_string(),
                host: "192.0.2.1".to_string(),
                port: 22,
                username: "deploy".to_string(),
                authentication: AuthenticationKind::PrivateKey,
                trusted_host_key_sha256: None,
            },
            "BEGIN\nnot-a-real-key\nPRIVATE KEY".to_string(),
            None,
        );

        assert_eq!(snapshot.state, ConnectionState::Disconnected);
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "Connection failed")
        );
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| { row.text.contains("private key parse failed") })
        );
        assert!(snapshot.rows.iter().all(|row| row.text != "Connected"));
    }

    #[cfg(feature = "native-integrations")]
    #[test]
    fn private_key_exec_reports_parse_error_before_network() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.connect_private_key_exec(
            HostProfile {
                name: "Bad Probe Key".to_string(),
                host: "192.0.2.1".to_string(),
                port: 22,
                username: "deploy".to_string(),
                authentication: AuthenticationKind::PrivateKey,
                trusted_host_key_sha256: None,
            },
            "BEGIN\nnot-a-real-key\nPRIVATE KEY".to_string(),
            None,
            "uname -s".to_string(),
        );

        assert_eq!(snapshot.state, ConnectionState::Disconnected);
        assert!(snapshot.rows.iter().any(|row| row.text == "Command failed"));
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text.contains("private key parse failed"))
        );
        assert!(
            snapshot
                .rows
                .iter()
                .all(|row| row.text != "Command completed")
        );
    }

    #[test]
    fn terminal_input_commits_command_on_enter() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("pwd");
        let snapshot = engine.send_terminal_input("\r");

        assert!(snapshot.rows.iter().any(|row| row.text == "/home/shellow"));
        assert_eq!(snapshot.cursor_column, 0);
    }

    #[test]
    fn terminal_input_recalls_local_command_history() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("pwd");
        engine.send_terminal_input("\r");
        engine.send_terminal_input("whoami");
        engine.send_terminal_input("\r");

        let snapshot = engine.send_terminal_input("\u{1b}[A");
        assert_eq!(snapshot.rows.last().unwrap().text, "whoami");
        assert_eq!(snapshot.cursor_column, 6);

        let snapshot = engine.send_terminal_input("\u{1b}[A");
        assert_eq!(snapshot.rows.last().unwrap().text, "pwd");
        assert_eq!(snapshot.cursor_column, 3);

        let snapshot = engine.send_terminal_input("\u{1b}[B");
        assert_eq!(snapshot.rows.last().unwrap().text, "whoami");

        let snapshot = engine.send_terminal_input("\u{1b}[B");
        assert_eq!(snapshot.rows.last().unwrap().text, "");
        assert_eq!(snapshot.cursor_column, 0);
    }

    #[test]
    fn terminal_input_restores_draft_after_history_navigation() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("pwd");
        engine.send_terminal_input("\r");
        engine.send_terminal_input("who");

        let snapshot = engine.send_terminal_input("\u{1b}OA");
        assert_eq!(snapshot.rows.last().unwrap().text, "pwd");

        let snapshot = engine.send_terminal_input("\u{1b}OB");
        assert_eq!(snapshot.rows.last().unwrap().text, "who");
        assert_eq!(snapshot.cursor_column, 3);

        let snapshot = engine.send_terminal_input("ami");
        assert_eq!(snapshot.rows.last().unwrap().text, "whoami");
    }

    #[test]
    fn terminal_input_reverse_searches_command_history() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("pwd");
        engine.send_terminal_input("\r");
        engine.send_terminal_input("shellow links");
        engine.send_terminal_input("\r");
        engine.send_terminal_input("shellow ssh");
        engine.send_terminal_input("\r");
        engine.send_terminal_input("whoami");
        engine.send_terminal_input("\r");

        let snapshot = engine.send_terminal_input("\u{12}");
        let row = snapshot.rows.last().unwrap();
        assert_eq!(row.prompt, "(reverse-i-search)`':");
        assert_eq!(row.text, "whoami");

        let snapshot = engine.send_terminal_input("sh");
        let row = snapshot.rows.last().unwrap();
        assert_eq!(row.prompt, "(reverse-i-search)`sh':");
        assert_eq!(row.text, "shellow ssh");
        assert_eq!(snapshot.cursor_column, "shellow ssh".len());

        let snapshot = engine.send_terminal_input("\u{12}");
        let row = snapshot.rows.last().unwrap();
        assert_eq!(row.prompt, "(reverse-i-search)`sh':");
        assert_eq!(row.text, "shellow links");

        let snapshot = engine.send_terminal_input("\r");
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.style == TerminalRowStyle::Command && row.text == "shellow links")
        );
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "Shellow link detection demo")
        );
    }

    #[test]
    fn terminal_input_reverse_search_failure_and_escape_restore_draft() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("pwd");
        engine.send_terminal_input("\r");
        engine.send_terminal_input("who");

        let snapshot = engine.send_terminal_input("\u{12}");
        assert_eq!(snapshot.rows.last().unwrap().text, "pwd");

        let snapshot = engine.send_terminal_input("zzz");
        let row = snapshot.rows.last().unwrap();
        assert_eq!(row.prompt, "(failed reverse-i-search)`zzz':");
        assert_eq!(row.text, "zzz");

        let snapshot = engine.send_terminal_input("\u{1b}");
        let row = snapshot.rows.last().unwrap();
        assert_eq!(row.prompt, "$");
        assert_eq!(row.text, "who");
        assert_eq!(snapshot.cursor_column, 3);
    }

    #[test]
    fn osc_title_updates_snapshot_title() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("shellow title");
        let snapshot = engine.send_terminal_input("\r");

        assert_eq!(snapshot.title, "Shellow OSC Title");
        assert_eq!(snapshot.bell_count, 0);
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "OSC 2 title set by terminal output")
        );
        assert!(
            snapshot
                .rows
                .iter()
                .all(|row| !row.text.contains("\u{1b}]2;"))
        );
    }

    #[test]
    fn bell_output_updates_snapshot_bell_count() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("shellow bell");
        let snapshot = engine.send_terminal_input("\r");

        assert_eq!(snapshot.bell_count, 1);
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "Shellow terminal bell demo")
        );
        assert!(snapshot.rows.iter().all(|row| !row.text.contains('\u{7}')));
    }

    #[test]
    fn remote_osc_title_updates_snapshot_title() {
        let mut engine = ShellowEngine::new();
        let rows = engine
            .terminal_rows_from_remote_output("\u{1b}]2;Remote Build Host\u{7}deploy finished");

        assert_eq!(engine.title, "Remote Build Host");
        assert_eq!(engine.bell_count, 0);
        assert!(rows.iter().any(|row| row.text == "deploy finished"));
    }

    #[test]
    fn remote_bell_updates_snapshot_bell_count() {
        let mut engine = ShellowEngine::new();
        let rows = engine.terminal_rows_from_remote_output("\u{7}remote alert");

        assert_eq!(engine.bell_count, 1);
        assert!(rows.iter().any(|row| row.text == "remote alert"));
    }

    #[test]
    fn osc52_clipboard_request_updates_snapshot() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("shellow osc52");
        let snapshot = engine.send_terminal_input("\r");

        assert_eq!(
            snapshot.pending_clipboard_text.as_deref(),
            Some("copied from remote via OSC 52")
        );
        assert_eq!(snapshot.clipboard_sequence, 1);
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "Shellow OSC 52 clipboard demo")
        );
        assert!(
            snapshot
                .rows
                .iter()
                .all(|row| !row.text.contains("\u{1b}]52;"))
        );
    }

    #[test]
    fn remote_osc52_clipboard_request_uses_st_terminator() {
        let mut engine = ShellowEngine::new();
        let rows = engine.terminal_rows_from_remote_output(
            "\u{1b}]52;c;cmVtb3RlIGNsaXBib2FyZCBwYXlsb2Fk\u{1b}\\remote copied",
        );

        assert_eq!(
            engine.pending_clipboard_text.as_deref(),
            Some("remote clipboard payload")
        );
        assert_eq!(engine.clipboard_sequence, 1);
        assert!(rows.iter().any(|row| row.text == "remote copied"));
    }

    #[test]
    fn osc52_clipboard_query_is_ignored() {
        let mut engine = ShellowEngine::new();
        let rows = engine.terminal_rows_from_remote_output("\u{1b}]52;c;?\u{7}query ignored");

        assert_eq!(engine.pending_clipboard_text, None);
        assert_eq!(engine.clipboard_sequence, 0);
        assert!(rows.iter().any(|row| row.text == "query ignored"));
    }

    #[test]
    fn link_demo_outputs_copyable_urls() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("shellow links");
        let snapshot = engine.send_terminal_input("\r");

        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "Shellow link detection demo")
        );
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text.contains("https://example.com/shellow/build/42"))
        );
        assert!(snapshot.rows.iter().any(|row| {
            row.text
                .contains("https://docs.example.com/shellow?view=terminal")
        }));
    }

    #[test]
    fn terminal_input_supports_backspace_and_control_keys() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("pwz");
        engine.send_terminal_input("\u{7f}d");
        let snapshot = engine.send_terminal_input("\u{3}");

        assert!(snapshot.rows.iter().any(|row| row.text == "pwd ^C"));
        assert_eq!(snapshot.rows.last().unwrap().text, "");
    }

    #[test]
    fn terminal_input_supports_readline_control_keys() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("pwd");
        let snapshot = engine.send_terminal_input("\u{1b}[D");
        assert_eq!(snapshot.rows.last().unwrap().text, "pwd");
        assert_eq!(snapshot.cursor_column, 2);

        let snapshot = engine.send_terminal_input("X");
        assert_eq!(snapshot.rows.last().unwrap().text, "pwXd");
        assert_eq!(snapshot.cursor_column, 3);

        let snapshot = engine.send_terminal_input("\u{7f}");
        assert_eq!(snapshot.rows.last().unwrap().text, "pwd");
        assert_eq!(snapshot.cursor_column, 2);

        let snapshot = engine.send_terminal_input("\u{1}");
        assert_eq!(snapshot.cursor_column, 0);

        let snapshot = engine.send_terminal_input("echo ");
        assert_eq!(snapshot.rows.last().unwrap().text, "echo pwd");
        assert_eq!(snapshot.cursor_column, 5);

        let snapshot = engine.send_terminal_input("\u{5}");
        assert_eq!(snapshot.cursor_column, "echo pwd".len());

        let snapshot = engine.send_terminal_input("\u{1b}[D\u{1b}[D\u{1b}[D\u{b}");
        assert_eq!(snapshot.rows.last().unwrap().text, "echo ");
        assert_eq!(snapshot.cursor_column, "echo ".len());

        let snapshot = engine.send_terminal_input("alpha beta\u{17}");
        assert_eq!(snapshot.rows.last().unwrap().text, "echo alpha ");
        assert_eq!(snapshot.cursor_column, "echo alpha ".len());

        let snapshot = engine.send_terminal_input("\u{15}");
        assert_eq!(snapshot.rows.last().unwrap().text, "");
        assert_eq!(snapshot.cursor_column, 0);

        engine.send_terminal_input("vim");
        engine.send_terminal_input("\r");
        let snapshot = engine.send_terminal_input("\u{15}");
        let grid = snapshot.grid.expect("vim demo should still be active");

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Ctrl-U sent to alternate-screen app"))
        );
    }

    #[test]
    fn terminal_input_supports_alt_meta_keys() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.send_terminal_input("\u{1b}b");

        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "Alt-b sent to terminal input stream")
        );
        assert_eq!(snapshot.rows.last().unwrap().text, "");

        engine.send_terminal_input("vim");
        engine.send_terminal_input("\r");
        let snapshot = engine.send_terminal_input("\u{1b}f");
        let grid = snapshot.grid.expect("vim demo should still be active");

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Alt-f sent to alternate-screen app"))
        );
    }

    #[test]
    fn terminal_input_decodes_csi_navigation_keys() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.send_terminal_input("\u{1b}[5~");

        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "PageUp sent to terminal input stream")
        );
        assert_eq!(snapshot.rows.last().unwrap().text, "");

        engine.send_terminal_input("vim");
        engine.send_terminal_input("\r");
        let snapshot = engine.send_terminal_input("\u{1b}[6~");
        let grid = snapshot.grid.expect("vim demo should still be active");

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("PageDown key handled inside alternate screen"))
        );
    }

    #[test]
    fn terminal_input_decodes_function_keys() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.send_terminal_input("\u{1b}[15~");

        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "F5 sent to terminal input stream")
        );
        assert_eq!(snapshot.rows.last().unwrap().text, "");

        let snapshot = engine.send_terminal_input("\u{1b}OP");

        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "F1 sent to terminal input stream")
        );

        engine.send_terminal_input("vim");
        engine.send_terminal_input("\r");
        let snapshot = engine.send_terminal_input("\u{1b}[15~");
        let grid = snapshot.grid.expect("vim demo should still be active");

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("F5 key handled inside alternate screen"))
        );

        let snapshot = engine.send_terminal_input("\u{1b}OP");
        let grid = snapshot.grid.expect("vim demo should still be active");

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("F1 key handled inside alternate screen"))
        );
        assert!(
            grid.lines
                .iter()
                .all(|line| !line.contains("Application cursor F1"))
        );
    }

    #[test]
    fn mouse_demo_reports_sgr_mouse_presses() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("shellow mouse");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("mouse demo should expose a grid");

        assert_eq!(grid.active_screen, TerminalScreenKind::Alternate);
        assert!(grid.mouse_reporting);
        assert!(grid.mouse_drag_reporting);
        assert!(grid.sgr_mouse);
        assert!(
            grid.lines
                .iter()
                .any(|line| { line.contains("Shellow mouse reporting demo - alternate screen") })
        );

        let snapshot = engine.send_terminal_input("\u{1b}[<0;7;5M");
        let grid = snapshot
            .grid
            .expect("mouse demo should still expose a grid");

        assert!(
            grid.lines
                .iter()
                .any(|line| { line.contains("Mouse press at col 7 row 5 (SGR 1006 left)") })
        );

        let snapshot = engine.send_terminal_input("\u{1b}[<32;9;6M");
        let grid = snapshot
            .grid
            .expect("mouse demo should still expose a grid after drag");

        assert!(
            grid.lines
                .iter()
                .any(|line| { line.contains("Mouse drag at col 9 row 6 (SGR 1006 left drag)") })
        );

        let snapshot = engine.send_terminal_input("\u{1b}[<0;9;6m");
        let grid = snapshot
            .grid
            .expect("mouse demo should still expose a grid after release");

        assert!(
            grid.lines
                .iter()
                .any(|line| { line.contains("Mouse release at col 9 row 6 (SGR 1006 left)") })
        );
    }

    #[test]
    fn vt_render_strips_basic_escape_sequences() {
        let rendered = ghostty_adapter::render_vt_plain_text("\x1b[31mhello\x1b[0m\r\n");
        assert!(rendered.contains("hello"));

        #[cfg(feature = "official-libghostty-vt-rs")]
        assert!(!rendered.contains("\x1b"));
    }

    #[test]
    fn libghostty_vt_ready_path_exposes_official_backend_contract() {
        if !ghostty_adapter::is_libghostty_vt_ready() {
            return;
        }

        let bytes = b"\x1b]2;libghostty-vt Contract\x07hello libghostty\x07";
        assert_eq!(
            ghostty_adapter::terminal_title_from_vt_bytes(bytes).as_deref(),
            Some("libghostty-vt Contract")
        );
        assert_eq!(ghostty_adapter::terminal_bell_count_from_vt_bytes(bytes), 1);
        assert!(
            ghostty_adapter::render_vt_plain_text("\x1b[31mhello libghostty\x1b[0m")
                .contains("hello libghostty")
        );

        let grid = ghostty_adapter::terminal_grid_from_vt_bytes(bytes, 40, 8);
        assert_eq!(grid.cols, 40);
        assert_eq!(grid.rows, 8);
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("hello libghostty"))
        );
    }

    #[cfg(feature = "native-integrations")]
    #[test]
    fn live_terminal_state_applies_incremental_vt_output() {
        let mut state = ghostty_adapter::LiveTerminalState::new(40, 8)
            .expect("libghostty-vt live terminal should initialize");

        state.write(b"\x1b]2;Live Shell\x1b\\one\r\n");
        state.write(b"two\x07");

        assert_eq!(state.title().as_deref(), Some("Live Shell"));
        assert_eq!(state.take_bell_count(), 1);
        assert_eq!(state.take_bell_count(), 0);

        let grid = state
            .snapshot()
            .expect("live terminal should expose a grid snapshot");
        assert_eq!(grid.cols, 40);
        assert_eq!(grid.rows, 8);
        assert!(grid.lines.iter().any(|line| line.contains("one")));
        assert!(grid.lines.iter().any(|line| line.contains("two")));

        assert!(state.resize(50, 10));
        let grid = state
            .snapshot()
            .expect("resized live terminal should expose a grid snapshot");
        assert_eq!(grid.cols, 50);
        assert_eq!(grid.rows, 10);
    }

    #[test]
    fn resize_updates_snapshot_terminal_size() {
        let mut engine = ShellowEngine::new();
        let snapshot = engine.resize_terminal(120, 36);

        assert_eq!(snapshot.terminal_cols, 120);
        assert_eq!(snapshot.terminal_rows, 36);
        assert_eq!(snapshot.rows.last().unwrap().text, "");

        let snapshot = engine.send_command("shellow size");
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text == "terminal size=120x36")
        );
    }

    #[test]
    fn vim_demo_uses_alternate_screen_grid_and_exits_with_escape() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("vim");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("vim demo should expose a grid");

        #[cfg(feature = "official-libghostty-vt-rs")]
        assert_eq!(grid.active_screen, TerminalScreenKind::Alternate);
        #[cfg(feature = "official-libghostty-vt-rs")]
        assert!(grid.bracketed_paste);
        #[cfg(feature = "official-libghostty-vt-rs")]
        assert!(grid.application_cursor_keys);

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Shellow demo editor"))
        );

        let snapshot = engine.send_terminal_input("abc");
        let grid = snapshot.grid.expect("typing should keep editor active");
        assert!(grid.lines.iter().any(|line| line.contains("abc")));
        assert!(grid.cursor_visible);

        let snapshot = engine.send_terminal_input("\u{1b}");
        assert!(snapshot.grid.is_none());
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text.contains("closed with Esc"))
        );
    }

    #[test]
    fn grid_snapshots_report_dirty_rows_incrementally() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("vim");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("vim demo should expose a first frame");
        assert_eq!(grid.dirty_rows.len(), grid.lines.len());

        let snapshot = engine.send_terminal_input("a");
        let grid = snapshot.grid.expect("typing should keep editor active");
        assert!(!grid.dirty_rows.is_empty());
        assert!(grid.dirty_rows.len() < grid.lines.len());
        assert!(
            grid.dirty_rows
                .iter()
                .any(|row| grid.lines[*row].contains('a')),
            "dirty rows should include the edited text row"
        );

        let snapshot = engine.snapshot();
        let grid = snapshot
            .grid
            .expect("no-op snapshot should keep editor active");
        assert!(grid.dirty_rows.is_empty());
    }

    #[test]
    fn vim_demo_handles_bracketed_paste_wrappers() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("vim");
        engine.send_terminal_input("\r");
        let snapshot = engine.send_terminal_input("\u{1b}[200~pasted\ntext\u{1b}[201~");
        let grid = snapshot.grid.expect("vim demo should still be active");

        assert!(grid.lines.iter().any(|line| line.contains("pasted")));
        assert!(grid.lines.iter().any(|line| line.contains("text")));
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Bracketed paste wrapper received"))
        );
    }

    #[test]
    fn vim_demo_handles_application_cursor_keys() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("vim");
        engine.send_terminal_input("\r");
        let snapshot = engine.send_terminal_input("\u{1b}OA");
        let grid = snapshot.grid.expect("vim demo should still be active");

        assert!(grid.lines.iter().any(|line| {
            line.contains("Application cursor Up key handled inside alternate screen")
        }));
    }

    #[test]
    fn pager_demo_scrolls_and_exits_like_less() {
        let mut engine = ShellowEngine::new();
        engine.resize_terminal(80, 8);
        engine.send_terminal_input("less");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("pager demo should expose a grid");

        #[cfg(feature = "official-libghostty-vt-rs")]
        assert_eq!(grid.active_screen, TerminalScreenKind::Alternate);

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Shellow pager demo"))
        );
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("001 Shellow pager demo document"))
        );

        let snapshot = engine.send_terminal_input("\u{1b}[6~");
        let grid = snapshot.grid.expect("PageDown should keep pager active");
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("PageDown: showing lines"))
        );
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("005 PageDown moves by a viewport"))
                || grid
                    .lines
                    .iter()
                    .any(|line| line.contains("006 PageUp moves back by a viewport"))
        );

        let snapshot = engine.send_terminal_input("\u{1b}[F");
        let grid = snapshot.grid.expect("End should keep pager active");
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("030 Final page marker"))
        );

        let snapshot = engine.send_terminal_input("\u{1b}[H");
        let grid = snapshot.grid.expect("Home should keep pager active");
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("001 Shellow pager demo document"))
        );

        let snapshot = engine.send_terminal_input("q");
        assert!(snapshot.grid.is_none());
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text.contains("pager closed with q"))
        );
    }

    #[test]
    fn local_tui_demos_cover_nano_top_and_tmux() {
        let mut engine = ShellowEngine::new();
        engine.resize_terminal(80, 10);

        engine.send_terminal_input("nano");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("nano demo should expose a grid");
        #[cfg(feature = "official-libghostty-vt-rs")]
        assert_eq!(grid.active_screen, TerminalScreenKind::Alternate);
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Shellow GNU nano demo"))
        );

        let snapshot = engine.send_terminal_input("\u{f}");
        let grid = snapshot.grid.expect("Ctrl-O should keep nano active");
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("nano Ctrl-O writeout requested"))
        );

        let snapshot = engine.send_terminal_input("\u{18}");
        assert!(snapshot.grid.is_none());
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text.contains("nano closed with Ctrl-X"))
        );

        engine.send_terminal_input("top");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("top demo should expose a grid");
        #[cfg(not(feature = "official-libghostty-vt-rs"))]
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Shellow top demo"))
        );
        #[cfg(feature = "official-libghostty-vt-rs")]
        assert!(!grid.cursor_visible);

        let snapshot = engine.send_terminal_input("\u{1b}[B");
        let grid = snapshot.grid.expect("Down should keep top active");
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("top selected next process"))
        );

        let snapshot = engine.send_terminal_input("q");
        assert!(snapshot.grid.is_none());
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text.contains("top closed with q"))
        );

        engine.send_terminal_input("tmux");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("tmux demo should expose a grid");
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Shellow tmux demo"))
        );

        let snapshot = engine.send_terminal_input("\u{2}");
        let grid = snapshot.grid.expect("Ctrl-B should keep tmux active");
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("tmux prefix Ctrl-B armed"))
        );

        let snapshot = engine.send_terminal_input("n");
        let grid = snapshot
            .grid
            .expect("tmux prefix n should keep tmux active");
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("tmux prefix + n selected next window"))
        );

        engine.send_terminal_input("\u{2}");
        let snapshot = engine.send_terminal_input("d");
        assert!(snapshot.grid.is_none());
        assert!(
            snapshot
                .rows
                .iter()
                .any(|row| row.text.contains("tmux detached with Ctrl-B d"))
        );
    }

    #[test]
    fn ansi_demo_exposes_styled_grid_runs() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("shellow ansi");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("ansi demo should expose a grid");

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Shellow ANSI style grid demo"))
        );

        #[cfg(feature = "official-libghostty-vt-rs")]
        {
            let runs = grid
                .styled_lines
                .iter()
                .flat_map(|line| line.runs.iter())
                .collect::<Vec<_>>();

            assert!(
                runs.iter()
                    .any(|run| run.text.contains("red") && run.style.fg.is_some())
            );
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("bold") && run.style.bold)
            );
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("underline") && run.style.underline)
            );
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("inverse") && run.style.inverse)
            );
        }
    }

    #[test]
    fn vt_grid_preserves_foreground_background_and_text_attributes() {
        let grid = ghostty_adapter::terminal_grid_from_vt_bytes(
            b"\x1b[31mred8\x1b[0m \
              \x1b[38;5;45mfg256\x1b[0m \
              \x1b[38;2;12;34;56mfgtrue\x1b[0m\r\n\
              \x1b[48;5;196mbg256\x1b[0m \
              \x1b[48;2;90;45;210mbgtrue\x1b[0m \
              \x1b[7minverse\x1b[0m\r\n\
              \x1b[1mbold\x1b[0m \
              \x1b[4munder\x1b[0m \
              \x1b[9mstrike\x1b[0m",
            96,
            8,
        );

        #[cfg(feature = "official-libghostty-vt-rs")]
        {
            let runs = grid
                .styled_lines
                .iter()
                .flat_map(|line| line.runs.iter())
                .collect::<Vec<_>>();

            assert!(
                runs.iter()
                    .any(|run| run.text.contains("red8") && run.style.fg.is_some())
            );
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("fg256") && run.style.fg.is_some())
            );
            assert!(runs.iter().any(|run| {
                run.text.contains("fgtrue")
                    && run.style.fg
                        == Some(TerminalGridColor {
                            r: 12,
                            g: 34,
                            b: 56,
                        })
            }));
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("bg256") && run.style.bg.is_some())
            );
            assert!(runs.iter().any(|run| {
                run.text.contains("bgtrue")
                    && run.style.bg
                        == Some(TerminalGridColor {
                            r: 90,
                            g: 45,
                            b: 210,
                        })
            }));
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("inverse") && run.style.inverse)
            );
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("bold") && run.style.bold)
            );
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("under") && run.style.underline)
            );
            assert!(
                runs.iter()
                    .any(|run| run.text.contains("strike") && run.style.strikethrough)
            );
        }

        #[cfg(not(feature = "official-libghostty-vt-rs"))]
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("red8") && line.contains("fg256"))
        );
    }

    #[test]
    fn cursor_demo_exposes_cursor_shape() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("shellow cursor");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("cursor demo should expose a grid");

        assert_eq!(grid.cursor_shape, TerminalCursorShape::Bar);
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Shellow cursor shape demo"))
        );
        assert!(
            grid.lines
                .get(grid.cursor_row as usize)
                .is_some_and(|line| line.contains("Cursor sample:"))
        );
    }

    #[test]
    fn wide_demo_preserves_utf8_grid_text() {
        let mut engine = ShellowEngine::new();
        engine.send_terminal_input("shellow wide");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("wide demo should expose a grid");

        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("Shellow UTF-8 width demo"))
        );
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("CJK double-width: 你好 世界 终端"))
        );

        #[cfg(feature = "official-libghostty-vt-rs")]
        {
            assert!(
                grid.styled_lines
                    .iter()
                    .flat_map(|line| line.runs.iter())
                    .any(|run| run.text.contains("主机"))
            );
            assert_eq!(grid.cursor_row, 2);
            assert_eq!(grid.cursor_column, "CJK double-width: ".len() as u32);
            assert!(
                grid.lines
                    .get(grid.cursor_row as usize)
                    .is_some_and(|line| line.contains("你好 世界 终端"))
            );
        }
    }

    #[test]
    fn scrollback_demo_exposes_history_and_visible_rows() {
        let mut engine = ShellowEngine::new();
        engine.resize_terminal(40, 8);
        engine.send_terminal_input("shellow scrollback");
        let snapshot = engine.send_terminal_input("\r");
        let grid = snapshot.grid.expect("scrollback demo should expose a grid");

        assert!(grid.scrollback_len > 0);
        assert!(grid.lines.len() > grid.rows as usize);
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("scrollback demo line 001"))
        );
        assert!(
            grid.lines
                .iter()
                .any(|line| line.contains("scrollback footer"))
        );
        assert!(grid.cursor_row as usize >= grid.scrollback_len);
    }

    #[test]
    fn integration_report_names_final_terminal_and_renderer_targets() {
        let engine = ShellowEngine::new();
        let report = engine.snapshot().integration;

        assert_eq!(report.terminal_target_backend, "libghostty-vt");
        assert_eq!(report.libghostty_vt_abi_contract, "libghostty-vt-rs-0.2.0");
        let expected_link_status = if ghostty_adapter::is_libghostty_vt_link_configured() {
            "rust-crate-linked"
        } else {
            "not-selected"
        };
        assert_eq!(ghostty_adapter::link_status(), expected_link_status);
        assert_eq!(
            report.libghostty_vt_abi_status,
            ghostty_adapter::libghostty_vt_abi_status()
        );
        if ghostty_adapter::is_libghostty_vt_link_configured() {
            assert!(report.libghostty_vt_abi_status.contains("linked"));
        } else {
            assert!(report.libghostty_vt_abi_status.contains("not-linked"));
        }
        assert_eq!(
            report.libghostty_vt_link_configured,
            ghostty_adapter::is_libghostty_vt_link_configured()
        );
        assert_eq!(
            report.libghostty_vt_ready,
            ghostty_adapter::is_libghostty_vt_ready()
        );
        if report.libghostty_vt_ready {
            assert_eq!(
                report.terminal_backend_migration,
                "official-libghostty-vt-rs"
            );
        } else {
            assert!(report.terminal_backend_migration.contains("libghostty-vt"));
        }
        assert_eq!(report.renderer_target_backend, "wgpu-native-surface");
        assert!(!report.renderer_surface_ready);
    }

    #[test]
    fn renderer_surface_attachment_tracks_native_host_without_claiming_presentation() {
        let mut engine = ShellowEngine::new();

        let attachment = engine.attach_core_animation_layer_renderer_surface(0xfeed_beef, 640, 320);

        assert_eq!(
            attachment.kind,
            renderer::RendererSurfaceKind::CoreAnimationLayer
        );
        assert!(attachment.raw_handle_nonzero);
        assert_eq!(attachment.width_px, 640);
        assert_eq!(attachment.height_px, 320);
        assert!(attachment.attach_api_ready);
        assert!(!attachment.wgpu_surface_configured);
        assert!(!attachment.presentation_ready);

        let info = engine.renderer_info();
        assert!(info.native_surface_attached);
        assert_eq!(
            info.native_surface_kind,
            Some(renderer::RendererSurfaceKind::CoreAnimationLayer)
        );
        assert_eq!(info.native_surface_width_px, 640);
        assert_eq!(info.native_surface_height_px, 320);
        assert!(!info.native_surface_ready);
        assert!(!info.native_surface_presentation_ready);

        let frame = engine.render_frame(640, 320);
        assert!(frame.native_surface_attached);
        assert_eq!(
            frame.native_surface_kind,
            Some(renderer::RendererSurfaceKind::CoreAnimationLayer)
        );
        assert!(!frame.native_surface_ready);
        assert!(!frame.native_surface_presentation_ready);

        let detached = engine.detach_renderer_surface();
        assert_eq!(detached.status, "detached");
        assert!(!engine.renderer_info().native_surface_attached);
    }

    #[test]
    fn renderer_frame_uses_current_terminal_grid() {
        let mut engine = ShellowEngine::new();
        engine.resize_terminal(40, 8);
        engine.send_terminal_input("shellow ansi");
        engine.send_terminal_input("\r");

        let frame = engine.render_frame(800, 320);

        assert_eq!(frame.target_backend, "wgpu-native-surface");
        assert_eq!(frame.cols, 40);
        assert_eq!(frame.rows, 8);
        assert_eq!(frame.cell_width_px, 20);
        assert_eq!(frame.cell_height_px, 40);
        assert!(matches!(
            frame.active_screen,
            TerminalScreenKind::Primary | TerminalScreenKind::Alternate
        ));
        assert!(frame.visible_line_count > 0);
        assert!(!frame.dirty_rows.is_empty());
        assert!(frame.glyph_atlas_glyph_count >= 96);
        assert!(frame.glyph_atlas_revision > 0);
        assert_renderer_glyph_atlas_backend(
            &frame.glyph_atlas_backend,
            &frame.glyph_atlas_target_backend,
            frame.glyph_atlas_real_font_ready,
        );
        assert_renderer_glyph_layout_backend(
            &frame.glyph_layout_backend,
            &frame.glyph_layout_target_backend,
            frame.glyph_layout_shaping_ready,
        );
        assert!(frame.glyph_layout_cluster_count > 0);
        if renderer::is_text_shaping_available() {
            assert!(frame.glyph_layout_shaped_glyph_count > 0);
        } else {
            assert_eq!(frame.glyph_layout_shaped_glyph_count, 0);
        }
        assert_eq!(frame.dirty_row_upload_count, frame.dirty_rows.len());
        assert!(frame.dirty_row_upload_bytes > 0);
        #[cfg(feature = "official-libghostty-vt-rs")]
        assert!(frame.styled_run_count > 0);
        assert!(!frame.frame_signature.is_empty());

        #[cfg(feature = "native-integrations")]
        {
            if frame.offscreen_gpu_pass {
                assert!(frame.glyph_atlas_ready);
                assert!(frame.glyph_atlas_uploaded);
                assert_eq!(frame.gpu_dirty_row_upload_count, frame.dirty_rows.len());
                assert_eq!(
                    frame.gpu_dirty_row_upload_bytes,
                    frame.dirty_row_upload_bytes
                );
            } else {
                assert!(frame.notes.iter().any(|note| note.contains("wgpu")));
                assert!(!frame.glyph_atlas_uploaded);
                assert_eq!(frame.gpu_dirty_row_upload_count, 0);
                assert_eq!(frame.gpu_dirty_row_upload_bytes, 0);
            }
        }
    }

    #[test]
    fn renderer_runtime_keeps_identity_and_frame_lifecycle() {
        let mut engine = ShellowEngine::new();
        engine.resize_terminal(40, 8);
        engine.send_terminal_input("shellow ansi");
        engine.send_terminal_input("\r");

        let before = engine.renderer_info();
        assert_eq!(before.frame_count, 0);

        let first = engine.render_frame(800, 320);
        let second = engine.render_frame(800, 320);
        let after = engine.renderer_info();

        assert_eq!(first.renderer_id, second.renderer_id);
        assert_eq!(second.frame_index, first.frame_index + 1);
        assert_eq!(after.renderer_id, first.renderer_id);
        assert_eq!(after.frame_count, second.frame_index);
        assert_eq!(
            after.last_frame_signature.as_deref(),
            Some(second.frame_signature.as_str())
        );
        assert!(first.content_changed);
        assert!(!second.content_changed);
        assert!(second.dirty_rows.is_empty());
        assert_eq!(second.dirty_row_upload_count, 0);
        assert_eq!(second.dirty_row_upload_bytes, 0);
        assert_eq!(
            after.glyph_atlas_glyph_count,
            second.glyph_atlas_glyph_count
        );
        assert_eq!(after.glyph_atlas_revision, second.glyph_atlas_revision);
        assert_renderer_glyph_atlas_backend(
            &after.glyph_atlas_backend,
            &after.glyph_atlas_target_backend,
            after.glyph_atlas_real_font_ready,
        );
        assert_renderer_glyph_layout_backend(
            &after.glyph_layout_backend,
            &after.glyph_layout_target_backend,
            after.glyph_layout_shaping_ready,
        );
        if after.glyph_atlas_real_font_ready {
            assert!(
                after
                    .notes
                    .iter()
                    .any(|note| { note.contains("real font glyph rasterization is active") })
            );
        } else {
            assert!(after.notes.iter().any(|note| {
                note.contains("real font glyph rasterization is not available yet")
            }));
        }

        #[cfg(feature = "native-integrations")]
        {
            if first.offscreen_gpu_pass {
                assert!(first.glyph_atlas_ready);
                assert!(first.glyph_atlas_uploaded);
                assert!(first.gpu_dirty_row_upload_count > 0);
                assert!(second.persistent_device_ready);
                assert!(second.reused_gpu_device);
                assert!(second.glyph_atlas_ready);
                assert!(!second.glyph_atlas_uploaded);
                assert_eq!(second.gpu_dirty_row_upload_count, 0);
                assert!(after.gpu_glyph_atlas_upload_count > 0);
                assert!(after.gpu_dirty_row_upload_count > 0);
            } else {
                assert!(first.notes.iter().any(|note| note.contains("wgpu")));
                assert!(!first.glyph_atlas_uploaded);
                assert_eq!(first.gpu_dirty_row_upload_count, 0);
                assert_eq!(after.gpu_glyph_atlas_upload_count, 0);
                assert_eq!(after.gpu_dirty_row_upload_count, 0);
            }
        }
    }

    #[test]
    fn renderer_glyph_layout_clusters_wide_and_combining_cells() {
        let mut engine = ShellowEngine::new();
        engine.resize_terminal(40, 8);
        engine.send_terminal_input("shellow wide");
        engine.send_terminal_input("\r");

        let wide_frame = engine.render_frame(800, 320);

        assert_renderer_glyph_layout_backend(
            &wide_frame.glyph_layout_backend,
            &wide_frame.glyph_layout_target_backend,
            wide_frame.glyph_layout_shaping_ready,
        );
        assert!(wide_frame.glyph_layout_cluster_count > 0);
        if renderer::is_text_shaping_available() {
            assert!(wide_frame.glyph_layout_shaped_glyph_count > 0);
        } else {
            assert_eq!(wide_frame.glyph_layout_shaped_glyph_count, 0);
        }
        assert!(wide_frame.glyph_layout_wide_cluster_count > 0);
        assert!(wide_frame.text_cell_count > wide_frame.glyph_layout_cluster_count);

        engine.clear_terminal();
        engine.send_terminal_input("e\u{301}");

        let combining_frame = engine.render_frame(800, 320);

        assert_renderer_glyph_layout_backend(
            &combining_frame.glyph_layout_backend,
            &combining_frame.glyph_layout_target_backend,
            combining_frame.glyph_layout_shaping_ready,
        );
        assert!(combining_frame.glyph_layout_cluster_count > 0);
        assert!(combining_frame.glyph_layout_zero_width_cluster_count > 0);
        assert!(combining_frame.text_cell_count >= combining_frame.glyph_layout_cluster_count);
    }

    #[test]
    fn renderer_overlay_state_is_reported_in_surface_frame() {
        let mut engine = ShellowEngine::new();
        engine.resize_terminal(40, 8);
        engine.send_terminal_input("shellow ansi");
        engine.send_terminal_input("\r");

        let update = engine.set_renderer_overlay(renderer::RendererOverlayState {
            ranges: vec![
                renderer::RendererOverlayRange {
                    kind: renderer::RendererOverlayKind::Selection,
                    row: 0,
                    start_col: 0,
                    end_col: 8,
                },
                renderer::RendererOverlayRange {
                    kind: renderer::RendererOverlayKind::Search,
                    row: 1,
                    start_col: 2,
                    end_col: 16,
                },
                renderer::RendererOverlayRange {
                    kind: renderer::RendererOverlayKind::ActiveSearch,
                    row: 2,
                    start_col: 5,
                    end_col: 5,
                },
            ],
        });

        assert!(update.accepted);
        assert_eq!(update.range_count, 2);
        assert!(
            update
                .notes
                .iter()
                .any(|note| note.contains("empty overlay ranges were discarded"))
        );
        assert_eq!(engine.renderer_info().renderer_overlay_range_count, 2);

        let frame = engine.render_frame(800, 320);

        assert_eq!(frame.native_surface_terminal_overlay_range_count, 2);
    }

    #[test]
    fn renderer_frame_uploads_history_rows_when_no_vt_grid_exists() {
        let engine = ShellowEngine::new();
        assert!(engine.snapshot().grid.is_none());

        let frame = engine.render_frame(960, 480);

        assert!(frame.visible_line_count > 0);
        assert_eq!(frame.dirty_row_upload_count, frame.dirty_rows.len());
        assert!(frame.dirty_row_upload_count > 0);
        assert!(frame.dirty_row_upload_bytes > 0);
        assert!(frame.glyph_atlas_glyph_count >= 96);

        #[cfg(feature = "native-integrations")]
        {
            if frame.offscreen_gpu_pass {
                assert!(frame.glyph_atlas_ready);
                assert!(frame.glyph_atlas_uploaded);
                assert_eq!(
                    frame.gpu_dirty_row_upload_count,
                    frame.dirty_row_upload_count
                );
                assert_eq!(
                    frame.gpu_dirty_row_upload_bytes,
                    frame.dirty_row_upload_bytes
                );
            } else {
                assert!(frame.notes.iter().any(|note| note.contains("wgpu")));
                assert!(!frame.glyph_atlas_uploaded);
                assert_eq!(frame.gpu_dirty_row_upload_count, 0);
                assert_eq!(frame.gpu_dirty_row_upload_bytes, 0);
            }
        }
    }
}
