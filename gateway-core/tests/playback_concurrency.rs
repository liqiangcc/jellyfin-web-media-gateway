use gateway_core::playback::{
    Command, CommandEnvelope, CommandError, PlaybackSession, PlaybackState,
};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

fn session() -> PlaybackSession {
    PlaybackSession::new("item-a", "media-a", "display-a")
}

fn envelope(request_id: &str, expected_session_revision: u64, command: Command) -> CommandEnvelope {
    CommandEnvelope {
        request_id: request_id.to_owned(),
        expected_session_revision: Some(expected_session_revision),
        command,
    }
}

#[test]
fn duplicate_request_id_is_idempotent() {
    let mut session = session();
    let command = envelope("req-1", 0, Command::Pause);

    let first = session.execute(command.clone()).expect("first command");
    let second = session.execute(command).expect("duplicate command");

    assert_eq!(first, second);
    assert_eq!(session.session_revision(), 1);
    assert_eq!(session.state(), PlaybackState::Paused);
}

#[test]
fn request_id_reuse_with_different_command_is_rejected() {
    let mut session = session();
    session
        .execute(envelope("req-1", 0, Command::Pause))
        .expect("first command");

    let error = session
        .execute(envelope("req-1", 1, Command::Seek(30_000)))
        .expect_err("request id reuse must fail");

    assert_eq!(error, CommandError::RequestIdMismatch);
    assert_eq!(session.session_revision(), 1);
    assert_eq!(session.position_ms(), 0);
    assert_eq!(session.state(), PlaybackState::Paused);
}

#[test]
fn stale_expected_revision_has_no_side_effects() {
    let mut session = session();
    session
        .execute(envelope("req-1", 0, Command::Pause))
        .expect("first command");

    let before_state = session.state();
    let before_position = session.position_ms();
    let error = session
        .execute(envelope("req-2", 0, Command::Seek(42_000)))
        .expect_err("stale revision must fail");

    assert_eq!(
        error,
        CommandError::RevisionConflict {
            current_revision: 1
        }
    );
    assert_eq!(session.session_revision(), 1);
    assert_eq!(session.state(), before_state);
    assert_eq!(session.position_ms(), before_position);
}

#[test]
fn position_telemetry_does_not_advance_command_revision() {
    let mut session = session();
    let item_id = session.current_item_id().to_owned();
    let item_revision = session.item_revision();

    for sequence in 1..=100 {
        assert!(session.apply_position_callback(
            &item_id,
            item_revision,
            sequence,
            sequence * 1_000,
        ));
    }

    assert_eq!(session.session_revision(), 0);
    assert_eq!(session.telemetry_sequence(), 100);
    assert_eq!(session.position_ms(), 100_000);
}

#[test]
fn high_frequency_position_plus_pause_seek_has_no_telemetry_conflict() {
    let mut session = session();
    let item_id = session.current_item_id().to_owned();
    let item_revision = session.item_revision();

    for sequence in 1..=1_000 {
        assert!(session.apply_position_callback(&item_id, item_revision, sequence, sequence * 10,));
    }
    assert_eq!(session.session_revision(), 0);

    session
        .execute(envelope("pause", 0, Command::Pause))
        .expect("pause after telemetry");
    assert!(session.apply_position_callback(&item_id, item_revision, 1_001, 10_010));
    session
        .execute(envelope("seek", 1, Command::Seek(55_000)))
        .expect("seek after telemetry");

    assert_eq!(session.session_revision(), 2);
    assert_eq!(session.state(), PlaybackState::Paused);
    assert_eq!(session.position_ms(), 55_000);
}

#[test]
fn stale_item_callback_is_ignored() {
    let mut session = session();
    let old_item_id = session.current_item_id().to_owned();
    let old_item_revision = session.item_revision();

    session.switch_item("item-b", "media-b");
    let revision_after_switch = session.session_revision();

    assert!(!session.apply_position_callback(&old_item_id, old_item_revision, 1, 99_000));
    assert_eq!(session.current_item_id(), "item-b");
    assert_eq!(session.position_ms(), 0);
    assert_eq!(session.session_revision(), revision_after_switch);
}

