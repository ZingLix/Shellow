use std::{
    collections::HashMap,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{HostProfile, ssh};

const CODEX_APP_SERVER_COMMAND_BODY: &str = r#"SHELLOW_CODEX_CWD="$(pwd -P 2>/dev/null || pwd)"
PATH="$PATH:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.bun/bin:$HOME/.npm-global/bin:/home/linuxbrew/.linuxbrew/bin"
export PATH
if ! command -v codex >/dev/null 2>&1; then
    echo "codex executable not found in non-interactive SSH PATH. Install Codex CLI or expose it via PATH." >&2
    exit 127
fi
SHELLOW_CODEX_RUNTIME="$HOME/.cache/shellow/codex-app-server"
SHELLOW_CODEX_SOCKET="$SHELLOW_CODEX_RUNTIME/app-server.sock"
SHELLOW_CODEX_PID_FILE="$SHELLOW_CODEX_RUNTIME/app-server.pid"
SHELLOW_CODEX_LOG="$SHELLOW_CODEX_RUNTIME/app-server.log"
mkdir -p "$SHELLOW_CODEX_RUNTIME" || exit $?
chmod 700 "$SHELLOW_CODEX_RUNTIME" >/dev/null 2>&1 || true
SHELLOW_CODEX_SERVER_PID=""
if [ -f "$SHELLOW_CODEX_PID_FILE" ]; then
    SHELLOW_CODEX_SERVER_PID="$(sed -n '1p' "$SHELLOW_CODEX_PID_FILE" 2>/dev/null)"
fi

shellow_codex_pid_is_server() {
    case "$SHELLOW_CODEX_SERVER_PID" in
        ''|*[!0-9]*) return 1 ;;
    esac
    kill -0 "$SHELLOW_CODEX_SERVER_PID" >/dev/null 2>&1 || return 1
    SHELLOW_CODEX_SERVER_COMMAND="$(ps -p "$SHELLOW_CODEX_SERVER_PID" -o command= 2>/dev/null)"
    case "$SHELLOW_CODEX_SERVER_COMMAND" in
        *"codex app-server"*) return 0 ;;
        *) return 1 ;;
    esac
}

shellow_codex_start_server() {
    if shellow_codex_pid_is_server; then
        kill "$SHELLOW_CODEX_SERVER_PID" >/dev/null 2>&1 || true
        SHELLOW_CODEX_STOP_WAIT=0
        while kill -0 "$SHELLOW_CODEX_SERVER_PID" >/dev/null 2>&1 && [ "$SHELLOW_CODEX_STOP_WAIT" -lt 20 ]; do
            SHELLOW_CODEX_STOP_WAIT=$((SHELLOW_CODEX_STOP_WAIT + 1))
            sleep 0.1
        done
    fi
    rm -f "$SHELLOW_CODEX_SOCKET" "$SHELLOW_CODEX_PID_FILE"
    nohup codex app-server --listen "unix://$SHELLOW_CODEX_SOCKET" >"$SHELLOW_CODEX_LOG" 2>&1 </dev/null &
    SHELLOW_CODEX_SERVER_PID=$!
    printf '%s\n' "$SHELLOW_CODEX_SERVER_PID" >"$SHELLOW_CODEX_PID_FILE"
    SHELLOW_CODEX_WAIT=0
    while [ ! -S "$SHELLOW_CODEX_SOCKET" ] && [ "$SHELLOW_CODEX_WAIT" -lt 50 ]; do
        if ! kill -0 "$SHELLOW_CODEX_SERVER_PID" >/dev/null 2>&1; then
            break
        fi
        SHELLOW_CODEX_WAIT=$((SHELLOW_CODEX_WAIT + 1))
        sleep 0.1
    done
    if [ ! -S "$SHELLOW_CODEX_SOCKET" ]; then
        echo "Unable to start the background Codex app-server." >&2
        sed -n '1,12p' "$SHELLOW_CODEX_LOG" >&2
        return 1
    fi
}

shellow_codex_python_bridge() {
    python3 -c 'import os,select,socket,sys,time
s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
for attempt in range(20):
    try:
        s.connect(sys.argv[1])
        break
    except OSError as error:
        if attempt == 19:
            print("Unable to connect to the background Codex app-server: %s" % error, file=sys.stderr)
            raise SystemExit(75)
        time.sleep(0.1)
while True:
    ready,_,_=select.select([s,0],[],[])
    if s in ready:
        data=s.recv(65536)
        if not data: break
        os.write(1,data)
    if 0 in ready:
        data=os.read(0,65536)
        if not data: break
        s.sendall(data)' "$SHELLOW_CODEX_SOCKET"
}

if ! shellow_codex_pid_is_server || [ ! -S "$SHELLOW_CODEX_SOCKET" ]; then
    shellow_codex_start_server || exit 1
fi
printf 'SHELLOW_CODEX_CWD=%s\n' "$SHELLOW_CODEX_CWD" >&2
if command -v python3 >/dev/null 2>&1; then
    shellow_codex_python_bridge
    SHELLOW_CODEX_BRIDGE_STATUS=$?
    if [ "$SHELLOW_CODEX_BRIDGE_STATUS" -eq 75 ]; then
        echo "Restarting the unresponsive background Codex app-server." >&2
        shellow_codex_start_server || exit 1
        shellow_codex_python_bridge
        exit $?
    fi
    exit "$SHELLOW_CODEX_BRIDGE_STATUS"
fi
if command -v nc >/dev/null 2>&1; then
    exec nc -U "$SHELLOW_CODEX_SOCKET"
fi
if command -v socat >/dev/null 2>&1; then
    exec socat - "UNIX-CONNECT:$SHELLOW_CODEX_SOCKET"
fi
echo "Shellow needs nc, socat, or python3 to connect to the background Codex app-server socket." >&2
exit 127"#;
const REMOTE_CWD_PREFIX: &str = "SHELLOW_CODEX_CWD=";
const APP_SERVER_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const APP_SERVER_TRANSPORT_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(8);
const APP_SERVER_WEBSOCKET_KEY: &str = "c2hlbGxvdy1jb2RleC0wMQ==";
const APP_SERVER_WEBSOCKET_MAX_FRAME_BYTES: usize = 32 * 1024 * 1024;
const COMMAND_OUTPUT_PREVIEW_MAX_LINES: usize = 10;
const COMMAND_OUTPUT_PREVIEW_MAX_CHARS: usize = 2_400;
const STATUS_MESSAGE_MAX_CHARS: usize = 2_000;
const HISTORY_ITEMS_VIEW: &str = "full";
const COMPACT_TRANSCRIPT_MAX_CHARS: usize = 8_000;

fn codex_debug(args: std::fmt::Arguments<'_>) {
    #[cfg(debug_assertions)]
    println!("[Shellow Codex] {args}");
    #[cfg(not(debug_assertions))]
    let _ = args;
}

