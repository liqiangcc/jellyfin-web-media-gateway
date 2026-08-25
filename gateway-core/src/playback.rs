use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybackItem {
    pub item_id: String,
    pub item_revision: u64,
    pub resolved_media: String,
    pub media_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayAuthority {
    pub display_id: String,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaRefreshTicket {
    pub item_id: String,
    pub item_revision: u64,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandoffTicket {
    pub transition_id: u64,
    pub item_id: String,
    pub item_revision: u64,
    pub from_display_id: String,
    pub from_generation: u64,
    pub target_display_id: String,
    pub candidate_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Play,
    Pause,
    Seek(u64),
    Stop,
    BeginHandoff { target_display_id: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandEnvelope {
    pub request_id: String,
    pub expected_session_revision: Option<u64>,
    pub command: Command,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandResult {
    pub request_id: String,
    pub session_revision: u64,
    pub transition: Option<HandoffTicket>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandError {
    RevisionConflict { current_revision: u64 },
    RequestIdMismatch,
    HandoffInProgress { transition_id: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RequestFingerprint {
    expected_session_revision: Option<u64>,
    command: Command,
}

#[derive(Clone, Debug)]
struct RequestRecord {
    fingerprint: RequestFingerprint,
    outcome: Result<CommandResult, CommandError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HandoffTransition {
    ticket: HandoffTicket,
    candidate_position_ms: Option<u64>,
    candidate_telemetry_sequence: u64,
}

#[derive(Debug)]
pub struct PlaybackSession {
    session_revision: u64,
    state: PlaybackState,
    current_item: PlaybackItem,
    position_ms: u64,
    telemetry_sequence: u64,
    active_display: DisplayAuthority,
    handoff: Option<HandoffTransition>,
    next_transition_id: u64,
    next_display_generation: u64,
    request_records: HashMap<String, RequestRecord>,
}

impl PlaybackSession {
    pub fn new(
        item_id: impl Into<String>,
        resolved_media: impl Into<String>,
        display_id: impl Into<String>,
    ) -> Self {
        Self {
            session_revision: 0,
            state: PlaybackState::Playing,
            current_item: PlaybackItem {
                item_id: item_id.into(),
                item_revision: 1,
                resolved_media: resolved_media.into(),
                media_generation: 0,
            },
            position_ms: 0,
            telemetry_sequence: 0,
            active_display: DisplayAuthority {
                display_id: display_id.into(),
                generation: 1,
            },
            handoff: None,
            next_transition_id: 1,
            next_display_generation: 2,
            request_records: HashMap::new(),
        }
    }

    pub fn session_revision(&self) -> u64 {
        self.session_revision
    }

    pub fn state(&self) -> PlaybackState {
        self.state
    }

    pub fn current_item_id(&self) -> &str {
        &self.current_item.item_id
    }

    pub fn item_revision(&self) -> u64 {
        self.current_item.item_revision
    }

    pub fn resolved_media(&self) -> &str {
        &self.current_item.resolved_media
    }

    pub fn media_generation(&self) -> u64 {
        self.current_item.media_generation
    }

    pub fn position_ms(&self) -> u64 {
        self.position_ms
    }

    pub fn telemetry_sequence(&self) -> u64 {
        self.telemetry_sequence
    }

    pub fn active_display(&self) -> &DisplayAuthority {
        &self.active_display
    }

    pub fn active_handoff(&self) -> Option<&HandoffTicket> {
        self.handoff.as_ref().map(|transition| &transition.ticket)
    }

    pub fn has_request_id(&self, request_id: &str) -> bool {
        self.request_records.contains_key(request_id)
    }

    pub fn candidate_position_ms(&self) -> Option<u64> {
        self.handoff
            .as_ref()
            .and_then(|transition| transition.candidate_position_ms)
    }

    pub fn execute(&mut self, envelope: CommandEnvelope) -> Result<CommandResult, CommandError> {
        let fingerprint = RequestFingerprint {
            expected_session_revision: envelope.expected_session_revision,
            command: envelope.command.clone(),
        };

        if let Some(record) = self.request_records.get(&envelope.request_id) {
            if record.fingerprint == fingerprint {
                return record.outcome.clone();
            }
            return Err(CommandError::RequestIdMismatch);
        }

        let outcome = self.execute_fresh(&envelope);
        self.request_records.insert(
            envelope.request_id.clone(),
            RequestRecord {
                fingerprint,
                outcome: outcome.clone(),
            },
        );
        outcome
    }

    fn execute_fresh(&mut self, envelope: &CommandEnvelope) -> Result<CommandResult, CommandError> {
        if let Some(expected) = envelope.expected_session_revision {
            if expected != self.session_revision {
                return Err(CommandError::RevisionConflict {
                    current_revision: self.session_revision,
                });
            }
        }

        let transition = match &envelope.command {
            Command::Play => {
                self.state = PlaybackState::Playing;
                self.bump_session_revision();
                None
            }
            Command::Pause => {
                self.state = PlaybackState::Paused;
                self.bump_session_revision();
                None
            }
            Command::Seek(position_ms) => {
                self.position_ms = *position_ms;
                self.bump_session_revision();
                None
            }
            Command::Stop => {
                self.state = PlaybackState::Stopped;
                self.bump_session_revision();
                None
            }
            Command::BeginHandoff { target_display_id } => {
                if let Some(active) = &self.handoff {
                    return Err(CommandError::HandoffInProgress {
                        transition_id: active.ticket.transition_id,
                    });
                }

                let candidate_generation = self.next_display_generation;
                self.next_display_generation = self
                    .next_display_generation
                    .checked_add(1)
                    .expect("display generation overflow");

                let ticket = HandoffTicket {
                    transition_id: self.next_transition_id,
                    item_id: self.current_item.item_id.clone(),
                    item_revision: self.current_item.item_revision,
                    from_display_id: self.active_display.display_id.clone(),
                    from_generation: self.active_display.generation,
                    target_display_id: target_display_id.clone(),
                    candidate_generation,
                };
                self.next_transition_id = self
                    .next_transition_id
                    .checked_add(1)
                    .expect("handoff transition id overflow");
                self.handoff = Some(HandoffTransition {
                    ticket: ticket.clone(),
                    candidate_position_ms: None,
                    candidate_telemetry_sequence: 0,
                });
                self.bump_session_revision();
                Some(ticket)
            }
        };

        Ok(CommandResult {
            request_id: envelope.request_id.clone(),
            session_revision: self.session_revision,
            transition,
        })
    }

    pub fn switch_item(&mut self, item_id: impl Into<String>, resolved_media: impl Into<String>) {
        let item_revision = self
            .current_item
            .item_revision
            .checked_add(1)
            .expect("item revision overflow");
        self.current_item = PlaybackItem {
            item_id: item_id.into(),
            item_revision,
            resolved_media: resolved_media.into(),
            media_generation: 0,
        };
        self.position_ms = 0;
        self.telemetry_sequence = 0;
        self.handoff = None;
        self.bump_session_revision();
    }

    pub fn apply_position_callback(
        &mut self,
        item_id: &str,
        item_revision: u64,
        telemetry_sequence: u64,
        position_ms: u64,
    ) -> bool {
        if self.current_item.item_id.as_str() != item_id
            || self.current_item.item_revision != item_revision
            || telemetry_sequence <= self.telemetry_sequence
        {
            return false;
        }

        self.position_ms = position_ms;
        self.telemetry_sequence = telemetry_sequence;
        true
    }

    pub fn apply_display_position_callback(
        &mut self,
        display_id: &str,
        display_generation: u64,
        item_id: &str,
        item_revision: u64,
        telemetry_sequence: u64,
        position_ms: u64,
    ) -> bool {
        if self.active_display.display_id.as_str() != display_id
            || self.active_display.generation != display_generation
        {
            return false;
        }

        self.apply_position_callback(item_id, item_revision, telemetry_sequence, position_ms)
    }

    pub fn begin_media_refresh(&mut self) -> MediaRefreshTicket {
        self.current_item.media_generation = self
            .current_item
            .media_generation
            .checked_add(1)
            .expect("media generation overflow");
        MediaRefreshTicket {
            item_id: self.current_item.item_id.clone(),
            item_revision: self.current_item.item_revision,
            generation: self.current_item.media_generation,
        }
    }

    pub fn commit_media_refresh(
        &mut self,
        ticket: &MediaRefreshTicket,
        resolved_media: impl Into<String>,
    ) -> bool {
        if self.current_item.item_id != ticket.item_id
            || self.current_item.item_revision != ticket.item_revision
            || self.current_item.media_generation != ticket.generation
        {
            return false;
        }

        self.current_item.resolved_media = resolved_media.into();
        self.bump_session_revision();
        true
    }

    pub fn apply_candidate_callback(
        &mut self,
        ticket: &HandoffTicket,
        telemetry_sequence: u64,
        position_ms: u64,
    ) -> bool {
        let valid = match &self.handoff {
            Some(transition) => transition.ticket == *ticket,
            None => false,
        };
        if !valid
            || self.current_item.item_id != ticket.item_id
            || self.current_item.item_revision != ticket.item_revision
            || self.active_display.display_id != ticket.from_display_id
            || self.active_display.generation != ticket.from_generation
        {
            return false;
        }

        if let Some(transition) = &mut self.handoff {
            if telemetry_sequence <= transition.candidate_telemetry_sequence {
                return false;
            }
            transition.candidate_telemetry_sequence = telemetry_sequence;
            transition.candidate_position_ms = Some(position_ms);
        }
        true
    }

    pub fn commit_handoff(&mut self, ticket: &HandoffTicket) -> bool {
        let valid = match &self.handoff {
            Some(transition) => transition.ticket == *ticket,
            None => false,
        };
        if !valid
            || self.current_item.item_id != ticket.item_id
            || self.current_item.item_revision != ticket.item_revision
            || self.active_display.display_id != ticket.from_display_id
            || self.active_display.generation != ticket.from_generation
        {
            return false;
        }

        self.active_display = DisplayAuthority {
            display_id: ticket.target_display_id.clone(),
            generation: ticket.candidate_generation,
        };
        self.handoff = None;
        self.bump_session_revision();
        true
    }

    pub fn expire_handoff(&mut self, ticket: &HandoffTicket) -> bool {
        let valid = match &self.handoff {
            Some(transition) => transition.ticket == *ticket,
            None => false,
        };
        if !valid {
            return false;
        }

        self.handoff = None;
        self.bump_session_revision();
        true
    }

    pub fn cancel_handoff(&mut self, ticket: &HandoffTicket) -> bool {
        self.expire_handoff(ticket)
    }

    fn bump_session_revision(&mut self) {
        self.session_revision = self
            .session_revision
            .checked_add(1)
            .expect("session revision overflow");
    }
}
