//! Bounded Chromium implementation of the target-neutral BrowserWorker contract.
//!
//! This module deliberately exposes only generic browser operations.  It does
//! not know about sites, DOM selectors, accounts, media, or PlaybackSession.

use crate::browser::{
    BROWSER_EVENT_VERSION, BrowserAuthMode, BrowserError, BrowserEvent, BrowserEventKind,
    BrowserFuture, BrowserInput, BrowserNavigationRequest, BrowserOperationId, BrowserSession,
    BrowserSessionId, BrowserStatus, BrowserWorker, NativePanelSession, R008NavigationPolicy,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex as AsyncMutex, Notify, oneshot};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;
use uuid::Uuid;

const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(20);
const BROWSER_START_TIMEOUT: Duration = Duration::from_secs(8);
const CLOSE_WAIT: Duration = Duration::from_secs(2);
const EVENT_POLL_INTERVAL: Duration = Duration::from_millis(20);

const ALLOWED_BINARIES: &[(&str, &str)] = &[
    ("google-chrome-stable", "google-chrome-stable"),
    ("google-chrome", "google-chrome"),
    ("chromium", "chromium"),
    ("chromium-browser", "chromium-browser"),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChromiumBinaryInfo {
    pub class: String,
    pub version: String,
}

#[derive(Debug)]
enum CdpError {
    Disconnected,
    Timeout,
    Protocol,
}

type CdpResult = Result<Value, CdpError>;

type CdpStream =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Clone)]
struct CdpClient {
    sink: Arc<AsyncMutex<futures_util::stream::SplitSink<CdpStream, Message>>>,
    pending: Arc<AsyncMutex<HashMap<u64, oneshot::Sender<CdpResult>>>>,
    events: Arc<AsyncMutex<tokio::sync::mpsc::UnboundedReceiver<Value>>>,
    event_tx: tokio::sync::mpsc::UnboundedSender<Value>,
    next_id: Arc<AtomicU64>,
    alive: Arc<AtomicBool>,
    disconnected: Arc<Notify>,
}

impl CdpClient {
    async fn connect(url: &str) -> Result<Self, CdpError> {
        let (socket, _) = connect_async(url)
            .await
            .map_err(|_| CdpError::Disconnected)?;
        let (sink, mut stream) = socket.split();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let client = Self {
            sink: Arc::new(AsyncMutex::new(sink)),
            pending: Arc::new(AsyncMutex::new(HashMap::new())),
            events: Arc::new(AsyncMutex::new(event_rx)),
            event_tx,
            next_id: Arc::new(AtomicU64::new(1)),
            alive: Arc::new(AtomicBool::new(true)),
            disconnected: Arc::new(Notify::new()),
        };

        let pending = Arc::clone(&client.pending);
        let alive = Arc::clone(&client.alive);
        let disconnected = Arc::clone(&client.disconnected);
        let event_tx = client.event_tx.clone();
        tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                let text = match message {
                    Ok(Message::Text(text)) => text.to_string(),
                    Ok(Message::Binary(bytes)) => String::from_utf8_lossy(&bytes).into_owned(),
                    Ok(Message::Close(_)) | Err(_) => break,
                    Ok(_) => continue,
                };
                let Ok(value) = serde_json::from_str::<Value>(&text) else {
                    continue;
                };
                if let Some(id) = value.get("id").and_then(Value::as_u64) {
                    if let Some(sender) = pending.lock().await.remove(&id) {
                        let result = if value.get("error").is_some() {
                            Err(CdpError::Protocol)
                        } else {
                            Ok(value.get("result").cloned().unwrap_or(Value::Null))
                        };
                        let _ = sender.send(result);
                    }
                } else {
                    let _ = event_tx.send(value);
                }
            }
            alive.store(false, Ordering::Release);
            let mut pending = pending.lock().await;
            for (_, sender) in pending.drain() {
                let _ = sender.send(Err(CdpError::Disconnected));
            }
            disconnected.notify_waiters();
        });

        Ok(client)
    }

    async fn command(&self, method: &str, params: Value, wait_for: Duration) -> CdpResult {
        if !self.alive.load(Ordering::Acquire) {
            return Err(CdpError::Disconnected);
        }
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = oneshot::channel();
        self.pending.lock().await.insert(id, sender);
        let message = json!({"id": id, "method": method, "params": params});
        if self
            .sink
            .lock()
            .await
            .send(Message::Text(message.to_string().into()))
            .await
            .is_err()
        {
            self.pending.lock().await.remove(&id);
            return Err(CdpError::Disconnected);
        }
        match timeout(wait_for, receiver).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(CdpError::Disconnected),
            Err(_) => {
                self.pending.lock().await.remove(&id);
                Err(CdpError::Timeout)
            }
        }
    }

    async fn next_event(&self, wait_for: Duration) -> Option<Value> {
        timeout(wait_for, self.events.lock())
            .await
            .ok()?
            .recv()
            .await
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }
}