fn json_byte_len(value: &Value) -> usize {
    serde_json::to_vec(value)
        .map(|bytes| bytes.len())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebSocketConnectionState {
    Handshaking,
    Open,
    Closed,
}

#[derive(Debug)]
struct WebSocketTransportState {
    state: WebSocketConnectionState,
    buffer: Vec<u8>,
    fragmented_text: Vec<u8>,
    fragmented_opcode: Option<u8>,
    mask_seed: u32,
}

impl WebSocketTransportState {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.subsec_nanos())
            .unwrap_or(0x5348_4c57);
        Self {
            state: WebSocketConnectionState::Handshaking,
            buffer: Vec::new(),
            fragmented_text: Vec::new(),
            fragmented_opcode: None,
            mask_seed: nanos ^ 0xa5c3_7e19,
        }
    }

    fn next_mask(&mut self) -> [u8; 4] {
        let mut value = self.mask_seed;
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        if value == 0 {
            value = 0x6d2b_79f5;
        }
        self.mask_seed = value;
        value.to_be_bytes()
    }
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
    pub messages_start_index: usize,
    pub messages_replace_all: bool,
    pub pending_approvals: Vec<CodexApproval>,
    pub directory: CodexDirectoryState,
    pub threads: CodexThreadListState,
    pub projects: CodexProjectState,
    pub thread_detail: CodexThreadDetailState,
    pub active_turn: Option<CodexActiveTurn>,
    pub operation: CodexOperationState,
    pub settings: CodexSettingsState,
    pub usage: CodexUsageState,
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
            messages_start_index: 0,
            messages_replace_all: true,
            pending_approvals: Vec::new(),
            directory: CodexDirectoryState::default(),
            threads: CodexThreadListState::default(),
            projects: CodexProjectState::default(),
            thread_detail: CodexThreadDetailState::default(),
            active_turn: None,
            operation: CodexOperationState::idle(),
            settings: CodexSettingsState::default(),
            usage: CodexUsageState::default(),
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
            messages_start_index: 0,
            messages_replace_all: true,
            pending_approvals: Vec::new(),
            directory: CodexDirectoryState::default(),
            threads: CodexThreadListState::default(),
            projects: CodexProjectState::default(),
            thread_detail: CodexThreadDetailState::default(),
            active_turn: None,
            operation: CodexOperationState::failed(message.clone()),
            settings: CodexSettingsState::default(),
            usage: CodexUsageState::default(),
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
    pub active_flags: Vec<String>,
    pub pending_approval_count: usize,
    pub last_turn_status: Option<String>,
    pub last_turn_error: Option<String>,
    pub updated_at: u64,
    pub created_at: u64,
    pub source: String,
    pub model_provider: String,
    pub forked_from_id: Option<String>,
    pub parent_thread_id: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct CodexThreadActivity {
    status: Option<String>,
    active_flags: Vec<String>,
    last_turn_status: Option<String>,
    last_turn_error: Option<String>,
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
pub struct CodexSettingOption {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexModelOption {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub reasoning_efforts: Vec<CodexSettingOption>,
    pub default_reasoning_effort: Option<String>,
    #[serde(default)]
    pub service_tiers: Vec<CodexSettingOption>,
    pub default_service_tier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexSettingsState {
    pub model: Option<String>,
    pub reasoning_effort: Option<String>,
    pub service_tier: Option<String>,
    pub approval_policy: Option<String>,
    pub sandbox: Option<String>,
    pub available_models: Vec<CodexModelOption>,
    pub is_loading_models: bool,
    pub models_error: Option<String>,
}

impl Default for CodexSettingsState {
    fn default() -> Self {
        let available_models = default_model_options();
        Self {
            model: available_models.first().map(|model| model.id.clone()),
            reasoning_effort: None,
            service_tier: None,
            approval_policy: None,
            sandbox: None,
            available_models,
            is_loading_models: false,
            models_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CodexUsageState {
    pub thread: Option<CodexThreadTokenUsage>,
    pub rate_limits: Option<CodexRateLimitSnapshot>,
    pub is_loading_rate_limits: bool,
    pub rate_limits_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexThreadTokenUsage {
    pub last: CodexTokenUsageBreakdown,
    pub total: CodexTokenUsageBreakdown,
    pub model_context_window: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CodexTokenUsageBreakdown {
    pub cached_input_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CodexRateLimitSnapshot {
    pub limit_id: Option<String>,
    pub limit_name: Option<String>,
    pub plan_type: Option<String>,
    pub primary: Option<CodexRateLimitWindow>,
    pub secondary: Option<CodexRateLimitWindow>,
    pub credits: Option<CodexCreditsSnapshot>,
    pub individual_limit: Option<CodexSpendControlLimitSnapshot>,
    pub rate_limit_reached_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexRateLimitWindow {
    pub used_percent: u32,
    pub resets_at: Option<u64>,
    pub window_duration_mins: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexCreditsSnapshot {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexSpendControlLimitSnapshot {
    pub limit: String,
    pub used: String,
    pub remaining_percent: u32,
    pub resets_at: u64,
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
    pub delivery: Option<CodexMessageDelivery>,
}

impl CodexMessage {
    pub(crate) fn user(id: impl Into<String>, text: impl Into<String>) -> Self {
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
            delivery: Some(CodexMessageDelivery::Committed),
        };
        message.refresh_blocks();
        message
    }

    fn local_user(id: impl Into<String>, text: impl Into<String>) -> Self {
        let mut message = Self::user(id, text);
        message.delivery = Some(CodexMessageDelivery::Sent);
        message
    }

    pub(crate) fn assistant(id: impl Into<String>) -> Self {
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
            delivery: None,
        }
    }

    pub(crate) fn status(id: impl Into<String>, text: impl Into<String>) -> Self {
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
            delivery: None,
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
            delivery: None,
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
            delivery: None,
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
            delivery: None,
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
            delivery: None,
        }
    }

    pub(crate) fn refresh_blocks(&mut self) {
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexMessageDelivery {
    Queued,
    Sent,
    Committed,
    Failed,
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
    #[serde(default)]
    pub questions: Vec<CodexUserInputQuestion>,
    #[serde(default)]
    pub available_decisions: Vec<String>,
    pub permissions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexUserInputQuestion {
    pub id: String,
    pub header: String,
    pub question: String,
    pub is_other: bool,
    pub is_secret: bool,
    #[serde(default)]
    pub multi_select: bool,
    pub options: Vec<CodexUserInputOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexUserInputOption {
    pub label: String,
    pub description: String,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexApprovalKind {
    Command,
    FileChange,
    UserInput,
    Permissions,
    Elicitation,
    Tool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClientRequestKind {
    Initialize,
    ModelList,
    RateLimitsRead,
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
    thread_id: Option<String>,
    approval: CodexApproval,
    params: Value,
}

#[derive(Debug)]
pub struct CodexSession {
    title: String,
    endpoint: String,
    cwd: Option<String>,
    remote_cwd: Option<String>,
    status: CodexStatus,
    initialized: bool,
    connection_started_at: Instant,
    observed_host_key_sha256: Option<String>,
    thread_id: Option<String>,
    event_thread_id: Option<String>,
    turn_active: bool,
    messages: Vec<CodexMessage>,
    last_polled_messages: Vec<CodexMessage>,
    pending_approvals: Vec<PendingServerRequest>,
    thread_activity: HashMap<String, CodexThreadActivity>,
    directory: CodexDirectoryState,
    threads: CodexThreadListState,
    projects: CodexProjectState,
    thread_detail: CodexThreadDetailState,
    active_turn: Option<CodexActiveTurn>,
    operation: CodexOperationState,
    settings: CodexSettingsState,
    usage: CodexUsageState,
    model_explicitly_selected: bool,
    last_error: Option<String>,
    line_buffer: String,
    websocket: WebSocketTransportState,
    next_request_id: u64,
    next_local_message_id: u64,
    local_revision: u64,
    request_kinds: HashMap<u64, ClientRequestKind>,
    request_thread_ids: HashMap<u64, String>,
    request_message_ids: HashMap<u64, String>,
    pending_steers: Vec<(String, String)>,
    completed_requests: HashMap<u64, Result<(), String>>,
    operation_thread_id: Option<String>,
    assistant_message_indices: HashMap<String, usize>,
    command_output_indices: HashMap<String, usize>,
    event_message_indices: HashMap<String, usize>,
    reasoning_message_indices: HashMap<String, usize>,
    #[cfg(feature = "native-integrations")]
    transport: ssh::ExecStdioHandle,
    #[cfg(feature = "native-integrations")]
    media_transport_options: ssh::RusshConnectOptions,
}

impl CodexSession {
    #[cfg(feature = "native-integrations")]
    pub fn start_password(
        profile: HostProfile,
        password: String,
        cwd: Option<String>,
        keepalive_interval_secs: u64,
    ) -> Result<Self, String> {
        Self::start(
            profile,
            ssh::RusshAuthMethod::Password(password),
            cwd,
            keepalive_interval_secs,
        )
    }

    #[cfg(feature = "native-integrations")]
    pub fn start_private_key(
        profile: HostProfile,
        private_key_pem: String,
        passphrase: Option<String>,
        cwd: Option<String>,
        keepalive_interval_secs: u64,
    ) -> Result<Self, String> {
        ssh::validate_private_key_auth(&private_key_pem, passphrase.as_deref())?;
        Self::start(
            profile,
            ssh::RusshAuthMethod::PrivateKey {
                private_key_pem,
                passphrase,
            },
            cwd,
            keepalive_interval_secs,
        )
    }

    #[cfg(feature = "native-integrations")]
    fn start(
        profile: HostProfile,
        auth: ssh::RusshAuthMethod,
        cwd: Option<String>,
        keepalive_interval_secs: u64,
    ) -> Result<Self, String> {
        let title = profile.name.clone();
        let endpoint = profile.endpoint();
        let initial_cwd = normalize_cwd(cwd);
        let app_server_command = codex_app_server_command(initial_cwd.as_deref());
        let transport_options = ssh::RusshConnectOptions {
            host: profile.host,
            port: profile.port,
            username: profile.username,
            auth,
            expected_host_key_sha256: profile.trusted_host_key_sha256,
            keepalive_interval_secs: Some(keepalive_interval_secs.clamp(10, 120)),
            keepalive_max: ssh::DEFAULT_KEEPALIVE_MAX,
            detect_remote_ports: false,
            cols: 80,
            rows: 24,
            inactivity_timeout_secs: 3_600,
        };
        let media_transport_options = transport_options.clone();
        let transport = ssh::ExecStdioHandle::spawn(transport_options, app_server_command)?;

        let session = Self {
            title,
            endpoint,
            cwd: initial_cwd.clone(),
            remote_cwd: None,
            status: CodexStatus::Connecting,
            initialized: false,
            connection_started_at: Instant::now(),
            observed_host_key_sha256: None,
            thread_id: None,
            event_thread_id: None,
            turn_active: false,
            messages: vec![
                CodexMessage::status(
                    "status-0",
                    "Connecting to the background Codex app-server over SSH.",
                ),
                CodexMessage::status("status-1", "Waiting for the Unix WebSocket handshake."),
            ],
            last_polled_messages: Vec::new(),
            pending_approvals: Vec::new(),
            thread_activity: HashMap::new(),
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
            usage: CodexUsageState::default(),
            model_explicitly_selected: false,
            last_error: None,
            line_buffer: String::new(),
            websocket: WebSocketTransportState::new(),
            next_request_id: 1,
            next_local_message_id: 2,
            local_revision: 1,
            request_kinds: HashMap::new(),
            request_thread_ids: HashMap::new(),
            request_message_ids: HashMap::new(),
            pending_steers: Vec::new(),
            completed_requests: HashMap::new(),
            operation_thread_id: None,
            assistant_message_indices: HashMap::new(),
            command_output_indices: HashMap::new(),
            event_message_indices: HashMap::new(),
            reasoning_message_indices: HashMap::new(),
            transport,
            media_transport_options,
        };

        session.start_websocket_handshake()?;
        Ok(session)
    }

    pub fn snapshot(&self) -> CodexSnapshot {
        self.snapshot_from_message(0, true)
    }

    fn incremental_snapshot(&mut self) -> CodexSnapshot {
        let replace_all = self.messages.len() < self.last_polled_messages.len();
        let start = if replace_all {
            0
        } else {
            self.messages
                .iter()
                .zip(&self.last_polled_messages)
                .position(|(current, previous)| current != previous)
                .unwrap_or_else(|| self.last_polled_messages.len().min(self.messages.len()))
        };
        let snapshot = self.snapshot_from_message(start, replace_all || start == 0);
        self.last_polled_messages = self.messages.clone();
        snapshot
    }

    fn snapshot_from_message(&self, start: usize, replace_all: bool) -> CodexSnapshot {
        let mut threads = self.threads.clone();
        for thread in &mut threads.threads {
            self.decorate_thread_summary(thread);
        }
        let mut thread_detail = self.thread_detail.clone();
        if let Some(thread) = &mut thread_detail.thread {
            self.decorate_thread_summary(thread);
        }

        CodexSnapshot {
            title: self.title.clone(),
            endpoint: self.endpoint.clone(),
            cwd: self.cwd.clone(),
            status: self.status,
            observed_host_key_sha256: self.observed_host_key_sha256.clone(),
            thread_id: self.thread_id.clone(),
            turn_active: self.turn_active,
            messages: self.messages[start..].to_vec(),
            messages_start_index: start,
            messages_replace_all: replace_all,
            pending_approvals: self
                .pending_approvals
                .iter()
                .filter(|pending| {
                    thread_scope_matches(
                        pending.thread_id.as_deref(),
                        self.event_thread_id.as_deref(),
                    )
                })
                .map(|pending| pending.approval.clone())
                .collect(),
            directory: self.directory.clone(),
            threads,
            projects: self.projects.clone(),
            thread_detail,
            active_turn: self.active_turn.clone(),
            operation: self.operation.clone(),
            settings: self.settings.clone(),
            usage: self.usage.clone(),
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
        {
            if self.websocket.state == WebSocketConnectionState::Open {
                let _ = self.send_websocket_frame(0x8, &[]);
            }
            self.transport.disconnect();
        }
        self.websocket.state = WebSocketConnectionState::Closed;
        self.status = CodexStatus::Disconnected;
        self.fail_pending_steers("Disconnected before this message could be sent.");
        self.turn_active = false;
        self.push_status("Codex disconnected; the background app-server remains running.");
    }

    pub fn poll(&mut self) -> CodexSnapshot {
        #[cfg(feature = "native-integrations")]
        {
            let poll = self.transport.poll();
            self.apply_transport_status(poll.status);
            self.consume_transport_output(&poll.output);
            if app_server_transport_handshake_timed_out(
                self.initialized,
                self.status,
                self.connection_started_at.elapsed(),
            ) {
                self.transport.disconnect();
                self.websocket.state = WebSocketConnectionState::Closed;
                self.status = CodexStatus::Failed;
                self.report_error(
                    "Background Codex app-server WebSocket handshake timed out. Check the remote app-server log, then retry.",
                );
            }
        }

        self.incremental_snapshot()
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

        self.clear_error();
        let mut params = serde_json::Map::new();
        params.insert("threadId".to_string(), json!(thread_id));
        params.insert("input".to_string(), text_input_value(text));

        let request_id = if self.turn_active {
            if let Some(active_turn) = &self.active_turn {
                params.insert("expectedTurnId".to_string(), json!(active_turn.id));
                let request_id = self.send_request(
                    "turn/steer",
                    Value::Object(params),
                    ClientRequestKind::TurnSteer,
                )?;
                self.operation = CodexOperationState::running("Steering active turn");
                request_id
            } else {
                let user_message_id = self.next_message_id("user");
                let mut message = CodexMessage::local_user(user_message_id.clone(), text);
                message.delivery = Some(CodexMessageDelivery::Queued);
                self.messages.push(message);
                self.pending_steers
                    .push((user_message_id, text.to_string()));
                self.operation =
                    CodexOperationState::running("Message queued until the active turn is ready");
                self.bump_revision();
                return Ok(self.snapshot());
            }
        } else {
            self.apply_turn_settings(&mut params);
            let request_id = self.send_request(
                "turn/start",
                Value::Object(params),
                ClientRequestKind::TurnStart,
            )?;
            self.turn_active = true;
            self.operation = CodexOperationState::running("Starting turn");
            request_id
        };
        let user_message_id = self.next_message_id("user");
        self.messages.push(CodexMessage::local_user(
            user_message_id.clone(),
            text.to_string(),
        ));
        self.request_message_ids.insert(request_id, user_message_id);
        self.bump_revision();
        Ok(self.snapshot())
    }

    pub fn update_settings(
        &mut self,
        model: Option<&str>,
        reasoning_effort: Option<&str>,
        service_tier: Option<&str>,
        approval_policy: Option<&str>,
        sandbox: Option<&str>,
    ) -> Result<CodexSnapshot, String> {
        self.clear_error();
        let requested_model = normalize_setting(model).or_else(|| self.settings.model.clone());
        let selected_model = preferred_model_id(
            requested_model.as_deref(),
            &self.settings.available_models,
            None,
        );
        self.settings = CodexSettingsState {
            model: selected_model,
            reasoning_effort: normalize_setting(reasoning_effort),
            service_tier: normalize_setting(service_tier),
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
        self.model_explicitly_selected = normalize_setting(model).is_some();
        self.operation = CodexOperationState::succeeded(
            "Defaults saved. Model, reasoning, speed, and approval apply to the next turn; sandbox applies when a thread starts or resumes.",
        );
        self.bump_revision();
        Ok(self.snapshot())
    }

    pub fn browse_directory(&mut self, path: &str) -> Result<CodexSnapshot, String> {
        self.poll();
        self.clear_error();
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
                self.directory.error = Some(error.clone());
                self.report_error_if_absent(error);
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
        self.clear_error();
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
                self.threads.error = Some(error.clone());
                self.report_error_if_absent(error);
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
        self.clear_error();
        let cwd = normalize_path_input_opt(cwd)
            .or_else(|| self.cwd.clone())
            .or_else(|| self.remote_cwd.clone());
        self.cwd = cwd.clone();
        self.thread_id = None;
        self.event_thread_id = None;
        self.usage.thread = None;
        self.turn_active = false;
        self.clear_message_indices();
        self.request_message_ids.clear();
        self.pending_steers.clear();
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
                self.report_error_if_absent(error);
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
        self.clear_error();
        let thread_id = thread_id.trim();
        if thread_id.is_empty() {
            self.push_status("Choose a Codex thread to resume.");
            return Ok(self.snapshot());
        }

        self.thread_id = None;
        self.event_thread_id = Some(thread_id.to_string());
        self.usage.thread = None;
        self.turn_active = false;
        self.clear_message_indices();
        self.request_message_ids.clear();
        self.pending_steers.clear();
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
                    "sortDirection": "desc",
                    "itemsView": HISTORY_ITEMS_VIEW
                }),
            );
            let id = self.send_request(
                "thread/resume",
                Value::Object(params),
                ClientRequestKind::ThreadResume,
            )?;
            if let Err(error) = self.wait_for_request(id) {
                self.report_error_if_absent(error);
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
        self.clear_error();
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
                self.report_error_if_absent(error);
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
        self.clear_error();
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
                self.thread_detail.error = Some(error.clone());
                self.report_error_if_absent(error);
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
        self.clear_error();
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
                self.report_error_if_absent(error);
            }
        }

        Ok(self.snapshot())
    }

    pub fn interrupt_turn(&mut self) -> Result<CodexSnapshot, String> {
        self.poll();
        self.clear_error();
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
                self.report_error_if_absent(error);
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
        let raw_decision = decision.trim();
        let decision = normalize_approval_decision(raw_decision);
        let result = match pending.approval.kind {
            CodexApprovalKind::Command => json!({ "decision": decision }),
            CodexApprovalKind::FileChange => json!({ "decision": decision }),
            CodexApprovalKind::UserInput => user_input_response(raw_decision),
            CodexApprovalKind::Permissions => {
                let granted = if matches!(decision, "decline" | "cancel") {
                    json!({})
                } else {
                    pending
                        .params
                        .get("permissions")
                        .cloned()
                        .unwrap_or_else(|| json!({}))
                };
                json!({
                    "permissions": granted,
                    "scope": if decision == "acceptForSession" { "session" } else { "turn" }
                })
            }
            CodexApprovalKind::Elicitation => mcp_elicitation_response(raw_decision),
            CodexApprovalKind::Tool => dynamic_tool_response(raw_decision),
        };
        self.send_response(pending.id, result)?;
        if let Some(thread_id) = pending.thread_id.as_deref() {
            self.sync_pending_request_flags(thread_id);
        }
        self.operation = CodexOperationState::running("Working");
        self.bump_revision();
        Ok(self.snapshot())
    }

    #[cfg(feature = "native-integrations")]
    fn start_websocket_handshake(&self) -> Result<(), String> {
        let request = format!(
            "GET / HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {APP_SERVER_WEBSOCKET_KEY}\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        self.transport.send_bytes(request.into_bytes())
    }

    #[cfg(feature = "native-integrations")]
    fn consume_transport_output(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.websocket.buffer.extend_from_slice(bytes);

        if self.websocket.state == WebSocketConnectionState::Handshaking {
            let Some(http_start) = find_subslice(&self.websocket.buffer, b"HTTP/1.1") else {
                return;
            };
            let Some(header_relative_end) =
                find_subslice(&self.websocket.buffer[http_start..], b"\r\n\r\n")
            else {
                return;
            };
            let header_end = http_start + header_relative_end + 4;
            let prelude = self.websocket.buffer[..http_start].to_vec();
            let header =
                String::from_utf8_lossy(&self.websocket.buffer[http_start..header_end]).to_string();
            self.websocket.buffer.drain(..header_end);

            if !prelude.is_empty() {
                self.consume_output(&prelude);
            }
            if !header.starts_with("HTTP/1.1 101 ") && !header.starts_with("HTTP/1.1 101\r\n") {
                self.websocket.state = WebSocketConnectionState::Closed;
                self.status = CodexStatus::Failed;
                self.report_error(format!(
                    "Codex app-server WebSocket upgrade failed: {}",
                    truncate_status_message(header.replace(['\r', '\n'], " "))
                ));
                return;
            }

            self.websocket.state = WebSocketConnectionState::Open;
            self.push_status("Connected to the background app-server socket.");
            if let Err(error) = self.bootstrap() {
                self.websocket.state = WebSocketConnectionState::Closed;
                self.status = CodexStatus::Failed;
                self.report_error(error);
                return;
            }
        }

        if self.websocket.state == WebSocketConnectionState::Open {
            self.consume_websocket_frames();
        }
    }

    #[cfg(feature = "native-integrations")]
    fn consume_websocket_frames(&mut self) {
        loop {
            let decoded = match decode_websocket_frame(&self.websocket.buffer) {
                Ok(Some(frame)) => frame,
                Ok(None) => return,
                Err(error) => {
                    self.websocket.state = WebSocketConnectionState::Closed;
                    self.status = CodexStatus::Failed;
                    self.report_error(format!("Codex WebSocket protocol error: {error}"));
                    return;
                }
            };
            self.websocket.buffer.drain(..decoded.consumed);

            match decoded.opcode {
                0x0 => {
                    if self.websocket.fragmented_opcode.is_none() {
                        self.report_error("Codex WebSocket sent an unexpected continuation frame.");
                        continue;
                    }
                    self.websocket
                        .fragmented_text
                        .extend_from_slice(&decoded.payload);
                    if decoded.fin {
                        let opcode = self.websocket.fragmented_opcode.take().unwrap_or(0x1);
                        let payload = std::mem::take(&mut self.websocket.fragmented_text);
                        if opcode == 0x1 {
                            self.deliver_websocket_text(&payload);
                        }
                    }
                }
                0x1 => {
                    if decoded.fin {
                        self.deliver_websocket_text(&decoded.payload);
                    } else {
                        self.websocket.fragmented_opcode = Some(0x1);
                        self.websocket.fragmented_text = decoded.payload;
                    }
                }
                0x2 => {
                    if !decoded.fin {
                        self.websocket.fragmented_opcode = Some(0x2);
                        self.websocket.fragmented_text = decoded.payload;
                    }
                }
                0x8 => {
                    self.websocket.state = WebSocketConnectionState::Closed;
                    if self.status != CodexStatus::Disconnected {
                        self.status = CodexStatus::Disconnected;
                        self.report_error_if_absent(
                            "The background Codex app-server closed this connection.",
                        );
                    }
                    return;
                }
                0x9 => {
                    if let Err(error) = self.send_websocket_frame(0xA, &decoded.payload) {
                        self.report_error(error);
                    }
                }
                0xA => {}
                opcode => {
                    self.report_error(format!(
                        "Codex WebSocket sent unsupported opcode {opcode:#x}."
                    ));
                }
            }
        }
    }

    fn deliver_websocket_text(&mut self, payload: &[u8]) {
        self.consume_output(payload);
        self.consume_output(b"\n");
    }

    #[cfg(feature = "native-integrations")]
    fn send_websocket_frame(&mut self, opcode: u8, payload: &[u8]) -> Result<(), String> {
        if self.websocket.state != WebSocketConnectionState::Open {
            return Err("Codex app-server WebSocket is not connected".to_string());
        }
        let mask = self.websocket.next_mask();
        self.transport
            .send_bytes(encode_websocket_client_frame(opcode, payload, mask))
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

    #[cfg(feature = "native-integrations")]
    fn request_rate_limits(&mut self) {
        self.usage.is_loading_rate_limits = true;
        self.usage.rate_limits_error = None;

        if let Err(error) = self.send_request(
            "account/rateLimits/read",
            Value::Null,
            ClientRequestKind::RateLimitsRead,
        ) {
            self.usage.is_loading_rate_limits = false;
            self.usage.rate_limits_error = Some(error);
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
            let output = truncate_status_message(line.to_string());
            if app_server_output_is_error(&output) {
                self.report_error(format!("Codex app-server: {output}"));
            } else {
                self.push_status(format!("Codex app-server output: {output}"));
            }
            return;
        };

        if let Some(level) = message.get("level").and_then(Value::as_str) {
            let log_message = message
                .pointer("/fields/message")
                .and_then(Value::as_str)
                .or_else(|| message.get("message").and_then(Value::as_str));
            if let Some(log_message) = log_message {
                if level.eq_ignore_ascii_case("error") {
                    self.report_error(format!("Codex app-server error: {log_message}"));
                } else if level.eq_ignore_ascii_case("warn")
                    && log_message.to_ascii_lowercase().contains("unknown model")
                {
                    self.push_status(format!("Codex warning: {log_message}"));
                }
            }
            return;
        }

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
        let request_thread_id = id_u64.and_then(|id| self.request_thread_ids.remove(&id));
        let known_request = kind.is_some();
        let started = Instant::now();

        if matches!(
            kind,
            Some(ClientRequestKind::TurnStart | ClientRequestKind::TurnSteer)
        ) && let (Some(thread_id), Some(error)) =
            (request_thread_id.as_deref(), message.get("error"))
        {
            self.record_thread_request_failure(thread_id, error);
        }

        if !should_apply_response_to_active_thread(
            kind,
            request_thread_id.as_deref(),
            self.event_thread_id.as_deref(),
        ) {
            let result = message
                .get("error")
                .map(|error| {
                    Err(describe_codex_error(error)
                        .unwrap_or_else(|| truncate_status_message(error.to_string())))
                })
                .unwrap_or(Ok(()));
            if let Some(id) = id_u64.filter(|_| known_request) {
                self.completed_requests.insert(id, result);
            }
            codex_debug(format_args!(
                "ignored cross-thread response id={:?} kind={:?} request_thread={:?} active_thread={:?}",
                id_u64, kind, request_thread_id, self.event_thread_id
            ));
            return;
        }

        if let Some(error) = message.get("error") {
            let description = describe_codex_error(error)
                .unwrap_or_else(|| truncate_status_message(error.to_string()));
            if let Some(id) = id_u64 {
                self.set_message_delivery_for_request(
                    id,
                    CodexMessageDelivery::Failed,
                    Some(&description),
                );
            }
            if kind == Some(ClientRequestKind::RateLimitsRead) {
                self.usage.is_loading_rate_limits = false;
                self.usage.rate_limits_error = Some(description.clone());
                if let Some(id) = id_u64.filter(|_| known_request) {
                    self.completed_requests.insert(id, Err(description));
                }
                self.bump_revision();
                return;
            }
            self.report_error(format!("Codex request failed: {description}"));
            match kind {
                Some(ClientRequestKind::Initialize) => {
                    self.status = CodexStatus::Failed;
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
                Some(ClientRequestKind::ThreadStart) | Some(ClientRequestKind::ThreadResume) => {
                    self.thread_detail.is_loading = false;
                    self.thread_detail.error = Some(description.clone());
                }
                Some(ClientRequestKind::TurnStart)
                | Some(ClientRequestKind::TurnSteer)
                | Some(ClientRequestKind::TurnInterrupt) => {
                    self.turn_active = false;
                    self.active_turn = None;
                }
                _ => {}
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
                self.clear_error();
                self.push_status("JSON-RPC initialized.");
                #[cfg(feature = "native-integrations")]
                {
                    self.request_model_list();
                    self.request_rate_limits();
                }
            }
            Some(ClientRequestKind::ModelList) => {
                self.apply_model_list_response(message);
            }
            Some(ClientRequestKind::RateLimitsRead) => {
                self.apply_rate_limits_response(message);
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
                let _ = self.remove_thread_from_visible_list();
            }
            Some(ClientRequestKind::ThreadUnarchive) => {
                self.operation = CodexOperationState::succeeded("Thread restored.");
                let _ = self.remove_thread_from_visible_list();
            }
            Some(ClientRequestKind::ThreadDelete) => {
                self.operation = CodexOperationState::succeeded("Thread deleted.");
                let deleted_thread_id = self.remove_thread_from_visible_list();
                if deleted_thread_id.as_deref() == self.thread_id.as_deref() {
                    self.thread_id = None;
                    self.event_thread_id = None;
                    self.usage.thread = None;
                    self.turn_active = false;
                    self.active_turn = None;
                }
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

        if let Some(id) = id_u64 {
            self.set_message_delivery_for_request(id, CodexMessageDelivery::Committed, None);
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
        self.record_thread_lifecycle_notification(method, params);
        if !should_apply_notification_to_thread(method, params, self.event_thread_id.as_deref()) {
            codex_debug(format_args!(
                "ignored cross-thread notification method={} incoming_thread={:?} active_thread={:?}",
                method,
                notification_thread_id(params),
                self.event_thread_id
            ));
            return;
        }

        match method {
            "thread/status/changed" => {}
            "thread/started" => {
                if let Some(thread_id) = params
                    .pointer("/thread/id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                {
                    self.thread_id = Some(thread_id.clone());
                    self.event_thread_id = Some(thread_id);
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
                        self.event_thread_id = None;
                        self.usage.thread = None;
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
                self.flush_pending_steers();
                self.bump_revision();
            }
            "turn/completed" => {
                self.fail_pending_steers("The turn ended before this message could be sent.");
                self.turn_active = false;
                self.active_turn = None;
                self.mark_streaming_messages_complete();
                let status = params
                    .pointer("/turn/status")
                    .and_then(Value::as_str)
                    .unwrap_or("completed");
                if let Some(description) = turn_completed_error_description(params) {
                    self.report_error(description);
                } else {
                    self.clear_error();
                    self.operation = if status == "interrupted" {
                        CodexOperationState::succeeded("Turn interrupted.")
                    } else {
                        CodexOperationState::idle()
                    };
                    self.bump_revision();
                }
            }
            "item/agentMessage/delta" => {
                let item_id = params
                    .get("itemId")
                    .and_then(Value::as_str)
                    .unwrap_or("assistant");
                let delta = params.get("delta").and_then(Value::as_str).unwrap_or("");
                self.append_assistant_delta(item_id, delta);
            }
            "item/plan/delta" => {
                let item_id = params
                    .get("itemId")
                    .and_then(Value::as_str)
                    .unwrap_or("plan");
                let delta = params.get("delta").and_then(Value::as_str).unwrap_or("");
                self.append_plan_delta(item_id, delta);
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
            "item/mcpToolCall/progress" => {
                let item_id = params
                    .get("itemId")
                    .and_then(Value::as_str)
                    .unwrap_or("mcp");
                let detail = params
                    .get("message")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::ToolCall,
                    "Tool working",
                    detail,
                    None,
                );
                self.bump_revision();
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
                    .get("error")
                    .and_then(describe_codex_error)
                    .or_else(|| describe_codex_error(params))
                    .unwrap_or_else(|| "Codex app-server reported an error.".to_string());
                let will_retry = params
                    .get("willRetry")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if will_retry {
                    self.last_error = None;
                    self.operation =
                        CodexOperationState::running(format!("Retrying · {description}"));
                    self.bump_revision();
                } else {
                    self.report_error(format!("Codex error: {description}"));
                }
            }
            "thread/realtime/error" => {
                let description = params
                    .get("error")
                    .and_then(describe_codex_error)
                    .or_else(|| describe_codex_error(params))
                    .unwrap_or_else(|| "Codex realtime connection failed.".to_string());
                self.report_error(format!("Codex realtime error: {description}"));
            }
            "account/updated" => {
                #[cfg(feature = "native-integrations")]
                self.request_rate_limits();
            }
            "account/rateLimits/updated" => {
                if let Some(rate_limits) = params.get("rateLimits") {
                    merge_rate_limit_snapshot(&mut self.usage.rate_limits, rate_limits);
                    self.usage.is_loading_rate_limits = false;
                    self.usage.rate_limits_error = None;
                    self.bump_revision();
                }
            }
            "thread/tokenUsage/updated" => {
                if let Some(token_usage) =
                    params.get("tokenUsage").and_then(parse_thread_token_usage)
                {
                    self.usage.thread = Some(token_usage);
                    self.bump_revision();
                }
            }
            "warning" | "guardianWarning" | "configWarning" => {
                if let Some(message) = params.get("message").and_then(Value::as_str) {
                    self.push_status(format!("Warning: {message}"));
                }
            }
            "hook/started" | "hook/completed" => {
                let run = params.get("run").unwrap_or(&Value::Null);
                let id = run.get("id").and_then(Value::as_str).unwrap_or("hook");
                let name = run
                    .get("name")
                    .or_else(|| run.get("hookName"))
                    .and_then(Value::as_str)
                    .unwrap_or("Hook");
                self.upsert_compact_event(
                    id,
                    CodexMessageKind::ToolCall,
                    if method == "hook/started" {
                        "Running hook"
                    } else {
                        "Hook completed"
                    },
                    Some(name.to_string()),
                    pretty_json(run),
                );
                self.bump_revision();
            }
            "remoteControl/status/changed"
            | "model/verification"
            | "model/safetyBuffering/updated" => {}
            other => {
                codex_debug(format_args!(
                    "unhandled notification method={} params={}",
                    other,
                    truncate_status_message(params.to_string())
                ));
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
            self.thread_id = Some(thread_id.clone());
            self.event_thread_id = Some(thread_id);
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

        let response_model = message
            .pointer("/result/model")
            .and_then(Value::as_str)
            .and_then(clean_model_text);
        let response_model_is_available = response_model.as_deref().is_some_and(|model| {
            self.settings
                .available_models
                .iter()
                .any(|option| option.id == model)
        });
        self.settings.model = preferred_model_id(
            self.settings.model.as_deref(),
            &self.settings.available_models,
            response_model
                .as_deref()
                .filter(|_| response_model_is_available),
        );
        let model_warning = response_model
            .filter(|_| !response_model_is_available)
            .zip(self.settings.model.clone())
            .map(|(unsupported, fallback)| {
                format!(
                    "Thread model {unsupported} is unavailable in this Codex app-server; using {fallback} for the next turn."
                )
            });
        self.settings.approval_policy = message
            .pointer("/result/approvalPolicy")
            .map(approval_policy_to_string)
            .or_else(|| self.settings.approval_policy.clone());

        self.status = CodexStatus::Connected;
        self.thread_detail.thread = parse_thread_summary(thread);
        self.thread_detail.is_loading = false;
        self.thread_detail.error = None;
        self.operation = CodexOperationState::succeeded(status_message);
        self.last_error = None;

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
        self.restore_active_turn(message, thread);
        if let Some(model_warning) = model_warning {
            self.push_status(model_warning);
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

    fn apply_rate_limits_response(&mut self, message: &Value) {
        self.usage.is_loading_rate_limits = false;
        let rate_limits = message.pointer("/result/rateLimits");
        match rate_limits {
            Some(value) => {
                self.usage.rate_limits = parse_rate_limit_snapshot(value);
                self.usage.rate_limits_error = None;
            }
            None => {
                self.usage.rate_limits_error =
                    Some("account/rateLimits/read returned no rate limits".to_string());
            }
        }
        self.bump_revision();
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
        if let Some(thread) = message.pointer("/result/thread")
            && let Some(summary) = parse_thread_summary(thread)
        {
            self.upsert_thread_summary(summary.clone());
            if self.thread_id.as_deref() == Some(summary.id.as_str()) {
                self.thread_detail.thread = Some(summary);
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
        let Some(turns) = page.get("data").and_then(Value::as_array) else {
            return;
        };
        let existing_messages = if append {
            std::mem::take(&mut self.messages)
        } else {
            Vec::new()
        };
        if !append {
            self.messages.clear();
        }
        self.clear_message_indices();
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

        if append {
            let mut loaded_ids = self
                .messages
                .iter()
                .map(|message| message.id.clone())
                .collect::<std::collections::HashSet<_>>();
            self.messages.extend(
                existing_messages
                    .into_iter()
                    .filter(|message| loaded_ids.insert(message.id.clone())),
            );
            self.rebuild_message_indices();
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

    fn restore_active_turn(&mut self, response: &Value, thread: &Value) {
        if let Some(active_turn) = active_turn_from_response(response, thread) {
            self.turn_active = true;
            self.active_turn = Some(active_turn);
            self.operation = CodexOperationState::running("Working");
        } else {
            self.turn_active = false;
            self.active_turn = None;
        }
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
                    message.visibility = CodexMessageVisibility::Compact;
                    message.is_streaming = false;
                    self.reasoning_message_indices
                        .insert(item_id, self.messages.len());
                    self.messages.push(message);
                }
            }
            "commandExecution" => {
                self.upsert_command_execution_message(item);
                if let Some(output) = item.get("aggregatedOutput").and_then(Value::as_str)
                    && !output.trim().is_empty()
                {
                    self.set_command_output_text(&item_id, output);
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
                    file_change_detail(item),
                    item.get("changes").and_then(pretty_json),
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
            "dynamicToolCall" => {
                let tool = item.get("tool").and_then(Value::as_str).unwrap_or("tool");
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("completed");
                self.upsert_compact_event(
                    &item_id,
                    CodexMessageKind::ToolCall,
                    humanize_camel_status("Tool", status),
                    Some(tool.to_string()),
                    pretty_json(item.get("arguments").unwrap_or(&Value::Null)),
                );
            }
            "sleep" => {
                let duration = item
                    .get("durationMs")
                    .and_then(Value::as_u64)
                    .unwrap_or_default();
                self.upsert_compact_event(
                    &item_id,
                    CodexMessageKind::Status,
                    "Waiting",
                    Some(format!("{} ms", duration)),
                    None,
                );
            }
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
                    let url = self.image_url_for_client(path);
                    self.upsert_image_event(
                        &item_id,
                        "Viewed image",
                        &url,
                        item.get("alt")
                            .or_else(|| item.get("name"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    );
                }
            }
            "imageGeneration" => {
                let url = item
                    .get("result")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .or_else(|| item.get("savedPath").and_then(Value::as_str));
                if let Some(url) = url {
                    let url = self.image_url_for_client(url);
                    self.upsert_image_event(
                        &item_id,
                        "Generated image",
                        &url,
                        item.get("revisedPrompt")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    );
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
            "collabAgentToolCall" | "subAgentActivity" => {
                self.upsert_compact_event(
                    &item_id,
                    CodexMessageKind::ToolCall,
                    collaboration_event_title(item_type, item),
                    collaboration_event_detail(item),
                    pretty_json(item),
                );
            }
            "enteredReviewMode" | "exitedReviewMode" => {
                self.upsert_compact_event(
                    &item_id,
                    CodexMessageKind::Status,
                    if item_type == "enteredReviewMode" {
                        "Entered review mode"
                    } else {
                        "Exited review mode"
                    },
                    item.get("review")
                        .and_then(Value::as_str)
                        .map(str::to_string),
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
                    file_change_detail(item),
                    item.get("changes").and_then(pretty_json),
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
            "plan" => {
                if let Some(text) = item.get("text").and_then(Value::as_str) {
                    self.append_plan_delta(item_id, text);
                }
            }
            "dynamicToolCall" => {
                let tool = item.get("tool").and_then(Value::as_str).unwrap_or("tool");
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::ToolCall,
                    "Calling tool",
                    Some(tool.to_string()),
                    pretty_json(item.get("arguments").unwrap_or(&Value::Null)),
                );
                self.bump_revision();
            }
            "sleep" => {
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::Status,
                    "Waiting",
                    item.get("durationMs")
                        .and_then(Value::as_u64)
                        .map(|value| format!("{} ms", value)),
                    None,
                );
                self.bump_revision();
            }
            "collabAgentToolCall" | "subAgentActivity" => {
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::ToolCall,
                    collaboration_event_title(item_type, item),
                    collaboration_event_detail(item),
                    pretty_json(item),
                );
                self.bump_revision();
            }
            "enteredReviewMode" | "exitedReviewMode" => {
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::Status,
                    if item_type == "enteredReviewMode" {
                        "Entered review mode"
                    } else {
                        "Exited review mode"
                    },
                    item.get("review")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    None,
                );
                self.bump_revision();
            }
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
                    if let Some(index) = self.assistant_message_indices.get(item_id).copied()
                        && let Some(message) = self.messages.get_mut(index)
                    {
                        message.kind = agent_message_kind(item);
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
                if let Some(output) = item.get("aggregatedOutput").and_then(Value::as_str)
                    && !output.trim().is_empty()
                {
                    self.set_command_output_text(item_id, output);
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
                    file_change_detail(item),
                    item.get("changes").and_then(pretty_json),
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
                    let url = self.image_url_for_client(path);
                    self.upsert_image_event(
                        item_id,
                        "Viewed image",
                        &url,
                        item.get("alt")
                            .or_else(|| item.get("name"))
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    );
                    self.bump_revision();
                }
            }
            "plan" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or("plan");
                let text = item.get("text").and_then(Value::as_str).unwrap_or("");
                if let Some(index) = self.event_message_indices.get(item_id).copied()
                    && let Some(message) = self.messages.get_mut(index)
                {
                    message.text = text.to_string();
                    message.is_streaming = false;
                    message.refresh_blocks();
                    self.bump_revision();
                } else {
                    self.append_plan_delta(item_id, text);
                    if let Some(index) = self.event_message_indices.get(item_id).copied()
                        && let Some(message) = self.messages.get_mut(index)
                    {
                        message.is_streaming = false;
                    }
                }
            }
            "imageGeneration" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or("image");
                let url = item
                    .get("result")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .or_else(|| item.get("savedPath").and_then(Value::as_str));
                if let Some(url) = url {
                    let url = self.image_url_for_client(url);
                    self.upsert_image_event(
                        item_id,
                        "Generated image",
                        &url,
                        item.get("revisedPrompt")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    );
                    self.bump_revision();
                }
            }
            "dynamicToolCall" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or("tool");
                let status = item
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("completed");
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::ToolCall,
                    humanize_camel_status("Tool", status),
                    item.get("tool").and_then(Value::as_str).map(str::to_string),
                    pretty_json(item),
                );
                self.bump_revision();
            }
            "sleep" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or("sleep");
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::Status,
                    "Wait completed",
                    item.get("durationMs")
                        .and_then(Value::as_u64)
                        .map(|value| format!("{} ms", value)),
                    None,
                );
                self.bump_revision();
            }
            "collabAgentToolCall" | "subAgentActivity" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or(item_type);
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::ToolCall,
                    collaboration_event_title(item_type, item),
                    collaboration_event_detail(item),
                    pretty_json(item),
                );
                self.bump_revision();
            }
            "enteredReviewMode" | "exitedReviewMode" => {
                let item_id = item.get("id").and_then(Value::as_str).unwrap_or(item_type);
                self.upsert_compact_event(
                    item_id,
                    CodexMessageKind::Status,
                    if item_type == "enteredReviewMode" {
                        "Entered review mode"
                    } else {
                        "Exited review mode"
                    },
                    item.get("review")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    None,
                );
                self.bump_revision();
            }
            _ => {}
        }
    }

    fn handle_server_request(&mut self, message: &Value) {
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return;
        };
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        let params = message.get("params").unwrap_or(&Value::Null);
        let request_thread_id = params
            .get("threadId")
            .and_then(Value::as_str)
            .map(str::to_string);

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
                    request_thread_id,
                    params.clone(),
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
                        questions: Vec::new(),
                        available_decisions: approval_decisions(
                            params,
                            &["accept", "acceptForSession", "decline", "cancel"],
                        ),
                        permissions: params.get("additionalPermissions").and_then(pretty_json),
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
                    request_thread_id,
                    params.clone(),
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
                        questions: Vec::new(),
                        available_decisions: vec![
                            "accept".to_string(),
                            "acceptForSession".to_string(),
                            "decline".to_string(),
                            "cancel".to_string(),
                        ],
                        permissions: None,
                    },
                );
            }
            "item/tool/requestUserInput" => {
                self.add_pending_request(
                    id,
                    request_thread_id,
                    params.clone(),
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::UserInput,
                        title: "User input".to_string(),
                        detail: params
                            .get("questions")
                            .and_then(Value::as_array)
                            .and_then(|questions| questions.first())
                            .and_then(|question| question.get("question"))
                            .and_then(Value::as_str)
                            .unwrap_or("Codex needs your input to continue.")
                            .to_string(),
                        command: None,
                        cwd: None,
                        reason: None,
                        questions: parse_user_input_questions(params),
                        available_decisions: Vec::new(),
                        permissions: None,
                    },
                );
            }
            "item/permissions/requestApproval" => {
                self.add_pending_request(
                    id,
                    request_thread_id,
                    params.clone(),
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::Permissions,
                        title: "Permissions".to_string(),
                        detail: params
                            .get("reason")
                            .and_then(Value::as_str)
                            .unwrap_or("Codex is requesting additional access.")
                            .to_string(),
                        command: None,
                        cwd: params
                            .get("cwd")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        reason: params
                            .get("reason")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                        questions: Vec::new(),
                        available_decisions: vec![
                            "accept".to_string(),
                            "acceptForSession".to_string(),
                            "decline".to_string(),
                        ],
                        permissions: params.get("permissions").and_then(pretty_json),
                    },
                );
            }
            "mcpServer/elicitation/request" => {
                let mode = params.get("mode").and_then(Value::as_str).unwrap_or("form");
                self.add_pending_request(
                    id,
                    request_thread_id,
                    params.clone(),
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::Elicitation,
                        title: "MCP server input".to_string(),
                        detail: params
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("An MCP server needs structured input to continue.")
                            .to_string(),
                        command: None,
                        cwd: None,
                        reason: Some(format!("Mode: {mode}")),
                        questions: Vec::new(),
                        available_decisions: vec![
                            "accept".to_string(),
                            "decline".to_string(),
                            "cancel".to_string(),
                        ],
                        permissions: params
                            .get("requestedSchema")
                            .or_else(|| params.get("url"))
                            .and_then(pretty_json_or_string),
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
            "item/tool/call" => {
                self.add_pending_request(
                    id,
                    request_thread_id,
                    params.clone(),
                    CodexApproval {
                        request_id: String::new(),
                        kind: CodexApprovalKind::Tool,
                        title: method.to_string(),
                        detail: params.to_string(),
                        command: None,
                        cwd: None,
                        reason: None,
                        questions: Vec::new(),
                        available_decisions: vec!["submit".to_string(), "decline".to_string()],
                        permissions: None,
                    },
                );
            }
            _ => {
                let _ = self.send_error_response(
                    id,
                    -32601,
                    format!("Shellow does not support server request {method}"),
                );
                codex_debug(format_args!(
                    "rejected unsupported server request method={method}"
                ));
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
        self.clear_error();
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
                self.report_error_if_absent(error);
            }
        }

        Ok(self.snapshot())
    }

    fn apply_thread_settings(&self, params: &mut serde_json::Map<String, Value>) {
        apply_thread_settings(&self.settings, params);
    }

    fn apply_turn_settings(&self, params: &mut serde_json::Map<String, Value>) {
        apply_turn_settings(&self.settings, params);
    }

    fn apply_model_list_response(&mut self, message: &Value) {
        let (mut models, mut default_model) = parse_model_catalog(message);
        if models.is_empty() {
            models = default_model_options();
            default_model = models.first().map(|model| model.id.clone());
        }

        let current_model = self
            .model_explicitly_selected
            .then_some(self.settings.model.as_deref())
            .flatten();
        self.settings.model = preferred_model_id(current_model, &models, default_model.as_deref());
        self.settings.available_models = models;
        self.settings.is_loading_models = false;
        self.settings.models_error = None;
        self.bump_revision();
    }

    fn remove_thread_from_visible_list(&mut self) -> Option<String> {
        let thread_id = self.operation_thread_id.take();
        if let Some(thread_id) = thread_id.as_deref() {
            self.remove_thread_by_id(thread_id);
        }
        thread_id
    }

    fn remove_thread_by_id(&mut self, thread_id: &str) {
        let before = self.threads.threads.len();
        self.threads.threads.retain(|thread| thread.id != thread_id);
        self.thread_activity.remove(thread_id);
        self.pending_approvals
            .retain(|pending| pending.thread_id.as_deref() != Some(thread_id));
        if self
            .thread_detail
            .thread
            .as_ref()
            .is_some_and(|thread| thread.id == thread_id)
        {
            self.thread_detail.thread = None;
        }
        if before != self.threads.threads.len() {
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
        if let Some(thread) = &mut self.thread_detail.thread
            && thread.id == thread_id
        {
            thread.name = next_name;
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

    fn decorate_thread_summary(&self, summary: &mut CodexThreadSummary) {
        if let Some(activity) = self.thread_activity.get(&summary.id) {
            if let Some(status) = &activity.status {
                summary.status = status.clone();
                summary.active_flags = activity.active_flags.clone();
            }
            if activity.last_turn_status.is_some() {
                summary.last_turn_status = activity.last_turn_status.clone();
                summary.last_turn_error = activity.last_turn_error.clone();
            }
        }
        summary.pending_approval_count = self
            .pending_approvals
            .iter()
            .filter(|pending| {
                pending.thread_id.as_deref() == Some(summary.id.as_str())
                    && !matches!(
                        pending.approval.kind,
                        CodexApprovalKind::UserInput | CodexApprovalKind::Elicitation
                    )
            })
            .count();
    }

    fn record_thread_lifecycle_notification(&mut self, method: &str, params: &Value) {
        let Some(thread_id) = notification_thread_id(params).map(str::to_string) else {
            return;
        };
        let activity = self.thread_activity.entry(thread_id).or_default();
        match method {
            "thread/status/changed" => {
                let (status, active_flags) = parse_thread_status(params.get("status"));
                activity.status = (!status.is_empty()).then_some(status);
                activity.active_flags = active_flags;
            }
            "turn/started" => {
                activity.last_turn_status = Some("inProgress".to_string());
                activity.last_turn_error = None;
            }
            "turn/completed" => {
                activity.last_turn_status = params
                    .pointer("/turn/status")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                activity.last_turn_error =
                    params.pointer("/turn/error").and_then(describe_codex_error);
            }
            _ => {}
        }
    }

    fn record_thread_request_failure(&mut self, thread_id: &str, error: &Value) {
        let activity = self
            .thread_activity
            .entry(thread_id.to_string())
            .or_default();
        activity.last_turn_status = Some("failed".to_string());
        activity.last_turn_error = describe_codex_error(error)
            .or_else(|| Some(truncate_status_message(error.to_string())));
    }

    fn sync_pending_request_flags(&mut self, thread_id: &str) {
        let mut waiting_on_approval = false;
        let mut waiting_on_user_input = false;
        for pending in &self.pending_approvals {
            if pending.thread_id.as_deref() != Some(thread_id) {
                continue;
            }
            if matches!(
                pending.approval.kind,
                CodexApprovalKind::UserInput | CodexApprovalKind::Elicitation
            ) {
                waiting_on_user_input = true;
            } else {
                waiting_on_approval = true;
            }
        }

        let activity = self
            .thread_activity
            .entry(thread_id.to_string())
            .or_default();
        activity
            .active_flags
            .retain(|flag| flag != "waitingOnApproval" && flag != "waitingOnUserInput");
        if waiting_on_approval {
            activity.active_flags.push("waitingOnApproval".to_string());
        }
        if waiting_on_user_input {
            activity.active_flags.push("waitingOnUserInput".to_string());
        }
        if waiting_on_approval || waiting_on_user_input {
            activity.status = Some("active".to_string());
        }
    }

    fn add_pending_request(
        &mut self,
        id: Value,
        thread_id: Option<String>,
        params: Value,
        mut approval: CodexApproval,
    ) {
        approval.request_id = request_id_to_string(&id);
        let is_active_thread =
            thread_scope_matches(thread_id.as_deref(), self.event_thread_id.as_deref());
        self.pending_approvals.push(PendingServerRequest {
            id,
            thread_id: thread_id.clone(),
            approval,
            params,
        });
        if let Some(thread_id) = thread_id.as_deref() {
            self.sync_pending_request_flags(thread_id);
        }
        if is_active_thread {
            self.operation = CodexOperationState::running("Waiting for your response");
        }
        self.bump_revision();
    }

    fn remove_pending_request_by_value(&mut self, id: &Value) {
        let affected_thread = self
            .pending_approvals
            .iter()
            .find(|pending| pending.id == *id)
            .and_then(|pending| pending.thread_id.clone());
        let before = self.pending_approvals.len();
        self.pending_approvals.retain(|pending| pending.id != *id);
        if self.pending_approvals.len() != before {
            if let Some(thread_id) = affected_thread.as_deref() {
                self.sync_pending_request_flags(thread_id);
            }
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
                    self.fail_pending_steers(
                        "Connection closed before this message could be sent.",
                    );
                    self.status = CodexStatus::Disconnected;
                    self.turn_active = false;
                    self.active_turn = None;
                    self.report_error_if_absent("Codex app-server closed unexpectedly.");
                }
            }
            ssh::ExecStdioStatus::Failed(error) => {
                if self.last_error.as_deref() != Some(error.as_str()) {
                    self.fail_pending_steers(
                        "Connection failed before this message could be sent.",
                    );
                    self.status = CodexStatus::Failed;
                    self.turn_active = false;
                    self.active_turn = None;
                    self.report_error(error);
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

    fn rebuild_message_indices(&mut self) {
        self.clear_message_indices();
        for (index, message) in self.messages.iter().enumerate() {
            match message.kind {
                CodexMessageKind::FinalAnswer | CodexMessageKind::Commentary => {
                    self.assistant_message_indices
                        .insert(message.id.clone(), index);
                }
                CodexMessageKind::ReasoningSummary => {
                    self.reasoning_message_indices
                        .insert(message.id.clone(), index);
                }
                CodexMessageKind::CommandOutput => {
                    let item_id = message
                        .id
                        .strip_prefix("command-output-")
                        .unwrap_or(&message.id);
                    self.command_output_indices
                        .insert(item_id.to_string(), index);
                }
                CodexMessageKind::Command
                | CodexMessageKind::FileChange
                | CodexMessageKind::ToolCall
                | CodexMessageKind::ToolResult
                | CodexMessageKind::Plan => {
                    self.event_message_indices.insert(message.id.clone(), index);
                    if message.kind == CodexMessageKind::Command {
                        self.command_output_indices
                            .insert(message.id.clone(), index);
                    }
                }
                CodexMessageKind::UserMessage | CodexMessageKind::Status => {}
            }
        }
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

    fn append_plan_delta(&mut self, item_id: &str, delta: &str) {
        if delta.is_empty() {
            return;
        }
        let index = if let Some(index) = self.event_message_indices.get(item_id).copied() {
            index
        } else {
            let index = self.messages.len();
            let mut message = CodexMessage::assistant(item_id.to_string());
            message.kind = CodexMessageKind::Plan;
            message.title = Some("Plan".to_string());
            self.messages.push(message);
            self.event_message_indices
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
            if let Some(summary) = summary
                && !summary.trim().is_empty()
            {
                message.transcript = Some(summary);
            }
            let transcript = message.transcript.clone().unwrap_or_default();
            if let Some(header) = extract_first_bold_text(&transcript) {
                message.text = header.clone();
                message.detail = Some(header);
            }
            if message.text == "Thinking..." {
                message.text = transcript
                    .lines()
                    .map(str::trim)
                    .find(|line| !line.is_empty())
                    .unwrap_or("Thought through the task")
                    .trim_matches('*')
                    .to_string();
            }
            message.visibility = CodexMessageVisibility::Compact;
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

    fn upsert_image_event(
        &mut self,
        item_id: &str,
        title: &str,
        url: &str,
        alt: Option<String>,
    ) -> usize {
        upsert_image_message(
            &mut self.messages,
            &mut self.event_message_indices,
            item_id,
            title,
            url,
            alt,
        )
    }

    fn image_url_for_client(&self, value: &str) -> String {
        if value.starts_with("data:")
            || value.starts_with("https://")
            || value.starts_with("http://")
            || !value.starts_with('/')
        {
            return value.to_string();
        }

        #[cfg(feature = "native-integrations")]
        {
            let quoted = shell_quote(value);
            let command = format!(
                "FILE={quoted}; SIZE=$(wc -c < \"$FILE\" 2>/dev/null) || exit 2; [ \"$SIZE\" -le 8388608 ] || exit 3; base64 < \"$FILE\" | tr -d '\\r\\n'"
            );
            if let Ok(encoded) =
                ssh::exec_password_blocking(self.media_transport_options.clone(), &command)
            {
                let encoded = encoded.trim();
                if is_base64_payload(encoded) {
                    return format!("data:{};base64,{encoded}", image_mime_type(value));
                }
            }
        }

        value.to_string()
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
        let title = command_event_title(status, exit_code, command);
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

    fn push_status(&mut self, text: impl Into<String>) {
        let id = self.next_message_id("status");
        self.messages.push(CodexMessage::status(
            id,
            truncate_status_message(text.into()),
        ));
        self.bump_revision();
    }

    fn clear_error(&mut self) {
        self.last_error = None;
        if self.operation.last_error.is_some() {
            self.operation = CodexOperationState::idle();
        }
    }

    fn report_error(&mut self, message: impl Into<String>) {
        let message = truncate_status_message(message.into());
        self.last_error = Some(message.clone());
        self.operation = CodexOperationState::failed(message.clone());
        self.bump_revision();
    }

    fn report_error_if_absent(&mut self, message: impl Into<String>) {
        if self.last_error.is_none() {
            self.report_error(message);
        }
    }

    fn set_message_delivery_for_request(
        &mut self,
        request_id: u64,
        delivery: CodexMessageDelivery,
        error: Option<&str>,
    ) {
        let Some(message_id) = self.request_message_ids.remove(&request_id) else {
            return;
        };
        if let Some(message) = self
            .messages
            .iter_mut()
            .find(|message| message.id == message_id)
        {
            message.delivery = Some(delivery);
            if let Some(error) = error {
                message.detail = Some(truncate_status_message(error.to_string()));
            }
            self.bump_revision();
        }
    }

    fn flush_pending_steers(&mut self) {
        let (Some(thread_id), Some(active_turn)) =
            (self.thread_id.clone(), self.active_turn.clone())
        else {
            return;
        };
        let pending = std::mem::take(&mut self.pending_steers);
        for (message_id, text) in pending {
            let request = self.send_request(
                "turn/steer",
                json!({
                    "threadId": thread_id.clone(),
                    "expectedTurnId": active_turn.id.clone(),
                    "input": text_input_value(&text)
                }),
                ClientRequestKind::TurnSteer,
            );
            match request {
                Ok(request_id) => {
                    self.request_message_ids
                        .insert(request_id, message_id.clone());
                    if let Some(message) = self
                        .messages
                        .iter_mut()
                        .find(|message| message.id == message_id)
                    {
                        message.delivery = Some(CodexMessageDelivery::Sent);
                    }
                }
                Err(error) => {
                    if let Some(message) = self
                        .messages
                        .iter_mut()
                        .find(|message| message.id == message_id)
                    {
                        message.delivery = Some(CodexMessageDelivery::Failed);
                        message.detail = Some(truncate_status_message(error));
                    }
                }
            }
        }
    }

    fn fail_pending_steers(&mut self, reason: &str) {
        let pending = std::mem::take(&mut self.pending_steers);
        for (message_id, _) in pending {
            if let Some(message) = self
                .messages
                .iter_mut()
                .find(|message| message.id == message_id)
            {
                message.delivery = Some(CodexMessageDelivery::Failed);
                message.detail = Some(reason.to_string());
            }
        }
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
                self.request_kinds.remove(&id);
                self.request_thread_ids.remove(&id);
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
        if let Some(thread_id) = params
            .get("threadId")
            .and_then(Value::as_str)
            .map(str::to_string)
        {
            self.request_thread_ids.insert(id, thread_id);
        }
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
    fn send_error_response(&mut self, id: Value, code: i64, message: String) -> Result<(), String> {
        self.write_json(json!({
            "id": id,
            "error": { "code": code, "message": message }
        }))
    }

    #[cfg(feature = "native-integrations")]
    fn write_json(&mut self, message: Value) -> Result<(), String> {
        let payload = serde_json::to_vec(&message)
            .map_err(|error| format!("codex json encode failed: {error}"))?;
        self.send_websocket_frame(0x1, &payload)
    }
}

fn active_turn_from_response(response: &Value, thread: &Value) -> Option<CodexActiveTurn> {
    response
        .pointer("/result/initialTurnsPage/data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .chain(
            thread
                .get("turns")
                .and_then(Value::as_array)
                .into_iter()
                .flatten(),
        )
        .rev()
        .find_map(|turn| {
            (turn.get("status").and_then(Value::as_str) == Some("inProgress"))
                .then(|| turn.get("id").and_then(Value::as_str))
                .flatten()
        })
        .map(|id| CodexActiveTurn {
            id: id.to_string(),
            status: "inProgress".to_string(),
        })
}

fn apply_thread_settings(
    settings: &CodexSettingsState,
    params: &mut serde_json::Map<String, Value>,
) {
    if let Some(model) = &settings.model {
        params.insert("model".to_string(), json!(model));
    }
    if let Some(approval_policy) = &settings.approval_policy {
        params.insert(
            "approvalPolicy".to_string(),
            approval_policy_value(approval_policy),
        );
    }
    if let Some(sandbox) = &settings.sandbox {
        params.insert("sandbox".to_string(), json!(sandbox));
    }
    params.insert("serviceTier".to_string(), json!(settings.service_tier));
}

fn apply_turn_settings(settings: &CodexSettingsState, params: &mut serde_json::Map<String, Value>) {
    if let Some(model) = &settings.model {
        params.insert("model".to_string(), json!(model));
    }
    if let Some(approval_policy) = &settings.approval_policy {
        params.insert(
            "approvalPolicy".to_string(),
            approval_policy_value(approval_policy),
        );
    }
    params.insert("effort".to_string(), json!(settings.reasoning_effort));
    params.insert("serviceTier".to_string(), json!(settings.service_tier));
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

fn truncate_chars(value: &str, max: usize) -> String {
    let mut chars = value.chars();
    let truncated = chars.by_ref().take(max).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

fn image_mime_type(path: &str) -> &'static str {
    let path = path.to_ascii_lowercase();
    if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".webp") {
        "image/webp"
    } else {
        "image/png"
    }
}

fn is_base64_payload(value: &str) -> bool {
    !value.is_empty()
        && value.len().is_multiple_of(4)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
}

fn command_event_title(status: &str, exit_code: Option<i64>, command: &str) -> String {
    let summary = command
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| truncate_chars(line, 54));
    let action = match status {
        "inProgress" | "in_progress" => "Running".to_string(),
        "completed" => match exit_code {
            Some(0) | None => "Completed".to_string(),
            Some(code) => format!("Exited {code}"),
        },
        "failed" => match exit_code {
            Some(code) => format!("Failed ({code})"),
            None => "Failed".to_string(),
        },
        "declined" => "Declined".to_string(),
        other => humanize_camel_status("Command", other),
    };
    match summary {
        Some(summary) => format!("{action} · {summary}"),
        None => action,
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

fn file_change_detail(item: &Value) -> Option<String> {
    let paths = item
        .get("changes")
        .and_then(Value::as_object)
        .map(|changes| changes.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    match paths.as_slice() {
        [] => None,
        [path] => Some(path.clone()),
        _ => Some(format!(
            "{} files · {}",
            paths.len(),
            paths.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
        )),
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

fn collaboration_event_title(item_type: &str, item: &Value) -> String {
    let status = item
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("inProgress");
    let prefix = if item_type == "subAgentActivity" {
        "Sub-agent"
    } else {
        "Agent"
    };
    humanize_camel_status(prefix, status)
}

fn collaboration_event_detail(item: &Value) -> Option<String> {
    item.get("agentPath")
        .or_else(|| item.get("tool"))
        .or_else(|| item.get("receiverThreadId"))
        .and_then(Value::as_str)
        .map(str::to_string)
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

fn describe_codex_error(value: &Value) -> Option<String> {
    if let Some(message) = value.as_str().and_then(clean_model_text) {
        return Some(truncate_status_message(message));
    }

    let object = value.as_object()?;
    let mut parts = Vec::new();
    if let Some(message) = object
        .get("message")
        .and_then(Value::as_str)
        .and_then(clean_model_text)
    {
        parts.push(message);
    }
    if let Some(details) = object
        .get("additionalDetails")
        .or_else(|| object.get("additional_details"))
        .and_then(Value::as_str)
        .and_then(clean_model_text)
        .filter(|details| !parts.iter().any(|part| part == details))
    {
        parts.push(format!("Details: {details}"));
    }
    if let Some(info) = object
        .get("codexErrorInfo")
        .or_else(|| object.get("codex_error_info"))
        .filter(|info| !info.is_null())
    {
        let info = info
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| info.to_string());
        parts.push(format!("Code: {info}"));
    } else if let Some(code) = object.get("code").filter(|code| !code.is_null()) {
        parts.push(format!("Code: {code}"));
    }
    if let Some(data) = object.get("data").filter(|data| !data.is_null()) {
        let data = data
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| data.to_string());
        if !data.trim().is_empty() {
            parts.push(format!("Data: {data}"));
        }
    }

    if parts.is_empty() {
        Some(truncate_status_message(value.to_string()))
    } else {
        Some(truncate_status_message(parts.join("\n")))
    }
}

fn turn_completed_error_description(params: &Value) -> Option<String> {
    (params.pointer("/turn/status").and_then(Value::as_str) == Some("failed")).then(|| {
        let description = params
            .pointer("/turn/error")
            .and_then(describe_codex_error)
            .unwrap_or_else(|| "Codex turn failed without error details.".to_string());
        format!("Codex turn failed: {description}")
    })
}

fn app_server_output_is_error(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "error",
        "failed",
        "fatal",
        "panic",
        "not found",
        "permission denied",
        "operation not permitted",
    ]
    .iter()
    .any(|needle| value.contains(needle))
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

#[derive(Debug, PartialEq, Eq)]
struct DecodedWebSocketFrame {
    fin: bool,
    opcode: u8,
    payload: Vec<u8>,
    consumed: usize,
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn encode_websocket_client_frame(opcode: u8, payload: &[u8], mask: [u8; 4]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len().saturating_add(14));
    frame.push(0x80 | (opcode & 0x0f));
    match payload.len() {
        length @ 0..=125 => frame.push(0x80 | length as u8),
        length @ 126..=65_535 => {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(length as u16).to_be_bytes());
        }
        length => {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(length as u64).to_be_bytes());
        }
    }
    frame.extend_from_slice(&mask);
    frame.extend(
        payload
            .iter()
            .enumerate()
            .map(|(index, byte)| byte ^ mask[index % mask.len()]),
    );
    frame
}

fn decode_websocket_frame(buffer: &[u8]) -> Result<Option<DecodedWebSocketFrame>, String> {
    if buffer.len() < 2 {
        return Ok(None);
    }
    let fin = buffer[0] & 0x80 != 0;
    if buffer[0] & 0x70 != 0 {
        return Err("RSV bits are set without an agreed extension".to_string());
    }
    let opcode = buffer[0] & 0x0f;
    let masked = buffer[1] & 0x80 != 0;
    let mut cursor = 2usize;
    let payload_len = match buffer[1] & 0x7f {
        length @ 0..=125 => length as u64,
        126 => {
            if buffer.len() < cursor + 2 {
                return Ok(None);
            }
            let length = u16::from_be_bytes([buffer[cursor], buffer[cursor + 1]]) as u64;
            cursor += 2;
            length
        }
        127 => {
            if buffer.len() < cursor + 8 {
                return Ok(None);
            }
            let length = u64::from_be_bytes(
                buffer[cursor..cursor + 8]
                    .try_into()
                    .map_err(|_| "invalid 64-bit WebSocket frame length".to_string())?,
            );
            cursor += 8;
            length
        }
        _ => unreachable!(),
    };
    let payload_len = usize::try_from(payload_len)
        .map_err(|_| "WebSocket frame length does not fit this platform".to_string())?;
    if payload_len > APP_SERVER_WEBSOCKET_MAX_FRAME_BYTES {
        return Err(format!(
            "WebSocket frame exceeds {} bytes",
            APP_SERVER_WEBSOCKET_MAX_FRAME_BYTES
        ));
    }
    let mask = if masked {
        if buffer.len() < cursor + 4 {
            return Ok(None);
        }
        let mask: [u8; 4] = buffer[cursor..cursor + 4]
            .try_into()
            .map_err(|_| "invalid WebSocket mask".to_string())?;
        cursor += 4;
        Some(mask)
    } else {
        None
    };
    let frame_end = cursor
        .checked_add(payload_len)
        .ok_or_else(|| "WebSocket frame length overflow".to_string())?;
    if buffer.len() < frame_end {
        return Ok(None);
    }
    let mut payload = buffer[cursor..frame_end].to_vec();
    if let Some(mask) = mask {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % mask.len()];
        }
    }
    Ok(Some(DecodedWebSocketFrame {
        fin,
        opcode,
        payload,
        consumed: frame_end,
    }))
}

fn normalize_cwd(cwd: Option<String>) -> Option<String> {
    cwd.map(|cwd| cwd.trim().to_string())
        .filter(|cwd| !cwd.is_empty())
}

fn app_server_transport_handshake_timed_out(
    initialized: bool,
    status: CodexStatus,
    elapsed: Duration,
) -> bool {
    !initialized
        && status == CodexStatus::Connecting
        && elapsed >= APP_SERVER_TRANSPORT_HANDSHAKE_TIMEOUT
}

fn notification_thread_id(params: &Value) -> Option<&str> {
    params
        .get("threadId")
        .and_then(Value::as_str)
        .or_else(|| params.pointer("/thread/id").and_then(Value::as_str))
}

fn notification_is_cross_thread_metadata(method: &str) -> bool {
    matches!(
        method,
        "thread/name/updated"
            | "thread/archived"
            | "thread/unarchived"
            | "thread/deleted"
            | "serverRequest/resolved"
    )
}

fn thread_scope_matches(scoped_thread_id: Option<&str>, active_thread_id: Option<&str>) -> bool {
    scoped_thread_id.is_none() || scoped_thread_id == active_thread_id
}

fn should_apply_notification_to_thread(
    method: &str,
    params: &Value,
    active_thread_id: Option<&str>,
) -> bool {
    if notification_is_cross_thread_metadata(method) {
        return true;
    }

    match notification_thread_id(params) {
        Some(notification_thread_id) => {
            thread_scope_matches(Some(notification_thread_id), active_thread_id)
        }
        None => true,
    }
}

fn response_updates_active_turn(kind: Option<ClientRequestKind>) -> bool {
    matches!(
        kind,
        Some(ClientRequestKind::TurnStart)
            | Some(ClientRequestKind::TurnSteer)
            | Some(ClientRequestKind::TurnInterrupt)
    )
}

fn should_apply_response_to_active_thread(
    kind: Option<ClientRequestKind>,
    request_thread_id: Option<&str>,
    active_thread_id: Option<&str>,
) -> bool {
    !response_updates_active_turn(kind) || thread_scope_matches(request_thread_id, active_thread_id)
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
                    if !current_runs.is_empty()
                        && let Some(kind) = current_kind
                        && matches!(
                            kind,
                            CodexMarkdownBlockKind::Paragraph
                                | CodexMarkdownBlockKind::BlockQuote
                                | CodexMarkdownBlockKind::Heading
                        )
                    {
                        push_text_block(
                            &mut blocks,
                            message_id,
                            &mut block_index,
                            kind,
                            current_level,
                            std::mem::take(&mut current_runs),
                        );
                    }
                    image_url = Some(dest_url.to_string());
                    image_alt.clear();
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    if let Some(kind) = current_kind.take()
                        && (kind == CodexMarkdownBlockKind::Paragraph
                            || kind == CodexMarkdownBlockKind::BlockQuote)
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
                    if let Some(ordered) = list_ordered.take()
                        && !list_items.is_empty()
                    {
                        blocks.push(CodexMarkdownBlock::list(
                            markdown_block_id(message_id, block_index),
                            ordered,
                            std::mem::take(&mut list_items),
                        ));
                        block_index = block_index.saturating_add(1);
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
                        if let Some(row) = current_table_row.take()
                            && !row.is_empty()
                        {
                            table_rows.push(row);
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
    if let Some(ordered) = list_ordered.take()
        && !list_items.is_empty()
    {
        blocks.push(CodexMarkdownBlock::list(
            markdown_block_id(message_id, block_index),
            ordered,
            std::mem::take(&mut list_items),
        ));
        block_index = block_index.saturating_add(1);
    }
    if in_code_block || !code_text.is_empty() {
        blocks.push(CodexMarkdownBlock::code(
            markdown_block_id(message_id, block_index),
            code_language.take(),
            std::mem::take(&mut code_text),
            true,
        ));
    }

    if markdown_has_unclosed_fence(text)
        && let Some(block) = blocks
            .iter_mut()
            .rev()
            .find(|block| block.kind == CodexMarkdownBlockKind::CodeBlock)
    {
        block.incomplete = true;
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

fn upsert_image_message(
    messages: &mut Vec<CodexMessage>,
    event_message_indices: &mut HashMap<String, usize>,
    item_id: &str,
    title: &str,
    url: &str,
    alt: Option<String>,
) -> usize {
    let existing_index = event_message_indices
        .get(item_id)
        .copied()
        .filter(|index| {
            messages
                .get(*index)
                .is_some_and(|message| message.id == item_id)
        })
        .or_else(|| messages.iter().position(|message| message.id == item_id));
    let message = CodexMessage::image_event(item_id.to_string(), title, url, alt);
    let index = if let Some(index) = existing_index {
        messages[index] = message;
        index
    } else {
        messages.push(message);
        messages.len().saturating_sub(1)
    };

    event_message_indices.insert(item_id.to_string(), index);
    index
}

fn append_markdown_run(runs: &mut Vec<CodexMarkdownInlineRun>, run: CodexMarkdownInlineRun) {
    if let Some(last) = runs.last_mut()
        && last.style == run.style
        && last.url == run.url
    {
        last.text.push_str(&run.text);
        return;
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

fn parse_thread_status(value: Option<&Value>) -> (String, Vec<String>) {
    let Some(value) = value else {
        return (String::new(), Vec::new());
    };
    if let Some(status) = value.as_str() {
        return (status.to_string(), Vec::new());
    }

    let status = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let active_flags = value
        .get("activeFlags")
        .and_then(Value::as_array)
        .map(|flags| {
            flags
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    (status, active_flags)
}

fn parse_last_turn_state(value: &Value) -> (Option<String>, Option<String>) {
    let Some(turn) = value
        .get("turns")
        .and_then(Value::as_array)
        .and_then(|turns| turns.last())
    else {
        return (None, None);
    };
    let status = turn
        .get("status")
        .and_then(Value::as_str)
        .map(str::to_string);
    let error = turn.get("error").and_then(describe_codex_error);
    (status, error)
}

fn parse_thread_summary(value: &Value) -> Option<CodexThreadSummary> {
    let (status, active_flags) = parse_thread_status(value.get("status"));
    let (last_turn_status, last_turn_error) = parse_last_turn_state(value);
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
        status,
        active_flags,
        pending_approval_count: 0,
        last_turn_status,
        last_turn_error,
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

fn parse_thread_token_usage(value: &Value) -> Option<CodexThreadTokenUsage> {
    Some(CodexThreadTokenUsage {
        last: parse_token_usage_breakdown(value.get("last")?)?,
        total: parse_token_usage_breakdown(value.get("total")?)?,
        model_context_window: value.get("modelContextWindow").and_then(Value::as_u64),
    })
}

fn parse_token_usage_breakdown(value: &Value) -> Option<CodexTokenUsageBreakdown> {
    value.as_object()?;
    Some(CodexTokenUsageBreakdown {
        cached_input_tokens: value
            .get("cachedInputTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        input_tokens: value
            .get("inputTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        output_tokens: value
            .get("outputTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        reasoning_output_tokens: value
            .get("reasoningOutputTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        total_tokens: value
            .get("totalTokens")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
    })
}

fn parse_rate_limit_snapshot(value: &Value) -> Option<CodexRateLimitSnapshot> {
    value.as_object()?;
    Some(CodexRateLimitSnapshot {
        limit_id: value
            .get("limitId")
            .and_then(Value::as_str)
            .map(str::to_string),
        limit_name: value
            .get("limitName")
            .and_then(Value::as_str)
            .map(str::to_string),
        plan_type: value
            .get("planType")
            .and_then(Value::as_str)
            .map(str::to_string),
        primary: value.get("primary").and_then(parse_rate_limit_window),
        secondary: value.get("secondary").and_then(parse_rate_limit_window),
        credits: value.get("credits").and_then(parse_credits_snapshot),
        individual_limit: value
            .get("individualLimit")
            .and_then(parse_spend_control_limit),
        rate_limit_reached_type: value
            .get("rateLimitReachedType")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn parse_rate_limit_window(value: &Value) -> Option<CodexRateLimitWindow> {
    Some(CodexRateLimitWindow {
        used_percent: value.get("usedPercent")?.as_u64()?.min(100) as u32,
        resets_at: value.get("resetsAt").and_then(Value::as_u64),
        window_duration_mins: value.get("windowDurationMins").and_then(Value::as_u64),
    })
}

fn parse_credits_snapshot(value: &Value) -> Option<CodexCreditsSnapshot> {
    Some(CodexCreditsSnapshot {
        has_credits: value.get("hasCredits")?.as_bool()?,
        unlimited: value.get("unlimited")?.as_bool()?,
        balance: value
            .get("balance")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn parse_spend_control_limit(value: &Value) -> Option<CodexSpendControlLimitSnapshot> {
    Some(CodexSpendControlLimitSnapshot {
        limit: value.get("limit")?.as_str()?.to_string(),
        used: value.get("used")?.as_str()?.to_string(),
        remaining_percent: value.get("remainingPercent")?.as_u64()?.min(100) as u32,
        resets_at: value.get("resetsAt")?.as_u64()?,
    })
}

fn merge_rate_limit_snapshot(current: &mut Option<CodexRateLimitSnapshot>, update: &Value) {
    let Some(update) = update.as_object() else {
        return;
    };
    let snapshot = current.get_or_insert_with(CodexRateLimitSnapshot::default);

    if let Some(value) = update.get("limitId").and_then(Value::as_str) {
        snapshot.limit_id = Some(value.to_string());
    }
    if let Some(value) = update.get("limitName").and_then(Value::as_str) {
        snapshot.limit_name = Some(value.to_string());
    }
    if let Some(value) = update.get("planType").and_then(Value::as_str) {
        snapshot.plan_type = Some(value.to_string());
    }
    if let Some(value) = update.get("primary").and_then(parse_rate_limit_window) {
        snapshot.primary = Some(value);
    }
    if let Some(value) = update.get("secondary").and_then(parse_rate_limit_window) {
        snapshot.secondary = Some(value);
    }
    if let Some(value) = update.get("credits").and_then(parse_credits_snapshot) {
        snapshot.credits = Some(value);
    }
    if let Some(value) = update
        .get("individualLimit")
        .and_then(parse_spend_control_limit)
    {
        snapshot.individual_limit = Some(value);
    }
    if let Some(value) = update.get("rateLimitReachedType").and_then(Value::as_str) {
        snapshot.rate_limit_reached_type = Some(value.to_string());
    }
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
    .map(|(id, name)| {
        let service_tiers = if matches!(id, "gpt-5.5" | "gpt-5.4") {
            vec![CodexSettingOption {
                id: "fast".to_string(),
                name: "Fast".to_string(),
                description: Some("Higher speed with increased credit usage.".to_string()),
            }]
        } else {
            Vec::new()
        };
        CodexModelOption {
            id: id.to_string(),
            name: name.to_string(),
            reasoning_efforts: fallback_reasoning_efforts(),
            default_reasoning_effort: None,
            service_tiers,
            default_service_tier: None,
        }
    })
    .collect()
}

fn fallback_reasoning_efforts() -> Vec<CodexSettingOption> {
    ["low", "medium", "high", "xhigh"]
        .into_iter()
        .map(|id| CodexSettingOption {
            id: id.to_string(),
            name: setting_option_name(id),
            description: None,
        })
        .collect()
}

fn parse_model_catalog(message: &Value) -> (Vec<CodexModelOption>, Option<String>) {
    let result = message.get("result").unwrap_or(&Value::Null);
    let mut models = Vec::new();
    let mut default_model = None;

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
            default_model = find_default_model_id(candidate);
            break;
        }
    }

    (models, default_model)
}

fn find_default_model_id(value: &Value) -> Option<String> {
    match value {
        Value::Array(items) => items.iter().find_map(find_default_model_id),
        Value::Object(object) => {
            if object
                .get("isDefault")
                .or_else(|| object.get("is_default"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                return parse_model_option(value, None).map(|model| model.id);
            }

            object.iter().find_map(|(key, item)| {
                if item
                    .get("isDefault")
                    .or_else(|| item.get("is_default"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    parse_model_option(item, Some(key)).map(|model| model.id)
                } else {
                    find_default_model_id(item)
                }
            })
        }
        _ => None,
    }
}

fn preferred_model_id(
    current: Option<&str>,
    models: &[CodexModelOption],
    server_default: Option<&str>,
) -> Option<String> {
    current
        .filter(|current| models.iter().any(|model| model.id == *current))
        .or_else(|| {
            server_default.filter(|default| models.iter().any(|model| model.id == *default))
        })
        .map(str::to_string)
        .or_else(|| models.first().map(|model| model.id.clone()))
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
                            reasoning_efforts: Vec::new(),
                            default_reasoning_effort: None,
                            service_tiers: Vec::new(),
                            default_service_tier: None,
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
            reasoning_efforts: Vec::new(),
            default_reasoning_effort: None,
            service_tiers: Vec::new(),
            default_service_tier: None,
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

    let reasoning_efforts = object
        .get("supportedReasoningEfforts")
        .or_else(|| object.get("supported_reasoning_efforts"))
        .map(parse_reasoning_effort_options)
        .unwrap_or_default();
    let default_reasoning_effort = object
        .get("defaultReasoningEffort")
        .or_else(|| object.get("default_reasoning_effort"))
        .and_then(Value::as_str)
        .and_then(clean_model_text);
    let service_tiers = object
        .get("serviceTiers")
        .or_else(|| object.get("service_tiers"))
        .map(parse_setting_options)
        .unwrap_or_default();
    let default_service_tier = object
        .get("defaultServiceTier")
        .or_else(|| object.get("default_service_tier"))
        .and_then(Value::as_str)
        .and_then(clean_model_text);

    Some(CodexModelOption {
        id,
        name,
        reasoning_efforts,
        default_reasoning_effort,
        service_tiers,
        default_service_tier,
    })
}

fn parse_reasoning_effort_options(value: &Value) -> Vec<CodexSettingOption> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let id = item.as_str().and_then(clean_model_text).or_else(|| {
                item.get("reasoningEffort")
                    .or_else(|| item.get("reasoning_effort"))
                    .and_then(Value::as_str)
                    .and_then(clean_model_text)
            })?;
            let description = item
                .get("description")
                .and_then(Value::as_str)
                .and_then(clean_model_text);
            Some(CodexSettingOption {
                name: setting_option_name(&id),
                id,
                description,
            })
        })
        .collect()
}

fn parse_setting_options(value: &Value) -> Vec<CodexSettingOption> {
    let Some(items) = value.as_array() else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| {
            let id = item.as_str().and_then(clean_model_text).or_else(|| {
                item.get("id")
                    .and_then(Value::as_str)
                    .and_then(clean_model_text)
            })?;
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .and_then(clean_model_text)
                .unwrap_or_else(|| setting_option_name(&id));
            let description = item
                .get("description")
                .and_then(Value::as_str)
                .and_then(clean_model_text);
            Some(CodexSettingOption {
                id,
                name,
                description,
            })
        })
        .collect()
}

fn setting_option_name(value: &str) -> String {
    match value {
        "minimal" => "Minimal".to_string(),
        "low" => "Light".to_string(),
        "medium" => "Medium".to_string(),
        "high" => "High".to_string(),
        "xhigh" => "Extra High".to_string(),
        "ultra" => "Ultra".to_string(),
        "fast" => "Fast".to_string(),
        other => other.to_string(),
    }
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

fn pretty_json(value: &Value) -> Option<String> {
    serde_json::to_string_pretty(value).ok()
}

fn pretty_json_or_string(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::to_string)
        .or_else(|| pretty_json(value))
}

fn approval_decisions(params: &Value, fallback: &[&str]) -> Vec<String> {
    params
        .get("availableDecisions")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| fallback.iter().map(|value| (*value).to_string()).collect())
}

fn parse_user_input_questions(params: &Value) -> Vec<CodexUserInputQuestion> {
    params
        .get("questions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|question| {
            Some(CodexUserInputQuestion {
                id: question.get("id")?.as_str()?.to_string(),
                header: question
                    .get("header")
                    .and_then(Value::as_str)
                    .unwrap_or("Question")
                    .to_string(),
                question: question.get("question")?.as_str()?.to_string(),
                is_other: question
                    .get("isOther")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_secret: question
                    .get("isSecret")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                multi_select: question
                    .get("multiSelect")
                    .or_else(|| question.get("multi_select"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                options: question
                    .get("options")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|option| {
                        Some(CodexUserInputOption {
                            label: option.get("label")?.as_str()?.to_string(),
                            description: option
                                .get("description")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string(),
                            preview: option
                                .get("preview")
                                .and_then(Value::as_str)
                                .map(str::to_string),
                        })
                    })
                    .collect(),
            })
        })
        .collect()
}

fn user_input_response(value: &str) -> Value {
    if matches!(value, "decline" | "cancel") || value.is_empty() {
        return json!({ "answers": {} });
    }
    let Ok(parsed) = serde_json::from_str::<Value>(value) else {
        return json!({ "answers": {} });
    };
    if parsed.get("answers").is_some() {
        return parsed;
    }
    let answers = parsed
        .as_object()
        .map(|values| {
            values
                .iter()
                .map(|(id, value)| {
                    let answers = value
                        .as_array()
                        .cloned()
                        .unwrap_or_else(|| vec![value.clone()]);
                    (id.clone(), json!({ "answers": answers }))
                })
                .collect::<serde_json::Map<_, _>>()
        })
        .unwrap_or_default();
    json!({ "answers": answers })
}

fn dynamic_tool_response(value: &str) -> Value {
    serde_json::from_str::<Value>(value)
        .ok()
        .filter(|value| value.get("success").is_some() && value.get("contentItems").is_some())
        .unwrap_or_else(|| json!({ "contentItems": [], "success": false }))
}

fn mcp_elicitation_response(value: &str) -> Value {
    if matches!(value, "decline" | "cancel") {
        return json!({ "action": value, "content": null });
    }
    match serde_json::from_str::<Value>(value) {
        Ok(parsed) if parsed.get("action").is_some() => parsed,
        Ok(content) => json!({ "action": "accept", "content": content }),
        Err(_) => json!({ "action": "accept", "content": value }),
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
    fn builds_background_app_server_command_with_remote_cwd() {
        let command = codex_app_server_command(Some("/Users/zinglix/Shellow"));
        assert!(command.starts_with("cd '/Users/zinglix/Shellow' || exit $?; "));
        assert!(command.contains("SHELLOW_CODEX_CWD=\"$(pwd -P 2>/dev/null || pwd)\""));
        assert!(
            command.contains("nohup codex app-server --listen \"unix://$SHELLOW_CODEX_SOCKET\"")
        );
        assert!(command.contains("shellow_codex_pid_is_server"));
        assert!(command.contains("shellow_codex_python_bridge"));
        assert!(command.contains("raise SystemExit(75)"));
        assert!(command.contains("Restarting the unresponsive background Codex app-server."));
        assert!(command.contains("exec nc -U \"$SHELLOW_CODEX_SOCKET\""));
        assert!(command.contains("exec socat - \"UNIX-CONNECT:$SHELLOW_CODEX_SOCKET\""));
        assert!(!command.contains("app-server daemon"));
        assert!(!command.contains("app-server proxy"));
    }

    #[test]
    fn times_out_only_a_stalled_background_transport_handshake() {
        assert!(!app_server_transport_handshake_timed_out(
            false,
            CodexStatus::Connecting,
            APP_SERVER_TRANSPORT_HANDSHAKE_TIMEOUT - Duration::from_millis(1),
        ));
        assert!(app_server_transport_handshake_timed_out(
            false,
            CodexStatus::Connecting,
            APP_SERVER_TRANSPORT_HANDSHAKE_TIMEOUT,
        ));
        assert!(!app_server_transport_handshake_timed_out(
            true,
            CodexStatus::Connected,
            APP_SERVER_TRANSPORT_HANDSHAKE_TIMEOUT,
        ));
        assert!(!app_server_transport_handshake_timed_out(
            false,
            CodexStatus::Failed,
            APP_SERVER_TRANSPORT_HANDSHAKE_TIMEOUT,
        ));
    }

    #[test]
    fn websocket_client_frames_are_masked_and_round_trip() {
        let encoded = encode_websocket_client_frame(0x1, b"{\"id\":1}", [1, 2, 3, 4]);
        assert_eq!(encoded[0], 0x81);
        assert_ne!(encoded[1] & 0x80, 0);
        let decoded = decode_websocket_frame(&encoded)
            .expect("valid frame")
            .expect("complete frame");
        assert!(decoded.fin);
        assert_eq!(decoded.opcode, 0x1);
        assert_eq!(decoded.payload, b"{\"id\":1}");
        assert_eq!(decoded.consumed, encoded.len());
    }

    #[test]
    fn websocket_decoder_waits_for_complete_extended_frame() {
        let payload = vec![b'x'; 300];
        let encoded = encode_websocket_client_frame(0x1, &payload, [9, 8, 7, 6]);
        assert!(
            decode_websocket_frame(&encoded[..encoded.len() - 1])
                .expect("partial frame is not invalid")
                .is_none()
        );
        let decoded = decode_websocket_frame(&encoded)
            .expect("valid frame")
            .expect("complete frame");
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn filters_daemon_notifications_to_the_selected_thread() {
        let thread_a_delta = json!({
            "threadId": "thread-a",
            "turnId": "turn-a",
            "itemId": "item-a",
            "delta": "A"
        });
        assert!(should_apply_notification_to_thread(
            "item/agentMessage/delta",
            &thread_a_delta,
            Some("thread-a"),
        ));
        assert!(!should_apply_notification_to_thread(
            "item/agentMessage/delta",
            &thread_a_delta,
            Some("thread-b"),
        ));
        assert!(!should_apply_notification_to_thread(
            "item/agentMessage/delta",
            &thread_a_delta,
            None,
        ));

        let thread_started = json!({ "thread": { "id": "thread-a" } });
        assert!(!should_apply_notification_to_thread(
            "thread/started",
            &thread_started,
            Some("thread-b"),
        ));

        let renamed = json!({ "threadId": "thread-a", "name": "Renamed" });
        assert!(should_apply_notification_to_thread(
            "thread/name/updated",
            &renamed,
            Some("thread-b"),
        ));

        assert!(thread_scope_matches(Some("thread-a"), Some("thread-a")));
        assert!(!thread_scope_matches(Some("thread-a"), Some("thread-b")));
        assert!(!thread_scope_matches(Some("thread-a"), None));
        assert!(thread_scope_matches(None, Some("thread-b")));

        assert!(!should_apply_response_to_active_thread(
            Some(ClientRequestKind::TurnStart),
            Some("thread-a"),
            Some("thread-b"),
        ));
        assert!(should_apply_response_to_active_thread(
            Some(ClientRequestKind::TurnStart),
            Some("thread-b"),
            Some("thread-b"),
        ));
        assert!(should_apply_response_to_active_thread(
            Some(ClientRequestKind::ThreadRename),
            Some("thread-a"),
            Some("thread-b"),
        ));
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
    fn defaults_to_a_concrete_supported_model() {
        let settings = CodexSettingsState::default();

        assert_eq!(settings.model.as_deref(), Some("gpt-5.5"));
        assert!(
            settings
                .available_models
                .iter()
                .any(|model| Some(model.id.as_str()) == settings.model.as_deref())
        );
    }

    #[test]
    fn parses_and_selects_the_app_server_default_model() {
        let response = json!({
            "result": {
                "data": [
                    {
                        "id": "gpt-old",
                        "model": "gpt-old",
                        "displayName": "Old",
                        "isDefault": false
                    },
                    {
                        "id": "gpt-current",
                        "model": "gpt-current",
                        "displayName": "Current",
                        "isDefault": true,
                        "defaultReasoningEffort": "medium",
                        "supportedReasoningEfforts": [
                            { "reasoningEffort": "low", "description": "Quick tasks" },
                            { "reasoningEffort": "medium", "description": "Balanced" },
                            { "reasoningEffort": "xhigh", "description": "Deep work" }
                        ],
                        "defaultServiceTier": null,
                        "serviceTiers": [
                            { "id": "fast", "name": "Fast", "description": "1.5x speed" }
                        ]
                    }
                ]
            }
        });

        let (models, server_default) = parse_model_catalog(&response);
        assert_eq!(models.len(), 2);
        assert_eq!(server_default.as_deref(), Some("gpt-current"));
        let current = models
            .iter()
            .find(|model| model.id == "gpt-current")
            .expect("current model");
        assert_eq!(current.default_reasoning_effort.as_deref(), Some("medium"));
        assert_eq!(current.reasoning_efforts[0].name, "Light");
        assert_eq!(current.reasoning_efforts[2].name, "Extra High");
        assert_eq!(current.service_tiers[0].id, "fast");
        assert_eq!(
            preferred_model_id(Some("gpt-removed"), &models, server_default.as_deref()).as_deref(),
            Some("gpt-current")
        );
        assert_eq!(
            preferred_model_id(Some("gpt-old"), &models, server_default.as_deref()).as_deref(),
            Some("gpt-old")
        );
    }

    #[test]
    fn applies_reasoning_and_speed_settings_to_supported_requests() {
        let mut settings = CodexSettingsState {
            reasoning_effort: Some("high".to_string()),
            service_tier: Some("fast".to_string()),
            ..Default::default()
        };

        let mut thread_params = serde_json::Map::new();
        apply_thread_settings(&settings, &mut thread_params);
        assert_eq!(thread_params.get("serviceTier"), Some(&json!("fast")));
        assert!(!thread_params.contains_key("effort"));

        let mut turn_params = serde_json::Map::new();
        apply_turn_settings(&settings, &mut turn_params);
        assert_eq!(turn_params.get("effort"), Some(&json!("high")));
        assert_eq!(turn_params.get("serviceTier"), Some(&json!("fast")));

        settings.reasoning_effort = None;
        settings.service_tier = None;
        let mut cleared_turn_params = serde_json::Map::new();
        apply_turn_settings(&settings, &mut cleared_turn_params);
        assert_eq!(cleared_turn_params.get("effort"), Some(&Value::Null));
        assert_eq!(cleared_turn_params.get("serviceTier"), Some(&Value::Null));
    }

    #[test]
    fn exposes_nested_codex_error_details() {
        let error = json!({
            "message": "The selected model is unavailable.",
            "additionalDetails": "Choose a model returned by model/list.",
            "codexErrorInfo": "badRequest"
        });

        let description = describe_codex_error(&error).expect("error description");
        assert!(description.contains("The selected model is unavailable."));
        assert!(description.contains("Choose a model returned by model/list."));
        assert!(description.contains("badRequest"));
    }

    #[test]
    fn exposes_failed_turn_completion_errors() {
        let completed = json!({
            "threadId": "thread-1",
            "turn": {
                "id": "turn-1",
                "status": "failed",
                "items": [],
                "error": {
                    "message": "Model gpt-preview is not available.",
                    "codexErrorInfo": "badRequest"
                }
            }
        });

        let description =
            turn_completed_error_description(&completed).expect("failed turn description");
        assert!(description.contains("Model gpt-preview is not available."));
        assert!(description.contains("badRequest"));

        let successful = json!({
            "turn": { "id": "turn-2", "status": "completed", "items": [] }
        });
        assert_eq!(turn_completed_error_description(&successful), None);
    }

    #[test]
    fn distinguishes_app_server_errors_from_plain_output() {
        assert!(app_server_output_is_error(
            "Error: failed to initialize Codex state"
        ));
        assert!(app_server_output_is_error("codex executable not found"));
        assert!(!app_server_output_is_error("Codex app-server ready"));
    }

    #[test]
    fn parses_thread_summary_with_optional_lineage() {
        let value = json!({
            "id": "thread-1",
            "name": "Build native Codex",
            "preview": "hello",
            "cwd": "/Users/zinglix/Shellow",
            "status": {
                "type": "active",
                "activeFlags": ["waitingOnApproval"]
            },
            "turns": [{
                "id": "turn-1",
                "status": "failed",
                "error": { "message": "Build failed" },
                "items": []
            }],
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
        assert_eq!(summary.status, "active");
        assert_eq!(summary.active_flags, vec!["waitingOnApproval"]);
        assert_eq!(summary.pending_approval_count, 0);
        assert_eq!(summary.last_turn_status.as_deref(), Some("failed"));
        assert_eq!(summary.last_turn_error.as_deref(), Some("Build failed"));
        assert_eq!(summary.forked_from_id.as_deref(), Some("thread-0"));
        assert_eq!(summary.parent_thread_id, None);
    }

    #[test]
    fn keeps_legacy_string_thread_status_compatible() {
        let (status, flags) = parse_thread_status(Some(&json!("idle")));
        assert_eq!(status, "idle");
        assert!(flags.is_empty());
    }

    #[test]
    fn restores_the_latest_in_progress_turn_from_a_descending_resume_page() {
        let response = json!({
            "result": {
                "initialTurnsPage": {
                    "data": [
                        { "id": "turn-3", "status": "inProgress", "items": [] },
                        { "id": "turn-2", "status": "completed", "items": [] }
                    ]
                }
            }
        });
        let thread = json!({
            "turns": [{ "id": "turn-1", "status": "completed", "items": [] }]
        });

        let active = active_turn_from_response(&response, &thread).expect("active turn");
        assert_eq!(active.id, "turn-3");
        assert_eq!(active.status, "inProgress");
        assert_eq!(HISTORY_ITEMS_VIEW, "full");
    }

    #[test]
    fn encodes_structured_user_input_answers_for_app_server() {
        assert_eq!(
            user_input_response(r#"{"choice":["Fast"],"notes":"Ship it"}"#),
            json!({
                "answers": {
                    "choice": { "answers": ["Fast"] },
                    "notes": { "answers": ["Ship it"] }
                }
            })
        );
        assert_eq!(user_input_response("decline"), json!({ "answers": {} }));
    }

    #[test]
    fn parses_user_input_questions_and_server_decisions() {
        let params = json!({
            "availableDecisions": ["accept", "decline"],
            "questions": [{
                "id": "scope",
                "header": "Scope",
                "question": "Which scope?",
                "isOther": true,
                "isSecret": false,
                "options": [{ "label": "Workspace", "description": "Current workspace" }]
            }]
        });
        let questions = parse_user_input_questions(&params);
        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].options[0].label, "Workspace");
        assert_eq!(
            approval_decisions(&params, &["accept"]),
            vec!["accept", "decline"]
        );
    }

    #[test]
    fn gives_completed_commands_meaningful_compact_titles() {
        assert_eq!(
            command_event_title("completed", Some(0), "cargo test -p shellow-core"),
            "Completed · cargo test -p shellow-core"
        );
        assert_eq!(image_mime_type("/tmp/result.webp"), "image/webp");
        assert!(is_base64_payload("aGVsbG8="));
        assert!(!is_base64_payload("not base64 output"));
        assert_eq!(
            mcp_elicitation_response(r#"{"project":"Shellow"}"#),
            json!({ "action": "accept", "content": { "project": "Shellow" } })
        );
        assert_eq!(
            mcp_elicitation_response("decline"),
            json!({ "action": "decline", "content": null })
        );
    }

    #[test]
    fn parses_thread_token_usage_for_context_display() {
        let usage = parse_thread_token_usage(&json!({
            "last": {
                "cachedInputTokens": 512,
                "inputTokens": 8_000,
                "outputTokens": 1_000,
                "reasoningOutputTokens": 250,
                "totalTokens": 9_000
            },
            "total": {
                "cachedInputTokens": 1_024,
                "inputTokens": 15_000,
                "outputTokens": 2_000,
                "reasoningOutputTokens": 500,
                "totalTokens": 17_000
            },
            "modelContextWindow": 128_000
        }))
        .expect("thread token usage");

        assert_eq!(usage.last.total_tokens, 9_000);
        assert_eq!(usage.total.input_tokens, 15_000);
        assert_eq!(usage.model_context_window, Some(128_000));
    }

    #[test]
    fn parses_and_merges_sparse_rate_limit_updates() {
        let mut snapshot = parse_rate_limit_snapshot(&json!({
            "limitId": "codex",
            "planType": "plus",
            "primary": {
                "usedPercent": 21,
                "resetsAt": 1_800_000_000,
                "windowDurationMins": 300
            },
            "secondary": {
                "usedPercent": 42,
                "resetsAt": 1_800_500_000,
                "windowDurationMins": 10_080
            },
            "credits": {
                "hasCredits": true,
                "unlimited": false,
                "balance": "12.50"
            }
        }));

        merge_rate_limit_snapshot(
            &mut snapshot,
            &json!({
                "limitId": "codex",
                "primary": {
                    "usedPercent": 25,
                    "resetsAt": 1_800_000_000,
                    "windowDurationMins": 300
                },
                "secondary": null,
                "planType": null
            }),
        );

        let snapshot = snapshot.expect("rate limits");
        assert_eq!(
            snapshot.primary.as_ref().map(|value| value.used_percent),
            Some(25)
        );
        assert_eq!(
            snapshot.secondary.as_ref().map(|value| value.used_percent),
            Some(42)
        );
        assert_eq!(snapshot.plan_type.as_deref(), Some("plus"));
        assert_eq!(
            snapshot.credits.and_then(|credits| credits.balance),
            Some("12.50".to_string())
        );
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
    fn upserts_replayed_image_events_by_item_id() {
        let mut messages = vec![CodexMessage::image_event(
            "item-6",
            "Viewed image",
            "file:///tmp/old.png",
            Some("Old".to_string()),
        )];
        let mut event_message_indices = HashMap::new();

        let index = upsert_image_message(
            &mut messages,
            &mut event_message_indices,
            "item-6",
            "Viewed image",
            "file:///tmp/new.png",
            Some("New".to_string()),
        );

        assert_eq!(index, 0);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "item-6");
        assert_eq!(messages[0].detail.as_deref(), Some("file:///tmp/new.png"));
        assert_eq!(messages[0].blocks[0].image_alt.as_deref(), Some("New"));
        assert_eq!(event_message_indices.get("item-6"), Some(&0));
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
