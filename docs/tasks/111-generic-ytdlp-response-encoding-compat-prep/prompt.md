# Session Bootstrap — GENERIC-YTDLP-RESPONSE-ENCODING-COMPAT-PREP

Execute Issue #111 using the repository Worker protocol.

## Claim gate

Claim only if live #111 is exactly:

```text
status:ready
env:cloud
no active owner
```

If draft, blocked, review, done, closed, or already owned: STOP.

## Frozen task

```text
Task: #111 GENERIC-YTDLP-RESPONSE-ENCODING-COMPAT-PREP
Attempt: 1
Planning Base: 57d03a6875f957805c4dcb3dc09a139e65548fee
Runtime authority under repair: af65b2e2fec4cd3b3303db19415890f4052aa026
Parent: #67 R13 Attempt 13
Parent real result:
  process_error: UNSUPPORTED_FORMAT
  unsupported_stage: FALLBACK_WEBPAGE
  fallback_reason: RESPONSE_ENCODING
  broker_status_class: 2xx
  broker_request_count: 4
Environment: env:cloud
Public/real-site network: forbidden
```

Read `AGENTS.md`, live #111, `docs/tasks/111-generic-ytdlp-response-encoding-compat-prep/task.md`, lifecycle/freshness/recovery protocols, #67 R13 Execution Report + Coordinator Review, and accepted #95/#101/#105/#107/#109 authorities before implementation.

## Source-first starting point

The accepted Candidate currently has this repository-owned path:

```text
BrokerResponse body bytes
→ BrokerRH Response
→ _fallback_response_body()
→ response.read(bound)
→ body.decode("utf-8")
→ UnicodeDecodeError => RESPONSE_ENCODING
```

The real R13 Evidence intentionally retained no response body or headers. Do **not** claim that the actual source used gzip, br, deflate, a specific charset, anti-bot content, DASH or any other unproved site content.

## Goal

Implement one bounded generic repair for response byte-to-text/content-coding compatibility before the existing UTF-8/JSON/HTML fallback admission.

This is not another diagnostic Task. The Candidate must add an actual deterministic compatibility mechanism with a closed admitted encoding/content-coding set and fail-closed behavior for everything else.

## Required properties

- normalized text output remains capped at 96 KiB;
- expansion bound is enforced after decoding/decompression;
- existing broker/R008 body/frame limits are not enlarged speculatively;
- malformed/truncated/unknown/ambiguous/over-expanding inputs fail closed;
- `RESPONSE_READ`, `RESPONSE_ENCODING`, `RESPONSE_JSON`, `WEBPAGE_NOT_HTML`, `RESPONSE_BODY_TOO_LARGE` remain distinct;
- identity UTF-8 behavior remains unchanged;
- #109 positive full fallback traversal still reaches current muxed `http-file` ResolvedMedia;
- #109 negative + forged stage/reason matrix remains PASS;
- hosted x86_64 and native hosted ARM64 must prove equivalent behavior.

The exact implementation layer is yours to determine from repository architecture, but normalized body bytes and response metadata must remain internally consistent and bounded.

## Hard boundaries

- no public Bilibili or other real-site request;
- no #67 or #68 execution;
- no raw response/header/body/page diagnostics in durable output;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/access-control bypass;
- no R008/#95/broker/#83/#85/#97/#99 weakening;
- no broadened #105 admission;
- no changes to #101/#107/#109 public error semantics except implementation needed to preserve them;
- no DASH/separate-A/V/remux/FFmpeg/transcoding/navigation/Browser/Web-E2E/performance scope;
- production `DisabledRunner` remains intact;
- no speculative claim about what encoding the real Bilibili response used.

## Required proof

Run J1-J4 exactly as defined in `task.md`.

Report at minimum:

```text
Candidate
PR
Actions run(s)
final admitted response-encoding/content-coding allowlist
normalization layer / invariant
normalized output bound
identity fixtures
admitted encoding fixtures
malformed/truncated/unknown/expansion fixtures
full #109 positive traversal
#109 negative/forged matrix
J1/J2/J3/J4
C1-C12
x86_64 identity
native ARM64 identity
safe-output sentinel scan
freshness
Overall
#67 rerun readiness
```

Never include prohibited raw materials.

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

Worker must not merge, set `status:done`, close #111, rerun #67, execute #68, or create another Task.

This prompt becomes executable only after Coordinator Publication Gate records PUBLISH and live #111 is `status:ready + env:cloud + no active owner`.
