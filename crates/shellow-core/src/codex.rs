use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{HostProfile, ssh};

const CODEX_APP_SERVER_COMMAND_BODY: &str = r#"SHELLOW_CODEX_CWD="$(pwd -P 2>/dev/null || pwd)"; printf 'SHELLOW_CODEX_CWD=%s\n' "$SHELLOW_CODEX_CWD" >&2; PATH="$PATH:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.bun/bin:$HOME/.npm-global/bin:/home/linuxbrew/.linuxbrew/bin"; export PATH; if command -v codex >/dev/null 2>&1; then exec codex app-server --stdio; fi; echo "codex executable not found in non-interactive SSH PATH. Install Codex CLI or expose it via PATH." >&2; exit 127"#;
const REMOTE_CWD_PREFIX: &str = "SHELLOW_CODEX_CWD=";
const APP_SERVER_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const COMMAND_OUTPUT_PREVIEW_MAX_LINES: usize = 10;
const COMMAND_OUTPUT_PREVIEW_MAX_CHARS: usize = 2_400;
const STATUS_MESSAGE_MAX_CHARS: usize = 2_000;
const HISTORY_ITEMS_VIEW: &str = "summary";
const COMPACT_TRANSCRIPT_MAX_CHARS: usize = 8_000;

fn codex_debug(args: std::fmt::Arguments<'_>) {
    println!("[Shellow Codex] {args}");
}