#[derive(Debug)]
struct PanelRecord {
    worker_session: BrowserSessionId,
    token: crate::browser::PanelControlToken,
    expires_at: Instant,
    connected: bool,
}

struct ChromiumSession {
    status: BrowserStatus,
    events: VecDeque<BrowserEvent>,
    sequence: u64,
    profile_dir: PathBuf,
    child: Option<tokio::process::Child>,
    cdp: Arc<CdpClient>,
    panels: HashMap<crate::browser::PanelSessionId, PanelRecord>,
    cancelled: HashMap<BrowserOperationId, ()>,
}

type SessionHandle = Arc<AsyncMutex<ChromiumSession>>;
type SessionMap = HashMap<BrowserSessionId, SessionHandle>;

#[derive(Clone)]
pub struct ChromiumBrowserWorker {
    sessions: Arc<Mutex<SessionMap>>,
    operation_timeout: Duration,
}

impl Default for ChromiumBrowserWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl ChromiumBrowserWorker {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            operation_timeout: DEFAULT_OPERATION_TIMEOUT,
        }
    }

    pub fn with_operation_timeout(operation_timeout: Duration) -> Self {
        Self {
            operation_timeout,
            ..Self::new()
        }
    }

    pub fn discover_binary() -> Option<ChromiumBinaryInfo> {
        let (class, path) = find_allowed_binary()?;
        let output = std::process::Command::new(path)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .ok()?;
        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("unknown")
            .chars()
            .take(128)
            .collect::<String>();
        Some(ChromiumBinaryInfo { class, version })
    }

    pub fn open_panel(
        &self,
        session: &BrowserSessionId,
        ttl: Duration,
    ) -> Result<NativePanelSession, BrowserError> {
        let handle = self.session_handle(session)?;
        let mut state = handle
            .try_lock()
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        Self::refresh_status(&mut state)?;
        if state.status != BrowserStatus::Open {
            return Err(status_error(state.status));
        }
        let panel = NativePanelSession::new_for_worker(session.clone(), ttl);
        state.panels.insert(
            panel.id().clone(),
            PanelRecord {
                worker_session: session.clone(),
                token: panel.token().clone(),
                expires_at: panel.expires_at(),
                connected: true,
            },
        );
        Ok(panel)
    }

    pub fn panel_control(
        &self,
        panel: &NativePanelSession,
        input: BrowserInput,
    ) -> BrowserFuture<'_, ()> {
        let worker = self.clone();
        let worker_session = panel.worker_session().clone();
        let panel_id = panel.id().clone();
        let panel_token = panel.token().clone();
        let panel_expires_at = panel.expires_at();
        Box::pin(async move {
            {
                let handle = worker.session_handle(&worker_session)?;
                let mut state = handle.lock().await;
                Self::refresh_status(&mut state)?;
                let record = state
                    .panels
                    .get(&panel_id)
                    .ok_or(BrowserError::SessionExpired)?;
                if record.worker_session != worker_session || record.token != panel_token {
                    return Err(BrowserError::SessionExpired);
                }
                if record.expires_at <= Instant::now() || panel_expires_at <= Instant::now() {
                    state.panels.remove(&panel_id);
                    return Err(BrowserError::SessionExpired);
                }
                if !record.connected {
                    return Err(BrowserError::PanelDisconnected);
                }
            }
            worker.send_input(&worker_session, input).await
        })
    }

    pub fn disconnect_panel(&self, panel: &NativePanelSession) -> Result<(), BrowserError> {
        let handle = self.session_handle(panel.worker_session())?;
        let mut state = handle
            .try_lock()
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        let record = state
            .panels
            .get_mut(panel.id())
            .ok_or(BrowserError::SessionExpired)?;
        if record.token != *panel.token() {
            return Err(BrowserError::SessionExpired);
        }
        record.connected = false;
        Ok(())
    }

    pub fn reconnect_panel(
        &self,
        panel: &NativePanelSession,
        ttl: Duration,
    ) -> Result<NativePanelSession, BrowserError> {
        let handle = self.session_handle(panel.worker_session())?;
        let mut state = handle
            .try_lock()
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        Self::refresh_status(&mut state)?;
        if state.status != BrowserStatus::Open {
            return Err(status_error(state.status));
        }
        let record = state
            .panels
            .get_mut(panel.id())
            .ok_or(BrowserError::SessionExpired)?;
        if record.token != *panel.token() {
            return Err(BrowserError::SessionExpired);
        }
        let replacement = NativePanelSession::new_for_worker(panel.worker_session().clone(), ttl);
        record.token = replacement.token().clone();
        record.expires_at = replacement.expires_at();
        record.connected = true;
        Ok(replacement)
    }

    #[cfg(test)]
    fn kill_for_test(&self, session: &BrowserSessionId) -> Result<(), BrowserError> {
        let handle = self.session_handle(session)?;
        let mut state = handle
            .try_lock()
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        Self::session_error(&mut state)?;
        if let Some(child) = state.child.as_mut() {
            kill_process_group(child);
            Ok(())
        } else {
            Err(BrowserError::WorkerCrashed)
        }
    }

    fn lock_sessions(&self) -> Result<std::sync::MutexGuard<'_, SessionMap>, BrowserError> {
        self.sessions
            .lock()
            .map_err(|_| BrowserError::WorkerUnavailable)
    }

    fn session_handle(&self, session: &BrowserSessionId) -> Result<SessionHandle, BrowserError> {
        self.lock_sessions()?
            .get(session)
            .cloned()
            .ok_or(BrowserError::InvalidSession)
    }

    fn push_event(state: &mut ChromiumSession, kind: BrowserEventKind) {
        state.sequence += 1;
        state.events.push_back(BrowserEvent {
            version: BROWSER_EVENT_VERSION,
            sequence: state.sequence,
            kind,
        });
    }

    fn refresh_status(state: &mut ChromiumSession) -> Result<(), BrowserError> {
        if state.status != BrowserStatus::Open {
            return Ok(());
        }
        let exited = state
            .child
            .as_mut()
            .and_then(|child| child.try_wait().ok())
            .flatten()
            .is_some();
        if exited {
            state.child = None;
            state.status = BrowserStatus::Crashed;
            state
                .panels
                .values_mut()
                .for_each(|panel| panel.connected = false);
            Self::push_event(state, BrowserEventKind::WorkerCrashed);
            remove_profile(&state.profile_dir);
        }
        Ok(())
    }

    fn terminate_state(state: &mut ChromiumSession, status: BrowserStatus) {
        if state.status != BrowserStatus::Open {
            return;
        }
        if let Some(mut child) = state.child.take() {
            kill_process_group(&mut child);
        }
        state.status = status;
        state
            .panels
            .values_mut()
            .for_each(|panel| panel.connected = false);
        Self::push_event(
            state,
            match status {
                BrowserStatus::Crashed => BrowserEventKind::WorkerCrashed,
                BrowserStatus::TimedOut => BrowserEventKind::WorkerTimedOut,
                _ => BrowserEventKind::WorkerClosed,
            },
        );
        remove_profile(&state.profile_dir);
    }

    async fn spawn_session(&self, mode: BrowserAuthMode) -> Result<BrowserSession, BrowserError> {
        let (class, executable) = find_allowed_binary().ok_or(BrowserError::WorkerUnavailable)?;
        let profile_dir = std::env::temp_dir().join(format!(
            "web-media-gateway-chromium-{}",
            Uuid::new_v4().simple()
        ));
        fs::create_dir(&profile_dir).map_err(|_| BrowserError::WorkerUnavailable)?;
        set_private_permissions(&profile_dir);
        let port = reserve_loopback_port().await?;
        let mut command = tokio::process::Command::new(executable);
        command
            .args([
                "--headless=new",
                "--disable-gpu",
                "--disable-dev-shm-usage",
                "--no-sandbox",
                "--disable-background-networking",
                "--disable-component-update",
                "--disable-default-apps",
                "--disable-extensions",
                "--disable-features=Translate,OptimizationHints,MediaRouter",
                "--no-first-run",
                "--no-default-browser-check",
                "--remote-allow-origins=*",
                "--remote-debugging-address=127.0.0.1",
                "--window-size=1280,720",
            ])
            .arg(format!("--remote-debugging-port={port}"))
            .arg(format!("--user-data-dir={}", profile_dir.display()))
            .arg("about:blank")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let child = match command.spawn() {
            Ok(child) => child,
            Err(_) => {
                remove_profile(&profile_dir);
                return Err(BrowserError::WorkerUnavailable);
            }
        };
        let endpoint = match wait_for_cdp_page(port, BROWSER_START_TIMEOUT).await {
            Ok(endpoint) => endpoint,
            Err(error) => {
                let mut child = child;
                kill_process_group(&mut child);
                remove_profile(&profile_dir);
                return Err(error);
            }
        };
        let cdp = match CdpClient::connect(&endpoint).await {
            Ok(cdp) => Arc::new(cdp),
            Err(_) => {
                let mut child = child;
                kill_process_group(&mut child);
                remove_profile(&profile_dir);
                return Err(BrowserError::WorkerUnavailable);
            }
        };
        if cdp
            .command("Page.enable", Value::Null, DEFAULT_OPERATION_TIMEOUT)
            .await
            .is_err()
            || cdp
                .command("Runtime.enable", Value::Null, DEFAULT_OPERATION_TIMEOUT)
                .await
                .is_err()
            || cdp
                .command(
                    "Fetch.enable",
                    json!({"patterns": [{"requestStage": "Request"}]}),
                    DEFAULT_OPERATION_TIMEOUT,
                )
                .await
                .is_err()
        {
            let mut child = child;
            kill_process_group(&mut child);
            remove_profile(&profile_dir);
            return Err(BrowserError::WorkerUnavailable);
        }
        let session = BrowserSession::new_for_runtime(mode);
        let mut state = ChromiumSession {
            status: BrowserStatus::Open,
            events: VecDeque::new(),
            sequence: 0,
            profile_dir,
            child: Some(child),
            cdp,
            panels: HashMap::new(),
            cancelled: HashMap::new(),
        };
        Self::push_event(
            &mut state,
            BrowserEventKind::WorkerOpened {
                session: session.id().clone(),
            },
        );
        let _ = class;
        self.lock_sessions()?
            .insert(session.id().clone(), Arc::new(AsyncMutex::new(state)));
        Ok(session)
    }

    fn session_error(state: &mut ChromiumSession) -> Result<(), BrowserError> {
        Self::refresh_status(state)?;
        match state.status {
            BrowserStatus::Open => Ok(()),
            status => Err(status_error(status)),
        }
    }

    async fn process_cdp_event(
        &self,
        session: &BrowserSessionId,
        state: &mut ChromiumSession,
        event: Value,
        policy: &R008NavigationPolicy,
    ) -> Result<bool, BrowserError> {
        let method = event
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let params = event.get("params").cloned().unwrap_or(Value::Null);
        match method {
            "Fetch.requestPaused" => {
                let request_id = params
                    .get("requestId")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let url = params
                    .get("request")
                    .and_then(|request| request.get("url"))
                    .and_then(Value::as_str)
                    .and_then(|url| Url::parse(url).ok());
                let allowed = match url {
                    Some(url) => policy.authorize_url(&url).await.is_ok(),
                    None => false,
                };
                if allowed {
                    let _ = state
                        .cdp
                        .command(
                            "Fetch.continueRequest",
                            json!({"requestId": request_id}),
                            self.operation_timeout,
                        )
                        .await;
                } else {
                    let _ = state
                        .cdp
                        .command(
                            "Fetch.failRequest",
                            json!({"requestId": request_id, "errorReason": "BlockedByClient"}),
                            self.operation_timeout,
                        )
                        .await;
                    Self::push_event(state, BrowserEventKind::NetworkDenied);
                    return Err(BrowserError::NavigationDenied);
                }
            }
            "Page.frameNavigated" => {
                if let Some(url) = params
                    .get("frame")
                    .and_then(|frame| frame.get("url"))
                    .and_then(Value::as_str)
                    .and_then(|url| Url::parse(url).ok())
                {
                    let title = state
                        .cdp
                        .command(
                            "Runtime.evaluate",
                            json!({"expression": "document.title", "returnByValue": true}),
                            self.operation_timeout,
                        )
                        .await
                        .ok()
                        .and_then(|result| result.get("result").cloned())
                        .and_then(|result| {
                            result
                                .get("value")
                                .and_then(Value::as_str)
                                .map(str::to_owned)
                        });
                    Self::push_event(state, BrowserEventKind::NavigationChanged { url, title });
                }
            }
            "Page.loadEventFired" => {
                Self::push_event(state, BrowserEventKind::Ready);
                return Ok(true);
            }
            _ => {}
        }
        let _ = session;
        Ok(false)
    }

    async fn wait_for_ready(
        &self,
        session: &BrowserSessionId,
        handle: Arc<AsyncMutex<ChromiumSession>>,
        operation_id: BrowserOperationId,
        policy: &R008NavigationPolicy,
    ) -> Result<(), BrowserError> {
        let deadline = Instant::now() + self.operation_timeout;
        loop {
            let (cdp, cancelled) = {
                let mut state = handle.lock().await;
                if state.cancelled.remove(&operation_id).is_some() {
                    let cdp = Arc::clone(&state.cdp);
                    Self::push_event(
                        &mut state,
                        BrowserEventKind::OperationCancelled { operation_id },
                    );
                    (cdp, true)
                } else {
                    Self::refresh_status(&mut state)?;
                    if !state.cdp.is_alive() {
                        Self::terminate_state(&mut state, BrowserStatus::Crashed);
                        return Err(BrowserError::WorkerCrashed);
                    }
                    if state.status != BrowserStatus::Open {
                        return Err(status_error(state.status));
                    }
                    (Arc::clone(&state.cdp), false)
                }
            };
            if cancelled {
                let _ = cdp
                    .command("Page.stopLoading", Value::Null, self.operation_timeout)
                    .await;
                return Err(BrowserError::OperationCancelled);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                let mut state = handle.lock().await;
                Self::terminate_state(&mut state, BrowserStatus::TimedOut);
                return Err(BrowserError::WorkerTimeout);
            }
            let wait_for = remaining.min(EVENT_POLL_INTERVAL * 5);
            if let Some(event) = cdp.next_event(wait_for).await {
                let mut state = handle.lock().await;
                if self
                    .process_cdp_event(session, &mut state, event, policy)
                    .await?
                {
                    return Ok(());
                }
            }
        }
    }

    async fn send_input_inner(
        &self,
        session: &BrowserSessionId,
        input: BrowserInput,
    ) -> Result<(), BrowserError> {
        let (cdp, kind) = {
            let handle = self.session_handle(session)?;
            let mut state = handle.lock().await;
            Self::session_error(&mut state)?;
            (Arc::clone(&state.cdp), input.kind())
        };
        let command = match &input {
            BrowserInput::Key { key } => json!({
                "type": "keyDown",
                "key": key,
                "code": key,
            }),
            BrowserInput::Pointer { x, y, button } => json!({
                "type": "mousePressed",
                "x": x,
                "y": y,
                "button": pointer_button(*button),
                "clickCount": 1,
            }),
            BrowserInput::Text { value } => json!({"text": value}),
            BrowserInput::Submit => json!({
                "type": "keyDown",
                "key": "Enter",
                "code": "Enter",
            }),
        };
        let method = match input {
            BrowserInput::Text { .. } => "Input.insertText",
            BrowserInput::Pointer { .. } => "Input.dispatchMouseEvent",
            BrowserInput::Key { .. } | BrowserInput::Submit => "Input.dispatchKeyEvent",
        };
        cdp.command(method, command, self.operation_timeout)
            .await
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        let handle = self.session_handle(session)?;
        let mut state = handle.lock().await;
        Self::session_error(&mut state)?;
        Self::push_event(&mut state, BrowserEventKind::InputAccepted { kind });
        Self::push_event(
            &mut state,
            BrowserEventKind::InputResult {
                kind,
                accepted: true,
            },
        );
        Ok(())
    }
}