#[test]
fn stale_media_resolve_result_is_ignored() {
    let mut session = session();
    let stale_ticket = session.begin_media_refresh();

    session.switch_item("item-b", "media-b");
    let revision_after_switch = session.session_revision();

    assert!(!session.commit_media_refresh(&stale_ticket, "stale-media"));
    assert_eq!(session.resolved_media(), "media-b");
    assert_eq!(session.session_revision(), revision_after_switch);
}

#[test]
fn newer_media_resolve_wins_when_old_result_arrives_late() {
    let mut session = session();
    let old_ticket = session.begin_media_refresh();
    let new_ticket = session.begin_media_refresh();

    assert_eq!(old_ticket.generation + 1, new_ticket.generation);
    assert!(session.commit_media_refresh(&new_ticket, "media-new"));
    let revision_after_new = session.session_revision();

    assert!(!session.commit_media_refresh(&old_ticket, "media-old-late"));
    assert_eq!(session.resolved_media(), "media-new");
    assert_eq!(session.session_revision(), revision_after_new);
}

#[test]
fn stale_display_generation_callback_is_ignored() {
    let mut session = session();
    let old_display = session.active_display().clone();
    let item_id = session.current_item_id().to_owned();
    let item_revision = session.item_revision();

    let result = session
        .execute(envelope(
            "handoff",
            0,
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve handoff");
    let ticket = result.transition.expect("handoff ticket");
    assert!(session.commit_handoff(&ticket));
    let revision_after_handoff = session.session_revision();

    assert!(!session.apply_display_position_callback(
        &old_display.display_id,
        old_display.generation,
        &item_id,
        item_revision,
        1,
        88_000,
    ));
    assert_eq!(session.position_ms(), 0);
    assert_eq!(session.session_revision(), revision_after_handoff);
}

#[test]
fn handoff_candidate_callback_before_commit_has_no_global_authority() {
    let mut session = session();
    let original_display = session.active_display().clone();

    let result = session
        .execute(envelope(
            "handoff",
            0,
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve handoff");
    let ticket = result.transition.expect("handoff ticket");
    let revision_after_reservation = session.session_revision();

    assert!(session.apply_candidate_callback(&ticket, 1, 12_345));
    assert_eq!(session.active_display(), &original_display);
    assert_eq!(session.position_ms(), 0);
    assert_eq!(session.candidate_position_ms(), Some(12_345));
    assert_eq!(session.session_revision(), revision_after_reservation);
}

#[test]
fn handoff_timeout_invalidates_candidate() {
    let mut session = session();
    let original_display = session.active_display().clone();

    let result = session
        .execute(envelope(
            "handoff",
            0,
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve handoff");
    let ticket = result.transition.expect("handoff ticket");

    assert!(session.expire_handoff(&ticket));
    let revision_after_expiry = session.session_revision();
    assert!(!session.apply_candidate_callback(&ticket, 1, 9_999));
    assert!(!session.commit_handoff(&ticket));
    assert_eq!(session.active_display(), &original_display);
    assert_eq!(session.session_revision(), revision_after_expiry);
}

#[test]
fn handoff_cancel_invalidates_candidate() {
    let mut session = session();
    let original_display = session.active_display().clone();

    let result = session
        .execute(envelope(
            "handoff-cancel",
            0,
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve handoff");
    let ticket = result.transition.expect("handoff ticket");

    assert!(session.apply_candidate_callback(&ticket, 1, 4_321));
    assert!(session.cancel_handoff(&ticket));
    let revision_after_cancel = session.session_revision();

    assert!(!session.apply_candidate_callback(&ticket, 2, 9_999));
    assert!(!session.commit_handoff(&ticket));
    assert_eq!(session.active_display(), &original_display);
    assert_eq!(session.position_ms(), 0);
    assert_eq!(session.session_revision(), revision_after_cancel);
}

#[test]
fn cancelled_candidate_generation_is_not_reused_after_same_target_handoff() {
    let mut session = session();

    let old_result = session
        .execute(envelope(
            "handoff-cancel-old",
            0,
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve old handoff");
    let old_ticket = old_result.transition.expect("old handoff ticket");
    assert!(session.apply_candidate_callback(&old_ticket, 1, 4_321));
    assert!(session.cancel_handoff(&old_ticket));

    let new_result = session
        .execute(envelope(
            "handoff-cancel-new",
            session.session_revision(),
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve new handoff");
    let new_ticket = new_result.transition.expect("new handoff ticket");
    assert!(new_ticket.candidate_generation > old_ticket.candidate_generation);
    assert!(session.commit_handoff(&new_ticket));

    let item_id = session.current_item_id().to_owned();
    let item_revision = session.item_revision();
    let revision_after_commit = session.session_revision();

    assert_eq!(session.active_display().display_id, "display-b");
    assert_eq!(
        session.active_display().generation,
        new_ticket.candidate_generation
    );
    assert!(!session.apply_display_position_callback(
        &old_ticket.target_display_id,
        old_ticket.candidate_generation,
        &item_id,
        item_revision,
        1,
        99_000,
    ));
    assert!(!session.apply_candidate_callback(&old_ticket, 2, 99_000));
    assert_eq!(session.position_ms(), 0);
    assert_eq!(session.telemetry_sequence(), 0);
    assert_eq!(session.session_revision(), revision_after_commit);
}

#[test]
fn expired_candidate_generation_is_not_reused_after_same_target_handoff() {
    let mut session = session();

    let old_result = session
        .execute(envelope(
            "handoff-timeout-old",
            0,
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve old handoff");
    let old_ticket = old_result.transition.expect("old handoff ticket");
    assert!(session.apply_candidate_callback(&old_ticket, 1, 5_432));
    assert!(session.expire_handoff(&old_ticket));

    let new_result = session
        .execute(envelope(
            "handoff-timeout-new",
            session.session_revision(),
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve new handoff");
    let new_ticket = new_result.transition.expect("new handoff ticket");
    assert!(new_ticket.candidate_generation > old_ticket.candidate_generation);
    assert!(session.commit_handoff(&new_ticket));

    let item_id = session.current_item_id().to_owned();
    let item_revision = session.item_revision();
    let revision_after_commit = session.session_revision();

    assert_eq!(session.active_display().display_id, "display-b");
    assert_eq!(
        session.active_display().generation,
        new_ticket.candidate_generation
    );
    assert!(!session.apply_display_position_callback(
        &old_ticket.target_display_id,
        old_ticket.candidate_generation,
        &item_id,
        item_revision,
        1,
        88_000,
    ));
    assert!(!session.apply_candidate_callback(&old_ticket, 2, 88_000));
    assert_eq!(session.position_ms(), 0);
    assert_eq!(session.telemetry_sequence(), 0);
    assert_eq!(session.session_revision(), revision_after_commit);
}

#[test]
fn old_source_callback_after_handoff_commit_is_ignored() {
    let mut session = session();
    let source_display = session.active_display().clone();
    let item_id = session.current_item_id().to_owned();
    let item_revision = session.item_revision();

    let result = session
        .execute(envelope(
            "handoff",
            0,
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("reserve handoff");
    let ticket = result.transition.expect("handoff ticket");
    assert!(session.apply_candidate_callback(&ticket, 1, 1_234));
    assert!(session.commit_handoff(&ticket));

    assert_eq!(session.active_display().display_id, "display-b");
    assert_eq!(
        session.active_display().generation,
        ticket.candidate_generation
    );
    assert!(!session.apply_display_position_callback(
        &source_display.display_id,
        source_display.generation,
        &item_id,
        item_revision,
        1,
        77_000,
    ));
    assert_eq!(session.position_ms(), 0);
}

#[test]
fn overlapping_handoff_has_single_authority_path() {
    let mut session = session();

    let first = session
        .execute(envelope(
            "handoff-1",
            0,
            Command::BeginHandoff {
                target_display_id: "display-b".to_owned(),
            },
        ))
        .expect("first handoff reservation");
    let first_ticket = first.transition.expect("first ticket");

    let error = session
        .execute(envelope(
            "handoff-2",
            1,
            Command::BeginHandoff {
                target_display_id: "display-c".to_owned(),
            },
        ))
        .expect_err("overlapping handoff must fail");

    assert_eq!(
        error,
        CommandError::HandoffInProgress {
            transition_id: first_ticket.transition_id
        }
    );
    assert_eq!(session.active_handoff(), Some(&first_ticket));
    assert_eq!(session.session_revision(), 1);
    assert!(session.commit_handoff(&first_ticket));
    assert_eq!(session.active_display().display_id, "display-b");
}

fn run_two_control_race_iteration(iteration: usize) {
    let session = Arc::new(Mutex::new(session()));
    let expected_revision = session.lock().expect("session lock").session_revision();
    let barrier = Arc::new(Barrier::new(3));

    let commands = [
        CommandEnvelope {
            request_id: format!("pause-{iteration}"),
            expected_session_revision: Some(expected_revision),
            command: Command::Pause,
        },
        CommandEnvelope {
            request_id: format!("seek-{iteration}"),
            expected_session_revision: Some(expected_revision),
            command: Command::Seek(25_000),
        },
    ];

    let handles: Vec<_> = commands
        .into_iter()
        .map(|command| {
            let session = Arc::clone(&session);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                session.lock().expect("session lock").execute(command)
            })
        })
        .collect();

    barrier.wait();
    let outcomes: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().expect("control thread"))
        .collect();

    let success_count = outcomes.iter().filter(|outcome| outcome.is_ok()).count();
    let conflict_count = outcomes
        .iter()
        .filter(|outcome| {
            matches!(
                outcome,
                Err(CommandError::RevisionConflict {
                    current_revision: 1
                })
            )
        })
        .count();

    assert_eq!(success_count, 1, "iteration {iteration}");
    assert_eq!(conflict_count, 1, "iteration {iteration}");
    assert_eq!(
        session.lock().expect("session lock").session_revision(),
        1,
        "iteration {iteration}"
    );
}

#[test]
fn two_controls_same_expected_revision_only_one_authoritative_mutation_commits() {
    run_two_control_race_iteration(0);
}

fn run_r007_stress_iteration(iteration: usize) {
    position_telemetry_does_not_advance_command_revision();
    high_frequency_position_plus_pause_seek_has_no_telemetry_conflict();
    duplicate_request_id_is_idempotent();
    request_id_reuse_with_different_command_is_rejected();
    stale_expected_revision_has_no_side_effects();
    run_two_control_race_iteration(iteration);
    stale_item_callback_is_ignored();
    stale_media_resolve_result_is_ignored();
    newer_media_resolve_wins_when_old_result_arrives_late();
    stale_display_generation_callback_is_ignored();
    handoff_candidate_callback_before_commit_has_no_global_authority();
    handoff_timeout_invalidates_candidate();
    handoff_cancel_invalidates_candidate();
    cancelled_candidate_generation_is_not_reused_after_same_target_handoff();
    expired_candidate_generation_is_not_reused_after_same_target_handoff();
    old_source_callback_after_handoff_commit_is_ignored();
    overlapping_handoff_has_single_authority_path();
}

#[test]
fn repeated_r007_concurrency_stress() {
    let repetitions = std::env::var("R007_REPETITIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);

    for iteration in 0..repetitions {
        run_r007_stress_iteration(iteration);
    }
}
