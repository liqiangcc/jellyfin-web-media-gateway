# Task — GENERIC-YTDLP-BOUNDED-UNSUPPORTED-STAGE-PREP

## Metadata

```text
GitHub Issue: #107
Task ID: GENERIC-YTDLP-BOUNDED-UNSUPPORTED-STAGE-PREP
Task kind: implementation + deterministic security verification
Parent Goal / Research Item: #67 GENERIC-YTDLP-BILIBILI-REAL Attempt 11
Planning Base: 6034eb1cd1837988161d955ef0d1f67d60ce0257
Preferred worker: cloud-codex
Eligible environment: env:cloud
Execution plane: GitHub Actions
Downstream: #67 next Contract Revision / target Attempt
Freshness policy: dependency-aware
```

## Trigger Evidence

#67 Attempt 11 executed exact runtime Candidate `1a38e403a3252239822aeb2a784a20fdfd18c0a6` on the accepted low-privilege Ubuntu ARM64 target and produced one bounded current-contract compatibility result:

```text
J0: PASS
J1: PASS
J2: PASS
J3: COMPLETED
broker_status_class: 2xx
broker_error_code: n/a
broker_request_count: 4
process_error: UNSUPPORTED_FORMAT
protocol: n/a
stream_count: 0
J4: PASS
Overall: FAIL
```

The accepted target/runtime/security path was intact: no `SANDBOX_UNAVAILABLE`, `SPAWN_FAILED`, `BROKER_PROTOCOL`, `BROKER_RESPONSE_SECRET_REJECTED`, raw diagnostic exposure, or cleanup failure was observed.

This evidence does **not** prove DASH, separate A/V, a playurl shape, or any other concrete media-format cause. The Worker must not infer such a cause from `UNSUPPORTED_FORMAT` or broker request count.

## Goal

Add the smallest repository-owned, closed, Secret-safe secondary classification for `UNSUPPORTED_FORMAT` so a later Coordinator-controlled #67 target Attempt can identify **which bounded repository phase rejected the result** without exposing raw stderr, exception text, site response content, URLs, headers, tokens, or arbitrary extractor-provided strings.

The top-level #101 outcome remains exactly:

```text
UNSUPPORTED_FORMAT
```

A successful #107 Candidate adds only one optional fixed secondary field/classification for that top-level outcome.

## Frozen Stage Semantics

The secondary taxonomy must be a small closed repository enum covering these semantic phases (constant names may differ only if the mapping remains one-to-one and documented):

```text
PRE_FALLBACK
FALLBACK_WEBPAGE
FALLBACK_NAV
FALLBACK_VIEW
FALLBACK_DETAIL
FALLBACK_PLAYURL
MEDIA_SHAPE
UNCLASSIFIED
```

Meanings:

- `PRE_FALLBACK`: normal extractor / pre-fallback normalization rejected current media before the #105 continuation became the owning path.
- `FALLBACK_WEBPAGE`: #105 continuation rejected bounded webpage/admission structure.
- `FALLBACK_NAV`: #105 continuation rejected bounded nav/WBI structure.
- `FALLBACK_VIEW`: #105 continuation rejected bounded view structure.
- `FALLBACK_DETAIL`: #105 continuation rejected bounded detail structure.
- `FALLBACK_PLAYURL`: #105 continuation rejected bounded playurl structure before final media-shape acceptance.
- `MEDIA_SHAPE`: final current-contract stream/media normalization rejected an otherwise admitted bounded result.
- `UNCLASSIFIED`: the repository cannot safely map the unsupported result to a narrower admitted phase.

The taxonomy identifies a **repository phase only**. It must never encode response data, extractor text, HTTP details, URL hosts/paths, BVID/CID, media IDs, codec names, format IDs, site-provided error codes, or exception/message fragments.

## Task Decomposition Decision

```text
Implementation + verification: inline in #107
Real-site/target verification: downstream #67 only
Media-format repair: not authorized by #107
Reason: this Task adds only bounded diagnostic structure and deterministic security proof.
```

## Worker Routing Decision

```text
Worker: cloud-codex
Environment: env:cloud
J1/J3/J4: GitHub-hosted x86_64
J2: native hosted ARM64
Public/real-site network: forbidden
```

## Security Invariants

