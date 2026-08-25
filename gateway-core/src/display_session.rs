//! Generic Web Display registration and page-instance lease authority.
//!
//! This module owns browser-page liveness only. Playback item, session
//! revision, active display, and display-generation authority remain in
//! `PlaybackSession` through `ControlService`.

use crate::control::{
    ControlLookupError, ControlService, ControlSnapshot, DisplayPositionTelemetry,
};
use crate::control_view::{DisplayErrorInput, DisplayViewInput};
use display_adapter_api::{
    DisplayAdapterError, DisplayContext, DisplayInstance, DisplayStatus, PlaybackObservation,
    PositionSample,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const DEFAULT_WEB_DISPLAY_LEASE_TTL: Duration = Duration::from_secs(30);
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_LABEL_BYTES: usize = 128;
const MAX_CAPABILITIES: usize = 32;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayRegistration {
    /// Optional initial attachment. Registration/liveness is valid before a
    /// PlaybackSession exists; context lookup is the attachment boundary.
    pub session_id: Option<String>,
    /// This is a bounded R007 display identity. It is never used to select a
    /// generation; the current Playback snapshot remains authoritative.
    pub display_id: String,
    pub label: String,
    pub capabilities: Vec<String>,
    pub previous_registration_id: Option<String>,
    pub previous_lease_token: Option<String>,
}

impl fmt::Debug for DisplayRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DisplayRegistration")
            .field("session_id", &self.session_id)
            .field("display_id", &self.display_id)
            .field("label", &self.label)
            .field("capabilities", &self.capabilities)
            .field("previous_registration_id", &self.previous_registration_id)
            .field("previous_lease_token", &"[REDACTED]")
            .finish()
    }
}

#[derive(Clone, Serialize)]
pub struct DisplayRegistrationResponse {
    pub registration_id: String,
    pub display_id: String,
    pub lease_token: String,
    pub page_lease_epoch: u64,
    pub lease_ttl_ms: u64,
    pub context: Option<DisplayContextResponse>,
}

