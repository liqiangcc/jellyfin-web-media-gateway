# Session Bootstrap — R006-RUNTIME-FUNCTIONAL-PREP

Execute Issue #75 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #75 and relevant comments
3. `docs/tasks/75-r006-runtime-functional-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. Issue #27 current umbrella contract
7. Issue #33 Final Acceptance + `docs/tasks/33-r006-contract-prep/task.md`
8. `gateway-core/src/browser.rs`
9. canonical Browser Worker / Vault / R008 sections in architecture/security/implementation contracts
10. current #9 boundary only to understand what performance work is excluded

Claim only if live #75 remains:

```text
status:ready
env:cloud
no active owner
```

## Critical boundaries

- real Chromium functional runtime only; no site-specific DOM/API/login-success/media semantics;
- exercise a preinstalled allowlisted Chrome/Chromium binary in hosted Evidence; do not accept caller executable/argv authority;
- implement accepted BrowserWorker open/navigate/status/events/input/close semantics using CDP or equivalent generic browser automation;
- R008 remains navigation/security authority; no proxy/bypass weakening;
- no raw input values, page bodies, Cookie/profile/token material or Chromium stderr leakage in durable Evidence;
- deterministic process/temp-profile cleanup on close/crash/timeout/cancel;
- NativePanel proof is only the accepted session/token/input boundary, not a full remote desktop/UI;
- no clipboard/file-upload/audio permission expansion;
- no performance/capacity/phone placement claim; #9 remains separate;
- do **not** modify `site-adapter-api/**`, navigation semantics, `gateway-core/src/source_session.rs`, or Playback Next/Previous; #71 owns that parallel lane;
- if a hard dependency crosses #71 ownership, BLOCK/SPLIT rather than silently merging scopes;
- exact-Candidate J1-J4 required;
- completion: report -> status:review -> release owner -> STOP; never set done/close/merge own PR or start Auth/real-site work.