fn json_byte_len(value: &Value) -> usize {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexSnapshot {
    pub title: String,
    pub endpoint: String,
    pub cwd: Option<String>,
    pub status: CodexStatus,
    pub observed_host_key_sha256: Option<String>,
    pub thread_id: Option<String>,
    pub turn_active: bool,
    pub messages: Vec<CodexMessage>,
    pub pending_approvals: Vec<CodexApproval>,
    pub directory: CodexDirectoryState,
    pub threads: CodexThreadListState,
    pub projects: CodexProjectState,
    pub thread_detail: CodexThreadDetailState,
    pub active_turn: Option<CodexActiveTurn>,
    pub operation: CodexOperationState,
    pub settings: CodexSettingsState,
    pub last_error: Option<String>,
}

impl CodexSnapshot {
    pub fn disconnected() -> Self {
        Self {
            title: "Codex".to_string(),
            endpoint: "not connected".to_string(),
            cwd: None,
            status: CodexStatus::Disconnected,
            observed_host_key_sha256: None,
            thread_id: None,
            turn_active: false,
            messages: vec![CodexMessage::status(
                "status-0",
                "Connect to a host to start Codex.",
            )],
            pending_approvals: Vec::new(),
            directory: CodexDirectoryState::default(),
            threads: CodexThreadListState::default(),
            projects: CodexProjectState::default(),
            thread_detail: CodexThreadDetailState::default(),
            active_turn: None,
            operation: CodexOperationState::idle(),
            settings: CodexSettingsState::default(),
            last_error: None,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            title: "Codex".to_string(),
            endpoint: "bridge.error".to_string(),
            cwd: None,
            status: CodexStatus::Failed,
            observed_host_key_sha256: None,
            thread_id: None,
            turn_active: false,
            messages: vec![
                CodexMessage::status("status-0", "Codex bridge failed"),
                CodexMessage::status("status-1", &message),
            ],
            pending_approvals: Vec::new(),
            directory: CodexDirectoryState::default(),
            threads: CodexThreadListState::default(),
            projects: CodexProjectState::default(),
            thread_detail: CodexThreadDetailState::default(),
            active_turn: None,
            operation: CodexOperationState::failed(message.clone()),
            settings: CodexSettingsState::default(),
            last_error: Some(message),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CodexProjectState {
    pub current: Option<String>,
    pub remote_home: Option<String>,
    pub recent: Vec<String>,
    pub favorites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CodexDirectoryState {
    pub path: Option<String>,
    pub parent: Option<String>,
    pub entries: Vec<CodexDirectoryEntry>,
    pub is_loading: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexDirectoryEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub is_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CodexThreadListState {
    pub cwd: Option<String>,
    pub search_term: Option<String>,
    pub archived: bool,
    pub threads: Vec<CodexThreadSummary>,
    pub next_cursor: Option<String>,
    pub backwards_cursor: Option<String>,
    pub is_loading: bool,
    pub is_loading_more: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexThreadSummary {
    pub id: String,
    pub name: Option<String>,
    pub preview: String,
    pub cwd: String,
    pub status: String,
    pub updated_at: u64,
    pub created_at: u64,
    pub source: String,
    pub model_provider: String,
    pub forked_from_id: Option<String>,
    pub parent_thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CodexThreadDetailState {
    pub thread: Option<CodexThreadSummary>,
    pub turns_next_cursor: Option<String>,
    pub turns_backwards_cursor: Option<String>,
    pub is_loading: bool,
    pub is_loading_more: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexActiveTurn {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexOperationState {
    pub is_running: bool,
    pub label: Option<String>,
    pub last_success: Option<String>,
    pub last_error: Option<String>,
}

impl CodexOperationState {
    fn idle() -> Self {
        Self {
            is_running: false,
            label: None,
            last_success: None,
            last_error: None,
        }
    }

    fn running(label: impl Into<String>) -> Self {
        Self {
            is_running: true,
            label: Some(label.into()),
            last_success: None,
            last_error: None,
        }
    }

    fn succeeded(message: impl Into<String>) -> Self {
        Self {
            is_running: false,
            label: None,
            last_success: Some(message.into()),
            last_error: None,
        }
    }

    fn failed(message: impl Into<String>) -> Self {
        Self {
            is_running: false,
            label: None,
            last_success: None,
            last_error: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexModelOption {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexSettingsState {
    pub model: Option<String>,
    pub approval_policy: Option<String>,
    pub sandbox: Option<String>,
    pub available_models: Vec<CodexModelOption>,
    pub is_loading_models: bool,
    pub models_error: Option<String>,
}

impl Default for CodexSettingsState {
    fn default() -> Self {
        Self {
            model: None,
            approval_policy: None,
            sandbox: None,
            available_models: default_model_options(),
            is_loading_models: false,
            models_error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexMessage {
    pub id: String,
    pub role: CodexMessageRole,
    pub kind: CodexMessageKind,
    pub visibility: CodexMessageVisibility,
    pub title: Option<String>,
    pub detail: Option<String>,
    pub text: String,
    pub transcript: Option<String>,
    pub format: CodexMessageFormat,
    pub blocks: Vec<CodexMarkdownBlock>,
    pub is_streaming: bool,
    pub truncated: bool,
}

impl CodexMessage {
    fn user(id: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut message = Self {
            id: id.into(),
            role: CodexMessageRole::User,
            kind: CodexMessageKind::UserMessage,
            visibility: CodexMessageVisibility::Primary,
            title: Some("You".to_string()),
            detail: None,
            text,
            transcript: None,
            format: CodexMessageFormat::Markdown,
            blocks: Vec::new(),
            is_streaming: false,
            truncated: false,
        };
        message.refresh_blocks();
        message
    }

    fn assistant(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: CodexMessageRole::Assistant,
            kind: CodexMessageKind::FinalAnswer,
            visibility: CodexMessageVisibility::Primary,
            title: Some("Codex".to_string()),
            detail: None,
            text: String::new(),
            transcript: None,
            format: CodexMessageFormat::Markdown,
            blocks: Vec::new(),
            is_streaming: true,
            truncated: false,
        }
    }

    fn status(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: CodexMessageRole::Status,
            kind: CodexMessageKind::Status,
            visibility: CodexMessageVisibility::Compact,
            title: Some("Status".to_string()),
            detail: None,
            text: text.into(),
            transcript: None,
            format: CodexMessageFormat::Status,
            blocks: Vec::new(),
            is_streaming: false,
            truncated: false,
        }
    }

    fn compact_event(
        id: impl Into<String>,
        kind: CodexMessageKind,
        title: impl Into<String>,
        detail: Option<String>,
        transcript: Option<String>,
    ) -> Self {
        let title = title.into();
        let text = detail.clone().unwrap_or_else(|| title.clone());
        Self {
            id: id.into(),
            role: CodexMessageRole::Tool,
            kind,
            visibility: CodexMessageVisibility::Compact,
            title: Some(title.clone()),
            detail,
            text,
            transcript,
            format: CodexMessageFormat::Plain,
            blocks: Vec::new(),
            is_streaming: false,
            truncated: false,
        }
    }

    fn image_event(
        id: impl Into<String>,
        title: impl Into<String>,
        url: impl Into<String>,
        alt: Option<String>,
    ) -> Self {
        let title = title.into();
        let url = url.into();
        let alt_text = alt.unwrap_or_else(|| title.clone());
        let mut message = Self {
            id: id.into(),
            role: CodexMessageRole::Tool,
            kind: CodexMessageKind::ToolResult,
            visibility: CodexMessageVisibility::Primary,
            title: Some(title),
            detail: Some(url.clone()),
            text: markdown_image_text(&url, &alt_text),
            transcript: None,
            format: CodexMessageFormat::Markdown,
            blocks: Vec::new(),
            is_streaming: false,
            truncated: false,
        };
        message.refresh_blocks();
        message
    }

    fn command_output(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: CodexMessageRole::CommandOutput,
            kind: CodexMessageKind::CommandOutput,
            visibility: CodexMessageVisibility::Compact,
            title: Some("Command output".to_string()),
            detail: None,
            text: String::new(),
            transcript: None,
            format: CodexMessageFormat::Code,
            blocks: Vec::new(),
            is_streaming: false,
            truncated: false,
        }
    }

    fn reasoning_summary(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            role: CodexMessageRole::Status,
            kind: CodexMessageKind::ReasoningSummary,
            visibility: CodexMessageVisibility::Compact,
            title: Some("Thinking".to_string()),
            detail: None,
            text: "Thinking...".to_string(),
            transcript: Some(String::new()),
            format: CodexMessageFormat::Status,
            blocks: Vec::new(),
            is_streaming: true,
            truncated: false,
        }
    }

    fn refresh_blocks(&mut self) {
        self.blocks = match self.format {
            CodexMessageFormat::Markdown => parse_markdown_blocks(&self.id, &self.text),
            CodexMessageFormat::Code => {
                if self.text.is_empty() {
                    Vec::new()
                } else {
                    vec![CodexMarkdownBlock::code(
                        format!("{}-code-0", self.id),
                        None,
                        self.text.clone(),
                        false,
                    )]
                }
            }
            CodexMessageFormat::Plain | CodexMessageFormat::Status => Vec::new(),
        };
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexMessageKind {
    UserMessage,
    FinalAnswer,
    Commentary,
    ReasoningSummary,
    Status,
    ToolCall,
    ToolResult,
    Command,
    CommandOutput,
    FileChange,
    Plan,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexMessageVisibility {
    Primary,
    Compact,
    TranscriptOnly,
    Hidden,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexMessageFormat {
    Plain,
    Markdown,
    Code,
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexMarkdownBlock {
    pub id: String,
    pub kind: CodexMarkdownBlockKind,
    pub text: String,
    pub image_url: Option<String>,
    pub image_alt: Option<String>,
    pub level: Option<u8>,
    pub language: Option<String>,
    pub ordered: bool,
    pub items: Vec<CodexMarkdownListItem>,
    pub table_headers: Vec<CodexMarkdownTableCell>,
    pub table_rows: Vec<Vec<CodexMarkdownTableCell>>,
    pub runs: Vec<CodexMarkdownInlineRun>,
    pub incomplete: bool,
}

impl CodexMarkdownBlock {
    fn text(
        id: impl Into<String>,
        kind: CodexMarkdownBlockKind,
        text: String,
        runs: Vec<CodexMarkdownInlineRun>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            text,
            image_url: None,
            image_alt: None,
            level: None,
            language: None,
            ordered: false,
            items: Vec::new(),
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            runs,
            incomplete: false,
        }
    }

    fn heading(
        id: impl Into<String>,
        level: u8,
        text: String,
        runs: Vec<CodexMarkdownInlineRun>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: CodexMarkdownBlockKind::Heading,
            text,
            image_url: None,
            image_alt: None,
            level: Some(level),
            language: None,
            ordered: false,
            items: Vec::new(),
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            runs,
            incomplete: false,
        }
    }

    fn code(
        id: impl Into<String>,
        language: Option<String>,
        text: String,
        incomplete: bool,
    ) -> Self {
        Self {
            id: id.into(),
            kind: CodexMarkdownBlockKind::CodeBlock,
            text,
            image_url: None,
            image_alt: None,
            level: None,
            language,
            ordered: false,
            items: Vec::new(),
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            runs: Vec::new(),
            incomplete,
        }
    }

    fn list(id: impl Into<String>, ordered: bool, items: Vec<CodexMarkdownListItem>) -> Self {
        let text = items
            .iter()
            .map(|item| item.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        Self {
            id: id.into(),
            kind: CodexMarkdownBlockKind::List,
            text,
            image_url: None,
            image_alt: None,
            level: None,
            language: None,
            ordered,
            items,
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            runs: Vec::new(),
            incomplete: false,
        }
    }

    fn table(
        id: impl Into<String>,
        table_headers: Vec<CodexMarkdownTableCell>,
        table_rows: Vec<Vec<CodexMarkdownTableCell>>,
    ) -> Self {
        let text = table_text(&table_headers, &table_rows);
        Self {
            id: id.into(),
            kind: CodexMarkdownBlockKind::Table,
            text,
            image_url: None,
            image_alt: None,
            level: None,
            language: None,
            ordered: false,
            items: Vec::new(),
            table_headers,
            table_rows,
            runs: Vec::new(),
            incomplete: false,
        }
    }

    fn rule(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: CodexMarkdownBlockKind::HorizontalRule,
            text: String::new(),
            image_url: None,
            image_alt: None,
            level: None,
            language: None,
            ordered: false,
            items: Vec::new(),
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            runs: Vec::new(),
            incomplete: false,
        }
    }

    fn image(id: impl Into<String>, url: String, alt: Option<String>) -> Self {
        let text = alt.clone().unwrap_or_default();
        Self {
            id: id.into(),
            kind: CodexMarkdownBlockKind::Image,
            text,
            image_url: Some(url),
            image_alt: alt,
            level: None,
            language: None,
            ordered: false,
            items: Vec::new(),
            table_headers: Vec::new(),
            table_rows: Vec::new(),
            runs: Vec::new(),
            incomplete: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexMarkdownBlockKind {
    Paragraph,
    Heading,
    List,
    BlockQuote,
    CodeBlock,
    Table,
    HorizontalRule,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexMarkdownListItem {
    pub text: String,
    pub runs: Vec<CodexMarkdownInlineRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexMarkdownTableCell {
    pub text: String,
    pub runs: Vec<CodexMarkdownInlineRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexMarkdownInlineRun {
    pub text: String,
    pub style: CodexMarkdownInlineStyle,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexMarkdownInlineStyle {
    Text,
    Bold,
    Italic,
    BoldItalic,
    Code,
    Link,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexMessageRole {
    User,
    Assistant,
    Status,
    Tool,
    CommandOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexApproval {
    pub request_id: String,
    pub kind: CodexApprovalKind,
    pub title: String,
    pub detail: String,
    pub command: Option<String>,
    pub cwd: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexApprovalKind {
    Command,
    FileChange,
    UserInput,
    Permissions,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClientRequestKind {
    Initialize,
    ModelList,
    ThreadStart,
    ThreadResume,
    DirectoryList,
    ThreadList,
    ThreadListMore,
    ThreadRead,
    ThreadTurnsMore,
    ThreadArchive,
    ThreadUnarchive,
    ThreadDelete,
    ThreadRename,
    ThreadFork,
    TurnStart,
    TurnSteer,
    TurnInterrupt,
}

#[derive(Debug, Clone)]
struct PendingServerRequest {
    id: Value,
    approval: CodexApproval,
}

#[derive(Debug)]
pub struct CodexSession {
    title: String,
    endpoint: String,
    cwd: Option<String>,
    remote_cwd: Option<String>,
    status: CodexStatus,
    initialized: bool,
    observed_host_key_sha256: Option<String>,
    thread_id: Option<String>,
    turn_active: bool,
    messages: Vec<CodexMessage>,
    pending_approvals: Vec<PendingServerRequest>,
    directory: CodexDirectoryState,
    threads: CodexThreadListState,
    projects: CodexProjectState,
    thread_detail: CodexThreadDetailState,
    active_turn: Option<CodexActiveTurn>,
    operation: CodexOperationState,
    settings: CodexSettingsState,
    last_error: Option<String>,
    line_buffer: String,
    next_request_id: u64,
    next_local_message_id: u64,
    local_revision: u64,
    request_kinds: HashMap<u64, ClientRequestKind>,
    completed_requests: HashMap<u64, Result<(), String>>,
    operation_thread_id: Option<String>,
    assistant_message_indices: HashMap<String, usize>,
    command_output_indices: HashMap<String, usize>,
    event_message_indices: HashMap<String, usize>,
    reasoning_message_indices: HashMap<String, usize>,
    #[cfg(feature = "native-integrations")]
    transport: ssh::ExecStdioHandle,
}

impl CodexSession {
    #[cfg(feature = "native-integrations")]
    pub fn start_password(
        profile: HostProfile,
        password: String,
        cwd: Option<String>,
    ) -> Result<Self, String> {
        Self::start(profile, ssh::RusshAuthMethod::Password(password), cwd)
    }

    #[cfg(feature = "native-integrations")]
    pub fn start_private_key(
        profile: HostProfile,
        private_key_pem: String,
        passphrase: Option<String>,
        cwd: Option<String>,
    ) -> Result<Self, String> {
        ssh::validate_private_key_auth(&private_key_pem, passphrase.as_deref())?;
        Self::start(
            profile,
            ssh::RusshAuthMethod::PrivateKey {
                private_key_pem,
                passphrase,
            },
            cwd,
        )
    }

    #[cfg(feature = "native-integrations")]
    fn start(
        profile: HostProfile,
        auth: ssh::RusshAuthMethod,
        cwd: Option<String>,
    ) -> Result<Self, String> {
        let title = profile.name.clone();
        let endpoint = profile.endpoint();
        let initial_cwd = normalize_cwd(cwd);
        let app_server_command = codex_app_server_command(initial_cwd.as_deref());
        let transport = ssh::ExecStdioHandle::spawn(
            ssh::RusshConnectOptions {
                host: profile.host,
                port: profile.port,
                username: profile.username,
                auth,
                expected_host_key_sha256: profile.trusted_host_key_sha256,
                keepalive_interval_secs: Some(ssh::DEFAULT_LIVE_KEEPALIVE_INTERVAL_SECS),
                keepalive_max: ssh::DEFAULT_KEEPALIVE_MAX,
                cols: 80,
                rows: 24,
                inactivity_timeout_secs: 3_600,
            },
            app_server_command,
        )?;

        let mut session = Self {
            title,
            endpoint,
            cwd: initial_cwd.clone(),
            remote_cwd: None,
            status: CodexStatus::Connecting,
            initialized: false,
            observed_host_key_sha256: None,
            thread_id: None,
            turn_active: false,
            messages: vec![
                CodexMessage::status("status-0", "Starting codex app-server over SSH."),
                CodexMessage::status("status-1", "Waiting for JSON-RPC handshake."),
            ],
            pending_approvals: Vec::new(),
            directory: CodexDirectoryState {
                path: initial_cwd.clone(),
                parent: None,
                entries: Vec::new(),
                is_loading: false,
                error: None,
            },
            threads: CodexThreadListState::default(),
            projects: CodexProjectState {
                current: initial_cwd.clone(),
                remote_home: None,
                recent: initial_cwd.iter().cloned().collect(),
                favorites: Vec::new(),
            },
            thread_detail: CodexThreadDetailState::default(),
            active_turn: None,
            operation: CodexOperationState::idle(),
            settings: CodexSettingsState::default(),
            last_error: None,
            line_buffer: String::new(),
            next_request_id: 1,
            next_local_message_id: 2,
            local_revision: 1,
            request_kinds: HashMap::new(),
            completed_requests: HashMap::new(),
            operation_thread_id: None,
            assistant_message_indices: HashMap::new(),
            command_output_indices: HashMap::new(),
            event_message_indices: HashMap::new(),
            reasoning_message_indices: HashMap::new(),
            transport,
        };

        session.bootstrap()?;
        Ok(session)
    }

    pub fn snapshot(&self) -> CodexSnapshot {
        CodexSnapshot {
            title: self.title.clone(),
            endpoint: self.endpoint.clone(),
            cwd: self.cwd.clone(),
            status: self.status,
            observed_host_key_sha256: self.observed_host_key_sha256.clone(),
            thread_id: self.thread_id.clone(),
            turn_active: self.turn_active,
            messages: self.messages.clone(),
            pending_approvals: self
                .pending_approvals
                .iter()
                .map(|pending| pending.approval.clone())
                .collect(),
            directory: self.directory.clone(),
            threads: self.threads.clone(),
            projects: self.projects.clone(),
            thread_detail: self.thread_detail.clone(),
            active_turn: self.active_turn.clone(),
            operation: self.operation.clone(),
            settings: self.settings.clone(),
            last_error: self.last_error.clone(),
        }
    }

    pub fn event_revision(&self) -> u64 {
        #[cfg(feature = "native-integrations")]
        {
            self.local_revision
                .saturating_add(self.transport.event_revision())
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            self.local_revision
        }
    }

    pub fn disconnect(&mut self) {
        #[cfg(feature = "native-integrations")]
        self.transport.disconnect();
        self.status = CodexStatus::Disconnected;
        self.turn_active = false;
        self.push_status("Codex connection closed.");
    }

    pub fn poll(&mut self) -> CodexSnapshot {
        #[cfg(feature = "native-integrations")]
        {
            let poll = self.transport.poll();
            self.apply_transport_status(poll.status);
            self.consume_output(&poll.output);
        }

        self.snapshot()
    }

    pub fn send_user_message(&mut self, text: &str) -> Result<CodexSnapshot, String> {
        self.poll();
        let text = text.trim();
        if text.is_empty() {
            return Ok(self.snapshot());
        }

        let Some(thread_id) = self.thread_id.clone() else {
            self.push_status("Codex is still starting; wait for the thread to become ready.");
            return Ok(self.snapshot());
        };

        let user_message_id = self.next_message_id("user");
        self.messages
            .push(CodexMessage::user(user_message_id, text.to_string()));
        self.bump_revision();

        let mut params = serde_json::Map::new();
        params.insert("threadId".to_string(), json!(thread_id));
        params.insert("input".to_string(), text_input_value(text));
        self.apply_turn_settings(&mut params);

        if self.turn_active {
            if let Some(active_turn) = &self.active_turn {
                params.insert("expectedTurnId".to_string(), json!(active_turn.id));
                self.send_request(
                    "turn/steer",
                    Value::Object(params),
                    ClientRequestKind::TurnSteer,
                )?;
                self.operation = CodexOperationState::running("Steering active turn");
            } else {
                self.push_status(
                    "Codex is working, but this server did not report an active turn id yet.",
                );
            }
        } else {
            self.send_request(
                "turn/start",
                Value::Object(params),
                ClientRequestKind::TurnStart,
            )?;
            self.turn_active = true;
            self.operation = CodexOperationState::running("Starting turn");
        }
        Ok(self.snapshot())
    }

    pub fn update_settings(
        &mut self,
        model: Option<&str>,
        approval_policy: Option<&str>,
        sandbox: Option<&str>,
    ) -> Result<CodexSnapshot, String> {
        self.settings = CodexSettingsState {
            model: normalize_setting(model),
            approval_policy: normalize_approval_policy(approval_policy),
            sandbox: normalize_sandbox(sandbox),
            available_models: if self.settings.available_models.is_empty() {
                default_model_options()
            } else {
                self.settings.available_models.clone()
            },
            is_loading_models: self.settings.is_loading_models,
            models_error: self.settings.models_error.clone(),
        };
        self.operation = CodexOperationState::succeeded("Codex settings updated.");
        self.bump_revision();
        Ok(self.snapshot())
    }

    pub fn browse_directory(&mut self, path: &str) -> Result<CodexSnapshot, String> {
        self.poll();
        let path = normalize_path_input(path)
            .or_else(|| self.cwd.clone())
            .or_else(|| self.remote_cwd.clone())
            .unwrap_or_else(|| "/".to_string());

        self.directory.path = Some(path.clone());
        self.directory.parent = parent_path(&path);
        self.directory.entries.clear();
        self.directory.error = None;
        self.directory.is_loading = true;
        self.bump_revision();

        #[cfg(feature = "native-integrations")]
        {
            let id = self.send_request(
                "fs/readDirectory",
                json!({ "path": path }),
                ClientRequestKind::DirectoryList,
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.directory.is_loading = false;
                self.directory.error = Some(error);
                self.bump_revision();
            }
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            self.directory.is_loading = false;
            self.directory.error =
                Some("russh native integration is not compiled into this build".to_string());
        }

        Ok(self.snapshot())
    }

    pub fn list_threads(
        &mut self,
        cwd: Option<&str>,
        search_term: Option<&str>,
    ) -> Result<CodexSnapshot, String> {
        self.list_threads_page(cwd, search_term, None, false, false)
    }

    pub fn list_threads_page(
        &mut self,
        cwd: Option<&str>,
        search_term: Option<&str>,
        cursor: Option<&str>,
        archived: bool,
        append: bool,
    ) -> Result<CodexSnapshot, String> {
        self.poll();
        let cwd = normalize_path_input_opt(cwd);
        let search_term = search_term
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let cursor = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        if !append {
            self.threads.threads.clear();
            self.threads.next_cursor = None;
            self.threads.backwards_cursor = None;
        }
        self.threads.cwd = cwd.clone();
        self.threads.search_term = search_term.clone();
        self.threads.archived = archived;
        self.threads.error = None;
        self.threads.is_loading = !append;
        self.threads.is_loading_more = append;
        self.bump_revision();

        #[cfg(feature = "native-integrations")]
        {
            let mut params = serde_json::Map::new();
            params.insert("limit".to_string(), json!(20));
            params.insert("sortKey".to_string(), json!("updated_at"));
            params.insert("sortDirection".to_string(), json!("desc"));
            params.insert("archived".to_string(), json!(archived));
            if let Some(cursor) = cursor {
                params.insert("cursor".to_string(), json!(cursor));
            }
            if let Some(cwd) = cwd {
                params.insert("cwd".to_string(), json!(cwd));
            }
            if let Some(search_term) = search_term {
                params.insert("searchTerm".to_string(), json!(search_term));
            }

            let id = self.send_request(
                "thread/list",
                Value::Object(params),
                if append {
                    ClientRequestKind::ThreadListMore
                } else {
                    ClientRequestKind::ThreadList
                },
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.threads.is_loading = false;
                self.threads.is_loading_more = false;
                self.threads.error = Some(error);
                self.bump_revision();
            }
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            self.threads.is_loading = false;
            self.threads.error =
                Some("russh native integration is not compiled into this build".to_string());
        }

        Ok(self.snapshot())
    }

    pub fn start_thread(&mut self, cwd: Option<&str>) -> Result<CodexSnapshot, String> {
        self.poll();
        let cwd = normalize_path_input_opt(cwd)
            .or_else(|| self.cwd.clone())
            .or_else(|| self.remote_cwd.clone());
        self.cwd = cwd.clone();
        self.thread_id = None;
        self.turn_active = false;
        self.pending_approvals.clear();
        self.clear_message_indices();
        self.thread_detail = CodexThreadDetailState::default();
        self.active_turn = None;
        self.messages.clear();
        self.push_status(match cwd.as_deref() {
            Some(cwd) => format!("Starting new Codex thread in {cwd}."),
            None => "Starting new Codex thread.".to_string(),
        });

        #[cfg(feature = "native-integrations")]
        {
            let mut params = serde_json::Map::new();
            if let Some(cwd) = cwd {
                params.insert("cwd".to_string(), json!(cwd));
            }
            self.apply_thread_settings(&mut params);
            let id = self.send_request(
                "thread/start",
                Value::Object(params),
                ClientRequestKind::ThreadStart,
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.push_status(error);
            }
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            self.last_error =
                Some("russh native integration is not compiled into this build".to_string());
        }

        Ok(self.snapshot())
    }

    pub fn resume_thread(&mut self, thread_id: &str) -> Result<CodexSnapshot, String> {
        self.poll();
        let thread_id = thread_id.trim();
        if thread_id.is_empty() {
            self.push_status("Choose a Codex thread to resume.");
            return Ok(self.snapshot());
        }

        self.thread_id = None;
        self.turn_active = false;
        self.pending_approvals.clear();
        self.clear_message_indices();
        self.thread_detail = CodexThreadDetailState::default();
        self.active_turn = None;
        self.messages.clear();
        self.operation = CodexOperationState::running("Opening thread");
        self.push_status("Resuming Codex thread.");

        #[cfg(feature = "native-integrations")]
        {
            let mut params = serde_json::Map::new();
            params.insert("threadId".to_string(), json!(thread_id));
            params.insert(
                "initialTurnsPage".to_string(),
                json!({
                    "limit": 30,
                    "sortDirection": "asc",
                    "itemsView": HISTORY_ITEMS_VIEW
                }),
            );
            let id = self.send_request(
                "thread/resume",
                Value::Object(params),
                ClientRequestKind::ThreadResume,
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.operation = CodexOperationState::failed(error.clone());
                self.push_status(error);
            }
        }

        #[cfg(not(feature = "native-integrations"))]
        {
            self.last_error =
                Some("russh native integration is not compiled into this build".to_string());
        }

        Ok(self.snapshot())
    }

    pub fn read_thread(&mut self, thread_id: &str) -> Result<CodexSnapshot, String> {
        self.poll();
        let thread_id = thread_id.trim();
        if thread_id.is_empty() {
            self.thread_detail.error = Some("Choose a Codex thread to inspect.".to_string());
            self.bump_revision();
            return Ok(self.snapshot());
        }

        self.thread_detail.is_loading = true;
        self.thread_detail.is_loading_more = false;
        self.thread_detail.error = None;
        self.operation = CodexOperationState::running("Loading thread");
        self.bump_revision();

        #[cfg(feature = "native-integrations")]
        {
            let id = self.send_request(
                "thread/read",
                json!({ "threadId": thread_id, "includeTurns": true }),
                ClientRequestKind::ThreadRead,
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.thread_detail.is_loading = false;
                self.thread_detail.error = Some(error.clone());
                self.operation = CodexOperationState::failed(error);
                self.bump_revision();
            }
        }

        Ok(self.snapshot())
    }

    pub fn load_more_thread_turns(
        &mut self,
        thread_id: &str,
        cursor: Option<&str>,
    ) -> Result<CodexSnapshot, String> {
        self.poll();
        let thread_id = thread_id.trim();
        if thread_id.is_empty() {
            return Ok(self.snapshot());
        }
        let cursor = cursor
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        self.thread_detail.is_loading_more = true;
        self.thread_detail.error = None;
        self.bump_revision();

        #[cfg(feature = "native-integrations")]
        {
            let mut params = serde_json::Map::new();
            params.insert("threadId".to_string(), json!(thread_id));
            params.insert("limit".to_string(), json!(20));
            params.insert("sortDirection".to_string(), json!("desc"));
            params.insert("itemsView".to_string(), json!(HISTORY_ITEMS_VIEW));
            if let Some(cursor) = cursor {
                params.insert("cursor".to_string(), json!(cursor));
            }

            let id = self.send_request(
                "thread/turns/list",
                Value::Object(params),
                ClientRequestKind::ThreadTurnsMore,
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.thread_detail.is_loading_more = false;
                self.thread_detail.error = Some(error);
                self.bump_revision();
            }
        }

        Ok(self.snapshot())
    }

    pub fn rename_thread(&mut self, thread_id: &str, name: &str) -> Result<CodexSnapshot, String> {
        self.run_thread_operation(
            "thread/name/set",
            json!({ "threadId": thread_id.trim(), "name": name.trim() }),
            ClientRequestKind::ThreadRename,
            "Renaming thread",
        )
    }

    pub fn archive_thread(&mut self, thread_id: &str) -> Result<CodexSnapshot, String> {
        self.run_thread_operation(
            "thread/archive",
            json!({ "threadId": thread_id.trim() }),
            ClientRequestKind::ThreadArchive,
            "Archiving thread",
        )
    }

    pub fn unarchive_thread(&mut self, thread_id: &str) -> Result<CodexSnapshot, String> {
        self.run_thread_operation(
            "thread/unarchive",
            json!({ "threadId": thread_id.trim() }),
            ClientRequestKind::ThreadUnarchive,
            "Restoring thread",
        )
    }

    pub fn delete_thread(&mut self, thread_id: &str) -> Result<CodexSnapshot, String> {
        self.run_thread_operation(
            "thread/delete",
            json!({ "threadId": thread_id.trim() }),
            ClientRequestKind::ThreadDelete,
            "Deleting thread",
        )
    }

    pub fn fork_thread(
        &mut self,
        thread_id: &str,
        cwd: Option<&str>,
    ) -> Result<CodexSnapshot, String> {
        self.poll();
        let thread_id = thread_id.trim();
        if thread_id.is_empty() {
            self.operation = CodexOperationState::failed("Choose a Codex thread to fork.");
            self.bump_revision();
            return Ok(self.snapshot());
        }

        let mut params = serde_json::Map::new();
        params.insert("threadId".to_string(), json!(thread_id));
        params.insert("excludeTurns".to_string(), json!(false));
        if let Some(cwd) = normalize_path_input_opt(cwd).or_else(|| self.cwd.clone()) {
            params.insert("cwd".to_string(), json!(cwd));
        }
        self.apply_thread_settings(&mut params);

        self.operation = CodexOperationState::running("Forking thread");
        self.bump_revision();

        #[cfg(feature = "native-integrations")]
        {
            let id = self.send_request(
                "thread/fork",
                Value::Object(params),
                ClientRequestKind::ThreadFork,
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.operation = CodexOperationState::failed(error.clone());
                self.push_status(error);
            }
        }

        Ok(self.snapshot())
    }

    pub fn interrupt_turn(&mut self) -> Result<CodexSnapshot, String> {
        self.poll();
        let Some(thread_id) = self.thread_id.clone() else {
            self.push_status("No Codex thread is active.");
            return Ok(self.snapshot());
        };
        let Some(active_turn) = self.active_turn.clone() else {
            self.push_status("No active Codex turn to interrupt.");
            return Ok(self.snapshot());
        };

        self.operation = CodexOperationState::running("Interrupting turn");
        self.bump_revision();

        #[cfg(feature = "native-integrations")]
        {
            let id = self.send_request(
                "turn/interrupt",
                json!({ "threadId": thread_id, "turnId": active_turn.id }),
                ClientRequestKind::TurnInterrupt,
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.operation = CodexOperationState::failed(error.clone());
                self.push_status(error);
            }
        }

        Ok(self.snapshot())
    }

    pub fn answer_approval(
        &mut self,
        request_id: &str,
        decision: &str,
    ) -> Result<CodexSnapshot, String> {
        self.poll();
        let Some(index) = self
            .pending_approvals
            .iter()
            .position(|pending| pending.approval.request_id == request_id)
        else {
            self.push_status(format!(
                "Approval request {request_id} is no longer pending."
            ));
            return Ok(self.snapshot());
        };

        let pending = self.pending_approvals.remove(index);
        let decision = normalize_approval_decision(decision);
        let result = match pending.approval.kind {
            CodexApprovalKind::Command => json!({ "decision": decision }),
            CodexApprovalKind::FileChange => json!({ "decision": decision }),
            CodexApprovalKind::UserInput => json!({ "answers": {} }),
            CodexApprovalKind::Permissions => json!({ "permissions": null, "scope": "once" }),
            CodexApprovalKind::Tool => json!({ "contentItems": [], "success": false }),
        };
        self.send_response(pending.id, result)?;
        self.push_status(format!(
            "Answered {} approval: {}",
            pending.approval.title, decision
        ));
        Ok(self.snapshot())
    }

    #[cfg(feature = "native-integrations")]
    fn bootstrap(&mut self) -> Result<(), String> {
        self.send_request(
            "initialize",
            json!({
                "clientInfo": {
                    "name": "shellow_native",
                    "title": "Shellow",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "experimentalApi": true
                }
            }),
            ClientRequestKind::Initialize,
        )?;
        self.send_notification("initialized", json!({}))?;
        Ok(())
    }

    #[cfg(feature = "native-integrations")]
    fn request_model_list(&mut self) {
        self.settings.is_loading_models = true;
        self.settings.models_error = None;

        if let Err(error) = self.send_request("model/list", json!({}), ClientRequestKind::ModelList)
        {
            self.settings.is_loading_models = false;
            self.settings.models_error = Some(error);
        }

        self.bump_revision();
    }

    fn consume_output(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }

        self.line_buffer
            .push_str(&String::from_utf8_lossy(bytes).replace('\r', ""));

        while let Some(newline) = self.line_buffer.find('\n') {
            let line = self.line_buffer[..newline].trim().to_string();
            self.line_buffer.drain(..=newline);
            if !line.is_empty() {
                self.handle_line(&line);
            }
        }
    }

    fn handle_line(&mut self, line: &str) {
        if let Some(cwd) = line.strip_prefix(REMOTE_CWD_PREFIX) {
            let cwd = cwd.trim();
            if !cwd.is_empty() {
                let cwd = cwd.to_string();
                self.remote_cwd = Some(cwd.clone());
                self.projects.remote_home = Some(cwd.clone());
                if self.cwd.is_none() {
                    self.cwd = Some(cwd.clone());
                    self.projects.current = Some(cwd.clone());
                    remember_unique(&mut self.projects.recent, cwd.clone(), 12);
                }
                if self.directory.path.is_none() {
                    self.directory.path = Some(cwd.clone());
                    self.directory.parent = parent_path(&cwd);
                }
                self.bump_revision();
            }
            return;
        }

        let Ok(message) = serde_json::from_str::<Value>(line) else {
            codex_debug(format_args!(
                "non-json app-server line bytes={}",
                line.len()
            ));
            self.push_status(format!(
                "app-server sent non-JSON output ({} bytes).",
                line.len()
            ));
            return;
        };

        if let Some(id) = message.get("id") {
            if message.get("method").is_some() && message.get("params").is_some() {
                self.handle_server_request(&message);
            } else {
                let id_u64 = id.as_u64();
                codex_debug(format_args!(
                    "response line id={} kind={:?} bytes={}",
                    id_u64
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    id_u64.and_then(|id| self.request_kinds.get(&id).copied()),
                    line.len()
                ));
                self.handle_response(id.clone(), &message);
            }
            return;
        }

        if let Some(method) = message.get("method").and_then(Value::as_str) {
            self.handle_notification(method, message.get("params").unwrap_or(&Value::Null));
        }
    }

    fn handle_response(&mut self, id: Value, message: &Value) {
        let id_u64 = id.as_u64();
        let kind = id_u64.and_then(|id| self.request_kinds.remove(&id));
        let known_request = kind.is_some();
        let started = Instant::now();

        if let Some(error) = message.get("error") {
            let description = error
                .get("message")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| error.to_string());
            if !matches!(kind, Some(ClientRequestKind::ModelList)) {
                self.last_error = Some(description.clone());
            }
            match kind {
                Some(ClientRequestKind::Initialize) => {
                    self.status = CodexStatus::Failed;
                    self.push_status(format!("Codex request failed: {description}"));
                }
                Some(ClientRequestKind::ModelList) => {
                    self.settings.is_loading_models = false;
                    self.settings.models_error = Some(description.clone());
                }
                Some(ClientRequestKind::DirectoryList) => {
                    self.directory.is_loading = false;
                    self.directory.error = Some(description.clone());
                }
                Some(ClientRequestKind::ThreadList) => {
                    self.threads.is_loading = false;
                    self.threads.error = Some(description.clone());
                }
                Some(ClientRequestKind::ThreadListMore) => {
                    self.threads.is_loading_more = false;
                    self.threads.error = Some(description.clone());
                }
                Some(ClientRequestKind::ThreadRead) | Some(ClientRequestKind::ThreadTurnsMore) => {
                    self.thread_detail.is_loading = false;
                    self.thread_detail.is_loading_more = false;
                    self.thread_detail.error = Some(description.clone());
                }
                Some(ClientRequestKind::ThreadResume) => {
                    self.operation = CodexOperationState::failed(description.clone());
                    self.thread_detail.is_loading = false;
                    self.thread_detail.error = Some(description.clone());
                    self.push_status(format!("Codex request failed: {description}"));
                }
                Some(ClientRequestKind::ThreadArchive)
                | Some(ClientRequestKind::ThreadUnarchive)
                | Some(ClientRequestKind::ThreadDelete)
                | Some(ClientRequestKind::ThreadRename)
                | Some(ClientRequestKind::ThreadFork)
                | Some(ClientRequestKind::TurnSteer)
                | Some(ClientRequestKind::TurnInterrupt) => {
                    self.operation = CodexOperationState::failed(description.clone());
                    self.push_status(format!("Codex request failed: {description}"));
                }
                _ => self.push_status(format!("Codex request failed: {description}")),
            }
            if let Some(id) = id_u64.filter(|_| known_request) {
                self.completed_requests.insert(id, Err(description));
            }
            codex_debug(format_args!(
                "response error applied id={} kind={:?} elapsed_ms={} op_error={}",
                id_u64
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                kind,
                started.elapsed().as_millis(),
                self.operation.last_error.as_deref().unwrap_or("")
            ));
            self.bump_revision();
            return;
        }

        match kind {
            Some(ClientRequestKind::Initialize) => {
                self.initialized = true;
                self.status = CodexStatus::Connected;
                self.push_status("JSON-RPC initialized.");
                #[cfg(feature = "native-integrations")]
                self.request_model_list();
            }
            Some(ClientRequestKind::ModelList) => {
                self.apply_model_list_response(message);
            }
            Some(ClientRequestKind::ThreadStart) => {
                self.apply_thread_response(message, "Codex thread ready.");
            }
            Some(ClientRequestKind::ThreadResume) => {
                self.apply_thread_response(message, "Codex thread resumed.");
            }
            Some(ClientRequestKind::DirectoryList) => {
                self.apply_directory_list_response(message);
            }
            Some(ClientRequestKind::ThreadList) => {
                self.apply_thread_list_response(message, false);
            }
            Some(ClientRequestKind::ThreadListMore) => {
                self.apply_thread_list_response(message, true);
            }
            Some(ClientRequestKind::ThreadRead) => {
                self.apply_thread_read_response(message);
            }
            Some(ClientRequestKind::ThreadTurnsMore) => {
                self.apply_thread_turns_response(message, true);
            }
            Some(ClientRequestKind::ThreadArchive) => {
                self.operation = CodexOperationState::succeeded("Thread archived.");
                self.remove_thread_from_visible_list();
            }
            Some(ClientRequestKind::ThreadUnarchive) => {
                self.operation = CodexOperationState::succeeded("Thread restored.");
                self.remove_thread_from_visible_list();
            }
            Some(ClientRequestKind::ThreadDelete) => {
                self.operation = CodexOperationState::succeeded("Thread deleted.");
                self.remove_thread_from_visible_list();
                self.thread_id = None;
                self.active_turn = None;
            }
            Some(ClientRequestKind::ThreadRename) => {
                self.operation = CodexOperationState::succeeded("Thread renamed.");
                self.apply_thread_operation_thread(message);
            }
            Some(ClientRequestKind::ThreadFork) => {
                self.apply_thread_response(message, "Forked Codex thread.");
                self.operation = CodexOperationState::succeeded("Thread forked.");
            }
            Some(ClientRequestKind::TurnStart) => {
                self.turn_active = true;
            }
            Some(ClientRequestKind::TurnSteer) => {
                self.operation = CodexOperationState::succeeded("Steer message sent.");
            }
            Some(ClientRequestKind::TurnInterrupt) => {
                self.turn_active = false;
                self.active_turn = None;
                self.operation = CodexOperationState::succeeded("Turn interrupted.");
            }
            None => {}
        }

        if let Some(id) = id_u64.filter(|_| known_request) {
            self.completed_requests.insert(id, Ok(()));
        }
        codex_debug(format_args!(
            "response applied id={} kind={:?} elapsed_ms={} thread_id={:?} messages={} op_running={} op_error={}",
            id_u64
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            kind,
            started.elapsed().as_millis(),
            self.thread_id,
            self.messages.len(),
            self.operation.is_running,
            self.operation.last_error.as_deref().unwrap_or("")
        ));
    }

    fn handle_notification(&mut self, method: &str, params: &Value) {
        match method {
            "thread/started" => {
                if let Some(thread_id) = params
                    .pointer("/thread/id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                {
                    self.thread_id = Some(thread_id);
                    self.status = CodexStatus::Connected;
                }
            }
            "thread/name/updated" => {
                let thread_id = params
                    .get("threadId")
                    .and_then(Value::as_str)
                    .or_else(|| params.pointer("/thread/id").and_then(Value::as_str));
                let name = params
                    .get("name")
                    .and_then(Value::as_str)
                    .or_else(|| params.pointer("/thread/name").and_then(Value::as_str));
                if let Some(thread_id) = thread_id {
                    self.update_thread_name(thread_id, name);
                }
            }
            "thread/archived" | "thread/unarchived" | "thread/deleted" => {
                let thread_id = params
                    .get("threadId")
                    .and_then(Value::as_str)
                    .or_else(|| params.pointer("/thread/id").and_then(Value::as_str))
                    .map(str::to_string);
                if let Some(thread_id) = thread_id {
                    self.remove_thread_by_id(&thread_id);
                    if method == "thread/deleted"
                        && self.thread_id.as_deref() == Some(thread_id.as_str())
                    {
                        self.thread_id = None;
                        self.turn_active = false;
                        self.active_turn = None;
                    }
                }
            }
            "turn/started" => {
                self.turn_active = true;
                self.active_turn = params
                    .pointer("/turn/id")
                    .and_then(Value::as_str)
                    .map(|id| CodexActiveTurn {
                        id: id.to_string(),
                        status: "inProgress".to_string(),
                    });
                self.bump_revision();
            }
            "turn/completed" => {
                self.turn_active = false;
                self.active_turn = None;
                self.mark_streaming_messages_complete();
                self.hide_reasoning_summaries();
            }
            "item/agentMessage/delta" => {
                let item_id = params
                    .get("itemId")
                    .and_then(Value::as_str)
                    .unwrap_or("assistant");
                let delta = params.get("delta").and_then(Value::as_str).unwrap_or("");
                self.append_assistant_delta(item_id, delta);
            }
            "item/reasoning/summaryTextDelta" => {
                let item_id = params
                    .get("itemId")
                    .and_then(Value::as_str)
                    .unwrap_or("reasoning");
                let delta = params.get("delta").and_then(Value::as_str).unwrap_or("");
                self.append_reasoning_summary_delta(item_id, delta);
            }
            "item/reasoning/summaryPartAdded" => {
                let item_id = params
                    .get("itemId")
                    .and_then(Value::as_str)
                    .unwrap_or("reasoning");
                self.append_reasoning_summary_delta(item_id, "\n\n");
            }
            "item/reasoning/textDelta" => {}
            "item/commandExecution/outputDelta" => {
                let item_id = params
                    .get("itemId")
                    .and_then(Value::as_str)
                    .unwrap_or("command");
                let delta = params.get("delta").and_then(Value::as_str).unwrap_or("");
                self.append_command_output_delta(item_id, delta);
            }
            "item/started" => self.handle_item_started(params),
            "item/completed" => self.handle_item_completed(params),
            "serverRequest/resolved" => {
                if let Some(request_id) = params.get("requestId") {
                    self.remove_pending_request_by_value(request_id);
                }
            }
            "error" => {
                let description = params
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Codex app-server reported an error")
                    .to_string();
                self.last_error = Some(description.clone());
                self.push_status(description);
            }
            "remoteControl/status/changed"
            | "account/updated"
            | "account/rateLimits/updated"
            | "thread/tokenUsage/updated"
            | "model/verification"
            | "model/safetyBuffering/updated" => {}
            other => {
                if other.ends_with("/updated") || other.ends_with("/changed") {
                    return;
                }
            }
        }
    }

    fn apply_thread_response(&mut self, message: &Value, status_message: &str) {
        let started = Instant::now();
        let Some(thread) = message.pointer("/result/thread") else {
            self.operation = CodexOperationState::failed("thread response returned no thread");
            self.thread_detail.error = Some("thread response returned no thread".to_string());
            codex_debug(format_args!(
                "thread response missing thread elapsed_ms={}",
                started.elapsed().as_millis()
            ));
            return;
        };
        let response_bytes = json_byte_len(message);
        let thread_bytes = json_byte_len(thread);
        let thread_turns = thread
            .get("turns")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or_default();
        let initial_page_bytes = message
            .pointer("/result/initialTurnsPage")
            .map(json_byte_len)
            .unwrap_or_default();
        let initial_page_turns = message
            .pointer("/result/initialTurnsPage/data")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or_default();
        codex_debug(format_args!(
            "thread response start response_bytes={} thread_bytes={} thread_turns={} initial_page_bytes={} initial_page_turns={}",
            response_bytes, thread_bytes, thread_turns, initial_page_bytes, initial_page_turns
        ));

        if let Some(thread_id) = thread.get("id").and_then(Value::as_str).map(str::to_string) {
            self.thread_id = Some(thread_id);
        }

        if let Some(cwd) = message
            .pointer("/result/cwd")
            .and_then(Value::as_str)
            .or_else(|| thread.get("cwd").and_then(Value::as_str))
            .map(str::to_string)
        {
            self.cwd = Some(cwd.clone());
            self.directory.path = Some(cwd.clone());
            self.directory.parent = parent_path(&cwd);
            self.projects.current = Some(cwd.clone());
            remember_unique(&mut self.projects.recent, cwd, 12);
        }

        self.settings.model = message
            .pointer("/result/model")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| self.settings.model.clone());
        self.settings.approval_policy = message
            .pointer("/result/approvalPolicy")
            .map(approval_policy_to_string)
            .or_else(|| self.settings.approval_policy.clone());

        self.status = CodexStatus::Connected;
        self.thread_detail.thread = parse_thread_summary(thread);
        self.thread_detail.is_loading = false;
        self.thread_detail.error = None;
        self.operation = CodexOperationState::succeeded(status_message);

        if let Some(page) = message.pointer("/result/initialTurnsPage") {
            self.load_turn_page_messages(page, false);
            self.thread_detail.turns_next_cursor = page
                .get("nextCursor")
                .and_then(Value::as_str)
                .map(str::to_string);
            self.thread_detail.turns_backwards_cursor = page
                .get("backwardsCursor")
                .and_then(Value::as_str)
                .map(str::to_string);
        } else if thread
            .get("turns")
            .and_then(Value::as_array)
            .is_some_and(|turns| !turns.is_empty())
        {
            self.load_thread_messages(thread);
        }
        self.push_status(status_message);
        codex_debug(format_args!(
            "thread response done elapsed_ms={} thread_id={:?} cwd={:?} messages={} next_cursor={:?}",
            started.elapsed().as_millis(),
            self.thread_id,
            self.cwd,
            self.messages.len(),
            self.thread_detail.turns_next_cursor
        ));
    }

    fn apply_directory_list_response(&mut self, message: &Value) {
        let path = self
            .directory
            .path
            .clone()
            .or_else(|| self.cwd.clone())
            .unwrap_or_else(|| "/".to_string());
        let mut entries = Vec::new();
        if let Some(items) = message.pointer("/result/entries").and_then(Value::as_array) {
            for item in items {
                let Some(name) = item.get("fileName").and_then(Value::as_str) else {
                    continue;
                };
                entries.push(CodexDirectoryEntry {
                    name: name.to_string(),
                    path: join_path(&path, name),
                    is_directory: item
                        .get("isDirectory")
                        .and_then(Value::as_bool)
                        .unwrap_or(false),
                    is_file: item.get("isFile").and_then(Value::as_bool).unwrap_or(false),
                });
            }
        }
        entries.sort_by(|a, b| {
            b.is_directory
                .cmp(&a.is_directory)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        });
        self.directory.entries = entries;
        self.directory.parent = parent_path(&path);
        self.directory.is_loading = false;
        self.directory.error = None;
        self.bump_revision();
    }

    fn apply_thread_list_response(&mut self, message: &Value, append: bool) {
        let should_refresh_recent_projects = self.threads.cwd.is_none()
            && self.threads.search_term.is_none()
            && !self.threads.archived;
        let threads = message
            .pointer("/result/data")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(parse_thread_summary)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if append {
            append_unique_threads(&mut self.threads.threads, threads);
        } else {
            self.threads.threads = threads;
        }
        self.threads.next_cursor = message
            .pointer("/result/nextCursor")
            .and_then(Value::as_str)
            .map(str::to_string);
        self.threads.backwards_cursor = message
            .pointer("/result/backwardsCursor")
            .and_then(Value::as_str)
            .map(str::to_string);
        self.threads.is_loading = false;
        self.threads.is_loading_more = false;
        self.threads.error = None;
        if should_refresh_recent_projects {
            self.projects.recent = recent_projects_from_threads(&self.threads.threads, 24);
            let current_empty = self
                .projects
                .current
                .as_deref()
                .map(str::trim)
                .unwrap_or_default()
                .is_empty();
            if current_empty {
                self.projects.current = self.projects.recent.first().cloned();
            }
        }
        self.bump_revision();
    }

    fn apply_thread_read_response(&mut self, message: &Value) {
        let Some(thread) = message.pointer("/result/thread") else {
            self.thread_detail.is_loading = false;
            self.thread_detail.error = Some("thread/read returned no thread".to_string());
            self.operation = CodexOperationState::failed("thread/read returned no thread");
            self.bump_revision();
            return;
        };

        self.thread_detail.thread = parse_thread_summary(thread);
        self.thread_detail.is_loading = false;
        self.thread_detail.is_loading_more = false;
        self.thread_detail.error = None;
        self.operation = CodexOperationState::succeeded("Thread loaded.");
        if thread
            .get("turns")
            .and_then(Value::as_array)
            .is_some_and(|turns| !turns.is_empty())
        {
            self.load_thread_messages(thread);
        }
        self.bump_revision();
    }

    fn apply_thread_turns_response(&mut self, message: &Value, append: bool) {
        if let Some(page) = message.pointer("/result") {
            self.load_turn_page_messages(page, append);
            self.thread_detail.turns_next_cursor = page
                .get("nextCursor")
                .and_then(Value::as_str)
                .map(str::to_string);
            self.thread_detail.turns_backwards_cursor = page
                .get("backwardsCursor")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        self.thread_detail.is_loading = false;
        self.thread_detail.is_loading_more = false;
        self.thread_detail.error = None;
        self.bump_revision();
    }

    fn apply_thread_operation_thread(&mut self, message: &Value) {
        if let Some(thread) = message.pointer("/result/thread") {
            if let Some(summary) = parse_thread_summary(thread) {
                self.upsert_thread_summary(summary.clone());
                if self.thread_id.as_deref() == Some(summary.id.as_str()) {
                    self.thread_detail.thread = Some(summary);
                }
            }
        }
        self.bump_revision();
    }

    fn load_thread_messages(&mut self, thread: &Value) {
        let started = Instant::now();
        self.messages.clear();
        self.clear_message_indices();

        let Some(turns) = thread.get("turns").and_then(Value::as_array) else {
            return;
        };
        let item_count = turns
            .iter()
            .filter_map(|turn| turn.get("items").and_then(Value::as_array))
            .map(Vec::len)
            .sum::<usize>();

        for turn in turns {
            let Some(items) = turn.get("items").and_then(Value::as_array) else {
                continue;
            };
            for item in items {
                self.load_thread_item_message(item);
            }
        }

        if self.messages.is_empty() {
            let message_id = self.next_message_id("status");
            self.messages.push(CodexMessage::status(
                message_id,
                "No loaded messages in this thread yet.",
            ));
        }
        self.bump_revision();
        codex_debug(format_args!(
            "load thread messages turns={} items={} messages={} elapsed_ms={}",
            turns.len(),
            item_count,
            self.messages.len(),
            started.elapsed().as_millis()
        ));
    }

    fn load_turn_page_messages(&mut self, page: &Value, append: bool) {
        let started = Instant::now();
        if !append {
            self.messages.clear();
            self.clear_message_indices();
        }

        let Some(turns) = page.get("data").and_then(Value::as_array) else {
            return;
        };
        let item_count = turns
            .iter()
            .filter_map(|turn| turn.get("items").and_then(Value::as_array))
            .map(Vec::len)
            .sum::<usize>();

        let mut ordered_turns = turns.iter().collect::<Vec<_>>();
        ordered_turns.sort_by_key(|turn| {
            turn.get("startedAt")
                .and_then(Value::as_u64)
                .unwrap_or_default()
        });

        for turn in ordered_turns {
            let Some(items) = turn.get("items").and_then(Value::as_array) else {
                continue;
            };
            for item in items {
                self.load_thread_item_message(item);
            }
        }

        if self.messages.is_empty() {
            let message_id = self.next_message_id("status");
            self.messages.push(CodexMessage::status(
                message_id,
                "No loaded messages in this thread yet.",
            ));
        }
        codex_debug(format_args!(
            "load turn page append={} turns={} items={} messages={} elapsed_ms={}",
            append,
            turns.len(),
            item_count,
            self.messages.len(),
            started.elapsed().as_millis()
        ));
    }

    fn load_thread_item_message(&mut self, item: &Value) {
        let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        let item_id = item
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or(item_type)
            .to_string();

        match item_type {
            "userMessage" => {
                let text = item
                    .get("content")
                    .and_then(Value::as_array)
                    .map(|content| {
                        content
                            .iter()
                            .filter_map(|input| match input.get("type").and_then(Value::as_str) {
                                Some("text") => input
                                    .get("text")
                                    .and_then(Value::as_str)
                                    .map(str::to_string),
                                Some("mention") | Some("skill") => input
                                    .get("path")
                                    .and_then(Value::as_str)
                                    .map(str::to_string),
                                Some("image") | Some("localImage") => input
                                    .get("url")
                                    .or_else(|| input.get("path"))
                                    .and_then(Value::as_str)
                                    .map(|url| {
                                        let alt = input
                                            .get("alt")
                                            .or_else(|| input.get("name"))
                                            .and_then(Value::as_str)
                                            .unwrap_or("Image");
                                        markdown_image_text(url, alt)
                                    }),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_default();
                if !text.trim().is_empty() {
                    self.messages
                        .push(CodexMessage::user(item_id, text.trim().to_string()));
                }
            }
            "agentMessage" => {
                let text = item.get("text").and_then(Value::as_str).unwrap_or("");
                if !text.trim().is_empty() {
                    self.messages.push(CodexMessage::assistant(item_id.clone()));
                    if let Some(message) = self.messages.last_mut() {
                        message.text = text.to_string();
                        message.kind = agent_message_kind(item);
                        message.is_streaming = false;
                        message.refresh_blocks();
                    }
                    self.assistant_message_indices
                        .insert(item_id, self.messages.len().saturating_sub(1));
                }
            }
            "reasoning" => {
                let summary = item
                    .get("summary")
                    .and_then(Value::as_array)
                    .map(|parts| {
                        parts
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    })
                    .unwrap_or_default();
                if !summary.trim().is_empty() {
                    let mut message = CodexMessage::reasoning_summary(item_id.clone());
                    message.transcript = Some(summary.clone());
                    if let Some(header) = extract_first_bold_text(&summary) {
                        message.text = header.clone();
                        message.detail = Some(header);
                    }
                    message.visibility = CodexMessageVisibility::TranscriptOnly;
                    message.is_streaming = false;
                    self.reasoning_message_indices
                        .insert(item_id, self.messages.len());
                    self.messages.push(message);
                }
            }
            "commandExecution" => {
                self.upsert_command_execution_message(item);
                if let Some(output) = item.get("aggregatedOutput").and_then(Value::as_str) {
                    if !output.trim().is_empty() {
                        self.set_command_output_text(&item_id, output);
                    }
                }
            }
            "fileChange" => {
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("completed");
                self.upsert_compact_event(
                    &item_id,
                    CodexMessageKind::FileChange,
                    file_change_title(status),
                    Some("File changes were prepared.".to_string()),
                    None,
                );
            }
            "mcpToolCall" => {
                let server = item.get("server").and_then(Value::as_str).unwrap_or("mcp");
                let tool = item.get("tool").and_then(Value::as_str).unwrap_or("tool");
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("completed");
                self.upsert_compact_event(
                    &item_id,
                    CodexMessageKind::ToolCall,
                    mcp_tool_title(status),
                    Some(format!("{server}.{tool}")),
                    mcp_tool_transcript(item),
                );
            }
            "plan" => {
                let text = item.get("text").and_then(Value::as_str).unwrap_or("");
                if !text.trim().is_empty() {
                    let mut message = CodexMessage::assistant(item_id.clone());
                    message.kind = CodexMessageKind::Plan;
                    message.title = Some("Plan".to_string());
                    message.text = text.to_string();
                    message.is_streaming = false;
                    message.refresh_blocks();
                    self.messages.push(message);
                }
            }
            "dynamicToolCall" | "sleep" => {}
            "webSearch" => {
                let query = item.get("query").and_then(Value::as_str).unwrap_or("");
                if !query.trim().is_empty() {
                    self.upsert_compact_event(
                        &item_id,
                        CodexMessageKind::ToolCall,
                        "Web search",
                        Some(query.to_string()),
                        None,
                    );
                }
            }
            "imageView" => {
                if let Some(path) = item.get("path").and_then(Value::as_str) {
                    self.messages.push(CodexMessage::image_event(
                        item_id,
                        "Viewed image",
                        path,
                        item.get("alt")
                            .or_else(|| item.get("name"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    ));
                }
            }
            "contextCompaction" => {
                self.upsert_compact_event(
                    &item_id,
                    CodexMessageKind::Status,
                    "Context compacted",
                    None,
                    None,
                );
            }
            _ => {}
        }
    }

    fn handle_item_started(&mut self, params: &Value) {
        let Some(item) = params.get("item") else {
            return;
        };
        let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        let item_id = item.get("id").and_then(Value::as_str).unwrap_or(item_type);
        match item_type {
            "agentMessage" => {
                let index =
                    if let Some(index) = self.assistant_message_indices.get(item_id).copied() {
                        index
                    } else {
                        let index = self.messages.len();
                        self.messages
                            .push(CodexMessage::assistant(item_id.to_string()));
                        self.assistant_message_indices
                            .insert(item_id.to_string(), index);
                        index
                    };
                if let Some(message) = self.messages.get_mut(index) {
                    message.kind = agent_message_kind(item);
                }
                self.bump_revision();
            }
            "reasoning" => {
                let summary = item
                    .get("summary")
                    .and_then(Value::as_array)
                    .map(|parts| {
                        parts
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    })
                    .filter(|summary| !summary.trim().is_empty());
                self.finalize_reasoning_summary(item_id, summary);
            }
            "commandExecution" => {
                self.upsert_command_execution_message(item);
                self.bump_revision();
            }
            "fileChange" => {
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::FileChange,
                    "Preparing file changes",
                    None,
                    None,
                );
                self.bump_revision();
            }
            "mcpToolCall" => {
                let server = item.get("server").and_then(Value::as_str).unwrap_or("mcp");
                let tool = item.get("tool").and_then(Value::as_str).unwrap_or("tool");
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::ToolCall,
                    "Calling tool",
                    Some(format!("{server}.{tool}")),
                    None,
                );
                self.bump_revision();
            }
            "webSearch" => {
                let query = item.get("query").and_then(Value::as_str).unwrap_or("");
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::ToolCall,
                    "Web search",
                    Some(query.to_string()),
                    None,
                );
                self.bump_revision();
            }
            "dynamicToolCall" | "sleep" => {}
            _ => {}
        }
    }

    fn handle_item_completed(&mut self, params: &Value) {
        let Some(item) = params.get("item") else {
            return;
        };
        let item_type = item.get("type").and_then(Value::as_str).unwrap_or("");
        match item_type {
            "agentMessage" => {
                let item_id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("assistant");
                let text = item.get("text").and_then(Value::as_str).unwrap_or("");
                if !text.is_empty() {
                    self.set_assistant_text(item_id, text);
                    if let Some(index) = self.assistant_message_indices.get(item_id).copied() {
                        if let Some(message) = self.messages.get_mut(index) {
                            message.kind = agent_message_kind(item);
                        }
                    }
                }
            }
            "reasoning" => {
                let item_id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("reasoning");
                let summary = item
                    .get("summary")
                    .and_then(Value::as_array)
                    .map(|parts| {
                        parts
                            .iter()
                            .filter_map(Value::as_str)
                            .collect::<Vec<_>>()
                            .join("\n\n")
                    })
                    .filter(|summary| !summary.trim().is_empty());
                self.finalize_reasoning_summary(item_id, summary);
            }
            "commandExecution" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or("command");
                self.upsert_command_execution_message(item);
                if let Some(output) = item.get("aggregatedOutput").and_then(Value::as_str) {
                    if !output.trim().is_empty() {
                        self.set_command_output_text(item_id, output);
                    }
                }
                self.bump_revision();
            }
            "fileChange" => {
                let item_id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("file-change");
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("completed");
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::FileChange,
                    file_change_title(status),
                    None,
                    None,
                );
                self.bump_revision();
            }
            "mcpToolCall" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or("mcp");
                let server = item.get("server").and_then(Value::as_str).unwrap_or("mcp");
                let tool = item.get("tool").and_then(Value::as_str).unwrap_or("tool");
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("completed");
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::ToolCall,
                    mcp_tool_title(status),
                    Some(format!("{server}.{tool}")),
                    mcp_tool_transcript(item),
                );
                self.bump_revision();
            }
            "imageView" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or("image");
                if let Some(path) = item.get("path").and_then(Value::as_str) {
                    self.messages.push(CodexMessage::image_event(
                        item_id,
                        "Viewed image",
                        path,
                        item.get("alt")
                            .or_else(|| item.get("name"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    ));
                    self.bump_revision();
                }
            }
            "dynamicToolCall" | "sleep" => {}
            _ => {}
        }
    }

    fn handle_server_request(&mut self, message: &Value) {
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return;
        };
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        let params = message.get("params").unwrap_or(&Value::Null);

        match method {
            "item/commandExecution/requestApproval" => {
                let command = params
                    .get("command")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let cwd = params
                    .get("cwd")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let reason = params
                    .get("reason")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                self.add_pending_request(
                    id,
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::Command,
                        title: "Command approval".to_string(),
                        detail: command
                            .clone()
                            .or_else(|| reason.clone())
                            .unwrap_or_else(|| "Codex wants to run a command.".to_string()),
                        command,
                        cwd,
                        reason,
                    },
                );
            }
            "item/fileChange/requestApproval" => {
                let reason = params
                    .get("reason")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let grant_root = params
                    .get("grantRoot")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                self.add_pending_request(
                    id,
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::FileChange,
                        title: "File change approval".to_string(),
                        detail: grant_root
                            .clone()
                            .or_else(|| reason.clone())
                            .unwrap_or_else(|| "Codex wants to apply file changes.".to_string()),
                        command: None,
                        cwd: grant_root,
                        reason,
                    },
                );
            }
            "item/tool/requestUserInput" => {
                self.add_pending_request(
                    id,
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::UserInput,
                        title: "User input".to_string(),
                        detail: params.to_string(),
                        command: None,
                        cwd: None,
                        reason: None,
                    },
                );
            }
            "item/permissions/requestApproval" => {
                self.add_pending_request(
                    id,
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::Permissions,
                        title: "Permissions".to_string(),
                        detail: params.to_string(),
                        command: None,
                        cwd: None,
                        reason: None,
                    },
                );
            }
            "currentTime/read" => {
                let seconds = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|duration| duration.as_secs())
                    .unwrap_or(0);
                let _ = self.send_response(id, json!({ "currentTimeAt": seconds }));
            }
            _ => {
                self.add_pending_request(
                    id,
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::Tool,
                        title: method.to_string(),
                        detail: params.to_string(),
                        command: None,
                        cwd: None,
                        reason: None,
                    },
                );
            }
        }
    }

    fn run_thread_operation(
        &mut self,
        method: &str,
        params: Value,
        kind: ClientRequestKind,
        label: &str,
    ) -> Result<CodexSnapshot, String> {
        self.poll();
        let thread_id = params
            .get("threadId")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        if thread_id.is_empty() {
            self.operation = CodexOperationState::failed("Choose a Codex thread first.");
            self.bump_revision();
            return Ok(self.snapshot());
        }

        self.operation_thread_id = Some(thread_id);
        self.operation = CodexOperationState::running(label);
        self.bump_revision();

        #[cfg(feature = "native-integrations")]
        {
            let id = self.send_request(method, params, kind)?;
            if let Err(error) = self.wait_for_request(id) {
                self.operation = CodexOperationState::failed(error.clone());
                self.push_status(error);
            }
        }

        Ok(self.snapshot())
    }

    fn apply_thread_settings(&self, params: &mut serde_json::Map<String, Value>) {
        if let Some(model) = &self.settings.model {
            params.insert("model".to_string(), json!(model));
        }
        if let Some(approval_policy) = &self.settings.approval_policy {
            params.insert(
                "approvalPolicy".to_string(),
                approval_policy_value(approval_policy),
            );
        }
        if let Some(sandbox) = &self.settings.sandbox {
            params.insert("sandbox".to_string(), json!(sandbox));
        }
    }

    fn apply_turn_settings(&self, params: &mut serde_json::Map<String, Value>) {
        if let Some(model) = &self.settings.model {
            params.insert("model".to_string(), json!(model));
        }
        if let Some(approval_policy) = &self.settings.approval_policy {
            params.insert(
                "approvalPolicy".to_string(),
                approval_policy_value(approval_policy),
            );
        }
    }

    fn apply_model_list_response(&mut self, message: &Value) {
        let mut models = parse_model_options(message);
        if models.is_empty() {
            models = default_model_options();
        }

        self.settings.available_models = models;
        self.settings.is_loading_models = false;
        self.settings.models_error = None;
        self.bump_revision();
    }

    fn remove_thread_from_visible_list(&mut self) {
        if let Some(thread_id) = self.operation_thread_id.take() {
            self.remove_thread_by_id(&thread_id);
        }
    }

    fn remove_thread_by_id(&mut self, thread_id: &str) {
        let before = self.threads.threads.len();
        self.threads.threads.retain(|thread| thread.id != thread_id);
        if self
            .thread_detail
            .thread
            .as_ref()
            .is_some_and(|thread| thread.id == thread_id)
        {
            self.thread_detail.thread = None;
        }
        if self.threads.threads.len() != before {
            self.bump_revision();
        }
    }

    fn update_thread_name(&mut self, thread_id: &str, name: Option<&str>) {
        let next_name = name.map(str::to_string);
        for thread in &mut self.threads.threads {
            if thread.id == thread_id {
                thread.name = next_name.clone();
            }
        }
        if let Some(thread) = &mut self.thread_detail.thread {
            if thread.id == thread_id {
                thread.name = next_name;
            }
        }
        self.bump_revision();
    }

    fn upsert_thread_summary(&mut self, summary: CodexThreadSummary) {
        if let Some(thread) = self
            .threads
            .threads
            .iter_mut()
            .find(|thread| thread.id == summary.id)
        {
            *thread = summary;
        } else {
            self.threads.threads.insert(0, summary);
        }
    }

    fn add_pending_request(&mut self, id: Value, mut approval: CodexApproval) {
        approval.request_id = request_id_to_string(&id);
        let message = format!("Approval needed: {}", approval.title);
        self.pending_approvals
            .push(PendingServerRequest { id, approval });
        self.push_status(message);
    }

    fn remove_pending_request_by_value(&mut self, id: &Value) {
        let before = self.pending_approvals.len();
        self.pending_approvals.retain(|pending| pending.id != *id);
        if self.pending_approvals.len() != before {
            self.bump_revision();
        }
    }

    fn apply_transport_status(&mut self, status: ssh::ExecStdioStatus) {
        match status {
            ssh::ExecStdioStatus::Connecting => {
                if !self.initialized {
                    self.status = CodexStatus::Connecting;
                }
            }
            ssh::ExecStdioStatus::Connected {
                observed_host_key_sha256,
            } => {
                self.observed_host_key_sha256 = observed_host_key_sha256;
                if self.initialized {
                    self.status = CodexStatus::Connected;
                } else {
                    self.status = CodexStatus::Connecting;
                }
            }
            ssh::ExecStdioStatus::Closed => {
                if self.status != CodexStatus::Disconnected {
                    self.status = CodexStatus::Disconnected;
                    self.turn_active = false;
                    self.push_status("Codex app-server closed.");
                }
            }
            ssh::ExecStdioStatus::Failed(error) => {
                if self.last_error.as_deref() != Some(error.as_str()) {
                    self.last_error = Some(error.clone());
                    self.status = CodexStatus::Failed;
                    self.turn_active = false;
                    self.push_status(error);
                }
            }
        }
    }

    fn append_assistant_delta(&mut self, item_id: &str, delta: &str) {
        if delta.is_empty() {
            return;
        }
        let index = if let Some(index) = self.assistant_message_indices.get(item_id).copied() {
            index
        } else {
            let index = self.messages.len();
            self.messages
                .push(CodexMessage::assistant(item_id.to_string()));
            self.assistant_message_indices
                .insert(item_id.to_string(), index);
            index
        };
        if let Some(message) = self.messages.get_mut(index) {
            message.text.push_str(delta);
            message.is_streaming = true;
            message.refresh_blocks();
            self.bump_revision();
        }
    }

    fn clear_message_indices(&mut self) {
        self.assistant_message_indices.clear();
        self.command_output_indices.clear();
        self.event_message_indices.clear();
        self.reasoning_message_indices.clear();
    }

    fn set_assistant_text(&mut self, item_id: &str, text: &str) {
        if text.is_empty() {
            return;
        }
        let index = if let Some(index) = self.assistant_message_indices.get(item_id).copied() {
            index
        } else {
            let index = self.messages.len();
            self.messages
                .push(CodexMessage::assistant(item_id.to_string()));
            self.assistant_message_indices
                .insert(item_id.to_string(), index);
            index
        };
        if let Some(message) = self.messages.get_mut(index) {
            message.text = text.to_string();
            message.is_streaming = false;
            message.refresh_blocks();
            self.bump_revision();
        }
    }

    fn append_reasoning_summary_delta(&mut self, item_id: &str, delta: &str) {
        if delta.is_empty() {
            return;
        }
        let index = if let Some(index) = self.reasoning_message_indices.get(item_id).copied() {
            index
        } else {
            let index = self.messages.len();
            self.messages
                .push(CodexMessage::reasoning_summary(item_id.to_string()));
            self.reasoning_message_indices
                .insert(item_id.to_string(), index);
            index
        };
        if let Some(message) = self.messages.get_mut(index) {
            let mut transcript = message.transcript.clone().unwrap_or_default();
            transcript.push_str(delta);
            message.transcript = Some(transcript.clone());
            if let Some(header) = extract_first_bold_text(&transcript) {
                message.text = header.clone();
                message.detail = Some(header);
            } else {
                message.text = "Thinking...".to_string();
                message.detail = None;
            }
            message.is_streaming = true;
            self.bump_revision();
        }
    }

    fn finalize_reasoning_summary(&mut self, item_id: &str, summary: Option<String>) {
        let index = if let Some(index) = self.reasoning_message_indices.get(item_id).copied() {
            index
        } else if summary
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
        {
            let index = self.messages.len();
            self.messages
                .push(CodexMessage::reasoning_summary(item_id.to_string()));
            self.reasoning_message_indices
                .insert(item_id.to_string(), index);
            index
        } else {
            return;
        };

        if let Some(message) = self.messages.get_mut(index) {
            if let Some(summary) = summary {
                if !summary.trim().is_empty() {
                    message.transcript = Some(summary);
                }
            }
            let transcript = message.transcript.clone().unwrap_or_default();
            if let Some(header) = extract_first_bold_text(&transcript) {
                message.text = header.clone();
                message.detail = Some(header);
            }
            message.visibility = CodexMessageVisibility::TranscriptOnly;
            message.is_streaming = false;
            self.bump_revision();
        }
    }

    fn upsert_compact_event(
        &mut self,
        item_id: &str,
        kind: CodexMessageKind,
        title: impl Into<String>,
        detail: Option<String>,
        transcript: Option<String>,
    ) -> usize {
        let title = title.into();
        let index = if let Some(index) = self.event_message_indices.get(item_id).copied() {
            index
        } else {
            let index = self.messages.len();
            self.messages.push(CodexMessage::compact_event(
                item_id.to_string(),
                kind,
                title.clone(),
                detail.clone(),
                transcript.clone(),
            ));
            self.event_message_indices
                .insert(item_id.to_string(), index);
            index
        };

        if let Some(message) = self.messages.get_mut(index) {
            message.role = CodexMessageRole::Tool;
            message.kind = kind;
            message.visibility = CodexMessageVisibility::Compact;
            message.title = Some(title.clone());
            message.text = detail.clone().unwrap_or(title);
            message.detail = detail;
            if transcript.is_some() {
                message.transcript = transcript;
            }
            message.format = CodexMessageFormat::Plain;
            message.is_streaming = false;
            message.truncated = false;
            message.blocks.clear();
        }

        index
    }

    fn upsert_command_execution_message(&mut self, item: &Value) {
        let item_id = item.get("id").and_then(Value::as_str).unwrap_or("command");
        let command = item.get("command").and_then(Value::as_str).unwrap_or("");
        let cwd = item.get("cwd").and_then(Value::as_str).unwrap_or("");
        let status = item
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("inProgress");
        let exit_code = item.get("exitCode").and_then(Value::as_i64);
        let line = format_command_line(cwd, command);
        let title = command_event_title(status, exit_code);
        let index = self.upsert_compact_event(
            item_id,
            CodexMessageKind::Command,
            title,
            if line.is_empty() { None } else { Some(line) },
            None,
        );
        self.command_output_indices
            .insert(item_id.to_string(), index);
    }

    fn append_command_output_delta(&mut self, item_id: &str, delta: &str) {
        if delta.is_empty() {
            return;
        }
        let output_id = format!("command-output-{item_id}");
        let index = if let Some(index) = self.command_output_indices.get(item_id).copied() {
            index
        } else {
            let index = self.messages.len();
            self.messages.push(CodexMessage::command_output(output_id));
            self.command_output_indices
                .insert(item_id.to_string(), index);
            index
        };
        if let Some(message) = self.messages.get_mut(index) {
            let mut output = message.transcript.clone().unwrap_or_default();
            if output.is_empty() && message.kind == CodexMessageKind::CommandOutput {
                output.push_str(&message.text);
            }
            output.push_str(delta);
            apply_command_output_preview(message, &output);
            message.refresh_blocks();
            self.bump_revision();
        }
    }

    fn set_command_output_text(&mut self, item_id: &str, output: &str) {
        if output.is_empty() {
            return;
        }
        let output_id = format!("command-output-{item_id}");
        let index = if let Some(index) = self.command_output_indices.get(item_id).copied() {
            index
        } else {
            let index = self.messages.len();
            self.messages.push(CodexMessage::command_output(output_id));
            self.command_output_indices
                .insert(item_id.to_string(), index);
            index
        };
        if let Some(message) = self.messages.get_mut(index) {
            apply_command_output_preview(message, output);
            message.refresh_blocks();
            self.bump_revision();
        }
    }

    fn mark_streaming_messages_complete(&mut self) {
        let mut changed = false;
        for message in &mut self.messages {
            if message.is_streaming {
                message.is_streaming = false;
                message.refresh_blocks();
                changed = true;
            }
        }
        if changed {
            self.bump_revision();
        }
    }

    fn hide_reasoning_summaries(&mut self) {
        let mut changed = false;
        for message in &mut self.messages {
            if message.kind == CodexMessageKind::ReasoningSummary
                && message.visibility != CodexMessageVisibility::TranscriptOnly
            {
                message.visibility = CodexMessageVisibility::TranscriptOnly;
                message.is_streaming = false;
                changed = true;
            }
        }
        if changed {
            self.bump_revision();
        }
    }

    fn push_status(&mut self, text: impl Into<String>) {
        let id = self.next_message_id("status");
        self.messages.push(CodexMessage::status(
            id,
            truncate_status_message(text.into()),
        ));
        self.bump_revision();
    }

    fn next_message_id(&mut self, prefix: &str) -> String {
        let id = format!("{prefix}-{}", self.next_local_message_id);
        self.next_local_message_id = self.next_local_message_id.saturating_add(1);
        id
    }

    fn bump_revision(&mut self) {
        self.local_revision = self.local_revision.saturating_add(1);
    }

    #[cfg(feature = "native-integrations")]
    fn wait_for_request(&mut self, id: u64) -> Result<(), String> {
        let started = Instant::now();
        let deadline = Instant::now() + APP_SERVER_REQUEST_TIMEOUT;
        loop {
            self.poll();
            if let Some(result) = self.completed_requests.remove(&id) {
                codex_debug(format_args!(
                    "request wait done id={} elapsed_ms={} result={}",
                    id,
                    started.elapsed().as_millis(),
                    if result.is_ok() { "ok" } else { "error" }
                ));
                return result;
            }
            if Instant::now() >= deadline {
                let message = "Codex app-server request timed out".to_string();
                self.completed_requests.remove(&id);
                codex_debug(format_args!(
                    "request wait timeout id={} elapsed_ms={}",
                    id,
                    started.elapsed().as_millis()
                ));
                return Err(message);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[cfg(feature = "native-integrations")]
    fn send_request(
        &mut self,
        method: &str,
        params: Value,
        kind: ClientRequestKind,
    ) -> Result<u64, String> {
        let id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.request_kinds.insert(id, kind);
        let params_bytes = json_byte_len(&params);
        codex_debug(format_args!(
            "request send id={} kind={kind:?} method={method} params_bytes={params_bytes}",
            id
        ));
        self.write_json(json!({
            "method": method,
            "id": id,
            "params": params
        }))?;
        Ok(id)
    }

    #[cfg(feature = "native-integrations")]
    fn send_notification(&mut self, method: &str, params: Value) -> Result<(), String> {
        self.write_json(json!({
            "method": method,
            "params": params
        }))
    }

    #[cfg(feature = "native-integrations")]
    fn send_response(&mut self, id: Value, result: Value) -> Result<(), String> {
        self.write_json(json!({
            "id": id,
            "result": result
        }))
    }

    #[cfg(feature = "native-integrations")]
    fn write_json(&mut self, message: Value) -> Result<(), String> {
        let mut line = serde_json::to_vec(&message)
            .map_err(|error| format!("codex json encode failed: {error}"))?;
        line.push(b'\n');
        self.transport.send_bytes(line)
    }
}

fn agent_message_kind(item: &Value) -> CodexMessageKind {
    match item.get("phase").and_then(Value::as_str) {
        Some("commentary") => CodexMessageKind::Commentary,
        Some("final_answer") | None => CodexMessageKind::FinalAnswer,
        Some(_) => CodexMessageKind::FinalAnswer,
    }
}

fn format_command_line(cwd: &str, command: &str) -> String {
    let command = command.trim();
    if command.is_empty() {
        return String::new();
    }
    let cwd = cwd.trim();
    if cwd.is_empty() {
        command.to_string()
    } else {
        format!("{cwd}$ {command}")
    }
}

fn command_event_title(status: &str, exit_code: Option<i64>) -> String {
    match status {
        "inProgress" | "in_progress" => "Running command".to_string(),
        "completed" => match exit_code {
            Some(0) | None => "Command completed".to_string(),
            Some(code) => format!("Command exited {code}"),
        },
        "failed" => match exit_code {
            Some(code) => format!("Command failed ({code})"),
            None => "Command failed".to_string(),
        },
        "declined" => "Command declined".to_string(),
        other => humanize_camel_status("Command", other),
    }
}

fn file_change_title(status: &str) -> String {
    match status {
        "inProgress" | "in_progress" => "Preparing file changes".to_string(),
        "completed" => "File changes applied".to_string(),
        "failed" => "File changes failed".to_string(),
        "declined" => "File changes declined".to_string(),
        other => humanize_camel_status("File changes", other),
    }
}

fn mcp_tool_title(status: &str) -> String {
    match status {
        "inProgress" | "in_progress" => "Calling tool".to_string(),
        "completed" => "Tool completed".to_string(),
        "failed" => "Tool failed".to_string(),
        other => humanize_camel_status("Tool", other),
    }
}

fn humanize_camel_status(prefix: &str, status: &str) -> String {
    if status.trim().is_empty() {
        return prefix.to_string();
    }
    let mut words = String::new();
    for (index, ch) in status.chars().enumerate() {
        if index > 0 && ch.is_ascii_uppercase() {
            words.push(' ');
        }
        words.push(ch.to_ascii_lowercase());
    }
    format!("{prefix} {words}")
}

fn mcp_tool_transcript(item: &Value) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(arguments) = item.get("arguments") {
        parts.push(format!("Arguments:\n{}", pretty_json_or_compact(arguments)));
    }
    if let Some(result) = item.get("result") {
        parts.push(format!("Result:\n{}", pretty_json_or_compact(result)));
    }
    if let Some(error) = item.get("error") {
        parts.push(format!("Error:\n{}", pretty_json_or_compact(error)));
    }
    if parts.is_empty() {
        None
    } else {
        Some(truncate_compact_transcript(&parts.join("\n\n")))
    }
}

fn pretty_json_or_compact(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

fn truncate_compact_transcript(value: &str) -> String {
    if value.chars().count() <= COMPACT_TRANSCRIPT_MAX_CHARS {
        return value.to_string();
    }
    let mut truncated = value
        .chars()
        .take(COMPACT_TRANSCRIPT_MAX_CHARS)
        .collect::<String>();
    truncated.push_str("\n\n[Transcript truncated in Shellow preview.]");
    truncated
}

fn truncate_status_message(value: String) -> String {
    if value.chars().count() <= STATUS_MESSAGE_MAX_CHARS {
        return value;
    }
    let mut truncated = value
        .chars()
        .take(STATUS_MESSAGE_MAX_CHARS)
        .collect::<String>();
    truncated.push_str("\n\n[Status message truncated in Shellow preview.]");
    truncated
}

fn apply_command_output_preview(message: &mut CodexMessage, output: &str) {
    let (preview, truncated, line_count) = command_output_preview(output);
    let output_label = match line_count {
        0 => "No output".to_string(),
        1 => {
            if truncated {
                "1 line output, truncated".to_string()
            } else {
                "1 line output".to_string()
            }
        }
        count => {
            if truncated {
                format!("{count} lines output, truncated")
            } else {
                format!("{count} lines output")
            }
        }
    };

    message.transcript = Some(truncate_compact_transcript(output));
    message.truncated = truncated;
    match message.kind {
        CodexMessageKind::Command => {
            message.detail = Some(output_label);
        }
        _ => {
            message.text = preview;
            message.detail = Some(output_label);
        }
    }
}

fn command_output_preview(output: &str) -> (String, bool, usize) {
    let line_count = output.lines().count();
    let mut preview = output
        .lines()
        .take(COMMAND_OUTPUT_PREVIEW_MAX_LINES)
        .collect::<Vec<_>>()
        .join("\n");
    let mut truncated = line_count > COMMAND_OUTPUT_PREVIEW_MAX_LINES;

    if preview.chars().count() > COMMAND_OUTPUT_PREVIEW_MAX_CHARS {
        preview = preview
            .chars()
            .take(COMMAND_OUTPUT_PREVIEW_MAX_CHARS)
            .collect::<String>();
        truncated = true;
    }

    if truncated {
        if !preview.is_empty() && !preview.ends_with('\n') {
            preview.push('\n');
        }
        preview.push_str("...");
    }

    (preview, truncated, line_count)
}

fn extract_first_bold_text(text: &str) -> Option<String> {
    for marker in ["**", "__"] {
        let Some(start) = text.find(marker) else {
            continue;
        };
        let rest = &text[start + marker.len()..];
        let Some(end) = rest.find(marker) else {
            continue;
        };
        let candidate = rest[..end].trim();
        if !candidate.is_empty() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn normalize_cwd(cwd: Option<String>) -> Option<String> {
    cwd.map(|cwd| cwd.trim().to_string())
        .filter(|cwd| !cwd.is_empty())
}

fn codex_app_server_command(cwd: Option<&str>) -> String {
    let mut command = String::new();
    if let Some(cwd) = cwd.map(str::trim).filter(|cwd| !cwd.is_empty()) {
        command.push_str("cd ");
        command.push_str(&shell_quote(cwd));
        command.push_str(" || exit $?; ");
    }
    command.push_str(CODEX_APP_SERVER_COMMAND_BODY);
    command
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[derive(Debug, Clone, Default)]
struct MarkdownInlineState {
    bold: bool,
    italic: bool,
    link_url: Option<String>,
}

fn parse_markdown_blocks(message_id: &str, text: &str) -> Vec<CodexMarkdownBlock> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let mut blocks = Vec::new();
    let mut block_index = 0usize;
    let mut inline_state = MarkdownInlineState::default();
    let mut quote_depth = 0usize;

    let mut current_kind: Option<CodexMarkdownBlockKind> = None;
    let mut current_level: Option<u8> = None;
    let mut current_runs: Vec<CodexMarkdownInlineRun> = Vec::new();
    let mut image_url: Option<String> = None;
    let mut image_alt = String::new();

    let mut code_language: Option<String> = None;
    let mut code_text = String::new();
    let mut in_code_block = false;

    let mut list_ordered: Option<bool> = None;
    let mut list_items: Vec<CodexMarkdownListItem> = Vec::new();
    let mut current_item_runs: Option<Vec<CodexMarkdownInlineRun>> = None;

    let mut table_headers: Vec<CodexMarkdownTableCell> = Vec::new();
    let mut table_rows: Vec<Vec<CodexMarkdownTableCell>> = Vec::new();
    let mut current_table_row: Option<Vec<CodexMarkdownTableCell>> = None;
    let mut current_table_cell_runs: Option<Vec<CodexMarkdownInlineRun>> = None;
    let mut in_table_head = false;

    for event in Parser::new_ext(text, options) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {
                    if current_item_runs.is_none() {
                        current_kind = Some(if quote_depth > 0 {
                            CodexMarkdownBlockKind::BlockQuote
                        } else {
                            CodexMarkdownBlockKind::Paragraph
                        });
                        current_level = None;
                        current_runs.clear();
                    }
                }
                Tag::Heading { level, .. } => {
                    current_kind = Some(CodexMarkdownBlockKind::Heading);
                    current_level = Some(heading_level_to_u8(level));
                    current_runs.clear();
                }
                Tag::BlockQuote(_) => {
                    quote_depth = quote_depth.saturating_add(1);
                }
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_language = code_block_language(kind);
                    code_text.clear();
                }
                Tag::List(start) => {
                    list_ordered = Some(start.is_some());
                    list_items.clear();
                }
                Tag::Item => {
                    current_item_runs = Some(Vec::new());
                }
                Tag::Table(_) => {
                    table_headers.clear();
                    table_rows.clear();
                    current_table_row = None;
                    current_table_cell_runs = None;
                    in_table_head = false;
                }
                Tag::TableHead => {
                    in_table_head = true;
                }
                Tag::TableRow => {
                    current_table_row = Some(Vec::new());
                }
                Tag::TableCell => {
                    current_table_cell_runs = Some(Vec::new());
                }
                Tag::Emphasis => {
                    inline_state.italic = true;
                }
                Tag::Strong => {
                    inline_state.bold = true;
                }
                Tag::Link { dest_url, .. } => {
                    inline_state.link_url = Some(dest_url.to_string());
                }
                Tag::Image { dest_url, .. } => {
                    if !current_runs.is_empty() {
                        if let Some(kind) = current_kind {
                            if matches!(
                                kind,
                                CodexMarkdownBlockKind::Paragraph
                                    | CodexMarkdownBlockKind::BlockQuote
                                    | CodexMarkdownBlockKind::Heading
                            ) {
                                push_text_block(
                                    &mut blocks,
                                    message_id,
                                    &mut block_index,
                                    kind,
                                    current_level,
                                    std::mem::take(&mut current_runs),
                                );
                            }
                        }
                    }
                    image_url = Some(dest_url.to_string());
                    image_alt.clear();
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    if let Some(kind) = current_kind.take() {
                        if kind == CodexMarkdownBlockKind::Paragraph
                            || kind == CodexMarkdownBlockKind::BlockQuote
                        {
                            push_text_block(
                                &mut blocks,
                                message_id,
                                &mut block_index,
                                kind,
                                None,
                                std::mem::take(&mut current_runs),
                            );
                        }
                    }
                }
                TagEnd::Heading(_) => {
                    if current_kind.take() == Some(CodexMarkdownBlockKind::Heading) {
                        push_text_block(
                            &mut blocks,
                            message_id,
                            &mut block_index,
                            CodexMarkdownBlockKind::Heading,
                            current_level.take(),
                            std::mem::take(&mut current_runs),
                        );
                    }
                }
                TagEnd::BlockQuote(_) => {
                    quote_depth = quote_depth.saturating_sub(1);
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    blocks.push(CodexMarkdownBlock::code(
                        markdown_block_id(message_id, block_index),
                        code_language.take(),
                        std::mem::take(&mut code_text),
                        false,
                    ));
                    block_index = block_index.saturating_add(1);
                }
                TagEnd::List(_) => {
                    if let Some(ordered) = list_ordered.take() {
                        if !list_items.is_empty() {
                            blocks.push(CodexMarkdownBlock::list(
                                markdown_block_id(message_id, block_index),
                                ordered,
                                std::mem::take(&mut list_items),
                            ));
                            block_index = block_index.saturating_add(1);
                        }
                    }
                }
                TagEnd::Table => {
                    if !table_headers.is_empty() || !table_rows.is_empty() {
                        blocks.push(CodexMarkdownBlock::table(
                            markdown_block_id(message_id, block_index),
                            std::mem::take(&mut table_headers),
                            std::mem::take(&mut table_rows),
                        ));
                        block_index = block_index.saturating_add(1);
                    }
                    current_table_row = None;
                    current_table_cell_runs = None;
                    in_table_head = false;
                }
                TagEnd::TableHead => {
                    in_table_head = false;
                }
                TagEnd::TableRow => {
                    if !in_table_head {
                        if let Some(row) = current_table_row.take() {
                            if !row.is_empty() {
                                table_rows.push(row);
                            }
                        }
                    } else {
                        current_table_row = None;
                    }
                }
                TagEnd::TableCell => {
                    if let Some(runs) = current_table_cell_runs.take() {
                        let cell = table_cell_from_runs(runs);
                        if in_table_head {
                            table_headers.push(cell);
                        } else if let Some(row) = current_table_row.as_mut() {
                            row.push(cell);
                        }
                    }
                }
                TagEnd::Item => {
                    if let Some(runs) = current_item_runs.take() {
                        let text = runs_to_text(&runs).trim().to_string();
                        if !text.is_empty() {
                            list_items.push(CodexMarkdownListItem { text, runs });
                        }
                    }
                }
                TagEnd::Emphasis => {
                    inline_state.italic = false;
                }
                TagEnd::Strong => {
                    inline_state.bold = false;
                }
                TagEnd::Link => {
                    inline_state.link_url = None;
                }
                TagEnd::Image => {
                    if let Some(url) = image_url.take() {
                        let alt = normalize_image_alt(&image_alt);
                        blocks.push(CodexMarkdownBlock::image(
                            markdown_block_id(message_id, block_index),
                            url,
                            alt,
                        ));
                        block_index = block_index.saturating_add(1);
                        image_alt.clear();
                    }
                }
                _ => {}
            },
            Event::Text(value) => {
                if in_code_block {
                    code_text.push_str(&value);
                } else if image_url.is_some() {
                    image_alt.push_str(&value);
                } else {
                    if current_kind.is_none()
                        && current_item_runs.is_none()
                        && current_table_cell_runs.is_none()
                    {
                        current_kind = Some(if quote_depth > 0 {
                            CodexMarkdownBlockKind::BlockQuote
                        } else {
                            CodexMarkdownBlockKind::Paragraph
                        });
                    }
                    append_markdown_text(
                        &mut current_runs,
                        current_item_runs.as_mut(),
                        current_table_cell_runs.as_mut(),
                        &inline_state,
                        &value,
                        false,
                    );
                }
            }
            Event::Code(value) => {
                if image_url.is_some() {
                    image_alt.push_str(&value);
                } else {
                    append_markdown_text(
                        &mut current_runs,
                        current_item_runs.as_mut(),
                        current_table_cell_runs.as_mut(),
                        &inline_state,
                        &value,
                        true,
                    );
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if image_url.is_some() {
                    image_alt.push('\n');
                } else {
                    append_markdown_text(
                        &mut current_runs,
                        current_item_runs.as_mut(),
                        current_table_cell_runs.as_mut(),
                        &inline_state,
                        "\n",
                        false,
                    );
                }
            }
            Event::Rule => {
                blocks.push(CodexMarkdownBlock::rule(markdown_block_id(
                    message_id,
                    block_index,
                )));
                block_index = block_index.saturating_add(1);
            }
            Event::Html(value) | Event::InlineHtml(value) => {
                append_markdown_text(
                    &mut current_runs,
                    current_item_runs.as_mut(),
                    current_table_cell_runs.as_mut(),
                    &inline_state,
                    &value,
                    false,
                );
            }
            _ => {}
        }
    }

    if let Some(runs) = current_item_runs.take() {
        let text = runs_to_text(&runs).trim().to_string();
        if !text.is_empty() {
            list_items.push(CodexMarkdownListItem { text, runs });
        }
    }
    if let Some(ordered) = list_ordered.take() {
        if !list_items.is_empty() {
            blocks.push(CodexMarkdownBlock::list(
                markdown_block_id(message_id, block_index),
                ordered,
                std::mem::take(&mut list_items),
            ));
            block_index = block_index.saturating_add(1);
        }
    }
    if in_code_block || !code_text.is_empty() {
        blocks.push(CodexMarkdownBlock::code(
            markdown_block_id(message_id, block_index),
            code_language.take(),
            std::mem::take(&mut code_text),
            true,
        ));
    }

    if markdown_has_unclosed_fence(text) {
        if let Some(block) = blocks
            .iter_mut()
            .rev()
            .find(|block| block.kind == CodexMarkdownBlockKind::CodeBlock)
        {
            block.incomplete = true;
        }
    }

    if blocks.is_empty() {
        vec![CodexMarkdownBlock::text(
            markdown_block_id(message_id, 0),
            CodexMarkdownBlockKind::Paragraph,
            text.to_string(),
            vec![CodexMarkdownInlineRun {
                text: text.to_string(),
                style: CodexMarkdownInlineStyle::Text,
                url: None,
            }],
        )]
    } else {
        blocks
    }
}

fn push_text_block(
    blocks: &mut Vec<CodexMarkdownBlock>,
    message_id: &str,
    block_index: &mut usize,
    kind: CodexMarkdownBlockKind,
    level: Option<u8>,
    runs: Vec<CodexMarkdownInlineRun>,
) {
    let text = runs_to_text(&runs).trim_end().to_string();
    if text.trim().is_empty() {
        return;
    }
    let block = if kind == CodexMarkdownBlockKind::Heading {
        CodexMarkdownBlock::heading(
            markdown_block_id(message_id, *block_index),
            level.unwrap_or(1),
            text,
            runs,
        )
    } else {
        CodexMarkdownBlock::text(
            markdown_block_id(message_id, *block_index),
            kind,
            text,
            runs,
        )
    };
    blocks.push(block);
    *block_index = (*block_index).saturating_add(1);
}

fn append_markdown_text(
    current_runs: &mut Vec<CodexMarkdownInlineRun>,
    current_item_runs: Option<&mut Vec<CodexMarkdownInlineRun>>,
    current_table_cell_runs: Option<&mut Vec<CodexMarkdownInlineRun>>,
    inline_state: &MarkdownInlineState,
    text: &str,
    inline_code: bool,
) {
    if text.is_empty() {
        return;
    }
    let style = if inline_code {
        CodexMarkdownInlineStyle::Code
    } else if inline_state.link_url.is_some() {
        CodexMarkdownInlineStyle::Link
    } else if inline_state.bold && inline_state.italic {
        CodexMarkdownInlineStyle::BoldItalic
    } else if inline_state.bold {
        CodexMarkdownInlineStyle::Bold
    } else if inline_state.italic {
        CodexMarkdownInlineStyle::Italic
    } else {
        CodexMarkdownInlineStyle::Text
    };
    let run = CodexMarkdownInlineRun {
        text: text.to_string(),
        style,
        url: inline_state.link_url.clone(),
    };
    match (current_table_cell_runs, current_item_runs) {
        (Some(runs), _) => append_markdown_run(runs, run),
        (None, Some(runs)) => append_markdown_run(runs, run),
        (None, None) => append_markdown_run(current_runs, run),
    }
}

fn table_cell_from_runs(runs: Vec<CodexMarkdownInlineRun>) -> CodexMarkdownTableCell {
    CodexMarkdownTableCell {
        text: runs_to_text(&runs).trim().to_string(),
        runs,
    }
}

fn table_text(headers: &[CodexMarkdownTableCell], rows: &[Vec<CodexMarkdownTableCell>]) -> String {
    let mut lines = Vec::new();
    if !headers.is_empty() {
        lines.push(
            headers
                .iter()
                .map(|cell| cell.text.as_str())
                .collect::<Vec<_>>()
                .join("\t"),
        );
    }
    lines.extend(rows.iter().map(|row| {
        row.iter()
            .map(|cell| cell.text.as_str())
            .collect::<Vec<_>>()
            .join("\t")
    }));
    lines.join("\n")
}

fn normalize_image_alt(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn markdown_image_text(url: &str, alt: &str) -> String {
    let alt = alt.replace(']', "\\]");
    let url = url.replace(')', "%29");
    format!("![{alt}]({url})")
}

fn append_markdown_run(runs: &mut Vec<CodexMarkdownInlineRun>, run: CodexMarkdownInlineRun) {
    if let Some(last) = runs.last_mut() {
        if last.style == run.style && last.url == run.url {
            last.text.push_str(&run.text);
            return;
        }
    }
    runs.push(run);
}

fn runs_to_text(runs: &[CodexMarkdownInlineRun]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

fn markdown_block_id(message_id: &str, block_index: usize) -> String {
    format!("{message_id}-md-{block_index}")
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn code_block_language(kind: CodeBlockKind<'_>) -> Option<String> {
    match kind {
        CodeBlockKind::Fenced(info) => first_code_info_word(info),
        CodeBlockKind::Indented => None,
    }
}

fn first_code_info_word(info: CowStr<'_>) -> Option<String> {
    info.split_whitespace()
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn markdown_has_unclosed_fence(text: &str) -> bool {
    let mut backtick_count = 0usize;
    let mut tilde_count = 0usize;
    for line in text.lines() {
        let line = line.trim_start();
        if line.starts_with("```") {
            backtick_count = backtick_count.saturating_add(1);
        } else if line.starts_with("~~~") {
            tilde_count = tilde_count.saturating_add(1);
        }
    }
    backtick_count % 2 == 1 || tilde_count % 2 == 1
}

fn normalize_path_input(path: &str) -> Option<String> {
    let path = path.trim();
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

fn normalize_path_input_opt(path: Option<&str>) -> Option<String> {
    path.and_then(normalize_path_input)
}

fn parent_path(path: &str) -> Option<String> {
    let path = path.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return None;
    }
    match path.rfind('/') {
        Some(0) => Some("/".to_string()),
        Some(index) => Some(path[..index].to_string()),
        None => None,
    }
}

fn join_path(parent: &str, child: &str) -> String {
    if parent == "/" {
        format!("/{child}")
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), child)
    }
}

fn parse_thread_summary(value: &Value) -> Option<CodexThreadSummary> {
    Some(CodexThreadSummary {
        id: value.get("id")?.as_str()?.to_string(),
        name: value
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string),
        preview: value
            .get("preview")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        cwd: value
            .get("cwd")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        status: value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        updated_at: value
            .get("updatedAt")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        created_at: value
            .get("createdAt")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        source: value
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        model_provider: value
            .get("modelProvider")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        forked_from_id: value
            .get("forkedFromId")
            .and_then(Value::as_str)
            .map(str::to_string),
        parent_thread_id: value
            .get("parentThreadId")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn append_unique_threads(target: &mut Vec<CodexThreadSummary>, threads: Vec<CodexThreadSummary>) {
    for thread in threads {
        if let Some(existing) = target.iter_mut().find(|existing| existing.id == thread.id) {
            *existing = thread;
        } else {
            target.push(thread);
        }
    }
}

fn recent_projects_from_threads(threads: &[CodexThreadSummary], max_len: usize) -> Vec<String> {
    let mut projects = Vec::new();
    for thread in threads {
        let cwd = thread.cwd.trim();
        if cwd.is_empty() || projects.iter().any(|existing| existing == cwd) {
            continue;
        }
        projects.push(cwd.to_string());
        if projects.len() >= max_len {
            break;
        }
    }
    projects
}

fn remember_unique(target: &mut Vec<String>, value: String, max_len: usize) {
    target.retain(|existing| existing != &value);
    target.insert(0, value);
    target.truncate(max_len);
}

fn default_model_options() -> Vec<CodexModelOption> {
    [
        ("gpt-5.5", "GPT-5.5"),
        ("gpt-5.4", "GPT-5.4"),
        ("gpt-5.4-mini", "GPT-5.4 Mini"),
        ("gpt-5.3-codex-spark", "GPT-5.3 Codex Spark"),
        ("codex-auto-review", "Codex Auto Review"),
    ]
    .into_iter()
    .map(|(id, name)| CodexModelOption {
        id: id.to_string(),
        name: name.to_string(),
    })
    .collect()
}

fn parse_model_options(message: &Value) -> Vec<CodexModelOption> {
    let result = message.get("result").unwrap_or(&Value::Null);
    let mut models = Vec::new();

    for candidate in [
        result.get("models"),
        result.get("modelOptions"),
        result.get("model_options"),
        result.get("data"),
        result.get("items"),
        Some(result),
    ]
    .into_iter()
    .flatten()
    {
        collect_model_options(candidate, &mut models);
        if !models.is_empty() {
            break;
        }
    }

    models
}

fn collect_model_options(value: &Value, target: &mut Vec<CodexModelOption>) {
    match value {
        Value::Array(items) => {
            for item in items {
                push_model_option(target, parse_model_option(item, None));
            }
        }
        Value::Object(object) => {
            if let Some(option) = parse_model_option(value, None) {
                push_model_option(target, Some(option));
            } else {
                for (key, item) in object {
                    let fallback_id = key.as_str();
                    let option = if let Some(name) = item.as_str() {
                        Some(CodexModelOption {
                            id: fallback_id.to_string(),
                            name: clean_model_text(name).unwrap_or_else(|| fallback_id.to_string()),
                        })
                    } else {
                        parse_model_option(item, Some(fallback_id))
                    };
                    push_model_option(target, option);
                }
            }
        }
        Value::String(_) => push_model_option(target, parse_model_option(value, None)),
        _ => {}
    }
}

fn parse_model_option(value: &Value, fallback_id: Option<&str>) -> Option<CodexModelOption> {
    if let Some(id) = value.as_str().and_then(clean_model_text) {
        return Some(CodexModelOption {
            name: id.clone(),
            id,
        });
    }

    let object = value.as_object()?;
    let id = object
        .get("slug")
        .or_else(|| object.get("id"))
        .or_else(|| object.get("model"))
        .or_else(|| object.get("modelId"))
        .or_else(|| object.get("model_id"))
        .or_else(|| object.get("value"))
        .and_then(Value::as_str)
        .and_then(clean_model_text)
        .or_else(|| fallback_id.and_then(clean_model_text))?;

    let name = object
        .get("display_name")
        .or_else(|| object.get("displayName"))
        .or_else(|| object.get("name"))
        .or_else(|| object.get("title"))
        .or_else(|| object.get("label"))
        .and_then(Value::as_str)
        .and_then(clean_model_text)
        .unwrap_or_else(|| id.clone());

    Some(CodexModelOption { id, name })
}

fn push_model_option(target: &mut Vec<CodexModelOption>, option: Option<CodexModelOption>) {
    let Some(option) = option else {
        return;
    };
    if option.id.is_empty() || target.iter().any(|existing| existing.id == option.id) {
        return;
    }
    target.push(option);
}

fn clean_model_text(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value == "null" {
        None
    } else {
        Some(value.to_string())
    }
}

fn text_input_value(text: &str) -> Value {
    json!([
        {
            "type": "text",
            "text": text,
            "text_elements": []
        }
    ])
}

fn normalize_setting(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn normalize_approval_policy(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    match value {
        "untrusted" | "on-failure" | "on-request" | "never" => Some(value.to_string()),
        "on_failure" | "onFailure" => Some("on-failure".to_string()),
        "on_request" | "onRequest" => Some("on-request".to_string()),
        _ => None,
    }
}

fn normalize_sandbox(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    match value {
        "read-only" | "workspace-write" | "danger-full-access" => Some(value.to_string()),
        "read_only" | "readOnly" => Some("read-only".to_string()),
        "workspace_write" | "workspaceWrite" => Some("workspace-write".to_string()),
        "danger_full_access" | "dangerFullAccess" => Some("danger-full-access".to_string()),
        _ => None,
    }
}

fn approval_policy_value(value: &str) -> Value {
    json!(value)
}

fn approval_policy_to_string(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn request_id_to_string(id: &Value) -> String {
    if let Some(value) = id.as_str() {
        value.to_string()
    } else if let Some(value) = id.as_u64() {
        value.to_string()
    } else if let Some(value) = id.as_i64() {
        value.to_string()
    } else {
        id.to_string()
    }
}

fn normalize_approval_decision(decision: &str) -> &'static str {
    match decision {
        "acceptForSession" | "accept_for_session" => "acceptForSession",
        "decline" => "decline",
        "cancel" => "cancel",
        _ => "accept",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_and_parents_remote_paths() {
        assert_eq!(join_path("/", "Users"), "/Users");
        assert_eq!(
            join_path("/Users/zinglix/", "Shellow"),
            "/Users/zinglix/Shellow"
        );
        assert_eq!(
            parent_path("/Users/zinglix/Shellow"),
            Some("/Users/zinglix".to_string())
        );
        assert_eq!(parent_path("/Users"), Some("/".to_string()));
        assert_eq!(parent_path("/"), None);
    }

    #[test]
    fn builds_app_server_command_with_remote_cwd() {
        let command = codex_app_server_command(Some("/Users/zinglix/Shellow"));
        assert!(command.starts_with("cd '/Users/zinglix/Shellow' || exit $?; "));
        assert!(command.contains("SHELLOW_CODEX_CWD=\"$(pwd -P 2>/dev/null || pwd)\""));
        assert!(command.contains("exec codex app-server --stdio"));
    }

    #[test]
    fn shell_quotes_remote_cwd() {
        assert_eq!(shell_quote("/tmp/O'Brien"), "'/tmp/O'\\''Brien'");
    }

    #[test]
    fn normalizes_supported_settings_only() {
        assert_eq!(
            normalize_approval_policy(Some("onFailure")),
            Some("on-failure".to_string())
        );
        assert_eq!(
            normalize_approval_policy(Some("never")),
            Some("never".to_string())
        );
        assert_eq!(normalize_approval_policy(Some("bogus")), None);
        assert_eq!(
            normalize_sandbox(Some("workspaceWrite")),
            Some("workspace-write".to_string())
        );
        assert_eq!(
            normalize_sandbox(Some("danger-full-access")),
            Some("danger-full-access".to_string())
        );
        assert_eq!(normalize_sandbox(Some("root")), None);
    }

    #[test]
    fn parses_thread_summary_with_optional_lineage() {
        let value = json!({
            "id": "thread-1",
            "name": "Build native Codex",
            "preview": "hello",
            "cwd": "/Users/zinglix/Shellow",
            "status": "completed",
            "updatedAt": 1,
            "createdAt": 0,
            "source": "app-server",
            "modelProvider": "openai",
            "forkedFromId": "thread-0",
            "parentThreadId": null
        });

        let summary = parse_thread_summary(&value).expect("thread summary");
        assert_eq!(summary.id, "thread-1");
        assert_eq!(summary.name.as_deref(), Some("Build native Codex"));
        assert_eq!(summary.forked_from_id.as_deref(), Some("thread-0"));
        assert_eq!(summary.parent_thread_id, None);
    }

    #[test]
    fn parses_markdown_blocks_for_native_rendering() {
        let blocks = parse_markdown_blocks(
            "msg",
            "# Title\n\nHello **bold** _em_ `code` [site](https://example.com).\n\n- one\n- two\n\n```rust\nfn main() {}\n```",
        );

        assert_eq!(blocks.len(), 4);
        assert_eq!(blocks[0].kind, CodexMarkdownBlockKind::Heading);
        assert_eq!(blocks[0].level, Some(1));
        assert_eq!(blocks[0].text, "Title");

        assert_eq!(blocks[1].kind, CodexMarkdownBlockKind::Paragraph);
        assert!(
            blocks[1]
                .runs
                .iter()
                .any(|run| run.text == "bold" && run.style == CodexMarkdownInlineStyle::Bold)
        );
        assert!(
            blocks[1]
                .runs
                .iter()
                .any(|run| run.text == "em" && run.style == CodexMarkdownInlineStyle::Italic)
        );
        assert!(
            blocks[1]
                .runs
                .iter()
                .any(|run| run.text == "code" && run.style == CodexMarkdownInlineStyle::Code)
        );
        assert!(blocks[1].runs.iter().any(|run| {
            run.text == "site"
                && run.style == CodexMarkdownInlineStyle::Link
                && run.url.as_deref() == Some("https://example.com")
        }));

        assert_eq!(blocks[2].kind, CodexMarkdownBlockKind::List);
        assert!(!blocks[2].ordered);
        assert_eq!(blocks[2].items.len(), 2);
        assert_eq!(blocks[2].items[0].text, "one");

        assert_eq!(blocks[3].kind, CodexMarkdownBlockKind::CodeBlock);
        assert_eq!(blocks[3].language.as_deref(), Some("rust"));
        assert_eq!(blocks[3].text, "fn main() {}\n");
        assert!(!blocks[3].incomplete);
    }

    #[test]
    fn parses_markdown_image_blocks_for_native_rendering() {
        let blocks = parse_markdown_blocks(
            "image-msg",
            "Before\n\n![Plot](https://example.com/plot.png)\n\nAfter",
        );

        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].kind, CodexMarkdownBlockKind::Paragraph);
        assert_eq!(blocks[0].text, "Before");

        let image = &blocks[1];
        assert_eq!(image.kind, CodexMarkdownBlockKind::Image);
        assert_eq!(
            image.image_url.as_deref(),
            Some("https://example.com/plot.png")
        );
        assert_eq!(image.image_alt.as_deref(), Some("Plot"));
        assert_eq!(image.text, "Plot");

        assert_eq!(blocks[2].kind, CodexMarkdownBlockKind::Paragraph);
        assert_eq!(blocks[2].text, "After");
    }

    #[test]
    fn builds_image_event_as_primary_markdown_message() {
        let message = CodexMessage::image_event(
            "image-view-1",
            "Viewed image",
            "file:///tmp/chart.png",
            Some("Chart".to_string()),
        );

        assert_eq!(message.visibility, CodexMessageVisibility::Primary);
        assert_eq!(message.format, CodexMessageFormat::Markdown);
        assert_eq!(message.blocks.len(), 1);
        assert_eq!(message.blocks[0].kind, CodexMarkdownBlockKind::Image);
        assert_eq!(
            message.blocks[0].image_url.as_deref(),
            Some("file:///tmp/chart.png")
        );
        assert_eq!(message.blocks[0].image_alt.as_deref(), Some("Chart"));
    }

    #[test]
    fn marks_unclosed_fenced_code_as_incomplete() {
        let blocks = parse_markdown_blocks("stream", "Before\n\n```swift\nprint(\"hi\")");
        let code = blocks
            .iter()
            .find(|block| block.kind == CodexMarkdownBlockKind::CodeBlock)
            .expect("streaming code block");

        assert_eq!(code.language.as_deref(), Some("swift"));
        assert_eq!(code.text, "print(\"hi\")");
        assert!(code.incomplete);
    }

    #[test]
    fn refreshed_assistant_message_streams_markdown_blocks() {
        let mut message = CodexMessage::assistant("assistant-1");
        message
            .text
            .push_str("Working on it\n\n1. first\n2. second");
        message.refresh_blocks();

        assert!(message.is_streaming);
        assert_eq!(message.format, CodexMessageFormat::Markdown);
        assert_eq!(message.blocks.len(), 2);
        assert_eq!(message.blocks[1].kind, CodexMarkdownBlockKind::List);
        assert!(message.blocks[1].ordered);
    }

    #[test]
    fn parses_markdown_table_blocks_for_native_rendering() {
        let blocks = parse_markdown_blocks("table", "| Key | Value |\n| --- | --- |\n| a | `b` |");

        assert_eq!(blocks.len(), 1);
        let table = &blocks[0];
        assert_eq!(table.kind, CodexMarkdownBlockKind::Table);
        assert_eq!(table.table_headers.len(), 2);
        assert_eq!(table.table_headers[0].text, "Key");
        assert_eq!(table.table_headers[1].text, "Value");
        assert_eq!(table.table_rows.len(), 1);
        assert_eq!(table.table_rows[0].len(), 2);
        assert_eq!(table.table_rows[0][0].text, "a");
        assert_eq!(table.table_rows[0][1].text, "b");
        assert_eq!(
            table.table_rows[0][1].runs[0].style,
            CodexMarkdownInlineStyle::Code
        );
    }
}
