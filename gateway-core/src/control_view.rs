//! Read-only projection for the unified Control experience.
//!
//! `ControlView` is deliberately a value produced from domain-owned snapshots.
//! It has no store, command executor, replay log, or mutable revision of its
//! own.  The domain-specific freshness values are retained only so a client
//! can reconcile a projection with the authority that produced it.

use crate::ControlSnapshot;
use crate::auth::{AccountState, PendingIntent, PendingPlaybackAction, SiteAccount};
use crate::browser::{BrowserError, BrowserStatus};
use display_adapter_api::{
    DisplayAdapterError, DisplayInstance, DisplayStatus, PlaybackObservation,
};
use serde::Serialize;
use std::fmt;

/// Inputs are safe copies of authoritative snapshots/status.  In particular,
/// this type cannot carry `resolved_media`, Vault material, browser input, or
/// Native Panel control tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlViewInput {
    pub playback: ControlSnapshot,
    pub event_cursor: Option<u64>,
    pub site: SiteViewInput,
    pub browser: BrowserViewInput,
    pub display: DisplayViewInput,
}

impl ControlViewInput {
    pub fn new(playback: ControlSnapshot) -> Self {
        Self {
            playback,
            event_cursor: None,
            site: SiteViewInput::default(),
            browser: BrowserViewInput::default(),
            display: DisplayViewInput::default(),
        }
    }
}

/// Safe site/session facts.  `from_account` intentionally projects only the
/// account reference and state; it never copies the `SiteSessionRef`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SiteViewInput {
    pub site_id: Option<String>,
    pub label: Option<String>,
    pub account_ref: Option<String>,
    pub account_state: Option<AccountState>,
    pub pending_intent: Option<PendingIntentInput>,
}

impl SiteViewInput {
    pub fn from_account(account: &SiteAccount, pending: Option<&PendingIntent>) -> Self {
        Self {
            site_id: Some(account.site_id().to_owned()),
            label: Some(account.label().to_owned()),
            account_ref: Some(account.account_ref().to_owned()),
            account_state: Some(account.state()),
            pending_intent: pending.map(PendingIntentInput::from_intent),
        }
    }
}

/// Non-secret retry metadata.  The source locator payload is intentionally
/// omitted: Control only needs to say that a retry exists and what it does.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PendingIntentInput {
    pub intent_id: String,
    pub action: PendingActionView,
    pub expected_session_revision: Option<u64>,
    pub has_display_ref: bool,
}

