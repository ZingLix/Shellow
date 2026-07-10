pub fn backend_name() -> &'static str {
    if is_russh_available() {
        "russh"
    } else {
        "demo-transport"
    }
}

pub fn is_russh_available() -> bool {
    cfg!(feature = "native-integrations")
}

pub fn demo_transport_summary() -> String {
    #[cfg(feature = "native-integrations")]
    {
        format!(
            "russh native integration compiled; SessionActor can request PTY {:?}",
            russh::Pty::IUTF8
        )
    }

    #[cfg(not(feature = "native-integrations"))]
    {
        "demo transport is active; next step is binding russh SessionActor to the same snapshot stream"
            .to_string()
    }
}

pub const DEFAULT_LIVE_KEEPALIVE_INTERVAL_SECS: u64 = 30;
pub const DEFAULT_KEEPALIVE_MAX: usize = 3;
const DEFAULT_PTY_TERM: &str = "xterm-256color";
const DEFAULT_COLORTERM: &str = "truecolor";

pub fn keepalive_policy_summary(interval_secs: Option<u64>, max: usize) -> String {
    match interval_secs {
        Some(interval_secs) => format!("keepalive={interval_secs}s max-missed={max}"),
        None => "keepalive=disabled".to_string(),
    }
}

pub fn normalize_sha256_fingerprint_option(value: Option<&str>) -> Option<String> {
    normalize_sha256_fingerprint(value?)
}

pub fn normalize_sha256_fingerprint(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_prefix = trimmed
        .get(..7)
        .filter(|prefix| prefix.eq_ignore_ascii_case("SHA256:"))
        .map(|_| &trimmed[7..])
        .unwrap_or(trimmed);
    let fingerprint = without_prefix.split_whitespace().next()?.trim();
    if fingerprint.is_empty() {
        None
    } else {
        Some(format!("SHA256:{fingerprint}"))
    }
}

pub fn sha256_fingerprints_match(actual: &str, expected: &str) -> bool {
    normalize_sha256_fingerprint(actual) == normalize_sha256_fingerprint(expected)
}

#[cfg(feature = "native-integrations")]
pub fn exec_password_blocking(
    options: RusshConnectOptions,
    command: &str,
) -> Result<String, String> {
    exec_blocking(options, command)
}

#[cfg(feature = "native-integrations")]
pub fn exec_private_key_blocking(
    options: RusshConnectOptions,
    command: &str,
) -> Result<String, String> {
    exec_blocking(options, command)
}

#[cfg(feature = "native-integrations")]
fn exec_blocking(options: RusshConnectOptions, command: &str) -> Result<String, String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("shellow-russh")
        .build()
        .map_err(|error| format!("tokio runtime failed: {error}"))?;

    runtime.block_on(async move {
        let mut actor = RusshSessionActor::connect_password(options).await?;

        let output = actor
            .exec_collect_text(command)
            .await
            .map_err(|error| format!("ssh exec failed: {error}"));

        let _ = actor.disconnect().await;
        output
    })
}

#[cfg(feature = "native-integrations")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveShellStatus {
    Connecting,
    Connected {
        observed_host_key_sha256: Option<String>,
    },
    Closed,
    Failed(String),
}

#[cfg(feature = "native-integrations")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveShellPoll {
    pub output: Vec<u8>,
    pub status: LiveShellStatus,
}

#[cfg(feature = "native-integrations")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecStdioStatus {
    Connecting,
    Connected {
        observed_host_key_sha256: Option<String>,
    },
    Closed,
    Failed(String),
}

#[cfg(feature = "native-integrations")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecStdioPoll {
    pub output: Vec<u8>,
    pub status: ExecStdioStatus,
}

#[cfg(feature = "native-integrations")]
enum LiveShellInput {
    Bytes(Vec<u8>),
    Resize { cols: u32, rows: u32 },
    Disconnect,
}

#[cfg(feature = "native-integrations")]
enum ExecStdioInput {
    Bytes(Vec<u8>),
    Disconnect,
}

#[cfg(feature = "native-integrations")]
#[derive(Debug)]
pub struct LiveShellHandle {
    input: tokio::sync::mpsc::UnboundedSender<LiveShellInput>,
    output: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    status: std::sync::Arc<std::sync::Mutex<LiveShellStatus>>,
    revision: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

#[cfg(feature = "native-integrations")]
impl LiveShellHandle {
    pub fn spawn_password(options: RusshConnectOptions) -> Result<Self, String> {
        Self::spawn(options)
    }

