# Task — GENERIC-YTDLP-BILIBILI-FALLBACK-COMPAT-PREP

## Metadata

```text
GitHub Issue: #109
Task ID: GENERIC-YTDLP-BILIBILI-FALLBACK-COMPAT-PREP
Task kind: implementation + deterministic security/compatibility verification
Parent Goal / Research Item: #67 GENERIC-YTDLP-BILIBILI-REAL Attempt 12
Planning Base: 1dd9a213f95373beecf41aa1a5c4d2a08a7f597f
Runtime authority under diagnosis: #107 merge 234c616f128deaee55156675d480d03ac5e8670d
Preferred worker: cloud-codex
Eligible environment: env:cloud
Execution plane: GitHub Actions
Downstream: #67 next Contract Revision / target Attempt
Freshness policy: dependency-aware
Publication state: non-executable until Coordinator Publication Gate
```

## Trigger Evidence

#67 Attempt 12 executed exact Candidate `234c616f128deaee55156675d480d03ac5e8670d` on the accepted low-privilege Ubuntu ARM64 target. The accepted runtime/security path remained intact:

```text
J0 PASS
J1 PASS
J2 PASS
J3 COMPLETED
runtime_cache: offline-hit
broker_status_class: 2xx
broker_error_code: n/a
broker_request_count: 4
process_error: UNSUPPORTED_FORMAT
unsupported_stage: FALLBACK_WEBPAGE
protocol: n/a
stream_count: 0
J4 PASS
Overall: FAIL
```

This is a valid current-contract compatibility FAIL, not an infrastructure blocker. It proves that the earliest actionable repository rejection phase is `FALLBACK_WEBPAGE`. It does **not** prove the exact webpage rejection reason, DASH, separate A/V, media shape, or any site-provided diagnostic.

The current fallback implementation also collapses many repository-owned decisions into the same coarse stage. The same pattern repeats in later phases (`FALLBACK_NAV`, `FALLBACK_VIEW`, `FALLBACK_DETAIL`, `FALLBACK_PLAYURL`, `MEDIA_SHAPE`). Repeating one diagnostic Issue and one real target rerun per phase would create unnecessary feedback cycles.

## Goal

Close the **entire repository-owned Bilibili fallback state machine** in one deterministic offline Task:

```text
normal extract first
→ #105 narrow fallback admission
→ webpage
→ nav / WBI
→ view
→ detail
→ playurl
→ media-shape normalization
→ current ResolvedMedia OR one bounded actionable rejection
```

The Candidate must provide enough closed, Secret-safe control-flow attribution to distinguish the current repository-owned rejection classes across all fallback phases in a later #67 target run, while preserving the existing top-level #101 error and #107 stage taxonomy.

The Task must also build one deterministic positive traversal that reaches the end of the full fallback state machine and one deterministic negative matrix that exercises every repository-owned rejection family. This is intended to prevent another `webpage → rerun → nav → rerun → view → rerun` diagnostic chain.

## Core Contract

Top-level outcome remains unchanged:

```text
UNSUPPORTED_FORMAT
```

The #107 stage remains unchanged and closed:

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

A Candidate may add **at most one** optional fixed `fallback_reason` (exact field name is implementation-owned) only when the top-level outcome is `UNSUPPORTED_FORMAT` and the reason is valid for the accompanying #107 stage.

The reason is repository control-flow metadata only. It must never contain or derive from response text, exception text, HTTP reason text, URL components, BVID/CID, site error codes, format IDs, codec names, headers, body content, tokens, or arbitrary strings.

## Reason Coverage Requirement

The exact constant names are implementation-owned, but the final closed taxonomy must cover the current rejection families below without collapsing distinct repair authorities into one indistinguishable value.

### Common bounded response reasons

```text
RESPONSE_STATUS
RESPONSE_BODY_TOO_LARGE
RESPONSE_ENCODING
RESPONSE_JSON
RESPONSE_SECRET_FIELD
```

These may be reused across applicable fallback API stages only when stage + reason remains unambiguous.

### FALLBACK_WEBPAGE

At minimum distinguish:

```text
WEBPAGE_NOT_HTML
WEBPAGE_BANGUMI
```

`__INITIAL_STATE__` present is not an `UNSUPPORTED_FORMAT` repair reason if the existing control flow treats the fallback as not applicable; preserve that semantic distinction.

### FALLBACK_NAV