impl fmt::Debug for DisplayRegistrationResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DisplayRegistrationResponse")
            .field("registration_id", &self.registration_id)
            .field("display_id", &self.display_id)
            .field("lease_token", &"[REDACTED]")
            .field("page_lease_epoch", &self.page_lease_epoch)
            .field("lease_ttl_ms", &self.lease_ttl_ms)
            .field("context", &self.context)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DisplayHeartbeatResponse {
    pub registration_id: String,
    pub display_id: String,
    pub page_lease_epoch: u64,
    pub lease_ttl_ms: u64,
    pub online: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct LiveDisplayView {
    pub display_id: String,
    pub label: String,
    pub capabilities: Vec<String>,
    pub online: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DisplayContextResponse {
    pub registration_id: String,
    pub session_id: String,
    pub display_id: String,
    pub authority_display_id: String,
    pub item_id: String,
    pub item_revision: u64,
    pub session_revision: u64,
    pub display_generation: u64,
    pub state: String,
    pub position_ms: u64,
    pub telemetry_sequence: u64,
    pub handoff: Option<crate::ControlHandoffSnapshot>,
    pub is_current_display: bool,
    pub is_handoff_candidate: bool,
    pub media_capabilities: Vec<String>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisplayCallback {
    pub lease_token: String,
    pub session_id: String,
    pub item_id: String,
    pub item_revision: u64,
    pub session_revision: u64,
    pub display_id: String,
    pub display_generation: u64,
    pub telemetry_sequence: Option<u64>,
    pub position_ms: Option<u64>,
    pub observation: Option<WebDisplayObservation>,
    pub error_code: Option<WebDisplayErrorCode>,
}

impl fmt::Debug for DisplayCallback {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DisplayCallback")
            .field("lease_token", &"[REDACTED]")
            .field("session_id", &self.session_id)
            .field("item_id", &self.item_id)
            .field("item_revision", &self.item_revision)
            .field("session_revision", &self.session_revision)
            .field("display_id", &self.display_id)
            .field("display_generation", &self.display_generation)
            .field("telemetry_sequence", &self.telemetry_sequence)
            .field("position_ms", &self.position_ms)
            .field("observation", &self.observation)
            .field("error_code", &self.error_code)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebDisplayObservation {
    Playing,
    Paused,
    Stopped,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WebDisplayErrorCode {
    InvalidConfiguration,
    ServerUnavailable,
    MediaIncompatible,
    CommandRejected,
    PlaybackNotConfirmed,
    Timeout,
    Cancelled,
    ProtocolError,
}

impl WebDisplayErrorCode {
    fn adapter_error(self) -> DisplayAdapterError {
        match self {
            Self::InvalidConfiguration => DisplayAdapterError::InvalidConfiguration,
            Self::ServerUnavailable => DisplayAdapterError::ServerUnavailable,
            Self::MediaIncompatible => DisplayAdapterError::MediaIncompatible,
            Self::CommandRejected => DisplayAdapterError::CommandRejected,
            Self::PlaybackNotConfirmed => {
                DisplayAdapterError::PlaybackNotConfirmed { timeout_ms: 0 }
            }
            Self::Timeout => DisplayAdapterError::Timeout,
            Self::Cancelled => DisplayAdapterError::Cancelled,
            // No caller-provided protocol text is accepted or retained.
            Self::ProtocolError => DisplayAdapterError::Protocol("[redacted]".into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct DisplayCallbackResponse {
    pub accepted: bool,
    pub candidate_observation: bool,
    pub telemetry_accepted: bool,
    pub session_revision: u64,
    pub telemetry_sequence: u64,
    pub context: DisplayContextResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct DisplaySessionErrorResponse {
    pub code: &'static str,
    pub message: &'static str,
}

impl From<&DisplaySessionError> for DisplaySessionErrorResponse {
    fn from(error: &DisplaySessionError) -> Self {
        Self {
            code: error.code(),
            message: error.message(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplaySessionError {
    SessionNotFound,
    SessionNotAttached,
    InvalidIdentifier(&'static str),
    InvalidLabel,
    InvalidCapabilities,
    RegistrationNotFound,
    LeaseInvalid,
    LeaseExpired,
    AlreadyRegistered,
    StaleContext,
    StaleTelemetry,
    InvalidCallback,
}

impl DisplaySessionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::SessionNotFound => "SESSION_NOT_FOUND",
            Self::SessionNotAttached => "SESSION_NOT_ATTACHED",
            Self::InvalidIdentifier(_) => "IDENTIFIER_INVALID",
            Self::InvalidLabel => "LABEL_INVALID",
            Self::InvalidCapabilities => "CAPABILITIES_INVALID",
            Self::RegistrationNotFound => "REGISTRATION_NOT_FOUND",
            Self::LeaseInvalid => "LEASE_INVALID",
            Self::LeaseExpired => "LEASE_EXPIRED",
            Self::AlreadyRegistered => "DISPLAY_ALREADY_REGISTERED",
            Self::StaleContext => "STALE_DISPLAY_CONTEXT",
            Self::StaleTelemetry => "STALE_TELEMETRY",
            Self::InvalidCallback => "INVALID_CALLBACK",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::SessionNotFound => "playback session was not found",
            Self::SessionNotAttached => "display is not attached to a playback session",
            Self::InvalidIdentifier(_) => "identifier is invalid",
            Self::InvalidLabel => "display label is invalid",
            Self::InvalidCapabilities => "display capabilities are invalid",
            Self::RegistrationNotFound => "display registration was not found",
            Self::LeaseInvalid => "display lease is invalid",
            Self::LeaseExpired => "display lease has expired",
            Self::AlreadyRegistered => "display is already registered",
            Self::StaleContext => "display callback context is stale",
            Self::StaleTelemetry => "display telemetry is stale",
            Self::InvalidCallback => "display callback is invalid",
        }
    }
}

impl From<ControlLookupError> for DisplaySessionError {
    fn from(_: ControlLookupError) -> Self {
        Self::SessionNotFound
    }
}

#[derive(Clone, Debug)]
struct DisplayObservationRecord {
    context: DisplayContext,
    observation: WebDisplayObservation,
    position_ms: u64,
    error: Option<WebDisplayErrorCode>,
}

#[derive(Clone)]
struct DisplayRecord {
    registration_id: String,
    display_id: String,
    session_id: Option<String>,
    lease_token: String,
    page_lease_epoch: u64,
    expires_at: Instant,
    label: String,
    capabilities: Vec<String>,
    observation: Option<DisplayObservationRecord>,
}

impl fmt::Debug for DisplayRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DisplayRecord")
            .field("registration_id", &self.registration_id)
            .field("display_id", &self.display_id)
            .field("session_id", &self.session_id)
            .field("lease_token", &"[REDACTED]")
            .field("page_lease_epoch", &self.page_lease_epoch)
            .field("expires_at", &self.expires_at)
            .field("label", &self.label)
            .field("capabilities", &self.capabilities)
            .field("observation", &self.observation)
            .finish()
    }
}

#[derive(Default)]
struct Registry {
    records: HashMap<String, DisplayRecord>,
    registration_by_display: HashMap<String, String>,
}

#[derive(Clone)]
pub struct DisplaySessionService {
    inner: Arc<Mutex<Registry>>,
    lease_ttl: Duration,
}

impl Default for DisplaySessionService {
    fn default() -> Self {
        Self::new(DEFAULT_WEB_DISPLAY_LEASE_TTL)
    }
}

impl DisplaySessionService {
    pub fn new(lease_ttl: Duration) -> Self {
        assert!(!lease_ttl.is_zero(), "display lease TTL must be positive");
        Self {
            inner: Arc::new(Mutex::new(Registry::default())),
            lease_ttl,
        }
    }

    pub fn register(
        &self,
        control: &ControlService,
        input: DisplayRegistration,
    ) -> Result<DisplayRegistrationResponse, DisplaySessionError> {
        self.register_at(control, input, Instant::now())
    }

    /// Validate only the server-owned registration/liveness facts needed by
    /// source-session creation. This deliberately does not accept or expose
    /// a page lease, page epoch, session attachment, or Playback generation.
    pub(crate) fn validate_live_selector(
        &self,
        display_id: &str,
    ) -> Result<(), DisplaySessionError> {
        validate_identifier(display_id, "display_id")?;
        let registry = self.inner.lock().expect("display registry poisoned");
        let registration_id = registry
            .registration_by_display
            .get(display_id)
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        let record = registry
            .records
            .get(registration_id)
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        if record.expires_at <= Instant::now() {
            return Err(DisplaySessionError::LeaseExpired);
        }
        Ok(())
    }

    fn register_at(
        &self,
        control: &ControlService,
        input: DisplayRegistration,
        now: Instant,
    ) -> Result<DisplayRegistrationResponse, DisplaySessionError> {
        if let Some(session_id) = input.session_id.as_deref() {
            validate_identifier(session_id, "session_id")?;
            // An optional initial attachment is checked, but registration
            // itself does not depend on a PlaybackSession existing.
            control.snapshot(session_id)?;
        }
        validate_identifier(&input.display_id, "display_id")?;
        validate_label(&input.label)?;
        validate_capabilities(&input.capabilities)?;
        let mut registry = self.inner.lock().expect("display registry poisoned");
        let existing_registration = registry
            .registration_by_display
            .get(&input.display_id)
            .cloned();

        let (registration_id, page_lease_epoch, record) =
            if let Some(previous_id) = input.previous_registration_id.as_deref() {
                let Some(existing) = registry.records.get_mut(previous_id) else {
                    return Err(DisplaySessionError::RegistrationNotFound);
                };
                if input.previous_lease_token.as_deref() != Some(existing.lease_token.as_str()) {
                    return Err(DisplaySessionError::LeaseInvalid);
                }
                if existing.expires_at <= now {
                    return Err(DisplaySessionError::LeaseExpired);
                }
                if (input.session_id.is_some() && existing.session_id != input.session_id)
                    || existing.display_id != input.display_id
                {
                    return Err(DisplaySessionError::StaleContext);
                }
                existing.lease_token = new_token();
                existing.page_lease_epoch = existing
                    .page_lease_epoch
                    .checked_add(1)
                    .expect("page lease epoch overflow");
                existing.expires_at = now + self.lease_ttl;
                existing.label = input.label;
                existing.capabilities = safe_capabilities(&input.capabilities);
                if input.session_id.is_some() {
                    existing.session_id = input.session_id;
                }
                (
                    existing.registration_id.clone(),
                    existing.page_lease_epoch,
                    existing.clone(),
                )
            } else {
                if let Some(existing_id) = existing_registration {
                    if registry
                        .records
                        .get(&existing_id)
                        .is_some_and(|record| record.expires_at > now)
                    {
                        return Err(DisplaySessionError::AlreadyRegistered);
                    }
                    registry.records.remove(&existing_id);
                    registry.registration_by_display.remove(&input.display_id);
                }
                let registration_id = new_token();
                let record = DisplayRecord {
                    registration_id: registration_id.clone(),
                    display_id: input.display_id.clone(),
                    session_id: input.session_id,
                    lease_token: new_token(),
                    page_lease_epoch: 1,
                    expires_at: now + self.lease_ttl,
                    label: input.label,
                    capabilities: safe_capabilities(&input.capabilities),
                    observation: None,
                };
                registry
                    .registration_by_display
                    .insert(input.display_id, registration_id.clone());
                registry
                    .records
                    .insert(registration_id.clone(), record.clone());
                (registration_id, 1, record)
            };

        let response = DisplayRegistrationResponse {
            registration_id: registration_id.clone(),
            display_id: record.display_id.clone(),
            lease_token: record.lease_token.clone(),
            page_lease_epoch,
            lease_ttl_ms: self.ttl_ms(),
            context: record
                .session_id
                .as_deref()
                .map(|session_id| {
                    control
                        .snapshot(session_id)
                        .map(|snapshot| context_response(&record, &snapshot))
                })
                .transpose()?,
        };
        Ok(response)
    }

    pub fn heartbeat(
        &self,
        display_id: &str,
        lease_token: &str,
    ) -> Result<DisplayHeartbeatResponse, DisplaySessionError> {
        self.heartbeat_at(display_id, lease_token, Instant::now())
    }

    /// Return only bounded selector metadata for live Web Displays. Lease
    /// tokens, page epochs and Playback generations are deliberately absent.
    pub fn live_displays(&self) -> Vec<LiveDisplayView> {
        let now = Instant::now();
        let registry = self.inner.lock().expect("display registry poisoned");
        let mut displays: Vec<_> = registry
            .records
            .values()
            .filter(|record| record.expires_at > now)
            .map(|record| LiveDisplayView {
                display_id: record.display_id.clone(),
                label: record.label.clone(),
                capabilities: record.capabilities.clone(),
                online: true,
            })
            .collect();
        displays.sort_by(|left, right| left.display_id.cmp(&right.display_id));
        displays
    }

    fn heartbeat_at(
        &self,
        display_id: &str,
        lease_token: &str,
        now: Instant,
    ) -> Result<DisplayHeartbeatResponse, DisplaySessionError> {
        validate_identifier(display_id, "display_id")?;
        let mut registry = self.inner.lock().expect("display registry poisoned");
        let registration_id = registry
            .registration_by_display
            .get(display_id)
            .cloned()
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        let record = registry
            .records
            .get_mut(&registration_id)
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        validate_lease(record, lease_token, now)?;
        record.expires_at = now + self.lease_ttl;
        Ok(DisplayHeartbeatResponse {
            registration_id: record.registration_id.clone(),
            display_id: record.display_id.clone(),
            page_lease_epoch: record.page_lease_epoch,
            lease_ttl_ms: self.ttl_ms(),
            online: true,
        })
    }

    pub fn context(
        &self,
        control: &ControlService,
        display_id: &str,
        lease_token: &str,
    ) -> Result<DisplayContextResponse, DisplaySessionError> {
        self.context_for_session(control, display_id, lease_token, None)
    }

    /// Read the current Playback-owned context and, when a session is
    /// supplied, attach this page instance to that existing session. The
    /// supplied ID is only a lookup key; item/display generations always come
    /// from the accepted Playback snapshot.
    pub fn context_for_session(
        &self,
        control: &ControlService,
        display_id: &str,
        lease_token: &str,
        session_id: Option<&str>,
    ) -> Result<DisplayContextResponse, DisplaySessionError> {
        self.context_at(control, display_id, lease_token, session_id, Instant::now())
    }

    /// Attach a live page to the session currently authoritative for its
    /// display. The session and display facts still come from Playback; the
    /// registry only records the page's server-side attachment for callbacks.
    pub fn context_for_active_display(
        &self,
        control: &ControlService,
        display_id: &str,
        lease_token: &str,
    ) -> Result<DisplayContextResponse, DisplaySessionError> {
        validate_identifier(display_id, "display_id")?;
        let snapshot = control
            .snapshot_for_active_display(display_id)
            .map_err(DisplaySessionError::from)?;
        self.context_for_session(control, display_id, lease_token, Some(&snapshot.session_id))
    }

    fn context_at(
        &self,
        control: &ControlService,
        display_id: &str,
        lease_token: &str,
        session_id: Option<&str>,
        now: Instant,
    ) -> Result<DisplayContextResponse, DisplaySessionError> {
        validate_identifier(display_id, "display_id")?;
        if let Some(session_id) = session_id {
            validate_identifier(session_id, "session_id")?;
        }
        let mut registry = self.inner.lock().expect("display registry poisoned");
        let registration_id = registry
            .registration_by_display
            .get(display_id)
            .cloned()
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        let record = registry
            .records
            .get_mut(&registration_id)
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        validate_lease(record, lease_token, now)?;
        let session_id = session_id
            .or(record.session_id.as_deref())
            .ok_or(DisplaySessionError::SessionNotAttached)?;
        let snapshot = control.snapshot(session_id)?;
        if record.session_id.as_deref() != Some(session_id) {
            // This is registry attachment state only. It cannot alter
            // Playback's active display or display generation authority.
            record.session_id = Some(session_id.to_owned());
        }
        Ok(context_response(record, &snapshot))
    }

    pub fn callback(
        &self,
        control: &ControlService,
        callback: DisplayCallback,
    ) -> Result<DisplayCallbackResponse, DisplaySessionError> {
        self.callback_at(control, callback, Instant::now())
    }

    fn callback_at(
        &self,
        control: &ControlService,
        callback: DisplayCallback,
        now: Instant,
    ) -> Result<DisplayCallbackResponse, DisplaySessionError> {
        validate_identifier(&callback.session_id, "session_id")?;
        validate_identifier(&callback.display_id, "display_id")?;
        if callback.position_ms.is_some() != callback.telemetry_sequence.is_some() {
            return Err(DisplaySessionError::InvalidCallback);
        }
        if callback.observation.is_none()
            && callback.error_code.is_none()
            && callback.position_ms.is_none()
        {
            return Err(DisplaySessionError::InvalidCallback);
        }

        let mut registry = self.inner.lock().expect("display registry poisoned");
        let registration_id = registry
            .registration_by_display
            .get(&callback.display_id)
            .cloned()
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        let record = registry
            .records
            .get_mut(&registration_id)
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        validate_lease(record, &callback.lease_token, now)?;
        if record.session_id.as_deref() != Some(callback.session_id.as_str())
            || record.display_id != callback.display_id
        {
            return Err(DisplaySessionError::StaleContext);
        }

        // Keep the registry lock while checking and applying the R007 callback.
        // A reconnect therefore cannot supersede the lease between validation
        // and the side effect.
        let session_id = record
            .session_id
            .as_deref()
            .ok_or(DisplaySessionError::SessionNotAttached)?;
        let snapshot = control.snapshot(session_id)?;
        // `session_revision` is command-CAS freshness, not display telemetry
        // freshness. A callback may carry an older informational revision
        // after a same-item command; item/display/lease identity below is the
        // authoritative callback boundary.
        if snapshot.current_item.item_id != callback.item_id
            || snapshot.current_item.item_revision != callback.item_revision
        {
            return Err(DisplaySessionError::StaleContext);
        }

        let is_current = snapshot.active_display.display_id == callback.display_id
            && snapshot.active_display.generation == callback.display_generation;
        let is_candidate = snapshot.handoff.as_ref().is_some_and(|handoff| {
            handoff.target_display_id == callback.display_id
                && handoff.candidate_generation == callback.display_generation
        });
        if !is_current && !is_candidate {
            return Err(DisplaySessionError::StaleContext);
        }

        let telemetry_accepted = if let (Some(sequence), Some(position_ms)) =
            (callback.telemetry_sequence, callback.position_ms)
        {
            let accepted = if is_current {
                control.apply_display_position_telemetry(
                    session_id,
                    DisplayPositionTelemetry {
                        display_id: &callback.display_id,
                        display_generation: callback.display_generation,
                        item_id: &callback.item_id,
                        item_revision: callback.item_revision,
                        telemetry_sequence: sequence,
                        position_ms,
                    },
                )?
            } else {
                control.apply_candidate_position_telemetry(
                    session_id,
                    DisplayPositionTelemetry {
                        display_id: &callback.display_id,
                        display_generation: callback.display_generation,
                        item_id: &callback.item_id,
                        item_revision: callback.item_revision,
                        telemetry_sequence: sequence,
                        position_ms,
                    },
                )?
            };
            if !accepted {
                return Err(DisplaySessionError::StaleTelemetry);
            }
            true
        } else {
            false
        };

        let observation = callback
            .observation
            .unwrap_or(WebDisplayObservation::Unknown);
        let error = callback.error_code;
        let context = DisplayContext::new(
            callback.session_id.clone(),
            callback.item_id.clone(),
            callback.item_revision,
            callback.display_id.clone(),
            callback.display_generation,
        );
        record.observation = Some(DisplayObservationRecord {
            context,
            observation,
            position_ms: callback.position_ms.unwrap_or(snapshot.position_ms),
            error,
        });
        let current = control.snapshot(record.session_id.as_deref().expect("validated session"))?;
        Ok(DisplayCallbackResponse {
            accepted: true,
            candidate_observation: is_candidate,
            telemetry_accepted,
            session_revision: current.session_revision,
            telemetry_sequence: current.telemetry_sequence,
            context: context_response(record, &current),
        })
    }

    /// Safe generic facts for accepted #40 projection. The projection itself
    /// still filters instance/status/error by the R007 active context.
    pub fn display_view_input(
        &self,
        control: &ControlService,
        display_id: &str,
    ) -> Result<DisplayViewInput, DisplaySessionError> {
        self.display_view_input_at(control, display_id, Instant::now())
    }

    fn display_view_input_at(
        &self,
        control: &ControlService,
        display_id: &str,
        now: Instant,
    ) -> Result<DisplayViewInput, DisplaySessionError> {
        validate_identifier(display_id, "display_id")?;
        let registry = self.inner.lock().expect("display registry poisoned");
        let registration_id = registry
            .registration_by_display
            .get(display_id)
            .cloned()
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        let record = registry
            .records
            .get(&registration_id)
            .ok_or(DisplaySessionError::RegistrationNotFound)?;
        let session_id = record
            .session_id
            .as_deref()
            .ok_or(DisplaySessionError::SessionNotAttached)?;
        let snapshot = control.snapshot(session_id)?;
        let active = snapshot.active_display.display_id == record.display_id;
        let instance = active.then(|| DisplayInstance {
            id: record.display_id.clone(),
            adapter_type: "web".into(),
            label: record.label.clone(),
            online: record.expires_at > now,
            capabilities: record.capabilities.clone(),
        });
        let (status, error) = match record.observation.as_ref() {
            Some(observation) if active => (
                Some(DisplayStatus {
                    context: observation.context.clone(),
                    observation: observation.observation.into(),
                    position: PositionSample {
                        requested_ms: None,
                        reported_ms: observation.position_ms,
                        error_ms: None,
                    },
                }),
                observation.error.map(|error| {
                    DisplayErrorInput::new(observation.context.clone(), error.adapter_error())
                }),
            ),
            _ => (None, None),
        };
        Ok(DisplayViewInput {
            instance,
            status,
            error,
        })
    }

    fn ttl_ms(&self) -> u64 {
        self.lease_ttl.as_millis().try_into().unwrap_or(u64::MAX)
    }
}

impl From<WebDisplayObservation> for PlaybackObservation {
    fn from(value: WebDisplayObservation) -> Self {
        match value {
            WebDisplayObservation::Playing => Self::Playing,
            WebDisplayObservation::Paused => Self::Paused,
            WebDisplayObservation::Stopped => Self::Stopped,
            WebDisplayObservation::Unknown => Self::Unknown,
        }
    }
}

fn context_response(record: &DisplayRecord, snapshot: &ControlSnapshot) -> DisplayContextResponse {
    let is_current_display = snapshot.active_display.display_id == record.display_id;
    let is_handoff_candidate = snapshot
        .handoff
        .as_ref()
        .is_some_and(|handoff| handoff.target_display_id == record.display_id);
    let display_generation = if is_current_display {
        snapshot.active_display.generation
    } else if is_handoff_candidate {
        snapshot
            .handoff
            .as_ref()
            .expect("handoff candidate checked")
            .candidate_generation
    } else {
        snapshot.active_display.generation
    };
    DisplayContextResponse {
        registration_id: record.registration_id.clone(),
        session_id: snapshot.session_id.clone(),
        display_id: record.display_id.clone(),
        authority_display_id: snapshot.active_display.display_id.clone(),
        item_id: snapshot.current_item.item_id.clone(),
        item_revision: snapshot.current_item.item_revision,
        session_revision: snapshot.session_revision,
        display_generation,
        state: snapshot.state.to_owned(),
        position_ms: snapshot.position_ms,
        telemetry_sequence: snapshot.telemetry_sequence,
        handoff: snapshot.handoff.clone(),
        is_current_display,
        is_handoff_candidate,
        media_capabilities: record.capabilities.clone(),
    }
}

fn validate_lease(
    record: &DisplayRecord,
    lease_token: &str,
    now: Instant,
) -> Result<(), DisplaySessionError> {
    if record.lease_token != lease_token {
        return Err(DisplaySessionError::LeaseInvalid);
    }
    if record.expires_at <= now {
        return Err(DisplaySessionError::LeaseExpired);
    }
    Ok(())
}

fn validate_identifier(value: &str, field: &'static str) -> Result<(), DisplaySessionError> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_BYTES
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
    {
        return Err(DisplaySessionError::InvalidIdentifier(field));
    }
    Ok(())
}

fn validate_label(value: &str) -> Result<(), DisplaySessionError> {
    if value.is_empty()
        || value.len() > MAX_LABEL_BYTES
        || value.chars().any(|character| character.is_control())
    {
        return Err(DisplaySessionError::InvalidLabel);
    }
    Ok(())
}

fn validate_capabilities(values: &[String]) -> Result<(), DisplaySessionError> {
    if values.len() > MAX_CAPABILITIES
        || values.iter().any(|value| {
            value.is_empty()
                || value.len() > MAX_IDENTIFIER_BYTES
                || !value.chars().all(|character| {
                    character.is_ascii_alphanumeric() || ".:_-/".contains(character)
                })
        })
    {
        return Err(DisplaySessionError::InvalidCapabilities);
    }
    Ok(())
}

fn safe_capabilities(values: &[String]) -> Vec<String> {
    values
        .iter()
        .filter(|value| matches!(value.as_str(), "video" | "audio" | "subtitles"))
        .cloned()
        .collect()
}

fn new_token() -> String {
    Uuid::new_v4().simple().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{ControlCommand, ControlCommandRequest};

    fn registration(session_id: &str, display_id: &str) -> DisplayRegistration {
        DisplayRegistration {
            session_id: Some(session_id.into()),
            display_id: display_id.into(),
            label: "Living room browser".into(),
            capabilities: vec!["video".into(), "audio".into()],
            previous_registration_id: None,
            previous_lease_token: None,
        }
    }

    fn callback(
        lease_token: String,
        context: &DisplayContextResponse,
        observation: Option<WebDisplayObservation>,
        position_ms: Option<u64>,
        telemetry_sequence: Option<u64>,
    ) -> DisplayCallback {
        DisplayCallback {
            lease_token,
            session_id: context.session_id.clone(),
            item_id: context.item_id.clone(),
            item_revision: context.item_revision,
            session_revision: context.session_revision,
            display_id: context.display_id.clone(),
            display_generation: context.display_generation,
            telemetry_sequence,
            position_ms,
            observation,
            error_code: None,
        }
    }

    #[test]
    fn registration_heartbeat_and_context_are_server_owned_and_bounded() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "secret-media", "display-a");
        let service = DisplaySessionService::new(Duration::from_secs(5));
        let now = Instant::now();
        let registered = service
            .register_at(&control, registration(&session, "display-a"), now)
            .unwrap();
        assert_ne!(registered.registration_id, "display-a");
        assert_eq!(registered.page_lease_epoch, 1);
        assert!(
            registered
                .context
                .as_ref()
                .expect("attached context")
                .is_current_display
        );
        assert!(!format!("{registered:?}").contains("secret-media"));

        let heartbeat = service
            .heartbeat_at(
                "display-a",
                &registered.lease_token,
                now + Duration::from_secs(1),
            )
            .unwrap();
        assert_eq!(heartbeat.page_lease_epoch, 1);
        assert_eq!(control.snapshot(&session).unwrap().session_revision, 0);
        let context = service
            .context_at(
                &control,
                "display-a",
                &registered.lease_token,
                None,
                now + Duration::from_secs(1),
            )
            .unwrap();
        assert_eq!(context.display_generation, 1);
        assert!(!format!("{context:?}").contains("secret-media"));
    }

    #[test]
    fn registration_liveness_is_independent_from_playback_attachment() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "media", "display-a");
        let service = DisplaySessionService::default();
        let registered = service
            .register(
                &control,
                DisplayRegistration {
                    session_id: None,
                    display_id: "display-a".into(),
                    label: "Idle browser".into(),
                    capabilities: vec!["video".into(), "audio".into(), "subtitles".into()],
                    previous_registration_id: None,
                    previous_lease_token: None,
                },
            )
            .unwrap();
        assert!(registered.context.is_none());
        assert_eq!(
            service.context(&control, "display-a", &registered.lease_token),
            Err(DisplaySessionError::SessionNotAttached)
        );

        let context = service
            .context_for_session(
                &control,
                "display-a",
                &registered.lease_token,
                Some(&session),
            )
            .unwrap();
        assert_eq!(context.session_id, session);
        assert_eq!(
            context.media_capabilities,
            vec!["video", "audio", "subtitles"]
        );
    }

    #[test]
    fn same_item_callback_accepts_informationally_stale_command_revision() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "media", "display-a");
        let service = DisplaySessionService::default();
        let registered = service
            .register(&control, registration(&session, "display-a"))
            .unwrap();
        let context = registered.context.as_ref().expect("attached context");
        let old_revision = context.session_revision;
        control
            .execute_command(
                &session,
                ControlCommandRequest {
                    request_id: "pause-before-telemetry".into(),
                    expected_session_revision: Some(old_revision),
                    command: ControlCommand::Pause,
                },
            )
            .unwrap();

        let mut callback = callback(
            registered.lease_token,
            context,
            Some(WebDisplayObservation::Paused),
            Some(2_000),
            Some(1),
        );
        callback.session_revision = old_revision;
        assert!(
            service
                .callback(&control, callback)
                .unwrap()
                .telemetry_accepted
        );
    }

    #[test]
    fn candidate_sequence_is_local_and_cannot_change_active_telemetry() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "media", "display-a");
        let service = DisplaySessionService::default();
        let active = service
            .register(&control, registration(&session, "display-a"))
            .unwrap();
        service
            .callback(
                &control,
                callback(
                    active.lease_token,
                    active.context.as_ref().expect("attached context"),
                    Some(WebDisplayObservation::Playing),
                    Some(100_000),
                    Some(100),
                ),
            )
            .unwrap();
        let handoff = control
            .execute_command(
                &session,
                ControlCommandRequest {
                    request_id: "local-candidate-sequence".into(),
                    expected_session_revision: Some(0),
                    command: ControlCommand::BeginHandoff {
                        target_display_id: "display-b".into(),
                    },
                },
            )
            .unwrap();
        let candidate = service
            .register(&control, registration(&session, "display-b"))
            .unwrap();
        let mut context = candidate.context.expect("attached context");
        context.session_revision = handoff.session_revision;
        let result = service
            .callback(
                &control,
                callback(
                    candidate.lease_token,
                    &context,
                    Some(WebDisplayObservation::Playing),
                    Some(101_000),
                    Some(1),
                ),
            )
            .unwrap();
        assert!(result.candidate_observation);
        let snapshot = control.snapshot(&session).unwrap();
        assert_eq!(snapshot.position_ms, 100_000);
        assert_eq!(snapshot.telemetry_sequence, 100);
        assert_eq!(snapshot.active_display.display_id, "display-a");
        assert_eq!(
            snapshot.handoff.unwrap().candidate_position_ms,
            Some(101_000)
        );
    }

    #[test]
    fn reconnect_supersedes_old_lease_and_rejects_old_callback() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "media", "display-a");
        let service = DisplaySessionService::default();
        let now = Instant::now();
        let first = service
            .register_at(&control, registration(&session, "display-a"), now)
            .unwrap();
        let old_context = first.context.clone().expect("attached context");
        let second = service
            .register_at(
                &control,
                DisplayRegistration {
                    previous_registration_id: Some(first.registration_id.clone()),
                    previous_lease_token: Some(first.lease_token.clone()),
                    ..registration(&session, "display-a")
                },
                now + Duration::from_secs(1),
            )
            .unwrap();
        assert_eq!(second.page_lease_epoch, 2);
        let old = service.callback(
            &control,
            callback(
                first.lease_token,
                &old_context,
                Some(WebDisplayObservation::Paused),
                None,
                None,
            ),
        );
        assert_eq!(old, Err(DisplaySessionError::LeaseInvalid));
        assert_eq!(control.snapshot(&session).unwrap().position_ms, 0);
    }

    #[test]
    fn stale_context_matrix_is_rejected_before_side_effects() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "media", "display-a");
        let service = DisplaySessionService::default();
        let registered = service
            .register(&control, registration(&session, "display-a"))
            .unwrap();
        let context = registered.context.expect("attached context");
        let mut wrong_session = callback(
            registered.lease_token.clone(),
            &context,
            Some(WebDisplayObservation::Playing),
            Some(1_000),
            Some(1),
        );
        wrong_session.session_id = "other-session".into();
        assert_eq!(
            service.callback(&control, wrong_session),
            Err(DisplaySessionError::StaleContext)
        );
        for (item_id, item_revision, display_generation, session_revision) in [
            (
                "old-item",
                context.item_revision,
                context.display_generation,
                context.session_revision,
            ),
            (
                context.item_id.as_str(),
                context.item_revision + 1,
                context.display_generation,
                context.session_revision,
            ),
            (
                context.item_id.as_str(),
                context.item_revision,
                context.display_generation + 1,
                context.session_revision,
            ),
        ] {
            let result = service.callback(
                &control,
                DisplayCallback {
                    lease_token: registered.lease_token.clone(),
                    session_id: session.clone(),
                    item_id: item_id.into(),
                    item_revision,
                    session_revision,
                    display_id: "display-a".into(),
                    display_generation,
                    telemetry_sequence: Some(1),
                    position_ms: Some(1_000),
                    observation: Some(WebDisplayObservation::Playing),
                    error_code: None,
                },
            );
            assert_eq!(result, Err(DisplaySessionError::StaleContext));
        }
        assert_eq!(control.snapshot(&session).unwrap().position_ms, 0);
    }

    #[test]
    fn current_position_callback_routes_through_r007_and_candidate_cannot_commit() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "media", "display-a");
        let service = DisplaySessionService::default();
        let registered = service
            .register(&control, registration(&session, "display-a"))
            .unwrap();
        let accepted = service
            .callback(
                &control,
                callback(
                    registered.lease_token.clone(),
                    registered.context.as_ref().expect("attached context"),
                    Some(WebDisplayObservation::Playing),
                    Some(12_000),
                    Some(1),
                ),
            )
            .unwrap();
        assert!(accepted.telemetry_accepted);
        assert_eq!(control.snapshot(&session).unwrap().position_ms, 12_000);
        assert_eq!(control.snapshot(&session).unwrap().session_revision, 0);

        let handoff = control
            .execute_command(
                &session,
                ControlCommandRequest {
                    request_id: "handoff".into(),
                    expected_session_revision: Some(0),
                    command: ControlCommand::BeginHandoff {
                        target_display_id: "display-b".into(),
                    },
                },
            )
            .unwrap();
        let target = service
            .register(&control, registration(&session, "display-b"))
            .unwrap();
        let mut candidate = target.context.expect("attached context");
        candidate.session_revision = handoff.session_revision;
        let candidate_result = service
            .callback(
                &control,
                callback(
                    target.lease_token,
                    &candidate,
                    Some(WebDisplayObservation::Playing),
                    Some(13_000),
                    Some(2),
                ),
            )
            .unwrap();
        assert!(candidate_result.candidate_observation);
        assert_eq!(
            control
                .snapshot(&session)
                .unwrap()
                .active_display
                .display_id,
            "display-a"
        );
        assert!(control.snapshot(&session).unwrap().handoff.is_some());
    }

    #[test]
    fn accepted_control_view_consumes_generic_web_display_facts_without_branching() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "media", "display-a");
        let service = DisplaySessionService::default();
        let registered = service
            .register(&control, registration(&session, "display-a"))
            .unwrap();
        service
            .callback(
                &control,
                callback(
                    registered.lease_token,
                    registered.context.as_ref().expect("attached context"),
                    Some(WebDisplayObservation::Playing),
                    Some(5_000),
                    Some(1),
                ),
            )
            .unwrap();
        let input = crate::control_view::ControlViewInput {
            playback: control.snapshot(&session).unwrap(),
            event_cursor: None,
            site: Default::default(),
            browser: Default::default(),
            display: service.display_view_input(&control, "display-a").unwrap(),
        };
        let view = crate::control_view::ControlView::project(input);
        assert_eq!(view.active_display.adapter_type.as_deref(), Some("web"));
        assert!(view.active_display.online);
        assert_eq!(
            view.active_display.observation,
            Some(crate::control_view::PlaybackObservationView::Playing)
        );
        assert_eq!(view.now_playing.position_ms, 5_000);
    }

    #[test]
    fn expired_lease_is_offline_and_cannot_be_renewed() {
        let control = ControlService::default();
        let session = control.seed_test_session("item-a", "media", "display-a");
        let service = DisplaySessionService::new(Duration::from_secs(1));
        let now = Instant::now();
        let registered = service
            .register_at(&control, registration(&session, "display-a"), now)
            .unwrap();
        assert_eq!(
            service.heartbeat_at(
                "display-a",
                &registered.lease_token,
                now + Duration::from_secs(2)
            ),
            Err(DisplaySessionError::LeaseExpired)
        );
        let facts = service
            .display_view_input_at(&control, "display-a", now + Duration::from_secs(2))
            .unwrap();
        assert!(!facts.instance.unwrap().online);
    }
}