    pub fn spawn(options: RusshConnectOptions) -> Result<Self, String> {
        let (input, receiver) = tokio::sync::mpsc::unbounded_channel();
        let output = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let status = std::sync::Arc::new(std::sync::Mutex::new(LiveShellStatus::Connecting));
        let revision = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1));
        let thread_output = std::sync::Arc::clone(&output);
        let thread_status = std::sync::Arc::clone(&status);
        let thread_revision = std::sync::Arc::clone(&revision);

        std::thread::Builder::new()
            .name("shellow-live-russh".to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .thread_name("shellow-live-russh-runtime")
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        set_live_status(
                            &thread_status,
                            &thread_revision,
                            LiveShellStatus::Failed(format!("tokio runtime failed: {error}")),
                        );
                        return;
                    }
                };

                runtime.block_on(run_live_shell(
                    options,
                    receiver,
                    thread_output,
                    thread_status,
                    thread_revision,
                ));
            })
            .map_err(|error| format!("failed to spawn russh shell thread: {error}"))?;

        Ok(Self {
            input,
            output,
            status,
            revision,
        })
    }

    pub fn send_input(&self, input: &str) -> Result<(), String> {
        self.input
            .send(LiveShellInput::Bytes(input.as_bytes().to_vec()))
            .map_err(|_| "live shell channel is closed".to_string())
    }

    pub fn resize(&self, cols: u32, rows: u32) -> Result<(), String> {
        self.input
            .send(LiveShellInput::Resize { cols, rows })
            .map_err(|_| "live shell channel is closed".to_string())
    }

    pub fn disconnect(&self) {
        let _ = self.input.send(LiveShellInput::Disconnect);
    }

    pub fn poll(&self) -> LiveShellPoll {
        LiveShellPoll {
            output: take_live_output(&self.output),
            status: get_live_status(&self.status),
        }
    }

    pub fn event_revision(&self) -> u64 {
        self.revision.load(std::sync::atomic::Ordering::Acquire)
    }
}

#[cfg(feature = "native-integrations")]
impl Drop for LiveShellHandle {
    fn drop(&mut self) {
        let _ = self.input.send(LiveShellInput::Disconnect);
    }
}

#[cfg(feature = "native-integrations")]
#[derive(Debug)]
pub struct ExecStdioHandle {
    input: tokio::sync::mpsc::UnboundedSender<ExecStdioInput>,
    output: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    status: std::sync::Arc<std::sync::Mutex<ExecStdioStatus>>,
    revision: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

#[cfg(feature = "native-integrations")]
impl ExecStdioHandle {
    pub fn spawn(options: RusshConnectOptions, command: String) -> Result<Self, String> {
        let (input, receiver) = tokio::sync::mpsc::unbounded_channel();
        let output = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let status = std::sync::Arc::new(std::sync::Mutex::new(ExecStdioStatus::Connecting));
        let revision = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1));
        let thread_output = std::sync::Arc::clone(&output);
        let thread_status = std::sync::Arc::clone(&status);
        let thread_revision = std::sync::Arc::clone(&revision);

        std::thread::Builder::new()
            .name("shellow-exec-stdio-russh".to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .thread_name("shellow-exec-stdio-runtime")
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        set_exec_status(
                            &thread_status,
                            &thread_revision,
                            ExecStdioStatus::Failed(format!("tokio runtime failed: {error}")),
                        );
                        return;
                    }
                };

                runtime.block_on(run_exec_stdio(
                    options,
                    command,
                    receiver,
                    thread_output,
                    thread_status,
                    thread_revision,
                ));
            })
            .map_err(|error| format!("failed to spawn ssh exec stdio thread: {error}"))?;

        Ok(Self {
            input,
            output,
            status,
            revision,
        })
    }

    pub fn send_bytes(&self, bytes: Vec<u8>) -> Result<(), String> {
        self.input
            .send(ExecStdioInput::Bytes(bytes))
            .map_err(|_| "ssh exec stdio channel is closed".to_string())
    }

    pub fn disconnect(&self) {
        let _ = self.input.send(ExecStdioInput::Disconnect);
    }

    pub fn poll(&self) -> ExecStdioPoll {
        ExecStdioPoll {
            output: take_live_output(&self.output),
            status: get_exec_status(&self.status),
        }
    }

    pub fn event_revision(&self) -> u64 {
        self.revision.load(std::sync::atomic::Ordering::Acquire)
    }
}

