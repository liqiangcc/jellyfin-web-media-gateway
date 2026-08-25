use crate::playback::{Command, CommandEnvelope, CommandError, PlaybackSession, PlaybackState};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
#[cfg(test)]
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
#[cfg(test)]
use uuid::Uuid;

pub const MAX_CONTROL_BODY_BYTES: usize = 32 * 1024;
const DEFAULT_EVENT_LIMIT: usize = 256;
const MAX_REQUEST_ID_BYTES: usize = 128;
const MAX_DISPLAY_ID_BYTES: usize = 128;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ControlItemSnapshot {
    pub item_id: String,
    pub item_revision: u64,
    pub media_generation: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ControlDisplaySnapshot {
    pub display_id: String,
    pub generation: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ControlHandoffSnapshot {
    pub transition_id: u64,
    pub item_id: String,
    pub item_revision: u64,
    pub from_display_id: String,
    pub from_generation: u64,
    pub target_display_id: String,
    pub candidate_generation: u64,
    pub candidate_position_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ControlSnapshot {
    pub session_id: String,
    pub session_revision: u64,
    pub state: &'static str,
    pub current_item: ControlItemSnapshot,
    pub position_ms: u64,
    pub telemetry_sequence: u64,
    pub active_display: ControlDisplaySnapshot,
    pub handoff: Option<ControlHandoffSnapshot>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControlEventKind {
    CommandAccepted,
    PositionTelemetry,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ControlEvent {
    pub cursor: u64,
    pub kind: ControlEventKind,
    pub snapshot: ControlSnapshot,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ControlEventsResponse {
    pub session_id: String,
    pub cursor: u64,
    pub events: Vec<ControlEvent>,
    pub snapshot_required: bool,
    pub reason: Option<&'static str>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ControlCommandResponse {
    pub request_id: String,
    pub status: &'static str,
    pub session_revision: u64,
    pub snapshot: ControlSnapshot,
    pub event_cursor: u64,
}

/// A display-owned position observation. The surrounding session and
/// generation checks are performed by the Web Display lease authority before
/// this reaches R007.
pub struct DisplayPositionTelemetry<'a> {
    pub display_id: &'a str,
    pub display_generation: u64,
    pub item_id: &'a str,
    pub item_revision: u64,
    pub telemetry_sequence: u64,
    pub position_ms: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ControlErrorResponse {
    pub code: &'static str,
    pub message: &'static str,
    pub current_revision: Option<u64>,
    pub transition_id: Option<u64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlCommandRequest {
    pub request_id: String,
    pub expected_session_revision: Option<u64>,
    pub command: ControlCommand,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlCommand {
    Play,
    Pause,
    Seek { position_ms: u64 },
    Stop,
    BeginHandoff { target_display_id: String },
}

impl ControlCommand {
    fn validate(&self) -> Result<(), ControlValidationError> {
        match self {
            Self::Seek { .. } | Self::Play | Self::Pause | Self::Stop => Ok(()),
            Self::BeginHandoff { target_display_id } => {
                validate_bounded_identifier(target_display_id, MAX_DISPLAY_ID_BYTES, "display_id")
            }
        }
    }

    fn into_playback(self) -> Command {
        match self {
            Self::Play => Command::Play,
            Self::Pause => Command::Pause,
            Self::Seek { position_ms } => Command::Seek(position_ms),
            Self::Stop => Command::Stop,
            Self::BeginHandoff { target_display_id } => Command::BeginHandoff { target_display_id },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlValidationError {
    RequestIdTooLong,
    RequestIdInvalid,
    DisplayIdTooLong,
    DisplayIdInvalid,
}

impl ControlValidationError {
    pub fn response(&self) -> ControlErrorResponse {
        match self {
            Self::RequestIdTooLong => ControlErrorResponse {
                code: "REQUEST_ID_TOO_LONG",
                message: "request_id exceeds the maximum length",
                current_revision: None,
                transition_id: None,
            },
            Self::RequestIdInvalid => ControlErrorResponse {
                code: "REQUEST_ID_INVALID",
                message: "request_id contains an unsupported character",
                current_revision: None,
                transition_id: None,
            },
            Self::DisplayIdTooLong => ControlErrorResponse {
                code: "DISPLAY_ID_TOO_LONG",
                message: "display_id exceeds the maximum length",
                current_revision: None,
                transition_id: None,
            },
            Self::DisplayIdInvalid => ControlErrorResponse {
                code: "DISPLAY_ID_INVALID",
                message: "display_id contains an unsupported character",
                current_revision: None,
                transition_id: None,
            },
        }
    }
}

fn validate_bounded_identifier(
    value: &str,
    max_bytes: usize,
    field: &str,
) -> Result<(), ControlValidationError> {
    if value.is_empty() {
        return Err(if field == "display_id" {
            ControlValidationError::DisplayIdInvalid
        } else {
            ControlValidationError::RequestIdInvalid
        });
    }
    if value.len() > max_bytes {
        return Err(if field == "display_id" {
            ControlValidationError::DisplayIdTooLong
        } else {
            ControlValidationError::RequestIdTooLong
        });
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || ".:_-".contains(character))
    {
        return Err(if field == "display_id" {
            ControlValidationError::DisplayIdInvalid
        } else {
            ControlValidationError::RequestIdInvalid
        });
    }
    Ok(())
}

impl ControlCommandRequest {
    fn validate(&self) -> Result<(), ControlValidationError> {
        validate_bounded_identifier(&self.request_id, MAX_REQUEST_ID_BYTES, "request_id")?;
        self.command.validate()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlLookupError {
    NotFound,
}

#[derive(Debug)]
struct SessionRecord {
    playback: PlaybackSession,
    next_cursor: u64,
    events: VecDeque<ControlEvent>,
}

#[derive(Clone, Debug)]
pub struct ControlService {
    sessions: Arc<RwLock<HashMap<String, Arc<Mutex<SessionRecord>>>>>,
    #[cfg(test)]
    next_session_id: Arc<AtomicU64>,
    event_limit: usize,
}

impl Default for ControlService {
    fn default() -> Self {
        Self::new(DEFAULT_EVENT_LIMIT)
    }
}

impl ControlService {
    pub fn new(event_limit: usize) -> Self {
        assert!(
            event_limit > 0,
            "event journal must retain at least one event"
        );
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(test)]
            next_session_id: Arc::new(AtomicU64::new(1)),
            event_limit,
        }
    }

    /// Trusted service/test hook. Production callers receive opaque IDs from
    /// the service; no HTTP route accepts caller-selected media or sessions.
    #[cfg(test)]
    pub(crate) fn seed_test_session(
        &self,
        item_id: impl Into<String>,
        resolved_media: impl Into<String>,
        display_id: impl Into<String>,
    ) -> String {
        let session_id = loop {
            let sequence = self.next_session_id.fetch_add(1, Ordering::Relaxed);
            let candidate = format!("s-{}-{}", sequence, Uuid::new_v4().simple());
            if !self
                .sessions
                .read()
                .expect("control sessions poisoned")
                .contains_key(&candidate)
            {
                break candidate;
            }
        };
        let record = SessionRecord {
            playback: PlaybackSession::new(item_id, resolved_media, display_id),
            next_cursor: 0,
            events: VecDeque::with_capacity(self.event_limit),
        };
        self.sessions
            .write()
            .expect("control sessions poisoned")
            .insert(session_id.clone(), Arc::new(Mutex::new(record)));
        session_id
    }

    fn session(&self, session_id: &str) -> Result<Arc<Mutex<SessionRecord>>, ControlLookupError> {
        self.sessions
            .read()
            .expect("control sessions poisoned")
            .get(session_id)
            .cloned()
            .ok_or(ControlLookupError::NotFound)
    }

    pub fn snapshot(&self, session_id: &str) -> Result<ControlSnapshot, ControlLookupError> {
        let session = self.session(session_id)?;
        let record = session.lock().expect("control session poisoned");
        Ok(snapshot_from_playback(session_id, &record.playback))
    }

    pub fn events_after(
        &self,
        session_id: &str,
        after: u64,
    ) -> Result<ControlEventsResponse, ControlLookupError> {
        let session = self.session(session_id)?;
        let record = session.lock().expect("control session poisoned");
        let oldest = record
            .events
            .front()
            .map_or(record.next_cursor.saturating_add(1), |event| event.cursor);
        let (snapshot_required, reason) = if after > record.next_cursor {
            (true, Some("cursor_unknown"))
        } else if after.saturating_add(1) < oldest {
            (true, Some("cursor_expired"))
        } else {
            (false, None)
        };
        Ok(ControlEventsResponse {
            session_id: session_id.to_owned(),
            cursor: record.next_cursor,
            events: if snapshot_required {
                Vec::new()
            } else {
                record
                    .events
                    .iter()
                    .filter(|event| event.cursor > after)
                    .cloned()
                    .collect()
            },
            snapshot_required,
            reason,
        })
    }

    pub fn execute_command(
        &self,
        session_id: &str,
        request: ControlCommandRequest,
    ) -> Result<ControlCommandResponse, ControlCommandError> {
        request
            .validate()
            .map_err(ControlCommandError::Validation)?;
        let session = self
            .session(session_id)
            .map_err(|_| ControlCommandError::NotFound)?;
        let mut record = session.lock().expect("control session poisoned");
        let is_replay = record.playback.has_request_id(&request.request_id);
        let envelope = CommandEnvelope {
            request_id: request.request_id.clone(),
            expected_session_revision: request.expected_session_revision,
            command: request.command.into_playback(),
        };
        let result = record
            .playback
            .execute(envelope)
            .map_err(ControlCommandError::Playback)?;
        if !is_replay {
            append_event(
                &mut record,
                session_id,
                ControlEventKind::CommandAccepted,
                self.event_limit,
            );
        }
        let snapshot = snapshot_from_playback(session_id, &record.playback);
        Ok(ControlCommandResponse {
            request_id: result.request_id,
            status: "accepted",
            session_revision: result.session_revision,
            snapshot,
            event_cursor: record.next_cursor,
        })
    }

    /// Trusted/internal telemetry hook. Invalid or stale callbacks are
    /// rejected by the R007 PlaybackSession and do not create events.
    pub fn apply_position_telemetry(
        &self,
        session_id: &str,
        item_id: &str,
        item_revision: u64,
        telemetry_sequence: u64,
        position_ms: u64,
    ) -> Result<bool, ControlLookupError> {
        let session = self.session(session_id)?;
        let mut record = session.lock().expect("control session poisoned");
        let accepted = record.playback.apply_position_callback(
            item_id,
            item_revision,
            telemetry_sequence,
            position_ms,
        );
        if accepted {
            append_event(
                &mut record,
                session_id,
                ControlEventKind::PositionTelemetry,
                self.event_limit,
            );
        }
        Ok(accepted)
    }

    /// Trusted display callback hook. R007 remains the authority for active
    /// display identity and generation; this method only forwards a bounded
    /// observation to that authority.
    pub fn apply_display_position_telemetry(
        &self,
        session_id: &str,
        telemetry: DisplayPositionTelemetry<'_>,
    ) -> Result<bool, ControlLookupError> {
        let session = self.session(session_id)?;
        let mut record = session.lock().expect("control session poisoned");
        let accepted = record.playback.apply_display_position_callback(
            telemetry.display_id,
            telemetry.display_generation,
            telemetry.item_id,
            telemetry.item_revision,
            telemetry.telemetry_sequence,
            telemetry.position_ms,
        );
        if accepted {
            append_event(
                &mut record,
                session_id,
                ControlEventKind::PositionTelemetry,
                self.event_limit,
            );
        }
        Ok(accepted)
    }

    /// Trusted handoff-candidate callback hook. It records only the candidate
    /// observation in R007; it cannot commit `active_display`.
    pub fn apply_candidate_position_telemetry(
        &self,
        session_id: &str,
        telemetry: DisplayPositionTelemetry<'_>,
    ) -> Result<bool, ControlLookupError> {
        let session = self.session(session_id)?;
        let mut record = session.lock().expect("control session poisoned");
        let Some(ticket) = record.playback.active_handoff().cloned() else {
            return Ok(false);
        };
        if ticket.target_display_id != telemetry.display_id
            || ticket.candidate_generation != telemetry.display_generation
        {
            return Ok(false);
        }
        let accepted = telemetry.item_id == ticket.item_id
            && telemetry.item_revision == ticket.item_revision
            && telemetry.telemetry_sequence > record.playback.telemetry_sequence();
        if accepted {
            record
                .playback
                .apply_candidate_callback(&ticket, telemetry.position_ms);
        }
        if accepted {
            append_event(
                &mut record,
                session_id,
                ControlEventKind::PositionTelemetry,
                self.event_limit,
            );
        }
        Ok(accepted)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlCommandError {
    NotFound,
    Validation(ControlValidationError),
    Playback(CommandError),
}

impl ControlCommandError {
    pub fn response(&self) -> ControlErrorResponse {
        match self {
            Self::NotFound => ControlErrorResponse {
                code: "SESSION_NOT_FOUND",
                message: "session was not found",
                current_revision: None,
                transition_id: None,
            },
            Self::Validation(error) => error.response(),
            Self::Playback(CommandError::RevisionConflict { current_revision }) => {
                ControlErrorResponse {
                    code: "REVISION_CONFLICT",
                    message: "expected session revision is stale",
                    current_revision: Some(*current_revision),
                    transition_id: None,
                }
            }
            Self::Playback(CommandError::RequestIdMismatch) => ControlErrorResponse {
                code: "REQUEST_ID_MISMATCH",
                message: "request_id was reused with a different command envelope",
                current_revision: None,
                transition_id: None,
            },
            Self::Playback(CommandError::HandoffInProgress { transition_id }) => {
                ControlErrorResponse {
                    code: "HANDOFF_IN_PROGRESS",
                    message: "another handoff is already in progress",
                    current_revision: None,
                    transition_id: Some(*transition_id),
                }
            }
        }
    }
}

fn append_event(
    record: &mut SessionRecord,
    session_id: &str,
    kind: ControlEventKind,
    event_limit: usize,
) {
    record.next_cursor = record
        .next_cursor
        .checked_add(1)
        .expect("control event cursor overflow");
    record.events.push_back(ControlEvent {
        cursor: record.next_cursor,
        kind,
        snapshot: snapshot_from_playback(session_id, &record.playback),
    });
    while record.events.len() > event_limit {
        record.events.pop_front();
    }
}

fn snapshot_from_playback(session_id: &str, playback: &PlaybackSession) -> ControlSnapshot {
    ControlSnapshot {
        session_id: session_id.to_owned(),
        session_revision: playback.session_revision(),
        state: match playback.state() {
            PlaybackState::Playing => "playing",
            PlaybackState::Paused => "paused",
            PlaybackState::Stopped => "stopped",
        },
        current_item: ControlItemSnapshot {
            item_id: playback.current_item_id().to_owned(),
            item_revision: playback.item_revision(),
            media_generation: playback.media_generation(),
        },
        position_ms: playback.position_ms(),
        telemetry_sequence: playback.telemetry_sequence(),
        active_display: ControlDisplaySnapshot {
            display_id: playback.active_display().display_id.clone(),
            generation: playback.active_display().generation,
        },
        handoff: playback
            .active_handoff()
            .map(|ticket| ControlHandoffSnapshot {
                transition_id: ticket.transition_id,
                item_id: ticket.item_id.clone(),
                item_revision: ticket.item_revision,
                from_display_id: ticket.from_display_id.clone(),
                from_generation: ticket.from_generation,
                target_display_id: ticket.target_display_id.clone(),
                candidate_generation: ticket.candidate_generation,
                candidate_position_ms: playback.candidate_position_ms(),
            }),
    }
}