impl PendingIntentInput {
    pub fn from_intent(intent: &PendingIntent) -> Self {
        Self {
            intent_id: intent.intent_id.clone(),
            action: match intent.action {
                PendingPlaybackAction::Play => PendingActionView::Play,
                PendingPlaybackAction::Resume => PendingActionView::Resume,
                PendingPlaybackAction::Handoff => PendingActionView::Handoff,
            },
            expected_session_revision: intent.expected_session_revision,
            has_display_ref: intent.display_ref.is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingActionView {
    Play,
    Resume,
    Handoff,
}

/// Generic Browser Worker facts.  `sequence` is a browser-domain freshness
/// marker, not a Control revision.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BrowserViewInput {
    pub sequence: u64,
    pub status: Option<BrowserStatus>,
    pub error: Option<BrowserError>,
    pub panel: NativePanelInput,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct NativePanelInput {
    pub available: bool,
    pub error: Option<BrowserError>,
    pub capabilities: Vec<String>,
}

/// Generic display facts.  The projection looks up the active display by
/// identity and never branches on an adapter/site name.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct DisplayViewInput {
    pub instance: Option<DisplayInstance>,
    pub status: Option<DisplayStatus>,
    pub error: Option<DisplayAdapterError>,
}

impl fmt::Debug for DisplayViewInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DisplayViewInput")
            .field("instance", &self.instance)
            .field("status", &self.status)
            .field("error", &self.error.as_ref().map(display_error_code))
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlView {
    pub now_playing: NowPlayingView,
    pub playback_controls: PlaybackControlsView,
    pub playback_context: PlaybackContextView,
    pub active_display: ActiveDisplayView,
    pub site: SiteView,
    pub site_account_state: SiteAccountStateView,
    pub native_site_panel: NativeSitePanelView,
    pub action_required: Option<ActionRequiredView>,
    pub freshness: ControlFreshnessView,
}

impl ControlView {
    /// Build the current read model from fresh authoritative inputs.
    pub fn project(input: ControlViewInput) -> Self {
        let playback = &input.playback;
        let site_state = input.site.account_state.unwrap_or(AccountState::Unknown);
        let active_display = ActiveDisplayView::from_input(
            &playback.active_display,
            input.display.instance.as_ref(),
            input.display.status.as_ref(),
            input.display.error.as_ref(),
        );
        let native_site_panel = NativeSitePanelView::from_input(&input.browser.panel);
        let action_required = action_required(site_state, &input.browser, &input.display, playback);

        Self {
            now_playing: NowPlayingView {
                item_id: playback.current_item.item_id.clone(),
                item_revision: playback.current_item.item_revision,
                media_generation: playback.current_item.media_generation,
                state: playback.state.to_owned(),
                position_ms: playback.position_ms,
            },
            playback_controls: PlaybackControlsView::from_snapshot(playback),
            playback_context: PlaybackContextView {
                session_id: playback.session_id.clone(),
                state: playback.state.to_owned(),
                handoff: playback.handoff.clone(),
            },
            active_display,
            site: SiteView {
                site_id: input.site.site_id,
                label: input.site.label,
            },
            site_account_state: SiteAccountStateView {
                account_ref: input.site.account_ref,
                state: site_state,
                pending_intent: input.site.pending_intent,
            },
            native_site_panel,
            action_required,
            freshness: ControlFreshnessView {
                playback: PlaybackFreshnessView {
                    session_revision: playback.session_revision,
                    item_revision: playback.current_item.item_revision,
                    media_generation: playback.current_item.media_generation,
                    display_generation: playback.active_display.generation,
                    telemetry_sequence: playback.telemetry_sequence,
                },
                event_cursor: input.event_cursor,
                browser_sequence: input.browser.sequence,
                display_generation: playback.active_display.generation,
            },
        }
    }

    /// Compare only source-owned freshness markers. This helper is pure and
    /// caller-driven; it does not retain a previous view or create a global
    /// Control revision.
    pub fn is_fresh_for(&self, previous: &Self) -> bool {
        let current = &self.freshness;
        let old = &previous.freshness;
        let playback_fresh = if current.playback.session_revision != old.playback.session_revision {
            current.playback.session_revision > old.playback.session_revision
        } else if current.playback.item_revision != old.playback.item_revision {
            current.playback.item_revision > old.playback.item_revision
        } else {
            current.playback.media_generation >= old.playback.media_generation
                && current.playback.telemetry_sequence >= old.playback.telemetry_sequence
        };
        playback_fresh
            && current.playback.display_generation >= old.playback.display_generation
            && current.event_cursor >= old.event_cursor
            && current.browser_sequence >= old.browser_sequence
            && current.display_generation >= old.display_generation
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NowPlayingView {
    pub item_id: String,
    pub item_revision: u64,
    pub media_generation: u64,
    pub state: String,
    pub position_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PlaybackControlsView {
    pub can_play: bool,
    pub can_pause: bool,
    pub can_seek: bool,
    pub can_stop: bool,
    pub can_handoff: bool,
}

impl PlaybackControlsView {
    fn from_snapshot(snapshot: &ControlSnapshot) -> Self {
        let stopped = snapshot.state == "stopped";
        let handoff_in_progress = snapshot.handoff.is_some();
        Self {
            can_play: snapshot.state != "playing",
            can_pause: snapshot.state == "playing",
            can_seek: !stopped,
            can_stop: !stopped,
            can_handoff: !handoff_in_progress,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PlaybackContextView {
    pub session_id: String,
    pub state: String,
    pub handoff: Option<crate::ControlHandoffSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ActiveDisplayView {
    pub display_id: String,
    pub adapter_type: Option<String>,
    pub label: Option<String>,
    pub online: bool,
    pub observation: Option<PlaybackObservationView>,
    pub error_code: Option<String>,
    pub generation: u64,
}

impl ActiveDisplayView {
    fn from_input(
        authority: &crate::ControlDisplaySnapshot,
        instance: Option<&DisplayInstance>,
        status: Option<&DisplayStatus>,
        error: Option<&DisplayAdapterError>,
    ) -> Self {
        Self {
            display_id: authority.display_id.clone(),
            adapter_type: instance.map(|value| value.adapter_type.clone()),
            label: instance.map(|value| value.label.clone()),
            online: instance.is_some_and(|value| value.online),
            observation: status.map(|value| match value.observation {
                PlaybackObservation::Playing => PlaybackObservationView::Playing,
                PlaybackObservation::Paused => PlaybackObservationView::Paused,
                PlaybackObservation::Stopped => PlaybackObservationView::Stopped,
                PlaybackObservation::Unknown => PlaybackObservationView::Unknown,
            }),
            error_code: error.map(display_error_code),
            generation: authority.generation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackObservationView {
    Playing,
    Paused,
    Stopped,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SiteView {
    pub site_id: Option<String>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SiteAccountStateView {
    pub account_ref: Option<String>,
    pub state: AccountState,
    pub pending_intent: Option<PendingIntentInput>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeSitePanelView {
    pub status: NativePanelStatus,
    pub error_code: Option<String>,
    pub capabilities: Vec<String>,
}

impl NativeSitePanelView {
    fn from_input(input: &NativePanelInput) -> Self {
        Self {
            status: if input.available {
                NativePanelStatus::Available
            } else if input.error.is_some() {
                NativePanelStatus::Unavailable
            } else {
                NativePanelStatus::NotAttached
            },
            error_code: input.error.map(|error| error.code().to_owned()),
            capabilities: input.capabilities.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativePanelStatus {
    Available,
    NotAttached,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ActionRequiredView {
    pub kind: ActionRequiredKind,
    pub code: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionRequiredKind {
    SiteAuthentication,
    SiteAccountError,
    NativeSitePanel,
    Display,
    PlaybackHandoff,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ControlFreshnessView {
    pub playback: PlaybackFreshnessView,
    pub event_cursor: Option<u64>,
    pub browser_sequence: u64,
    pub display_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PlaybackFreshnessView {
    pub session_revision: u64,
    pub item_revision: u64,
    pub media_generation: u64,
    pub display_generation: u64,
    pub telemetry_sequence: u64,
}

fn action_required(
    site_state: AccountState,
    browser: &BrowserViewInput,
    display: &DisplayViewInput,
    playback: &ControlSnapshot,
) -> Option<ActionRequiredView> {
    // Precedence is presentation-only.  It never changes any domain value.
    match site_state {
        AccountState::LoginRequired | AccountState::Expired => {
            return Some(ActionRequiredView {
                kind: ActionRequiredKind::SiteAuthentication,
                code: "SITE_AUTH_REQUIRED".to_owned(),
            });
        }
        AccountState::Error => {
            return Some(ActionRequiredView {
                kind: ActionRequiredKind::SiteAccountError,
                code: "SITE_ACCOUNT_ERROR".to_owned(),
            });
        }
        _ => {}
    }

    if browser.error.is_some()
        || browser.panel.error.is_some()
        || matches!(
            browser.status,
            Some(BrowserStatus::Crashed | BrowserStatus::TimedOut)
        )
    {
        return Some(ActionRequiredView {
            kind: ActionRequiredKind::NativeSitePanel,
            code: browser.panel.error.or(browser.error).map_or_else(
                || "NATIVE_PANEL_UNAVAILABLE".to_owned(),
                |error| error.code().to_owned(),
            ),
        });
    }

    if display.error.is_some() || !display.instance.as_ref().is_some_and(|value| value.online) {
        return Some(ActionRequiredView {
            kind: ActionRequiredKind::Display,
            code: display
                .error
                .as_ref()
                .map_or_else(|| "DISPLAY_OFFLINE".to_owned(), display_error_code),
        });
    }

    playback.handoff.as_ref().map(|_| ActionRequiredView {
        kind: ActionRequiredKind::PlaybackHandoff,
        code: "HANDOFF_PENDING".to_owned(),
    })
}

fn display_error_code(error: &DisplayAdapterError) -> String {
    match error {
        DisplayAdapterError::InvalidConfiguration => "INVALID_CONFIGURATION",
        DisplayAdapterError::ServerUnavailable => "SERVER_UNAVAILABLE",
        DisplayAdapterError::AuthenticationFailed => "AUTHENTICATION_FAILED",
        DisplayAdapterError::TargetMissing => "TARGET_MISSING",
        DisplayAdapterError::TargetOffline => "TARGET_OFFLINE",
        DisplayAdapterError::TargetAmbiguous => "TARGET_AMBIGUOUS",
        DisplayAdapterError::MediaIncompatible => "MEDIA_INCOMPATIBLE",
        DisplayAdapterError::CommandRejected => "COMMAND_REJECTED",
        DisplayAdapterError::PlaybackNotConfirmed { .. } => "PLAYBACK_NOT_CONFIRMED",
        DisplayAdapterError::Timeout => "TIMEOUT",
        DisplayAdapterError::Cancelled => "CANCELLED",
        DisplayAdapterError::StaleContext => "STALE_CONTEXT",
        DisplayAdapterError::Protocol(_) => "PROTOCOL_ERROR",
    }
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{ControlDisplaySnapshot, ControlHandoffSnapshot, ControlItemSnapshot};
    use serde_json::to_string;

    fn playback(state: &str) -> ControlSnapshot {
        ControlSnapshot {
            session_id: "session-1".into(),
            session_revision: 9,
            state: match state {
                "playing" => "playing",
                "paused" => "paused",
                "stopped" => "stopped",
                _ => panic!("invalid test state"),
            },
            current_item: ControlItemSnapshot {
                item_id: "item-1".into(),
                item_revision: 4,
                media_generation: 2,
            },
            position_ms: 12_000,
            telemetry_sequence: 7,
            active_display: ControlDisplaySnapshot {
                display_id: "display-1".into(),
                generation: 6,
            },
            handoff: None,
        }
    }

    fn base_input() -> ControlViewInput {
        let mut input = ControlViewInput::new(playback("playing"));
        input.site = SiteViewInput {
            site_id: Some("site-a".into()),
            label: Some("Source A".into()),
            account_ref: Some("account-a".into()),
            account_state: Some(AccountState::Valid),
            pending_intent: None,
        };
        input.browser = BrowserViewInput {
            sequence: 11,
            status: Some(BrowserStatus::Open),
            error: None,
            panel: NativePanelInput {
                available: true,
                error: None,
                capabilities: vec!["search".into()],
            },
        };
        input.display = DisplayViewInput {
            instance: Some(DisplayInstance {
                id: "display-1".into(),
                adapter_type: "generic-adapter".into(),
                label: "Living room".into(),
                online: true,
                capabilities: vec!["pause".into()],
            }),
            status: None,
            error: None,
        };
        input.event_cursor = Some(20);
        input
    }

    #[test]
    fn projection_is_read_only_safe_and_preserves_domain_freshness() {
        let view = ControlView::project(base_input());
        assert_eq!(view.now_playing.item_id, "item-1");
        assert!(view.playback_controls.can_pause);
        assert!(!view.playback_controls.can_play);
        assert_eq!(view.freshness.playback.session_revision, 9);
        assert_eq!(view.freshness.playback.item_revision, 4);
        assert_eq!(view.freshness.playback.media_generation, 2);
        assert_eq!(view.freshness.event_cursor, Some(20));
        let serialized = to_string(&view).unwrap();
        assert!(!serialized.contains("resolved_media"));
        assert!(!serialized.contains("control_revision"));
    }

    #[test]
    fn action_precedence_is_deterministic_and_does_not_reset_playback() {
        let mut input = base_input();
        input.site.account_state = Some(AccountState::LoginRequired);
        input.browser.status = Some(BrowserStatus::Crashed);
        input.browser.panel.error = Some(BrowserError::PanelDisconnected);
        input.display.instance.as_mut().unwrap().online = false;
        input.display.error = Some(DisplayAdapterError::ServerUnavailable);
        input.playback = playback("playing");
        let view = ControlView::project(input);
        assert_eq!(
            view.action_required.unwrap().kind,
            ActionRequiredKind::SiteAuthentication
        );
        assert_eq!(view.now_playing.state, "playing");
        assert_eq!(view.now_playing.position_ms, 12_000);
    }

    #[test]
    fn independent_failures_keep_unrelated_sections_available() {
        let mut input = base_input();
        input.browser.status = Some(BrowserStatus::Crashed);
        input.browser.error = Some(BrowserError::WorkerCrashed);
        input.browser.panel = NativePanelInput {
            available: false,
            error: Some(BrowserError::WorkerCrashed),
            capabilities: Vec::new(),
        };
        let browser_failed = ControlView::project(input.clone());
        assert_eq!(browser_failed.now_playing.state, "playing");
        assert!(browser_failed.playback_controls.can_pause);
        assert_eq!(browser_failed.active_display.display_id, "display-1");

        input.browser = BrowserViewInput::default();
        input.display.instance.as_mut().unwrap().online = false;
        input.display.error = Some(DisplayAdapterError::TargetOffline);
        let display_failed = ControlView::project(input);
        assert_eq!(display_failed.site.site_id.as_deref(), Some("site-a"));
        assert_eq!(
            display_failed.native_site_panel.status,
            NativePanelStatus::NotAttached
        );
        assert_eq!(display_failed.now_playing.item_id, "item-1");
    }

    #[test]
    fn reconnect_rebuild_from_fresh_snapshots_is_equal_without_history() {
        let first = ControlView::project(base_input());
        let mut newer_input = base_input();
        newer_input.playback.session_revision = 10;
        newer_input.playback.position_ms = 13_000;
        newer_input.playback.telemetry_sequence = 8;
        newer_input.event_cursor = Some(21);
        let after_update = ControlView::project(newer_input.clone());
        let rebuilt = ControlView::project(newer_input);
        assert_ne!(first, after_update);
        assert_eq!(after_update, rebuilt);
        assert!(after_update.is_fresh_for(&first));
        assert!(!first.is_fresh_for(&after_update));
    }

    #[test]
    fn pending_intent_and_sensitive_source_boundaries_are_redacted() {
        let mut input = base_input();
        input.site.account_state = Some(AccountState::LoginRequired);
        input.site.pending_intent = Some(PendingIntentInput {
            intent_id: "intent-1".into(),
            action: PendingActionView::Resume,
            expected_session_revision: Some(9),
            has_display_ref: true,
        });
        let view = ControlView::project(input);
        let serialized = to_string(&view).unwrap();
        for sentinel in [
            "Cookie: secret-sentinel",
            "Bearer secret-sentinel",
            "https://cdn.invalid/signed?token=secret-sentinel",
            "password-secret-sentinel",
            "panel-token-secret-sentinel",
        ] {
            assert!(!serialized.contains(sentinel));
        }
        assert!(!format!("{view:?}").contains("secret-sentinel"));
    }

    #[test]
    fn handoff_is_projected_as_domain_state_and_controls_are_derived() {
        let mut input = base_input();
        input.playback.handoff = Some(ControlHandoffSnapshot {
            transition_id: 2,
            item_id: "item-1".into(),
            item_revision: 4,
            from_display_id: "display-1".into(),
            from_generation: 6,
            target_display_id: "display-2".into(),
            candidate_generation: 7,
            candidate_position_ms: Some(10_000),
        });
        let view = ControlView::project(input);
        assert!(!view.playback_controls.can_handoff);
        assert_eq!(view.action_required.unwrap().code, "HANDOFF_PENDING");
        assert_eq!(view.playback_context.handoff.unwrap().transition_id, 2);
    }
}