At minimum distinguish:

```text
NAV_API_ENVELOPE
NAV_SHAPE
NAV_WBI_SHAPE
NAV_WBI_URL
```

### FALLBACK_VIEW

At minimum distinguish:

```text
VIEW_API_ENVELOPE
VIEW_ID_MISMATCH
VIEW_TITLE
VIEW_PAGES
VIEW_CID
```

### FALLBACK_DETAIL

At minimum distinguish:

```text
DETAIL_API_ENVELOPE
DETAIL_SHAPE
DETAIL_ID_MISMATCH
DETAIL_TITLE
DETAIL_PAGES
DETAIL_CID_MISMATCH
DETAIL_TITLE_MISMATCH
```

### FALLBACK_PLAYURL

At minimum distinguish:

```text
PLAYURL_API_ENVELOPE
PLAYURL_DURL_SHAPE
PLAYURL_DASH_PRESENT
PLAYURL_SEGMENT_SHAPE
PLAYURL_SEGMENT_FIELDS
```

### MEDIA_SHAPE / final stream admission

At minimum distinguish the existing repository-owned classes for:

```text
MEDIA_URL_SHAPE
MEDIA_URL_SENSITIVE_QUERY
MEDIA_EXTENSION
MEDIA_HEADERS
MEDIA_TITLE
MEDIA_NO_MUXED_STREAM
```

### UNCLASSIFIED

Keep one fixed `UNCLASSIFIED` reason for any unsupported path that cannot safely be mapped from repository control flow alone.

The Worker may use a smaller taxonomy only if it proves that merged values share the same security boundary, repair authority, and downstream action. The Worker must document the final stage→reason mapping in code/tests and in its report.

## Full-State Deterministic Matrix

The Task must not stop after instrumenting `FALLBACK_WEBPAGE`.

Create deterministic offline fixtures that prove:

1. one positive end-to-end fallback traversal:

```text
webpage accepted
→ nav accepted
→ view accepted
→ detail accepted
→ playurl accepted
→ current muxed http-file ResolvedMedia
```

2. one negative fixture for every admitted stage→reason family;
3. later-stage fixtures are reachable without relying on a prior rejected phase;
4. forged/unknown/malformed/wrong-stage/wrong-error reason envelopes fail closed;
5. existing normal-extract success remains unchanged;
6. existing #101 non-unsupported failures remain unchanged and cannot carry `unsupported_stage`/`fallback_reason`;
7. the existing #105 narrow admission remains normal-extract-first and cannot be caller-selected;
8. sentinel-bearing raw diagnostics never cross worker/parser/smoke/log/artifact boundaries.

No public network is permitted in this Task.

## Bounded Compatibility Repair Authority

This Task is broader than #107 but is **not** permission for speculative site support.

The Worker may repair a repository-owned fallback compatibility defect in this same #109 only when all of the following are true:

1. the defect is reproducible by deterministic offline fixtures within the existing fallback state machine;
2. the repair does not change #105 fallback admission, R008/Secret/broker/sandbox/fd authority, frozen yt-dlp identity, or production registration;
3. the repair does not add DASH/separate-A/V/remux/FFmpeg or a new site capability;
4. the accepted behavior remains representable by the current `http-file | hls` first-playback contract;
5. positive and negative fixtures prove the widening is structural/compatibility-only and does not admit Secret-bearing, signed, malformed, redirect-bypass, arbitrary-host, or otherwise unsafe data;
6. the Worker records the exact deterministic reason and proof in the PR/report.

Examples of allowed repair classes are internal parser/normalizer inconsistencies or overly rigid structurally-equivalent schema checks that can be proven safe offline. If a proposed change requires real-site contents, unknown site semantics, access-control work, media recomposition, or weakened security policy, do not implement it here.

If no deterministic defect is proven, #109 may still succeed as a complete bounded-observability + full-state fixture Task; the later #67 rerun will provide the real stage+reason authority for the next concrete compatibility repair.

## Security Invariants

