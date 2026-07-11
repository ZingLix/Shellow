use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Value, json};

use crate::{
    HostProfile,
    codex::{
        CodexActiveTurn, CodexApproval, CodexApprovalKind, CodexDirectoryState, CodexMessage,
        CodexMessageFormat, CodexMessageKind, CodexMessageRole, CodexMessageVisibility,
        CodexModelOption, CodexOperationState, CodexProjectState, CodexSettingOption,
        CodexSettingsState, CodexSnapshot, CodexStatus, CodexThreadDetailState,
        CodexThreadListState, CodexThreadSummary, CodexUsageState, CodexUserInputOption,
        CodexUserInputQuestion,
    },
    ssh,
};

const CLAUDE_REMOTE_ROOT: &str = "$HOME/.shellow/claude/v1/sessions";
const CLAUDE_STREAM_MAX_BUFFER: usize = 16 * 1024 * 1024;
const CLAUDE_PATH: &str = "$PATH:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$HOME/.local/bin:$HOME/.cargo/bin:$HOME/.bun/bin:$HOME/.npm-global/bin:/home/linuxbrew/.linuxbrew/bin";

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
struct PendingClaudeApproval {
    approval: CodexApproval,
    original_input: Value,
}

#[derive(Debug)]
struct ClaudeConversation {
    title: String,
    endpoint: String,
    cwd: Option<String>,
    status: CodexStatus,
    initialized: bool,
    observed_host_key_sha256: Option<String>,
    session_id: String,
    turn_active: bool,
    messages: Vec<CodexMessage>,
    pending_approvals: Vec<PendingClaudeApproval>,
    operation: CodexOperationState,
    settings: CodexSettingsState,
    last_error: Option<String>,
    line_buffer: String,
    local_revision: u64,
    next_message_id: u64,
    current_assistant_id: Option<String>,
    assistant_indices: HashMap<String, usize>,
    reasoning_indices: HashMap<String, usize>,
    event_indices: HashMap<String, usize>,
}

impl ClaudeConversation {
    fn new(title: String, endpoint: String, cwd: Option<String>, session_id: String) -> Self {
        Self {
            title,
            endpoint,
            cwd,
            status: CodexStatus::Connecting,
            initialized: false,
            observed_host_key_sha256: None,
            session_id,
            turn_active: false,
            messages: vec![CodexMessage::status(
                "claude-status-0",
                "Connecting to a durable Claude Code session over SSH.",
            )],
            pending_approvals: Vec::new(),
            operation: operation_running("Starting Claude Code"),
            settings: claude_settings(),
            last_error: None,
            line_buffer: String::new(),
            local_revision: 1,
            next_message_id: 1,
            current_assistant_id: None,
            assistant_indices: HashMap::new(),
            reasoning_indices: HashMap::new(),
            event_indices: HashMap::new(),
        }
    }

    fn snapshot(&self) -> CodexSnapshot {
        let cwd = self.cwd.clone().unwrap_or_default();
        let thread = CodexThreadSummary {
            id: self.session_id.clone(),
            name: Some("Claude session".to_string()),
            preview: self
                .messages
                .iter()
                .rev()
                .find(|message| message.role != CodexMessageRole::Status)
                .map(|message| message.text.clone())
                .unwrap_or_else(|| "Durable Claude Code session".to_string()),
            cwd: cwd.clone(),
            status: if self.turn_active { "active" } else { "idle" }.to_string(),
            active_flags: if self.turn_active {
                vec!["waiting".to_string()]
            } else {
                Vec::new()
            },
            pending_approval_count: self.pending_approvals.len(),
            last_turn_status: Some(
                if self.turn_active {
                    "in_progress"
                } else {
                    "completed"
                }
                .to_string(),
            ),
            last_turn_error: self.last_error.clone(),
            updated_at: 0,
            created_at: 0,
            source: "claude-code".to_string(),
            model_provider: "anthropic".to_string(),
            forked_from_id: None,
            parent_thread_id: None,
        };

        CodexSnapshot {
            title: format!("{} · Claude", self.title),
            endpoint: self.endpoint.clone(),
            cwd: self.cwd.clone(),
            status: self.status,
            observed_host_key_sha256: self.observed_host_key_sha256.clone(),
            thread_id: Some(self.session_id.clone()),
            turn_active: self.turn_active,
            messages: self.messages.clone(),
            messages_start_index: 0,
            messages_replace_all: true,
            pending_approvals: self
                .pending_approvals
                .iter()
                .map(|pending| pending.approval.clone())
                .collect(),
            directory: CodexDirectoryState::default(),
            threads: CodexThreadListState {
                cwd: self.cwd.clone(),
                threads: vec![thread.clone()],
                ..Default::default()
            },
            projects: CodexProjectState {
                current: self.cwd.clone(),
                remote_home: None,
                recent: self.cwd.iter().cloned().collect(),
                favorites: Vec::new(),
            },
            thread_detail: CodexThreadDetailState {
                thread: Some(thread),
                ..Default::default()
            },
            active_turn: self.turn_active.then(|| CodexActiveTurn {
                id: "claude-current-turn".to_string(),
                status: "in_progress".to_string(),
            }),
            operation: self.operation.clone(),
            settings: self.settings.clone(),
            usage: CodexUsageState::default(),
            last_error: self.last_error.clone(),
        }
    }

    fn consume_output(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.line_buffer.push_str(&String::from_utf8_lossy(bytes));
        if self.line_buffer.len() > CLAUDE_STREAM_MAX_BUFFER {
            self.report_error("Claude stream message exceeded the local buffer limit.");
            self.line_buffer.clear();
            return;
        }

        while let Some(newline) = self.line_buffer.find('\n') {
            let line = self.line_buffer[..newline].trim().to_string();
            self.line_buffer.drain(..=newline);
            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<Value>(&line) {
                Ok(value) => self.apply_message(&value),
                Err(_) => {
                    if line.contains("claude") || line.contains("Error") {
                        self.push_status(line);
                    }
                }
            }
        }
    }

