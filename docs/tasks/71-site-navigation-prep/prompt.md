# Session Bootstrap — SITE-NAVIGATION-PREP

Execute Issue #71 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #71 and relevant comments
3. `docs/tasks/71-site-navigation-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. canonical `docs/implementation-contracts.md` sections for SiteAdapter/navigation/PlaybackContext/Playback Command
7. #39 SiteAdapter conformance Final Acceptance
8. #2/R007 Playback Final Acceptance
9. #44 SourceSession preparation Final Acceptance
10. current `site-adapter-api/src/lib.rs`, `gateway-core/src/playback.rs`, `gateway-core/src/source_session.rs`

Claim only if live #71 remains:

```text
status:ready
env:cloud
no active owner
```

## Critical boundaries

- generic navigation only; no Bilibili/BVID/page/episode/playlist implementation;
- add current-authority previous/next navigation without restoring stale #23 ResolveContext/DASH/expiry/site-specific errors;
- Registry owns plugin/site routing and returned locator validation;
- target resolve/preparation happens before current-item commit;
- stale prepared navigation cannot commit over a newer item/session;
- NextItem/PreviousItem must preserve existing R007 request_id/CAS/session_revision/item_revision authority;
- no second navigation revision/state machine;
- do not modify `gateway-core/src/browser.rs` or BrowserWorker/Chromium runtime surfaces; a separate Cloud Task owns that parallel lane;
- no login/Vault/Native Panel/DASH/remux/real-site/performance work;
- exact-Candidate J1-J4 Evidence required;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never auto-start #72, set done, close Issue, or merge own PR.

The purpose is to make continuous-content navigation a generic accepted capability reusable by later #72 and other sites, not to make Bilibili-specific assumptions.