1. #101 top-level taxonomy remains unchanged.
2. #107 stage taxonomy remains unchanged and closed.
3. `fallback_reason` is a closed repository enum and valid only for its declared stage.
4. No arbitrary diagnostic string is serialized, logged, retained, or exposed.
5. Raw stderr/traceback/exception text, request/response headers, response body/page text, source/redirect/media URLs, signed query material, credentials, Cookie/Auth/token/profile/account state, and media payloads remain outside durable Evidence.
6. Reason attribution is derived from repository control flow, never text matching.
7. #105 normal-extract-first admission remains unchanged; no caller-selectable fallback action is added.
8. R008/#95 request/response Secret containment, #97 broker framing, #99 clean-build binding, #83/#85 sandbox/fd isolation, broker-only egress and production `DisabledRunner` remain unchanged.
9. Unknown/forged/malformed/wrong-stage/wrong-error reason envelopes fail closed.
10. No public Bilibili or other real-site request occurs in #109.
11. No #67 or #68 execution occurs in #109.

## Claims

```text
C1: #101 UNSUPPORTED_FORMAT and all other top-level outcomes remain unchanged
C2: #107 stage semantics remain unchanged and closed
C3: full fallback state machine has closed stage+reason attribution for all repository-owned rejection families
C4: one deterministic positive fixture traverses webpage→nav→view→detail→playurl→current ResolvedMedia
C5: forged/unknown/malformed/wrong-stage/wrong-error reason envelopes fail closed
C6: no raw diagnostic/Secret/site-response/media material crosses worker/parser/smoke/log/artifact boundaries
C7: existing normal-extract success, #105 admission and non-unsupported #101 failures remain unchanged
C8: x86_64 and native hosted ARM64 behavior is equivalent on the complete deterministic matrix
C9: any inline compatibility repair is fixture-proven, current-contract-only, and does not weaken security authority
C10: R008/#95/#97/#99/#83/#85/#101/#105/#107/DisabledRunner authorities remain intact
C11: no real-site request, #67/#68 execution, DASH/remux/FFmpeg or new access capability occurs
C12: result is sufficient for one later Coordinator-controlled #67 rerun without needing another stage-only diagnostic child
```

## Job Matrix

### J1 — hosted x86_64 full fallback matrix

- assert exact Candidate identity;
- execute deterministic positive full-state traversal;
- execute every stage→reason negative fixture;
- execute forged/unknown/malformed envelope negatives;
- prove C1–C7 and C9;
- prove no public request.

### J2 — native hosted ARM64 equivalence

Runner must assert native `aarch64`.

- repeat the full positive + negative state-machine matrix on the same exact Candidate;
- preserve accepted sandbox/runtime tests;
- prove C8 and no public request.

### J3 — security/static guards

- reason values are fixed constants only;
- no exception/response/URL/header/body-derived reason construction;
- no Secret/signed material in fixtures or durable outputs;
- no new Cookie/Auth/profile/proxy/browser/direct-egress capability;
- no #105 admission broadening/caller-selectable fallback;
- R008/#95/#97/#99/#83/#85/#101/#107 and `DisabledRunner` guards remain green;
- if a compatibility widening is implemented, prove its positive/negative fixture boundary and no security-authority change.

### J4 — full affected regressions