#[cfg(feature = "native-integrations")]
impl Drop for ExecStdioHandle {
    fn drop(&mut self) {
        let _ = self.input.send(ExecStdioInput::Disconnect);
    }
}

#[cfg(feature = "native-integrations")]
pub struct RusshConnectOptions {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: RusshAuthMethod,
    pub expected_host_key_sha256: Option<String>,
    pub keepalive_interval_secs: Option<u64>,
    pub keepalive_max: usize,
    pub cols: u32,
    pub rows: u32,
    pub inactivity_timeout_secs: u64,
}

#[cfg(feature = "native-integrations")]
pub enum RusshAuthMethod {
    Password(String),
    PrivateKey {
        private_key_pem: String,
        passphrase: Option<String>,
    },
}

#[cfg(feature = "native-integrations")]
impl RusshAuthMethod {
    fn label(&self) -> &'static str {
        match self {
            Self::Password(_) => "password",
            Self::PrivateKey { .. } => "private-key",
        }
    }
}

#[cfg(feature = "native-integrations")]
async fn run_live_shell(
    options: RusshConnectOptions,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<LiveShellInput>,
    output: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    status: std::sync::Arc<std::sync::Mutex<LiveShellStatus>>,
    revision: std::sync::Arc<std::sync::atomic::AtomicU64>,
) {
    let mut actor = match RusshSessionActor::connect(options).await {
        Ok(actor) => actor,
        Err(error) => {
            append_live_output(&output, &revision, error.as_bytes());
            append_live_output(&output, &revision, b"\r\n");
            set_live_status(&status, &revision, LiveShellStatus::Failed(error));
            return;
        }
    };

    let channel_result = async {
        let channel = actor.session.channel_open_session().await?;
        channel
            .request_pty(false, DEFAULT_PTY_TERM, actor.cols, actor.rows, 0, 0, &[])
            .await?;
        let _ = channel.set_env(false, "COLORTERM", DEFAULT_COLORTERM).await;
        channel.request_shell(true).await?;
        Ok::<_, russh::Error>(channel)
    }
    .await;

    let channel = match channel_result {
        Ok(channel) => channel,
        Err(error) => {
            let message = format!("ssh pty/shell request failed: {error}");
            append_live_output(&output, &revision, message.as_bytes());
            append_live_output(&output, &revision, b"\r\n");
            set_live_status(&status, &revision, LiveShellStatus::Failed(message));
            let _ = actor.disconnect().await;
            return;
        }
    };

    let (mut read_half, write_half) = channel.split();
    set_live_status(
        &status,
        &revision,
        LiveShellStatus::Connected {
            observed_host_key_sha256: actor.observed_host_key_sha256.clone(),
        },
    );

    loop {
        while let Ok(message) = receiver.try_recv() {
            match message {
                LiveShellInput::Bytes(bytes) => {
                    if let Err(error) = write_half.data_bytes(bytes).await {
                        let message = format!("ssh input failed: {error}");
                        append_live_output(&output, &revision, message.as_bytes());
                        append_live_output(&output, &revision, b"\r\n");
                        set_live_status(&status, &revision, LiveShellStatus::Failed(message));
                        let _ = write_half.close().await;
                        let _ = actor.disconnect().await;
                        return;
                    }
                }
                LiveShellInput::Resize { cols, rows } => {
                    if let Err(error) = write_half.window_change(cols, rows, 0, 0).await {
                        let message = format!("ssh window-change failed: {error}");
                        append_live_output(&output, &revision, message.as_bytes());
                        append_live_output(&output, &revision, b"\r\n");
                        set_live_status(&status, &revision, LiveShellStatus::Failed(message));
                        let _ = write_half.close().await;
                        let _ = actor.disconnect().await;
                        return;
                    }
                }
                LiveShellInput::Disconnect => {
                    let _ = write_half.close().await;
                    let _ = actor.disconnect().await;
                    set_live_status(&status, &revision, LiveShellStatus::Closed);
                    return;
                }
            }
        }

        match tokio::time::timeout(std::time::Duration::from_millis(40), read_half.wait()).await {
            Ok(Some(russh::ChannelMsg::Data { data })) => {
                append_live_output(&output, &revision, &data)
            }
            Ok(Some(russh::ChannelMsg::ExtendedData { data, .. })) => {
                append_live_output(&output, &revision, &data)
            }
            Ok(Some(russh::ChannelMsg::ExitStatus { .. })) => {}
            Ok(Some(russh::ChannelMsg::Close)) | Ok(None) => {
                set_live_status(&status, &revision, LiveShellStatus::Closed);
                let _ = actor.disconnect().await;
                return;
            }
            Ok(Some(_)) | Err(_) => {}
        }
    }
}