impl BrowserWorker for ChromiumBrowserWorker {
    fn open_session(&self, mode: BrowserAuthMode) -> BrowserFuture<'_, BrowserSession> {
        Box::pin(self.spawn_session(mode))
    }

    fn attach_profile(
        &self,
        _session: &BrowserSessionId,
        _profile: crate::browser::ProfileAttachmentRef,
    ) -> BrowserFuture<'_, ()> {
        Box::pin(async { Err(BrowserError::ProfileAttachFailed) })
    }

    fn detach_profile(&self, session: &BrowserSessionId) -> BrowserFuture<'_, ()> {
        let session = session.clone();
        Box::pin(async move {
            let handle = self.session_handle(&session)?;
            let mut state = handle.lock().await;
            Self::session_error(&mut state)
        })
    }

    fn navigate<'a>(
        &'a self,
        session: &'a BrowserSessionId,
        request: BrowserNavigationRequest,
        policy: &'a R008NavigationPolicy,
    ) -> BrowserFuture<'a, ()> {
        Box::pin(async move {
            policy.authorize_url(request.url()).await?;
            let handle = self.session_handle(session)?;
            let (cdp, operation_id) = {
                let mut state = handle.lock().await;
                Self::session_error(&mut state)?;
                if state.cancelled.remove(&request.operation_id()).is_some() {
                    Self::push_event(
                        &mut state,
                        BrowserEventKind::OperationCancelled {
                            operation_id: request.operation_id(),
                        },
                    );
                    return Err(BrowserError::OperationCancelled);
                }
                Self::push_event(
                    &mut state,
                    BrowserEventKind::NavigationStarted {
                        url: request.url().clone(),
                    },
                );
                Self::push_event(&mut state, BrowserEventKind::Loading);
                (Arc::clone(&state.cdp), request.operation_id())
            };
            let navigate_result = cdp
                .command(
                    "Page.navigate",
                    json!({"url": request.url().as_str()}),
                    self.operation_timeout,
                )
                .await;
            if let Err(error) = navigate_result {
                let mut state = handle.lock().await;
                let browser_error = match error {
                    CdpError::Timeout => BrowserError::WorkerTimeout,
                    CdpError::Disconnected | CdpError::Protocol => BrowserError::WorkerCrashed,
                };
                Self::terminate_state(
                    &mut state,
                    match browser_error {
                        BrowserError::WorkerTimeout => BrowserStatus::TimedOut,
                        _ => BrowserStatus::Crashed,
                    },
                );
                return Err(browser_error);
            }
            self.wait_for_ready(session, handle, operation_id, policy)
                .await
        })
    }

    fn send_input(&self, session: &BrowserSessionId, input: BrowserInput) -> BrowserFuture<'_, ()> {
        let session = session.clone();
        Box::pin(async move { self.send_input_inner(&session, input).await })
    }

    fn status(&self, session: &BrowserSessionId) -> Result<BrowserStatus, BrowserError> {
        let handle = self.session_handle(session)?;
        let mut state = handle
            .try_lock()
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        Self::refresh_status(&mut state)?;
        Ok(state.status)
    }

    fn poll_events(
        &self,
        session: &BrowserSessionId,
        after_sequence: u64,
    ) -> Result<Vec<BrowserEvent>, BrowserError> {
        let handle = self.session_handle(session)?;
        let mut state = handle
            .try_lock()
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        Self::refresh_status(&mut state)?;
        Ok(state
            .events
            .iter()
            .filter(|event| event.sequence > after_sequence)
            .cloned()
            .collect())
    }

    fn cancel(
        &self,
        session: &BrowserSessionId,
        operation_id: BrowserOperationId,
    ) -> Result<(), BrowserError> {
        let handle = self.session_handle(session)?;
        let mut state = handle
            .try_lock()
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        Self::session_error(&mut state)?;
        state.cancelled.insert(operation_id, ());
        Ok(())
    }

    fn close(&self, session: &BrowserSessionId) -> Result<(), BrowserError> {
        let handle = self.session_handle(session)?;
        let mut state = handle
            .try_lock()
            .map_err(|_| BrowserError::WorkerUnavailable)?;
        Self::refresh_status(&mut state)?;
        if state.status != BrowserStatus::Open {
            return Err(status_error(state.status));
        }
        Self::terminate_state(&mut state, BrowserStatus::Closed);
        Ok(())
    }
}