    fn apply_message(&mut self, value: &Value) {
        match value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "system" => self.apply_system(value),
            "assistant" => self.apply_assistant(value),
            "user" => self.apply_user(value),
            "stream_event" => self.apply_stream_event(value),
            "result" => self.apply_result(value),
            "control_request" => self.apply_control_request(value),
            "control_cancel_request" => self.apply_control_cancel(value),
            "control_response" => self.apply_control_response(value),
            "shellow_error" => {
                let message = value
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Claude Code worker stopped unexpectedly.");
                let status = value.get("exit_status").and_then(Value::as_i64);
                self.report_error(match status {
                    Some(status) => format!("{message} (exit status {status})."),
                    None => message.to_string(),
                });
                self.status = CodexStatus::Failed;
                self.bump();
            }
            _ => {}
        }
    }

    fn apply_system(&mut self, value: &Value) {
        match value
            .get("subtype")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "init" => {
                self.initialized = true;
                self.status = CodexStatus::Connected;
                self.operation = operation_succeeded("Claude Code connected");
                if let Some(model) = value.get("model").and_then(Value::as_str) {
                    self.settings.model = Some(model.to_string());
                }
                self.push_status("Claude Code stream initialized.");
            }
            "api_retry" => {
                let attempt = value.get("attempt").and_then(Value::as_u64).unwrap_or(1);
                self.push_status(format!("Claude API retry {attempt}."));
            }
            "status" => {
                if let Some(status) = value.get("status").and_then(Value::as_str) {
                    self.push_status(status.to_string());
                }
            }
            _ => {}
        }
    }

    fn apply_assistant(&mut self, value: &Value) {
        let message = value.get("message").unwrap_or(value);
        let id = message
            .get("id")
            .and_then(Value::as_str)
            .or_else(|| value.get("uuid").and_then(Value::as_str))
            .map(str::to_string)
            .unwrap_or_else(|| self.next_id("assistant"));
        self.current_assistant_id = Some(id.clone());

        let Some(content) = message.get("content").and_then(Value::as_array) else {
            return;
        };
        let mut text = String::new();
        let mut thinking = String::new();
        for block in content {
            match block
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default()
            {
                "text" => {
                    if let Some(value) = block.get("text").and_then(Value::as_str) {
                        text.push_str(value);
                    }
                }
                "thinking" => {
                    if let Some(value) = block.get("thinking").and_then(Value::as_str) {
                        thinking.push_str(value);
                    }
                }
                "tool_use" => self.upsert_tool_use(block),
                _ => {}
            }
        }
        if !text.is_empty() {
            self.set_assistant_text(&id, &text, false);
        }
        if !thinking.is_empty() {
            self.set_reasoning_text(&format!("{id}-thinking"), &thinking, false);
        }
    }

    fn apply_user(&mut self, value: &Value) {
        let Some(content) = value.pointer("/message/content").and_then(Value::as_array) else {
            return;
        };
        for block in content {
            if block.get("type").and_then(Value::as_str) == Some("tool_result") {
                let id = block
                    .get("tool_use_id")
                    .and_then(Value::as_str)
                    .map(|id| format!("{id}-result"))
                    .unwrap_or_else(|| self.next_id("tool-result"));
                let text = value_to_text(block.get("content").unwrap_or(&Value::Null));
                self.upsert_event(&id, CodexMessageKind::ToolResult, "Tool result", text);
            }
        }
    }

    fn apply_stream_event(&mut self, value: &Value) {
        let Some(event) = value.get("event") else {
            return;
        };
        match event
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or_default()
        {
            "message_start" => {
                if let Some(id) = event.pointer("/message/id").and_then(Value::as_str) {
                    self.current_assistant_id = Some(id.to_string());
                }
            }
            "content_block_start" => {
                if let Some(block) = event.get("content_block")
                    && block.get("type").and_then(Value::as_str) == Some("tool_use")
                {
                    self.upsert_tool_use(block);
                }
            }
            "content_block_delta" => {
                let Some(delta) = event.get("delta") else {
                    return;
                };
                let assistant_id = self
                    .current_assistant_id
                    .clone()
                    .unwrap_or_else(|| "claude-streaming-assistant".to_string());
                match delta
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                {
                    "text_delta" => {
                        if let Some(text) = delta.get("text").and_then(Value::as_str) {
                            self.append_assistant_delta(&assistant_id, text);
                        }
                    }
                    "thinking_delta" => {
                        if let Some(text) = delta.get("thinking").and_then(Value::as_str) {
                            self.append_reasoning_delta(&format!("{assistant_id}-thinking"), text);
                        }
                    }
                    _ => {}
                }
            }
            "message_stop" => self.finish_streaming_messages(),
            _ => {}
        }
    }

    fn apply_result(&mut self, value: &Value) {
        self.turn_active = false;
        self.finish_streaming_messages();
        let is_error = value
            .get("is_error")
            .and_then(Value::as_bool)
            .unwrap_or(false)
            || value.get("subtype").and_then(Value::as_str) != Some("success");
        if is_error {
            let error = value
                .get("errors")
                .map(value_to_text)
                .filter(|text| !text.is_empty())
                .or_else(|| {
                    value
                        .get("result")
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .unwrap_or_else(|| "Claude Code turn failed.".to_string());
            self.report_error(error);
        } else {
            self.operation = operation_succeeded("Claude Code turn completed");
            self.last_error = None;
            self.bump();
        }
    }

    fn apply_control_request(&mut self, value: &Value) {
        let Some(request) = value.get("request") else {
            return;
        };
        if request.get("subtype").and_then(Value::as_str) != Some("can_use_tool") {
            return;
        }
        let request_id = id_to_string(value.get("request_id").unwrap_or(&Value::Null));
        if request_id.is_empty()
            || self
                .pending_approvals
                .iter()
                .any(|pending| pending.approval.request_id == request_id)
        {
            return;
        }
        let tool_name = request
            .get("tool_name")
            .and_then(Value::as_str)
            .unwrap_or("Tool");
        let input = request.get("input").cloned().unwrap_or_else(|| json!({}));
        let questions = if tool_name == "AskUserQuestion" {
            parse_user_questions(&input)
        } else {
            Vec::new()
        };
        let command = input
            .get("command")
            .and_then(Value::as_str)
            .map(str::to_string);
        let cwd = input
            .get("cwd")
            .or_else(|| input.get("path"))
            .or_else(|| input.get("file_path"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| self.cwd.clone());
        let kind = match tool_name {
            "Bash" => CodexApprovalKind::Command,
            "Edit" | "Write" | "NotebookEdit" => CodexApprovalKind::FileChange,
            "AskUserQuestion" => CodexApprovalKind::UserInput,
            _ => CodexApprovalKind::Tool,
        };
        let detail = match tool_name {
            "Bash" => command.clone().unwrap_or_else(|| value_to_text(&input)),
            "AskUserQuestion" => questions
                .iter()
                .map(|question| question.question.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            _ => value_to_text(&input),
        };
        self.pending_approvals.push(PendingClaudeApproval {
            approval: CodexApproval {
                request_id,
                kind,
                title: if tool_name == "AskUserQuestion" {
                    "Claude needs input".to_string()
                } else {
                    format!("Allow {tool_name}?")
                },
                detail,
                command,
                cwd,
                reason: request
                    .get("decision_reason")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                questions,
                available_decisions: vec!["accept".to_string(), "decline".to_string()],
                permissions: None,
            },
            original_input: input,
        });
        self.operation = operation_running("Waiting for approval");
        self.bump();
    }

    fn apply_control_cancel(&mut self, value: &Value) {
        let request_id = id_to_string(value.get("request_id").unwrap_or(&Value::Null));
        let before = self.pending_approvals.len();
        self.pending_approvals
            .retain(|pending| pending.approval.request_id != request_id);
        if self.pending_approvals.len() != before {
            self.bump();
        }
    }

    fn apply_control_response(&mut self, value: &Value) {
        let response = value.get("response").unwrap_or(value);
        if response.get("subtype").and_then(Value::as_str) == Some("error") {
            self.report_error(
                response
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("Claude control request failed."),
            );
            return;
        }

        if !self.initialized && response.get("subtype").and_then(Value::as_str) == Some("success") {
            let payload = response.get("response").unwrap_or(&Value::Null);
            let model = payload
                .get("models")
                .and_then(Value::as_array)
                .and_then(|models| {
                    models
                        .iter()
                        .find(|model| model.get("value").and_then(Value::as_str) == Some("default"))
                        .or_else(|| models.first())
                })
                .and_then(|model| {
                    model
                        .get("resolvedModel")
                        .or_else(|| model.get("value"))
                        .and_then(Value::as_str)
                });
            self.initialized = true;
            self.status = CodexStatus::Connected;
            self.operation = operation_succeeded("Claude Code connected");
            if let Some(model) = model {
                self.settings.model = Some(model.to_string());
            }
            self.push_status("Claude Code stream initialized.");
        }

        if let Some(pending) = response
            .get("pending_permission_requests")
            .and_then(Value::as_array)
            .cloned()
        {
            self.pending_approvals.clear();
            for request in pending {
                self.apply_control_request(&request);
            }
            self.bump();
            return;
        }

        let request_id = id_to_string(response.get("request_id").unwrap_or(&Value::Null));
        if !request_id.is_empty() {
            let before = self.pending_approvals.len();
            self.pending_approvals
                .retain(|pending| pending.approval.request_id != request_id);
            if self.pending_approvals.len() != before {
                self.bump();
            }
        }
    }

    fn set_assistant_text(&mut self, id: &str, text: &str, streaming: bool) {
        let index = self.assistant_index(id);
        if let Some(message) = self.messages.get_mut(index) {
            message.text = text.to_string();
            message.title = Some("Claude".to_string());
            message.is_streaming = streaming;
            message.refresh_blocks();
        }
        self.bump();
    }

    fn append_assistant_delta(&mut self, id: &str, delta: &str) {
        if delta.is_empty() {
            return;
        }
        let index = self.assistant_index(id);
        if let Some(message) = self.messages.get_mut(index) {
            message.text.push_str(delta);
            message.title = Some("Claude".to_string());
            message.is_streaming = true;
            message.refresh_blocks();
        }
        self.bump();
    }

    fn assistant_index(&mut self, id: &str) -> usize {
        if let Some(index) = self.assistant_indices.get(id).copied() {
            return index;
        }
        let index = self.messages.len();
        let mut message = CodexMessage::assistant(id.to_string());
        message.title = Some("Claude".to_string());
        self.messages.push(message);
        self.assistant_indices.insert(id.to_string(), index);
        index
    }

    fn set_reasoning_text(&mut self, id: &str, text: &str, streaming: bool) {
        let index = self.reasoning_index(id);
        if let Some(message) = self.messages.get_mut(index) {
            message.text = if streaming { "Thinking..." } else { "Thought" }.to_string();
            message.transcript = Some(text.to_string());
            message.is_streaming = streaming;
        }
        self.bump();
    }

    fn append_reasoning_delta(&mut self, id: &str, delta: &str) {
        let index = self.reasoning_index(id);
        if let Some(message) = self.messages.get_mut(index) {
            message
                .transcript
                .get_or_insert_with(String::new)
                .push_str(delta);
            message.is_streaming = true;
        }
        self.bump();
    }

    fn reasoning_index(&mut self, id: &str) -> usize {
        if let Some(index) = self.reasoning_indices.get(id).copied() {
            return index;
        }
        let index = self.messages.len();
        self.messages.push(CodexMessage {
            id: id.to_string(),
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
        });
        self.reasoning_indices.insert(id.to_string(), index);
        index
    }

    fn upsert_tool_use(&mut self, block: &Value) {
        let id = block
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| self.next_id("tool"));
        let name = block.get("name").and_then(Value::as_str).unwrap_or("Tool");
        let input = value_to_text(block.get("input").unwrap_or(&Value::Null));
        self.upsert_event(&id, CodexMessageKind::ToolCall, name, input);
    }

    fn upsert_event(&mut self, id: &str, kind: CodexMessageKind, title: &str, text: String) {
        if let Some(index) = self.event_indices.get(id).copied() {
            if let Some(message) = self.messages.get_mut(index) {
                message.kind = kind;
                message.title = Some(title.to_string());
                message.detail = (!text.is_empty()).then_some(text.clone());
                message.text = if text.is_empty() {
                    title.to_string()
                } else {
                    text
                };
            }
        } else {
            let index = self.messages.len();
            self.messages.push(CodexMessage {
                id: id.to_string(),
                role: CodexMessageRole::Tool,
                kind,
                visibility: CodexMessageVisibility::Compact,
                title: Some(title.to_string()),
                detail: (!text.is_empty()).then_some(text.clone()),
                text: if text.is_empty() {
                    title.to_string()
                } else {
                    text
                },
                transcript: None,
                format: CodexMessageFormat::Plain,
                blocks: Vec::new(),
                is_streaming: false,
                truncated: false,
                delivery: None,
            });
            self.event_indices.insert(id.to_string(), index);
        }
        self.bump();
    }

    fn finish_streaming_messages(&mut self) {
        for message in &mut self.messages {
            message.is_streaming = false;
        }
        self.bump();
    }

    fn push_user(&mut self, text: &str) {
        let id = self.next_id("claude-user");
        self.messages.push(CodexMessage::user(id, text.to_string()));
        self.turn_active = true;
        self.operation = operation_running("Claude Code is working");
        self.last_error = None;
        self.bump();
    }

    fn push_status(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self
            .messages
            .last()
            .is_some_and(|message| message.role == CodexMessageRole::Status && message.text == text)
        {
            return;
        }
        let id = self.next_id("claude-status");
        self.messages.push(CodexMessage::status(id, text));
        self.bump();
    }

    fn report_error(&mut self, error: impl Into<String>) {
        let error = error.into();
        self.last_error = Some(error.clone());
        self.status = if self.initialized {
            CodexStatus::Connected
        } else {
            CodexStatus::Failed
        };
        self.operation = operation_failed(error.clone());
        self.push_status(error);
    }

    fn next_id(&mut self, prefix: &str) -> String {
        let id = format!("{prefix}-{}", self.next_message_id);
        self.next_message_id = self.next_message_id.saturating_add(1);
        id
    }

    fn bump(&mut self) {
        self.local_revision = self.local_revision.saturating_add(1);
    }
}

pub struct ClaudeSession {
    conversation: ClaudeConversation,
    connect_options: ssh::RusshConnectOptions,
    remote_dir: String,
    next_input_sequence: u64,
    next_control_id: u64,
    transport: ssh::ExecStdioHandle,
}

impl ClaudeSession {
    pub fn start_password(
        profile: HostProfile,
        password: String,
        cwd: Option<String>,
        session_id: Option<String>,
    ) -> Result<Self, String> {
        Self::start(
            profile,
            ssh::RusshAuthMethod::Password(password),
            cwd,
            session_id,
        )
    }

    pub fn start_private_key(
        profile: HostProfile,
        private_key_pem: String,
        passphrase: Option<String>,
        cwd: Option<String>,
        session_id: Option<String>,
    ) -> Result<Self, String> {
        ssh::validate_private_key_auth(&private_key_pem, passphrase.as_deref())?;
        Self::start(
            profile,
            ssh::RusshAuthMethod::PrivateKey {
                private_key_pem,
                passphrase,
            },
            cwd,
            session_id,
        )
    }

    fn start(
        profile: HostProfile,
        auth: ssh::RusshAuthMethod,
        cwd: Option<String>,
        session_id: Option<String>,
    ) -> Result<Self, String> {
        let title = profile.name.clone();
        let endpoint = profile.endpoint();
        let cwd = cwd.and_then(|cwd| {
            let cwd = cwd.trim().to_string();
            (!cwd.is_empty()).then_some(cwd)
        });
        let session_id = session_id
            .filter(|id| valid_session_id(id))
            .unwrap_or_else(generate_session_id);
        let remote_dir = format!("{CLAUDE_REMOTE_ROOT}/{session_id}");
        let connect_options = ssh::RusshConnectOptions {
            host: profile.host,
            port: profile.port,
            username: profile.username,
            auth,
            expected_host_key_sha256: profile.trusted_host_key_sha256,
            keepalive_interval_secs: Some(ssh::DEFAULT_LIVE_KEEPALIVE_INTERVAL_SECS),
            keepalive_max: ssh::DEFAULT_KEEPALIVE_MAX,
            detect_remote_ports: false,
            cols: 80,
            rows: 24,
            inactivity_timeout_secs: 86_400,
        };
        let bootstrap = durable_session_bootstrap_command(&session_id, cwd.as_deref());
        ssh::exec_input_blocking(connect_options.clone(), &bootstrap, &[])?;
        let attach = durable_session_attach_command(&remote_dir);
        let transport = ssh::ExecStdioHandle::spawn(connect_options.clone(), attach)?;
        let mut session = Self {
            conversation: ClaudeConversation::new(title, endpoint, cwd, session_id),
            connect_options,
            remote_dir,
            next_input_sequence: 1,
            next_control_id: 1,
            transport,
        };
        session.enqueue_initialize()?;
        Ok(session)
    }

    pub fn snapshot(&self) -> CodexSnapshot {
        self.conversation.snapshot()
    }

    pub fn event_revision(&self) -> u64 {
        self.conversation
            .local_revision
            .saturating_add(self.transport.event_revision())
    }

    pub fn poll(&mut self) -> CodexSnapshot {
        let poll = self.transport.poll();
        match poll.status {
            ssh::ExecStdioStatus::Connecting => {
                if !self.conversation.initialized {
                    self.conversation.status = CodexStatus::Connecting;
                }
            }
            ssh::ExecStdioStatus::Connected {
                observed_host_key_sha256,
            } => {
                self.conversation.observed_host_key_sha256 = observed_host_key_sha256;
            }
            ssh::ExecStdioStatus::Closed => {
                if self.conversation.status != CodexStatus::Disconnected {
                    self.conversation.status = CodexStatus::Disconnected;
                    self.conversation.push_status(
                        "Detached from Claude Code; the remote worker may still be running.",
                    );
                }
            }
            ssh::ExecStdioStatus::Failed(error) => self.conversation.report_error(error),
        }
        self.conversation.consume_output(&poll.output);
        self.snapshot()
    }

    pub fn send_user_message(&mut self, text: &str) -> Result<CodexSnapshot, String> {
        self.poll();
        let text = text.trim();
        if text.is_empty() {
            return Ok(self.snapshot());
        }
        let message = json!({
            "type": "user",
            "message": { "role": "user", "content": text },
            "parent_tool_use_id": null,
            "session_id": self.conversation.session_id,
        });
        self.enqueue_value("user", &message)?;
        self.conversation.push_user(text);
        Ok(self.snapshot())
    }

    pub fn interrupt_turn(&mut self) -> Result<CodexSnapshot, String> {
        let request = self.control_request(json!({ "subtype": "interrupt" }));
        self.enqueue_value("interrupt", &request)?;
        self.conversation.operation = operation_running("Interrupting Claude Code");
        self.conversation.bump();
        Ok(self.snapshot())
    }

    pub fn update_settings(
        &mut self,
        model: Option<&str>,
        permission_mode: Option<&str>,
    ) -> Result<CodexSnapshot, String> {
        if let Some(model) = model.map(str::trim).filter(|value| !value.is_empty()) {
            let request = self.control_request(json!({ "subtype": "set_model", "model": model }));
            self.enqueue_value("model", &request)?;
            self.conversation.settings.model = Some(model.to_string());
        }
        if let Some(mode) = permission_mode
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let request = self.control_request(json!({
                "subtype": "set_permission_mode",
                "mode": mode,
            }));
            self.enqueue_value("permission", &request)?;
            self.conversation.settings.approval_policy = Some(mode.to_string());
        }
        self.conversation.operation = operation_succeeded("Claude settings updated");
        self.conversation.bump();
        Ok(self.snapshot())
    }

    pub fn answer_approval(
        &mut self,
        request_id: &str,
        decision: &str,
    ) -> Result<CodexSnapshot, String> {
        self.poll();
        let Some(index) = self
            .conversation
            .pending_approvals
            .iter()
            .position(|pending| pending.approval.request_id == request_id)
        else {
            return Ok(self.snapshot());
        };
        let pending = self.conversation.pending_approvals[index].clone();
        let allow = matches!(decision, "accept" | "acceptForSession" | "allow")
            || decision.trim_start().starts_with('{');
        let response = if allow {
            let updated_input = if pending.approval.questions.is_empty() {
                pending.original_input.clone()
            } else {
                answered_question_input(
                    pending.original_input.clone(),
                    &pending.approval.questions,
                    decision,
                )?
            };
            json!({
                "type": "control_response",
                "response": {
                    "subtype": "success",
                    "request_id": request_id,
                    "response": {
                        "behavior": "allow",
                        "updatedInput": updated_input,
                    }
                }
            })
        } else {
            json!({
                "type": "control_response",
                "response": {
                    "subtype": "success",
                    "request_id": request_id,
                    "response": {
                        "behavior": "deny",
                        "message": "Denied in Shellow"
                    }
                }
            })
        };
        self.enqueue_value("approval", &response)?;
        self.conversation.pending_approvals.remove(index);
        self.conversation.operation = operation_running("Claude Code is working");
        self.conversation.bump();
        Ok(self.snapshot())
    }

    pub fn disconnect(&mut self) {
        self.transport.disconnect();
        self.conversation.status = CodexStatus::Disconnected;
        self.conversation
            .push_status("Detached from Claude Code; the durable remote worker remains running.");
    }

    fn enqueue_initialize(&mut self) -> Result<(), String> {
        let request = self.control_request(json!({
            "subtype": "initialize",
            "hooks": {},
        }));
        self.enqueue_value("initialize", &request)
    }

    fn control_request(&mut self, request: Value) -> Value {
        let request_id = format!("shellow-{}", self.next_control_id);
        self.next_control_id = self.next_control_id.saturating_add(1);
        json!({
            "type": "control_request",
            "request_id": request_id,
            "request": request,
        })
    }

    fn enqueue_value(&mut self, kind: &str, value: &Value) -> Result<(), String> {
        let sequence = self.next_input_sequence;
        self.next_input_sequence = self.next_input_sequence.saturating_add(1);
        let name = format!("{sequence:020}-{kind}.json");
        let command = enqueue_command(&self.remote_dir, &name);
        let mut input = serde_json::to_vec(value)
            .map_err(|error| format!("Claude input encode failed: {error}"))?;
        input.push(b'\n');
        ssh::exec_input_blocking(self.connect_options.clone(), &command, &input)?;
        Ok(())
    }
}

fn claude_settings() -> CodexSettingsState {
    let mode = |id: &str, name: &str| CodexSettingOption {
        id: id.to_string(),
        name: name.to_string(),
        description: None,
    };
    CodexSettingsState {
        model: None,
        reasoning_effort: None,
        service_tier: None,
        approval_policy: Some("default".to_string()),
        sandbox: None,
        available_models: vec![
            CodexModelOption {
                id: "sonnet".to_string(),
                name: "Sonnet".to_string(),
                reasoning_efforts: Vec::new(),
                default_reasoning_effort: None,
                service_tiers: Vec::new(),
                default_service_tier: None,
            },
            CodexModelOption {
                id: "opus".to_string(),
                name: "Opus".to_string(),
                reasoning_efforts: Vec::new(),
                default_reasoning_effort: None,
                service_tiers: Vec::new(),
                default_service_tier: None,
            },
            CodexModelOption {
                id: "haiku".to_string(),
                name: "Haiku".to_string(),
                reasoning_efforts: vec![mode("default", "Default")],
                default_reasoning_effort: None,
                service_tiers: Vec::new(),
                default_service_tier: None,
            },
        ],
        is_loading_models: false,
        models_error: None,
    }
}

fn operation_running(label: impl Into<String>) -> CodexOperationState {
    CodexOperationState {
        is_running: true,
        label: Some(label.into()),
        last_success: None,
        last_error: None,
    }
}

fn operation_succeeded(message: impl Into<String>) -> CodexOperationState {
    CodexOperationState {
        is_running: false,
        label: None,
        last_success: Some(message.into()),
        last_error: None,
    }
}

fn operation_failed(message: impl Into<String>) -> CodexOperationState {
    CodexOperationState {
        is_running: false,
        label: None,
        last_success: None,
        last_error: Some(message.into()),
    }
}

fn durable_session_bootstrap_command(session_id: &str, cwd: Option<&str>) -> String {
    let remote_dir = format!("{CLAUDE_REMOTE_ROOT}/{session_id}");
    let cwd = cwd.unwrap_or("$HOME");
    let cwd_command = if cwd == "$HOME" {
        "cd \"$HOME\" || exit 1".to_string()
    } else {
        format!("cd {} || exit 1", shell_quote(cwd))
    };
    let run_script = format!(
        r#"#!/bin/sh
set -u
SESSION_DIR="{remote_dir}"
SESSION_ID={session_id_quoted}
PATH="{claude_path}"
export PATH
{cwd_command}
if ! command -v claude >/dev/null 2>&1; then
  echo "claude executable not found in non-interactive SSH PATH" >&2
  printf '%s\n' '{{"type":"shellow_error","message":"claude executable not found in non-interactive SSH PATH","exit_status":127}}' >> "$SESSION_DIR/stream.jsonl"
  exit 127
fi
if find "$HOME/.claude/projects" -type f -name "$SESSION_ID.jsonl" -print -quit 2>/dev/null | grep -q .; then
  set -- --resume "$SESSION_ID"
else
  set -- --session-id "$SESSION_ID"
fi
claude "$@" -p --output-format stream-json --input-format stream-json --verbose --include-partial-messages --replay-user-messages --permission-prompt-tool stdio < "$SESSION_DIR/input.fifo" >> "$SESSION_DIR/stream.jsonl" 2>> "$SESSION_DIR/stderr.log"
status=$?
printf '{{"type":"shellow_error","message":"Claude Code worker stopped","exit_status":%s}}\n' "$status" >> "$SESSION_DIR/stream.jsonl"
exit "$status"
"#,
        session_id_quoted = shell_quote(session_id),
        claude_path = CLAUDE_PATH,
    );
    let relay_script = format!(
        r#"#!/bin/sh
set -u
SESSION_DIR="{remote_dir}"
for file in "$SESSION_DIR/inbox/sending/"*.json; do
  [ -f "$file" ] || continue
  name=${{file##*/}}
  if [ -f "$SESSION_DIR/inbox/sent/$name" ]; then
    rm -f "$file"
  else
    mv "$file" "$SESSION_DIR/inbox/ready/$name"
  fi
done
exec 3> "$SESSION_DIR/input.fifo"
while :; do
  moved=0
  for file in "$SESSION_DIR/inbox/ready/"*.json; do
    [ -f "$file" ] || continue
    name=${{file##*/}}
    if mv "$file" "$SESSION_DIR/inbox/sending/$name"; then
      moved=1
      if cat "$SESSION_DIR/inbox/sending/$name" >&3; then
        case "$name" in
          *-approval.json) cat "$SESSION_DIR/inbox/sending/$name" >> "$SESSION_DIR/stream.jsonl" ;;
        esac
        mv "$SESSION_DIR/inbox/sending/$name" "$SESSION_DIR/inbox/sent/$name"
      else
        exit 1
      fi
    fi
  done
  [ "$moved" -eq 0 ] && sleep 1
done
"#,
    );

    format!(
        "umask 077; DIR={dir}; mkdir -p \"$DIR/inbox/ready\" \"$DIR/inbox/sending\" \"$DIR/inbox/sent\" || exit 1; [ -p \"$DIR/input.fifo\" ] || {{ rm -f \"$DIR/input.fifo\"; mkfifo \"$DIR/input.fifo\"; }}; printf %s {run} > \"$DIR/run.sh\"; printf %s {relay} > \"$DIR/relay.sh\"; chmod 700 \"$DIR/run.sh\" \"$DIR/relay.sh\"; touch \"$DIR/stream.jsonl\" \"$DIR/stderr.log\"; worker_alive=0; relay_alive=0; if [ -f \"$DIR/worker.pid\" ] && kill -0 \"$(sed -n '1p' \"$DIR/worker.pid\")\" 2>/dev/null; then worker_alive=1; fi; if [ -f \"$DIR/relay.pid\" ] && kill -0 \"$(sed -n '1p' \"$DIR/relay.pid\")\" 2>/dev/null; then relay_alive=1; fi; if [ \"$worker_alive\" -eq 0 ]; then nohup sh \"$DIR/run.sh\" </dev/null >/dev/null 2>&1 & echo $! > \"$DIR/worker.pid\"; fi; if [ \"$relay_alive\" -eq 0 ]; then nohup sh \"$DIR/relay.sh\" </dev/null >/dev/null 2>&1 & echo $! > \"$DIR/relay.pid\"; fi; printf 'SHELLOW_CLAUDE_SESSION={id}\\n'",
        dir = remote_home_dir_expression(&remote_dir),
        run = shell_quote(&run_script),
        relay = shell_quote(&relay_script),
        id = session_id,
    )
}

fn durable_session_attach_command(remote_dir: &str) -> String {
    format!(
        "umask 077; DIR={dir}; touch \"$DIR/stream.jsonl\" || exit 1; exec tail -n +1 -f \"$DIR/stream.jsonl\"",
        dir = remote_home_dir_expression(remote_dir),
    )
}

fn enqueue_command(remote_dir: &str, name: &str) -> String {
    format!(
        "umask 077; DIR={dir}; TMP=\"$DIR/inbox/ready/.{name}.tmp\"; FINAL=\"$DIR/inbox/ready/{name}\"; mkdir -p \"$DIR/inbox/ready\" || exit 1; if [ -f \"$FINAL\" ] || [ -f \"$DIR/inbox/sending/{name}\" ] || [ -f \"$DIR/inbox/sent/{name}\" ]; then cat >/dev/null; exit 0; fi; cat >\"$TMP\" && mv \"$TMP\" \"$FINAL\"",
        dir = remote_home_dir_expression(remote_dir),
    )
}

fn parse_user_questions(input: &Value) -> Vec<CodexUserInputQuestion> {
    input
        .get("questions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|value| {
            let question = value.get("question")?.as_str()?.trim().to_string();
            if question.is_empty() {
                return None;
            }
            let options = value
                .get("options")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|option| {
                    let label = option.get("label")?.as_str()?.trim().to_string();
                    if label.is_empty() {
                        return None;
                    }
                    Some(CodexUserInputOption {
                        label,
                        description: option
                            .get("description")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        preview: option
                            .get("preview")
                            .and_then(Value::as_str)
                            .map(str::to_string),
                    })
                })
                .collect();
            Some(CodexUserInputQuestion {
                id: value
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or(&question)
                    .to_string(),
                question,
                header: value
                    .get("header")
                    .and_then(Value::as_str)
                    .unwrap_or("Question")
                    .to_string(),
                is_other: value
                    .get("isOther")
                    .or_else(|| value.get("is_other"))
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                is_secret: value
                    .get("isSecret")
                    .or_else(|| value.get("is_secret"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                options,
                multi_select: value
                    .get("multiSelect")
                    .or_else(|| value.get("multi_select"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

fn answered_question_input(
    mut input: Value,
    questions: &[CodexUserInputQuestion],
    decision: &str,
) -> Result<Value, String> {
    let payload: Value = serde_json::from_str(decision)
        .map_err(|error| format!("Claude question answers are invalid JSON: {error}"))?;
    let submitted = payload
        .get("answers")
        .unwrap_or(&payload)
        .as_object()
        .ok_or_else(|| "Claude question answers are missing.".to_string())?;
    let mut answers = serde_json::Map::new();
    for question in questions {
        let value = submitted
            .get(&question.id)
            .or_else(|| submitted.get(&question.question));
        let answer = match value {
            Some(Value::Array(values)) => values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>()
                .join(", "),
            Some(Value::String(value)) => value.trim().to_string(),
            _ => String::new(),
        };
        if answer.is_empty() {
            return Err(format!("Answer required for: {}", question.question));
        }
        answers.insert(question.question.clone(), Value::String(answer));
    }
    let object = input
        .as_object_mut()
        .ok_or_else(|| "Claude question input is not an object.".to_string())?;
    object.insert("answers".to_string(), Value::Object(answers));
    if let Some(annotations) = payload.get("annotations") {
        object.insert("annotations".to_string(), annotations.clone());
    }
    Ok(input)
}

fn remote_home_dir_expression(remote_dir: &str) -> String {
    let relative = remote_dir.strip_prefix("$HOME/").unwrap_or(remote_dir);
    format!("\"$HOME/{}\"", relative.replace('"', "\\\""))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn generate_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed) as u128;
    let mut bytes = (nanos ^ (counter << 64)).to_be_bytes();
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}

fn valid_session_id(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}

fn id_to_string(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_string)
        .or_else(|| value.as_u64().map(|value| value.to_string()))
        .unwrap_or_default()
}

fn value_to_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .map(value_to_text)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(Value::as_str) {
                text.to_string()
            } else {
                serde_json::to_string_pretty(value).unwrap_or_default()
            }
        }
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conversation() -> ClaudeConversation {
        ClaudeConversation::new(
            "Test".to_string(),
            "test@host:22".to_string(),
            Some("/tmp/project".to_string()),
            "12345678-1234-4234-8234-123456789abc".to_string(),
        )
    }

    #[test]
    fn parses_init_stream_and_result() {
        let mut state = conversation();
        state.apply_message(&json!({
            "type": "system", "subtype": "init", "model": "sonnet",
            "session_id": state.session_id,
        }));
        state.apply_message(&json!({
            "type": "stream_event",
            "event": { "type": "message_start", "message": { "id": "msg-1" } }
        }));
        state.apply_message(&json!({
            "type": "stream_event",
            "event": { "type": "content_block_delta", "delta": { "type": "text_delta", "text": "hello" } }
        }));
        state.apply_message(&json!({ "type": "result", "subtype": "success", "is_error": false }));

        assert_eq!(state.status, CodexStatus::Connected);
        assert_eq!(state.settings.model.as_deref(), Some("sonnet"));
        assert!(state.messages.iter().any(|message| message.text == "hello"));
        assert!(!state.turn_active);
    }

    #[test]
    fn parses_control_response_initialize() {
        let mut state = conversation();
        state.apply_message(&json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": "shellow-1",
                "response": {
                    "models": [{
                        "value": "default",
                        "resolvedModel": "claude-opus-test"
                    }]
                }
            }
        }));

        assert!(state.initialized);
        assert_eq!(state.status, CodexStatus::Connected);
        assert_eq!(state.settings.model.as_deref(), Some("claude-opus-test"));
    }

    #[test]
    fn parses_and_cancels_permission_request() {
        let mut state = conversation();
        state.apply_message(&json!({
            "type": "control_request",
            "request_id": "permission-1",
            "request": {
                "subtype": "can_use_tool",
                "tool_name": "Bash",
                "input": { "command": "cargo test" }
            }
        }));
        assert_eq!(state.pending_approvals.len(), 1);
        assert_eq!(
            state.pending_approvals[0].approval.command.as_deref(),
            Some("cargo test")
        );
        state.apply_message(
            &json!({ "type": "control_cancel_request", "request_id": "permission-1" }),
        );
        assert!(state.pending_approvals.is_empty());

        state.apply_message(&json!({
            "type": "control_request",
            "request_id": "permission-2",
            "request": {
                "subtype": "can_use_tool",
                "tool_name": "Edit",
                "input": { "file_path": "/tmp/example" }
            }
        }));
        state.apply_message(&json!({
            "type": "control_response",
            "response": { "subtype": "success", "request_id": "permission-2" }
        }));
        assert!(state.pending_approvals.is_empty());

        state.apply_message(&json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": "shellow-initialize",
                "pending_permission_requests": [{
                    "type": "control_request",
                    "request_id": "permission-3",
                    "request": {
                        "subtype": "can_use_tool",
                        "tool_name": "Write",
                        "input": { "file_path": "/tmp/pending" }
                    }
                }]
            }
        }));
        assert_eq!(state.pending_approvals.len(), 1);
        assert_eq!(
            state.pending_approvals[0].approval.request_id,
            "permission-3"
        );
    }

    #[test]
    fn durable_command_uses_only_shell_and_claude() {
        let command = durable_session_bootstrap_command(
            "12345678-1234-4234-8234-123456789abc",
            Some("/tmp/project with spaces"),
        );
        assert!(command.contains("mkfifo"));
        assert!(command.contains("--input-format stream-json"));
        assert!(command.contains("--output-format stream-json"));
        assert!(command.contains("--permission-prompt-tool stdio"));
        assert!(!command.contains("--disallowedTools AskUserQuestion"));
        assert!(command.contains("inbox/sending/\"*.json"));
        assert!(command.contains("nohup sh"));
        assert!(command.contains("DIR=\"$HOME/.shellow/claude/v1/sessions/"));
        assert!(!command.contains("DIR='$HOME/"));
        assert!(!command.contains("tmux"));
        assert!(!command.contains("python"));
        assert!(!command.contains("node "));
        assert!(!command.contains("tail -n +1 -f"));

        let attach = durable_session_attach_command(
            "$HOME/.shellow/claude/v1/sessions/12345678-1234-4234-8234-123456789abc",
        );
        assert!(attach.contains("exec tail -n +1 -f"));
        assert!(!attach.contains("nohup"));
    }

    #[test]
    fn generated_session_ids_are_valid_and_unique() {
        let first = generate_session_id();
        let second = generate_session_id();
        assert!(valid_session_id(&first));
        assert!(valid_session_id(&second));
        assert_ne!(first, second);
    }

    #[test]
    fn reports_durable_worker_failure() {
        let mut state = conversation();
        state.apply_message(&json!({
            "type": "shellow_error",
            "message": "Claude Code worker stopped",
            "exit_status": 127
        }));
        assert_eq!(state.status, CodexStatus::Failed);
        assert_eq!(
            state.last_error.as_deref(),
            Some("Claude Code worker stopped (exit status 127).")
        );
    }

    #[test]
    fn parses_and_answers_ask_user_question() {
        let input = json!({
            "questions": [{
                "question": "Which targets?",
                "header": "Targets",
                "options": [
                    { "label": "iOS", "description": "Apple", "preview": "swift" },
                    { "label": "Android", "description": "Google" }
                ],
                "multiSelect": true
            }]
        });
        let questions = parse_user_questions(&input);
        assert_eq!(questions.len(), 1);
        assert!(questions[0].multi_select);
        assert_eq!(questions[0].options[0].preview.as_deref(), Some("swift"));

        let updated = answered_question_input(
            input,
            &questions,
            r#"{"answers":{"Which targets?":"iOS, Android"}}"#,
        )
        .expect("answers should be accepted");
        assert_eq!(
            updated
                .pointer("/answers/Which targets?")
                .and_then(Value::as_str),
            Some("iOS, Android")
        );

        let extended = json!({
            "questions": (0..6)
                .map(|index| json!({
                    "question": format!("Question {index}?"),
                    "header": format!("Q{index}"),
                    "options": [
                        { "label": "Yes", "description": "Proceed" },
                        { "label": "No", "description": "Stop" }
                    ],
                    "multiSelect": false
                }))
                .collect::<Vec<_>>()
        });
        assert_eq!(parse_user_questions(&extended).len(), 6);
    }
}