1. `UNSUPPORTED_FORMAT` remains the top-level fixed #101 classification.
2. The secondary stage is a closed enum of repository constants only; no arbitrary string is serialized or logged.
3. Raw stderr, traceback, exception text/message, source URL, request/response headers, page/body content, signed media data, credentials, tokens, Cookie/Auth/profile/account state, and media payloads never cross the existing bounded worker/runtime Evidence boundary.
4. #105 fallback admission remains unchanged: normal frozen yt-dlp extraction runs first, and only the accepted narrow BiliBiliIE missing-initial-state condition plus strict Bilibili video URL shape may enter its continuation.
5. The Worker gains no caller-selectable fallback action, direct socket authority, alternate egress, proxy capability, browser authority, or site-specific Core branch.
6. R008/#95 request/response Secret policy, #97 broker framing, #83/#85 sandbox/fd isolation, #99 clean-build binding, #101 top-level taxonomy, and production `DisabledRunner` remain unchanged.
7. Unknown/malformed/unmapped stage data fails closed and must not become a successful `ResolvedMedia` or a trusted arbitrary diagnostic.
8. No public Bilibili or other real-site request occurs in #107.

## Implementation Requirements

### A. Safe worker envelope

Extend only the bounded unsupported outcome so it may carry one secondary stage. The Rust parser/runtime/smoke path must accept only the fixed allowlist and only when the top-level classification is `UNSUPPORTED_FORMAT`.

Examples of acceptable semantics:

```text
error: UNSUPPORTED_FORMAT
unsupported_stage: FALLBACK_DETAIL
```

Exact JSON shape is implementation-owned, but all of these remain invalid/fail-closed:

- unknown stage value;
- stage attached to a different top-level error;
- missing required fields in a shape that claims a stage;
- additional arbitrary diagnostic/message fields;
- oversized/malformed JSON;
- arbitrary worker nonzero exit or crash masquerading as an admitted stage.

### B. Repository-owned phase attribution

Attribute a stage only at a repository-owned rejection boundary. Do not parse or pattern-match raw exception/error text to derive a stage. Do not retain source response content merely to classify it.

`UNCLASSIFIED` is the safe fallback whenever the repository cannot prove a narrower fixed phase from control flow alone.

### C. Deterministic offline fixture matrix

Add deterministic fixtures proving at minimum:

- one `PRE_FALLBACK` unsupported case;
- one rejection case for each #105 fallback phase: webpage, nav, view, detail, playurl;
- one `MEDIA_SHAPE` case;
- one `UNCLASSIFIED` case;
- unknown/forged stage and wrong top-level/stage combinations fail closed;
- sentinel-bearing exception text, URL/query token, headers/body, credentials and media data never appear in worker stdout, Rust parser/smoke output, logs, or durable test evidence.

Fixtures must not use public network traffic.

### D. Preserve normal success and other failures

Existing successful muxed `http-file`/HLS paths remain unchanged. Existing #101 fixed classifications other than `UNSUPPORTED_FORMAT` remain semantically unchanged and must not receive a fabricated unsupported stage.

### E. Cross-architecture proof

The same exact-Candidate worker/parser/runtime behavior must pass on hosted x86_64 and native hosted ARM64.

## Claims

```text
C1: UNSUPPORTED_FORMAT top-level contract is preserved
C2: closed secondary stage taxonomy maps repository-owned unsupported phases
C3: no arbitrary diagnostic/Secret/site-response string crosses the boundary
C4: forged/unknown/malformed/wrong-error stage envelopes fail closed
C5: existing success path and other #101 failure classes remain unchanged
C6: hosted x86_64 and native hosted ARM64 are equivalent on the deterministic matrix
C7: R008/#95/#97/#99/#83/#85/#101/#105/DisabledRunner authorities remain intact
C8: result is actionable only for a later Coordinator-controlled #67 rerun; no media-format cause is claimed here
```

## Job Matrix

### J1 — hosted x86_64 stage taxonomy

- assert exact Candidate identity;
- run deterministic worker/parser/runtime stage fixture matrix;
- prove C1/C2/C4/C5;
- prove no public request.

### J2 — native hosted ARM64 equivalence

Runner must assert native `aarch64`.

- repeat the stage taxonomy and negative-envelope matrix on exact Candidate;
- preserve accepted sandbox/runtime tests;
- prove C6 and no public request.