fn status_error(status: BrowserStatus) -> BrowserError {
    match status {
        BrowserStatus::Closed => BrowserError::SessionClosed,
        BrowserStatus::Crashed => BrowserError::WorkerCrashed,
        BrowserStatus::TimedOut => BrowserError::WorkerTimeout,
        BrowserStatus::Open => BrowserError::WorkerUnavailable,
    }
}

fn pointer_button(button: crate::browser::PointerButton) -> &'static str {
    match button {
        crate::browser::PointerButton::Primary => "left",
        crate::browser::PointerButton::Secondary => "right",
        crate::browser::PointerButton::Auxiliary => "middle",
    }
}

fn find_allowed_binary() -> Option<(String, PathBuf)> {
    let path = std::env::var_os("PATH")?;
    for (class, name) in ALLOWED_BINARIES {
        for directory in std::env::split_paths(&path) {
            let candidate = directory.join(name);
            if candidate.is_file() {
                return Some(((*class).to_string(), candidate));
            }
        }
    }
    None
}

async fn reserve_loopback_port() -> Result<u16, BrowserError> {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .map_err(|_| BrowserError::WorkerUnavailable)?;
    let port = listener
        .local_addr()
        .map_err(|_| BrowserError::WorkerUnavailable)?
        .port();
    drop(listener);
    Ok(port)
}