#[cfg(feature = "native-integrations")]
async fn run_exec_stdio(
    options: RusshConnectOptions,
    command: String,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<ExecStdioInput>,
    output: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    status: std::sync::Arc<std::sync::Mutex<ExecStdioStatus>>,
    revision: std::sync::Arc<std::sync::atomic::AtomicU64>,
) {
    let mut actor = match RusshSessionActor::connect(options).await {
        Ok(actor) => actor,
        Err(error) => {
            append_exec_output(&output, &revision, error.as_bytes());
            append_exec_output(&output, &revision, b"\n");
            set_exec_status(&status, &revision, ExecStdioStatus::Failed(error));
            return;
        }
    };

    let channel_result = async {
        let channel = actor.session.channel_open_session().await?;
        let _ = channel.set_env(false, "COLORTERM", DEFAULT_COLORTERM).await;
        channel.exec(true, command.as_str()).await?;
        Ok::<_, russh::Error>(channel)
    }
    .await;

    let channel = match channel_result {
        Ok(channel) => channel,
        Err(error) => {
            let message = format!("ssh exec request failed: {error}");
            append_exec_output(&output, &revision, message.as_bytes());
            append_exec_output(&output, &revision, b"\n");
            set_exec_status(&status, &revision, ExecStdioStatus::Failed(message));
            let _ = actor.disconnect().await;
            return;
        }
    };

    let (mut read_half, write_half) = channel.split();
    set_exec_status(
        &status,
        &revision,
        ExecStdioStatus::Connected {
            observed_host_key_sha256: actor.observed_host_key_sha256.clone(),
        },
    );

    loop {
        while let Ok(message) = receiver.try_recv() {
            match message {
                ExecStdioInput::Bytes(bytes) => {
                    if let Err(error) = write_half.data_bytes(bytes).await {
                        let message = format!("ssh exec stdin write failed: {error}");
                        append_exec_output(&output, &revision, message.as_bytes());
                        append_exec_output(&output, &revision, b"\n");
                        set_exec_status(&status, &revision, ExecStdioStatus::Failed(message));
                        let _ = write_half.close().await;
                        let _ = actor.disconnect().await;
                        return;
                    }
                }
                ExecStdioInput::Disconnect => {
                    let _ = write_half.close().await;
                    let _ = actor.disconnect().await;
                    set_exec_status(&status, &revision, ExecStdioStatus::Closed);
                    return;
                }
            }
        }

        match tokio::time::timeout(std::time::Duration::from_millis(40), read_half.wait()).await {
            Ok(Some(russh::ChannelMsg::Data { data })) => {
                append_exec_output(&output, &revision, &data)
            }
            Ok(Some(russh::ChannelMsg::ExtendedData { data, .. })) => {
                append_exec_output(&output, &revision, &data)
            }
            Ok(Some(russh::ChannelMsg::ExitStatus { .. })) => {}
            Ok(Some(russh::ChannelMsg::Close)) | Ok(None) => {
                set_exec_status(&status, &revision, ExecStdioStatus::Closed);
                let _ = actor.disconnect().await;
                return;
            }
            Ok(Some(_)) | Err(_) => {}
        }
    }
}

#[cfg(feature = "native-integrations")]
fn append_live_output(
    output: &std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    revision: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    bytes: &[u8],
) {
    append_buffered_output(output, revision, bytes, 128 * 1024);
}

#[cfg(feature = "native-integrations")]
fn append_exec_output(
    output: &std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    revision: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    bytes: &[u8],
) {
    append_buffered_output(output, revision, bytes, 8 * 1024 * 1024);
}

