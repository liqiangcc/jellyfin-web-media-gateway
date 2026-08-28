# Session Bootstrap — GENERIC-YTDLP-BILIBILI-FALLBACK-COMPAT-PREP

Execute Issue #109 using the repository Worker protocol.

## Claim gate

Claim only if live #109 is exactly:

```text
status:ready
env:cloud
no active owner
```

If it is draft, blocked, review, done, closed, or already owned: STOP.

## Frozen task

```text
Task: #109 GENERIC-YTDLP-BILIBILI-FALLBACK-COMPAT-PREP
Attempt: 1
Planning Base: 1dd9a213f95373beecf41aa1a5c4d2a08a7f597f
Parent Evidence Candidate: 234c616f128deaee55156675d480d03ac5e8670d
Parent real result:
  process_error: UNSUPPORTED_FORMAT
  unsupported_stage: FALLBACK_WEBPAGE
  broker_request_count: 4
Environment: env:cloud
Public/real-site network: forbidden
```

Read `AGENTS.md`, live #109, `docs/tasks/109-generic-ytdlp-bilibili-fallback-compat-prep/task.md`, lifecycle/freshness/recovery protocols, #67 Attempt 12 report/Coordinator Review, and accepted #101/#103/#105/#107 authorities before implementation.

## Goal

Do **not** implement another webpage-only diagnostic.

Close the full repository-owned fallback state machine in one deterministic offline Candidate:

```text
normal extract
→ #105 narrow fallback admission
→ webpage
→ nav
→ view
→ detail
→ playurl
→ media shape
→ current ResolvedMedia OR closed actionable unsupported stage+reason
```

Required outcomes:

1. preserve top-level #101 `UNSUPPORTED_FORMAT`;
2. preserve #107 `unsupported_stage` exactly;
3. add at most one closed repository-owned `fallback_reason`, valid only for the corresponding stage;
4. map all current repository-owned fallback rejection families, not just webpage;
5. build one deterministic positive fixture that traverses the whole fallback chain to current muxed `http-file` ResolvedMedia;
6. build deterministic negatives for every admitted stage→reason family and forged envelopes;
7. prove identical behavior on hosted x86_64 and native hosted ARM64;
8. preserve all Secret/R008/broker/sandbox/fd/no-direct-egress/DisabledRunner boundaries.

## Compatibility repair authority

You may repair a repository-owned fallback compatibility defect in this same Task **only** when it is demonstrated by deterministic offline positive/negative fixtures and stays inside the current muxed `http-file | hls` contract.

Do not make speculative site changes. Do not use live Bilibili contents to invent a fixture. Do not add DASH/separate-A/V/remux/FFmpeg/transcoding. Do not broaden #105 admission, R008 policy, Secret handling, broker authority, sandbox/fd authority, or production registration.

If no deterministic compatibility defect is proven, complete the full observability + fixture matrix and report that fact; a later Coordinator-controlled #67 rerun will supply the real stage+reason.

## Hard boundaries

- no public Bilibili or other real-site request;
- no #67 or #68 execution;
- no raw stderr/traceback/exception text;
- no request/response header or page/body capture in durable output;
- no source/redirect/media URL, query token, signed material, credentials, Cookie/Auth/token/profile/account state or media payload in Evidence;
- no reason derived by string matching diagnostics/site contents;
- no caller-selectable fallback action;
- no new direct network/socket/proxy/browser authority;
- preserve frozen yt-dlp 2026.08.19 identity/provenance;
- preserve #79/#83/#85/#95/#97/#99/#101/#103/#105/#107/R008/ADR0007 and production `DisabledRunner`.

## Required proof

Run J1-J4 exactly as defined in `task.md`.

The Worker report must explicitly include:

```text
Candidate
PR
Actions run
final stage→reason allowlist mapping
positive full-state fixture: PASS/FAIL
negative matrix: PASS/FAIL
forged-envelope matrix: PASS/FAIL
compatibility repair implemented: yes/no
J1/J2/J3/J4
C1-C12
x86_64 identity
native ARM64 identity
safe-output sentinel scan
freshness
Overall
#67 rerun readiness
```

Never include the prohibited raw materials listed above.

## Stop boundary

Normal:

```text
[EXECUTION REPORT]
→ status:review
→ release active owner
→ STOP
```

Blocked:

```text
[BLOCKER REPORT]
→ status:blocked
→ release active owner
→ STOP
```

Worker must not merge, set `status:done`, close #109, rerun #67, execute #68, or create the downstream compatibility child.

This prompt is execution authority only after Coordinator Publication Gate records PUBLISH and live #109 is `status:ready + env:cloud + no active owner`.