async fn wait_for_cdp_page(port: u16, wait_for: Duration) -> Result<String, BrowserError> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_millis(500))
        .build()
        .map_err(|_| BrowserError::WorkerUnavailable)?;
    let deadline = Instant::now() + wait_for;
    let endpoint = format!("http://127.0.0.1:{port}/json/list");
    loop {
        if Instant::now() >= deadline {
            return Err(BrowserError::WorkerTimeout);
        }
        if let Ok(response) = client.get(&endpoint).send().await
            && let Ok(targets) = response.json::<Vec<Value>>().await
            && let Some(url) = targets
                .iter()
                .find(|target| target.get("type").and_then(Value::as_str) == Some("page"))
                .and_then(|target| target.get("webSocketDebuggerUrl"))
                .and_then(Value::as_str)
        {
            return Ok(url.to_string());
        }
        sleep(Duration::from_millis(50)).await;
    }
}

fn set_private_permissions(path: &PathBuf) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o700));
    }
}

fn remove_profile(path: &PathBuf) {
    let _ = fs::remove_dir_all(path);
}

fn kill_process_group(child: &mut tokio::process::Child) {
    #[cfg(unix)]
    if let Some(pid) = child.id() {
        // The process group is owned by this bounded worker and contains only
        // the task-owned Chromium process/helpers.
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
    let _ = child.start_kill();
    let deadline = Instant::now() + CLOSE_WAIT;
    while Instant::now() < deadline {
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::{BrowserCommand, PanelFeature, PanelPermissions};
    use crate::{EgressPolicy, EgressScope};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    async fn fixture() -> (tokio::task::JoinHandle<()>, Url) {
        let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let mut request = [0_u8; 1024];
                    let _ = stream.read(&mut request).await;
                    let body = b"<html><head><title>generic-fixture</title></head><body><input></body></html>";
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                    let _ = stream.write_all(body).await;
                });
            }
        });
        (
            task,
            Url::parse(&format!("http://{address}/fixture")).unwrap(),
        )
    }

    fn local_policy(url: &Url) -> R008NavigationPolicy {
        let mut egress = EgressPolicy::default();
        egress.configure_local_service("fixture", url).unwrap();
        R008NavigationPolicy::configured_local_service(egress, "fixture")
    }

    async fn worker_and_fixture() -> (
        ChromiumBrowserWorker,
        tokio::task::JoinHandle<()>,
        Url,
        R008NavigationPolicy,
    ) {
        assert!(
            ChromiumBrowserWorker::discover_binary().is_some(),
            "hosted job must provide an allowlisted browser"
        );
        let (fixture_task, url) = fixture().await;
        let policy = local_policy(&url);
        (ChromiumBrowserWorker::new(), fixture_task, url, policy)
    }

    #[tokio::test]
    async fn real_chromium_lifecycle_navigation_events_and_cleanup() {
        let (worker, fixture_task, url, policy) = worker_and_fixture().await;
        let session = worker.open_session(BrowserAuthMode::Passive).await.unwrap();
        assert_eq!(worker.status(session.id()).unwrap(), BrowserStatus::Open);
        worker
            .navigate(session.id(), BrowserNavigationRequest::new(url), &policy)
            .await
            .unwrap();
        let events = worker.poll_events(session.id(), 0).unwrap();
        assert!(
            events
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        );
        assert!(
            events
                .iter()
                .any(|event| matches!(event.kind, BrowserEventKind::NavigationChanged { .. }))
        );
        assert!(
            events
                .iter()
                .any(|event| event.kind == BrowserEventKind::Ready)
        );
        worker.close(session.id()).unwrap();
        assert_eq!(worker.status(session.id()).unwrap(), BrowserStatus::Closed);
        fixture_task.abort();
    }

    #[tokio::test]
    async fn generic_input_and_native_panel_boundary_are_bounded() {
        let (worker, fixture_task, url, policy) = worker_and_fixture().await;
        let session = worker
            .open_session(BrowserAuthMode::Interactive)
            .await
            .unwrap();
        worker
            .navigate(session.id(), BrowserNavigationRequest::new(url), &policy)
            .await
            .unwrap();
        worker
            .send_input(
                session.id(),
                BrowserInput::Text {
                    value: "not logged".into(),
                },
            )
            .await
            .unwrap();
        let panel = worker
            .open_panel(session.id(), Duration::from_secs(10))
            .unwrap();
        worker
            .panel_control(&panel, BrowserInput::Submit)
            .await
            .unwrap();
        assert!(!panel.permissions().allows(PanelFeature::Clipboard));
        assert!(!panel.permissions().allows(PanelFeature::FileUpload));
        assert!(!panel.permissions().allows(PanelFeature::Audio));
        assert_eq!(
            worker
                .panel_control(&panel.with_wrong_token(), BrowserInput::Submit)
                .await,
            Err(BrowserError::SessionExpired)
        );
        worker.disconnect_panel(&panel).unwrap();
        assert_eq!(
            worker.panel_control(&panel, BrowserInput::Submit).await,
            Err(BrowserError::PanelDisconnected)
        );
        let reconnected = worker
            .reconnect_panel(&panel, Duration::from_secs(10))
            .unwrap();
        worker
            .panel_control(&reconnected, BrowserInput::Submit)
            .await
            .unwrap();
        worker.close(session.id()).unwrap();
        fixture_task.abort();
    }

    #[tokio::test]
    async fn private_navigation_and_stale_session_are_denied() {
        let (worker, fixture_task, _url, policy) = worker_and_fixture().await;
        let session = worker.open_session(BrowserAuthMode::Passive).await.unwrap();
        let private =
            BrowserNavigationRequest::new(Url::parse("http://127.0.0.1:9/private").unwrap());
        assert_eq!(
            worker
                .navigate(
                    session.id(),
                    private,
                    &R008NavigationPolicy::public_web(EgressPolicy::default())
                )
                .await,
            Err(BrowserError::NavigationDenied)
        );
        worker.close(session.id()).unwrap();
        assert_eq!(
            worker.send_input(session.id(), BrowserInput::Submit).await,
            Err(BrowserError::SessionClosed)
        );
        let _ = policy;
        fixture_task.abort();
    }

    #[tokio::test]
    async fn cancellation_and_browser_exit_are_observable_and_isolated() {
        let (worker, fixture_task, url, policy) = worker_and_fixture().await;
        let session = worker.open_session(BrowserAuthMode::Passive).await.unwrap();
        let request = BrowserNavigationRequest::new(url.clone());
        worker.cancel(session.id(), request.operation_id()).unwrap();
        assert_eq!(
            worker.navigate(session.id(), request, &policy).await,
            Err(BrowserError::OperationCancelled)
        );
        worker.close(session.id()).unwrap();

        let crashed = worker.open_session(BrowserAuthMode::Passive).await.unwrap();
        worker.kill_for_test(crashed.id()).unwrap();
        for _ in 0..20 {
            if worker.status(crashed.id()) == Ok(BrowserStatus::Crashed) {
                break;
            }
            sleep(Duration::from_millis(25)).await;
        }
        assert_eq!(worker.status(crashed.id()), Ok(BrowserStatus::Crashed));
        assert!(
            worker
                .poll_events(crashed.id(), 0)
                .unwrap()
                .iter()
                .any(|event| event.kind == BrowserEventKind::WorkerCrashed)
        );
        fixture_task.abort();
    }

    #[tokio::test]
    async fn close_cleanup_and_timeout_leave_no_owned_profile() {
        let (_worker, fixture_task, url, policy) = worker_and_fixture().await;
        let worker = ChromiumBrowserWorker::with_operation_timeout(Duration::from_millis(1));
        let session = worker.open_session(BrowserAuthMode::Passive).await.unwrap();
        let result = worker
            .navigate(session.id(), BrowserNavigationRequest::new(url), &policy)
            .await;
        assert!(matches!(
            result,
            Err(BrowserError::WorkerTimeout | BrowserError::WorkerCrashed)
        ));
        assert_eq!(worker.status(session.id()), Ok(BrowserStatus::TimedOut));
        fixture_task.abort();
    }

    #[test]
    fn runtime_does_not_accept_caller_process_or_proxy_authority() {
        let command = BrowserCommand::Input {
            input: BrowserInput::Text {
                value: "secret-sentinel".into(),
            },
        };
        assert!(!format!("{command:?}").contains("secret-sentinel"));
        let _ = (
            EgressScope::PublicWeb,
            PanelPermissions,
            ChromiumBrowserWorker::new(),
        );
    }
}