#[cfg(feature = "native-integrations")]
fn append_buffered_output(
    output: &std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    revision: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    bytes: &[u8],
    max_buffered_bytes: usize,
) {
    if bytes.is_empty() {
        return;
    }

    if let Ok(mut output) = output.lock() {
        output.extend_from_slice(bytes);
        if output.len() > max_buffered_bytes {
            let drain = output.len() - max_buffered_bytes;
            output.drain(..drain);
        }
        revision.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }
}

#[cfg(feature = "native-integrations")]
fn take_live_output(output: &std::sync::Arc<std::sync::Mutex<Vec<u8>>>) -> Vec<u8> {
    output
        .lock()
        .map(|mut output| std::mem::take(&mut *output))
        .unwrap_or_default()
}

#[cfg(feature = "native-integrations")]
fn set_live_status(
    status: &std::sync::Arc<std::sync::Mutex<LiveShellStatus>>,
    revision: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    next: LiveShellStatus,
) {
    if let Ok(mut status) = status.lock() {
        if *status != next {
            *status = next;
            revision.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        }
    }
}

#[cfg(feature = "native-integrations")]
fn get_live_status(status: &std::sync::Arc<std::sync::Mutex<LiveShellStatus>>) -> LiveShellStatus {
    status
        .lock()
        .map(|status| status.clone())
        .unwrap_or_else(|_| LiveShellStatus::Failed("live shell status lock failed".to_string()))
}

#[cfg(feature = "native-integrations")]
fn set_exec_status(
    status: &std::sync::Arc<std::sync::Mutex<ExecStdioStatus>>,
    revision: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    next: ExecStdioStatus,
) {
    if let Ok(mut status) = status.lock() {
        if *status != next {
            *status = next;
            revision.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        }
    }
}

#[cfg(feature = "native-integrations")]
fn get_exec_status(status: &std::sync::Arc<std::sync::Mutex<ExecStdioStatus>>) -> ExecStdioStatus {
    status
        .lock()
        .map(|status| status.clone())
        .unwrap_or_else(|_| {
            ExecStdioStatus::Failed("ssh exec stdio status lock failed".to_string())
        })
}

#[cfg(feature = "native-integrations")]
pub struct RusshSessionActor {
    session: russh::client::Handle<ShellowClient>,
    cols: u32,
    rows: u32,
    observed_host_key_sha256: Option<String>,
}

#[cfg(feature = "native-integrations")]
impl RusshSessionActor {
    pub async fn connect_password(options: RusshConnectOptions) -> Result<Self, String> {
        Self::connect(options).await
    }

    pub async fn connect(options: RusshConnectOptions) -> Result<Self, String> {
        let config = russh::client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(
                options.inactivity_timeout_secs,
            )),
            keepalive_interval: options
                .keepalive_interval_secs
                .map(std::time::Duration::from_secs),
            keepalive_max: options.keepalive_max,
            nodelay: true,
            ..Default::default()
        };

        let auth_label = options.auth.label();
        let auth = prepare_auth(options.auth)?;
        let expected_host_key_sha256 = options.expected_host_key_sha256.clone();
        let observed_host_key_sha256 = std::sync::Arc::new(std::sync::Mutex::new(None));

        let mut session = russh::client::connect(
            std::sync::Arc::new(config),
            (options.host.as_str(), options.port),
            ShellowClient {
                expected_host_key_sha256,
                observed_host_key_sha256: std::sync::Arc::clone(&observed_host_key_sha256),
            },
        )
        .await
        .map_err(|error| {
            describe_connect_error(
                error,
                &options.expected_host_key_sha256,
                &observed_host_key_sha256,
            )
        })?;

        let authenticated = match auth {
            PreparedRusshAuth::Password(password) => session
                .authenticate_password(options.username, password)
                .await
                .map_err(|error| format!("ssh password authentication failed: {error}"))?,
            PreparedRusshAuth::PrivateKey(private_key) => {
                let hash_alg = session
                    .best_supported_rsa_hash()
                    .await
                    .map_err(|error| format!("ssh signature algorithm query failed: {error}"))?
                    .flatten();
                session
                    .authenticate_publickey(
                        options.username,
                        russh::keys::PrivateKeyWithHashAlg::new(
                            std::sync::Arc::new(private_key),
                            hash_alg,
                        ),
                    )
                    .await
                    .map_err(|error| format!("ssh private-key authentication failed: {error}"))?
            }
        };

        if !authenticated.success() {
            return Err(format!("ssh {auth_label} authentication rejected"));
        }

        let observed_host_key_sha256 = observed_host_key_sha256
            .lock()
            .ok()
            .and_then(|observed| observed.clone());

        Ok(Self {
            session,
            cols: options.cols,
            rows: options.rows,
            observed_host_key_sha256,
        })
    }

    pub async fn exec_collect_text(&mut self, command: &str) -> Result<String, russh::Error> {
        let mut channel = self.session.channel_open_session().await?;
        channel
            .request_pty(false, DEFAULT_PTY_TERM, self.cols, self.rows, 0, 0, &[])
            .await?;
        let _ = channel.set_env(false, "COLORTERM", DEFAULT_COLORTERM).await;
        channel.exec(true, command).await?;

        let mut output = Vec::new();
        while let Some(message) = channel.wait().await {
            match message {
                russh::ChannelMsg::Data { data } => output.extend_from_slice(&data),
                russh::ChannelMsg::ExitStatus { .. } => {}
                _ => {}
            }
        }

        Ok(String::from_utf8_lossy(&output).into_owned())
    }

    pub async fn disconnect(&mut self) -> Result<(), russh::Error> {
        self.session
            .disconnect(russh::Disconnect::ByApplication, "", "English")
            .await
    }
}