### J3 — security/static guards

- sentinel absence for stderr/exception/message/URL/query/header/body/token/credential/media payload material;
- fixed enum only; no exception-, response-, URL- or extractor-derived stage string;
- no new Cookie/Auth/profile/proxy/browser/direct-egress capability;
- no #105 admission broadening or caller-selectable fallback action;
- R008/#95/#97/#83/#85/#99/#101 and production `DisabledRunner` guards remain green.

### J4 — full affected regressions

At minimum:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p gateway-egress --all-targets
cargo test -p generic-ytdlp --features runtime-prep --test runtime -- --nocapture
python compile/static checks for the generic-ytdlp worker
```

Equivalent repository selectors are acceptable only if they are stricter and fully cover the same affected surfaces.

## Expected Files

Smallest necessary subset only, primarily:

```text
plugins/generic-ytdlp/worker/worker.py
plugins/generic-ytdlp/src/lib.rs
plugins/generic-ytdlp/src/smoke.rs
plugins/generic-ytdlp/tests/**
.github/workflows/generic-ytdlp-prep.yml   # only if needed for deterministic CI assertions
```

Do not change Gateway Core, R008 policy, sandbox/fd authority, frozen yt-dlp provenance, product playback, or production registration.

## Out of Scope

- real Bilibili/site extraction or any public-site request;
- executing or modifying #67 or #68;
- DASH/separate-A/V/remux/FFmpeg/media-format support;
- changing frozen yt-dlp version/source/wheel;
- changing #105 fallback admission or adding a generic retry/fallback path;
- Cookie/login/profile/fingerprint/CAPTCHA/proxy/access-control bypass;
- raw diagnostic inspection/persistence/exposure;
- R008/broker/fd/sandbox/Secret policy weakening;
- Core site-specific branches, Browser/Web E2E, TV/device work, performance tuning.

## Preconditions

- Read `AGENTS.md`, Issue #107 and its parent #67 Attempt 11 report/Coordinator Review, this `task.md`, `prompt.md`, lifecycle/freshness/recovery protocols, and accepted #101/#105 authorities before implementation.
- Confirm live #107 is exactly `status:ready + env:cloud + no active owner` before claim.
- Planning Base is `6034eb1cd1837988161d955ef0d1f67d60ce0257`; record final Task Candidate and Evidence Base explicitly.
- No real-site request may be used to discover or validate the stage taxonomy.

## Success Criteria

1. C1–C8 PASS on one exact Task Candidate.
2. J1–J4 PASS, including native hosted ARM64 equivalence.
3. `UNSUPPORTED_FORMAT` remains the top-level fixed result, with at most one admitted fixed secondary stage.
4. Every admitted stage is derived only from repository-owned control flow and is deterministic under offline fixtures.
5. Forged/unknown/malformed stage shapes remain fail closed.
6. Sentinel leak tests prove prohibited content does not cross worker/parser/smoke/log/artifact boundaries.
7. Existing generic-ytdlp successful media behavior and all non-unsupported #101 classifications remain unchanged.
8. No public network, #67/#68 execution, media-format implementation, or security-authority weakening occurs.
9. Worker creates/updates one focused PR, reports exact Candidate and J1–J4 Evidence, transitions to review/blocked, releases owner, and STOPs.

## Evidence Contract

Worker report may include only bounded facts:

```text
Issue / Attempt / Worker / Environment
Planning Base / Task Candidate / Evidence Base / observed Current Main
PR
J1-J4 run/job identifiers
architecture assertions
C1-C8 PASS/FAIL/BLOCKED
fixed top-level error code
fixed unsupported-stage enum values used by deterministic fixtures
sentinel-absence / cleanup booleans
Freshness fields required by protocol
```

Never publish raw stderr/exception text, source or media URLs, request/response headers or bodies, signed query data, credentials, Cookie/Auth/token/profile state, or media payloads.

## Failure / Blocked Handling

- `BLOCKED` if a trustworthy stage cannot be derived without prohibited diagnostics/public traffic or requires changing #101/#105/R008/sandbox/fd/security authority.
- `FAIL` if the implementation broadens trusted strings, changes top-level taxonomy semantics, falsely classifies malformed data, or breaks required regressions.
- A deterministic `MEDIA_SHAPE` fixture is **not** authorization to implement DASH/remux/media-format support.
- Worker must not create the downstream compatibility Task or rerun #67.

## Freshness / Integration Contract

Freshness policy: dependency-aware

### Semantic authorities

- #67 Attempt 11 bounded report: exact runtime Candidate `1a38e403a3252239822aeb2a784a20fdfd18c0a6`, `UNSUPPORTED_FORMAT`, 4 broker 2xx requests, no current ResolvedMedia, no raw diagnostic Evidence.
- #105 Attempt 3 Final Acceptance / PR #106 merge `1a38e403a3252239822aeb2a784a20fdfd18c0a6`: normal `extract` first, narrow missing-initial-state continuation, no caller-selectable fallback action.
- #101 fixed bounded top-level worker taxonomy.
- #99 clean-build sandbox binding; #97 broker framing; #95/R008 Secret containment; #85/#83 fd/sandbox authority; #79 frozen offline runtime.
- current `GenericYtdlpAdapter` / `ResolvedMedia` parser contract and production `DisabledRunner`.

### Semantic freshness domains

- `plugins/generic-ytdlp/worker/**` unsupported/fallback control flow;
- `plugins/generic-ytdlp/src/**` worker envelope parsing and safe smoke summaries;
- generic-ytdlp deterministic runtime fixtures/tests;
- #101 top-level taxonomy and #105 fallback admission;
- R008/Secret/broker/fd/sandbox boundaries when they affect the bounded worker envelope.

### Integration surfaces

- Cargo workspace / `Cargo.toml` / `Cargo.lock` if touched by concurrent accepted work;
- generic-ytdlp runtime-prep feature composition;
- `.github/workflows/generic-ytdlp-prep.yml` when changed;
- shared `gateway-egress` types used by generic-ytdlp runtime tests.

### Task-owned surfaces

- smallest necessary generic-ytdlp worker/parser/smoke implementation for the fixed stage envelope;
- deterministic stage fixtures/tests;
- narrowly necessary generic-ytdlp CI assertions.

### Authority/domain → Claim mapping

- #101 top-level taxonomy → C1, C4, C5, C7.
- #105 admission/control-flow authority → C2, C5, C7, C8.
- worker/parser/smoke bounded output → C1, C2, C3, C4, C5.
- R008/#95/#97/#83/#85/#99 security/runtime authority → C3, C7.
- architecture/runtime matrix → C6, C7.
- #67 Attempt 11 bounded evidence → C8 and Task justification only; it does not prove a media-format cause.

### Integration verification jobs

- `JI1`: on exact Integration Candidate, run focused `generic-ytdlp` runtime/parser/smoke tests including the closed stage matrix and negative envelopes.
- `JI2`: if shared workspace/dependency or `gateway-egress` composition changed, run the affected workspace/gateway-egress build/tests plus stage sentinel guards; otherwise `n/a`.

### Unrelated-main policy

Unrelated main changes do not invalidate exact-Candidate Task Evidence and do not require full rerun or rebase solely for freshness.

### Integration-overlap policy

If accepted main changes only shared build/workspace/composition surfaces without changing #101/#105 or bounded-output semantics, preserve Task semantic Evidence and run declared `JI1/JI2` on a frozen Integration Candidate.

### Semantic-authority-change policy

If #101 taxonomy, #105 admission/control flow, current bounded parser/output contract, R008/Secret/broker/fd/sandbox authority, or another mapped semantic domain changes, Coordinator must classify `SEMANTIC_AUTHORITY`, reconcile the accepted authority, and reverify mapped affected Claims before merge.

### Contract-invalidating policy

If Scope, Goal, allowed stage semantics, top-level `UNSUPPORTED_FORMAT` requirement, security boundary, or downstream decomposition changes, return #107 to `status:draft` for Contract Revision and a new Publication Gate.

### Strict-main reason

`n/a` — #107 is dependency-aware, not a proof of the complete moving main snapshot.

## Completion Protocol

```text
status:ready
→ Worker claim / Attempt 1
→ status:in-progress
→ implementation + exact-Candidate J1-J4
→ [EXECUTION REPORT] | [BLOCKER REPORT]
→ status:review | status:blocked
→ release owner
→ STOP
```

Coordinator alone reviews/merges/closes #107 and decides whether/when to re-freeze #67. Worker must not start #67/#68 or create a media-format child Task.