At minimum:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p gateway-egress --all-targets
cargo test -p generic-ytdlp --features runtime-prep --test runtime -- --nocapture
python compile/static checks for plugins/generic-ytdlp/worker/worker.py
```

Equivalent selectors are acceptable only if stricter and demonstrably cover the same affected surfaces.

## Expected Files

Smallest necessary subset only, primarily:

```text
plugins/generic-ytdlp/worker/worker.py
plugins/generic-ytdlp/src/lib.rs
plugins/generic-ytdlp/src/smoke.rs
plugins/generic-ytdlp/tests/**
.github/workflows/generic-ytdlp-prep.yml   # only if needed for deterministic CI assertions
```

Do not modify Gateway Core, R008 policy, sandbox/fd authority, frozen yt-dlp version/provenance, product playback routing, site account/login state, or production plugin registration.

## Out of Scope

- public Bilibili/site traffic;
- executing/modifying #67 or #68;
- DASH, separate A/V, remux, FFmpeg, transcoding or media recomposition;
- changing frozen yt-dlp version/source/wheel;
- broadening #105 initial-state fallback admission or adding a generic retry path;
- Cookie/login/profile/fingerprint/CAPTCHA/proxy/access-control bypass;
- raw diagnostic inspection/persistence/exposure;
- R008/broker/fd/sandbox/Secret policy weakening;
- Core site-specific branches, Browser/Web E2E, TV/device work or performance tuning.

## Preconditions

- Read `AGENTS.md`, live Issue #109, #67 Attempt 12 report/Coordinator Review, this task, prompt, lifecycle/freshness/recovery protocols, and accepted #101/#103/#105/#107 authorities.
- Confirm live #109 is exactly `status:ready + env:cloud + no active owner` before claim.
- Planning Base is `1dd9a213f95373beecf41aa1a5c4d2a08a7f597f`; record final Candidate and Evidence Base explicitly.
- No real-site request may be used to discover or validate stage/reason behavior.

## Success Criteria

1. C1–C12 PASS on one exact Candidate.
2. J1–J4 PASS, including native hosted ARM64 equivalence.
3. The deterministic positive fixture traverses the full fallback chain through current `ResolvedMedia`.
4. Every current repository-owned unsupported rejection family maps to a closed stage+reason or documented safe `UNCLASSIFIED` fallback.
5. The stage+reason pair is derived only from repository control flow.
6. Forged/unknown/malformed combinations fail closed.
7. Existing generic-ytdlp success and non-unsupported failures remain unchanged.
8. Any compatibility repair is proven by deterministic positive/negative fixtures and remains within current `http-file | hls` semantics.
9. Sentinel leak tests prove prohibited content does not cross bounded boundaries.
10. No public network, #67/#68 execution, media recomposition, access-control work, or security-authority weakening occurs.
11. Worker creates one focused PR, reports exact Candidate + J1–J4 Evidence, transitions to `status:review|status:blocked`, releases owner, and STOPs.
12. After Final Acceptance, Coordinator can re-freeze #67 directly for one target rerun; do not create another stage-only diagnostic child from #109.

## Evidence Contract

Worker report may contain only bounded facts:

```text
Attempt / Candidate / PR / Actions run
final stage→reason allowlist mapping
positive full-state fixture PASS/FAIL
negative matrix PASS/FAIL
forged-envelope matrix PASS/FAIL
compatibility repair implemented: yes/no + fixed repository-owned reason only
J1/J2/J3/J4
C1-C12
x86_64 / native ARM64 identity
safe-output sentinel scan
freshness classification
Overall
#67 rerun readiness yes/no
```

Do not include raw response bodies, URLs, query strings, headers, exception/stderr text, site payloads, media payloads, credentials, Secret material, or site-provided diagnostics.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #107 Final Accepted / merge `234c616f128deaee55156675d480d03ac5e8670d` — closed unsupported stage;
- #105 Final Accepted / merge `1a38e403a3252239822aeb2a784a20fdfd18c0a6` — normal-extract-first narrow Bilibili initial-state continuation;
- #103 Final Accepted / merge `bec606fe0346e60fa5f05f98e27981fca8feffb2` — ResolvedMedia compatibility seam;
- #101/#99/#97/#95/#85/#83/#79 and R008/ADR0007 — failure taxonomy, clean-build, broker framing, Secret containment, fd/sandbox, frozen runtime.

Semantic freshness domains:
- `plugins/generic-ytdlp/**`;
- generic-ytdlp worker/parser/smoke/test state machine;
- #101/#105/#107 failure/stage semantics;
- #103 current ResolvedMedia normalization;
- R008/broker/Secret/sandbox/fd boundaries consumed by generic-ytdlp.

Integration surfaces:
- worker unsupported envelope → Rust parser/runtime → smoke output;
- full offline fallback fixture path → current ResolvedMedia;
- x86_64/native ARM64 deterministic equivalence.

Task-owned surfaces:
- generic-ytdlp fallback observability, deterministic fixtures, and fixture-proven repository-owned compatibility repairs only.

Authority/domain → Claims:
- #101/#107 → C1/C2/C3/C5/C6;
- #105/#103 → C4/C7/C9/C12;
- #79/#83/#85/#95/#97/#99/R008 → C6/C8/C10/C11;
- full deterministic state-machine matrix → C3/C4/C5/C8/C12.

Unrelated-main policy:
- unrelated documentation/product movement does not invalidate exact-Candidate Evidence; classify before merge/review.

Semantic-authority-change policy:
- if accepted main changes any listed semantic authority/domain before Candidate review, Coordinator must reconcile mapped Claims before ACCEPT.

## Stop Boundary

```text
normal: [EXECUTION REPORT] → status:review → release owner → STOP
blocked: [BLOCKER REPORT] → status:blocked → release owner → STOP
```

Worker must not merge, mark done/close #109, execute #67/#68, create the next compatibility child, or broaden the Task beyond this contract.