#[cfg(feature = "native-integrations")]
enum PreparedRusshAuth {
    Password(String),
    PrivateKey(russh::keys::PrivateKey),
}

#[cfg(feature = "native-integrations")]
fn prepare_auth(auth: RusshAuthMethod) -> Result<PreparedRusshAuth, String> {
    match auth {
        RusshAuthMethod::Password(password) => Ok(PreparedRusshAuth::Password(password)),
        RusshAuthMethod::PrivateKey {
            private_key_pem,
            passphrase,
        } => private_key_from_openssh(&private_key_pem, passphrase.as_deref())
            .map(PreparedRusshAuth::PrivateKey),
    }
}

#[cfg(feature = "native-integrations")]
pub fn validate_private_key_auth(
    private_key_pem: &str,
    passphrase: Option<&str>,
) -> Result<(), String> {
    private_key_from_openssh(private_key_pem, passphrase).map(|_| ())
}

#[cfg(feature = "native-integrations")]
fn describe_connect_error(
    error: russh::Error,
    expected_host_key_sha256: &Option<String>,
    observed_host_key_sha256: &std::sync::Arc<std::sync::Mutex<Option<String>>>,
) -> String {
    let expected = expected_host_key_sha256.as_deref();
    let observed = observed_host_key_sha256
        .lock()
        .ok()
        .and_then(|observed| observed.clone());

    if let (Some(expected), Some(observed)) = (expected, observed.as_deref()) {
        if !sha256_fingerprints_match(observed, expected) {
            let expected =
                normalize_sha256_fingerprint(expected).unwrap_or_else(|| expected.to_string());
            return format!("ssh host key mismatch: expected {expected}, got {observed}");
        }
    }

    format!("ssh connection failed: {error}")
}

#[cfg(feature = "native-integrations")]
fn private_key_from_openssh(
    private_key_pem: &str,
    passphrase: Option<&str>,
) -> Result<russh::keys::PrivateKey, String> {
    let key = russh::keys::PrivateKey::from_openssh(private_key_pem.as_bytes())
        .map_err(|error| format!("private key parse failed: {error}"))?;

    if !key.is_encrypted() {
        return Ok(key);
    }

    let passphrase = passphrase
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "private key is encrypted; passphrase required".to_string())?;

    key.decrypt(passphrase.as_bytes())
        .map_err(|error| format!("private key decrypt failed: {error}"))
}

#[cfg(feature = "native-integrations")]
struct ShellowClient {
    expected_host_key_sha256: Option<String>,
    observed_host_key_sha256: std::sync::Arc<std::sync::Mutex<Option<String>>>,
}

#[cfg(feature = "native-integrations")]
impl russh::client::Handler for ShellowClient {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        let actual = format!(
            "{}",
            server_public_key.fingerprint(russh::keys::ssh_key::HashAlg::Sha256)
        );
        if let Ok(mut observed) = self.observed_host_key_sha256.lock() {
            *observed = Some(actual.clone());
        }

        let Some(expected) = self.expected_host_key_sha256.as_deref() else {
            return Ok(true);
        };

        Ok(sha256_fingerprints_match(&actual, expected))
    }
}
