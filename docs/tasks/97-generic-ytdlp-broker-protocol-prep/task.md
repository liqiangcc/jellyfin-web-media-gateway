# Task — GENERIC-YTDLP-BROKER-PROTOCOL-PREP

## Metadata

```text
GitHub Issue: #97
Task ID: GENERIC-YTDLP-BROKER-PROTOCOL-PREP
Task kind: implementation + deterministic runtime/protocol verification
Planning Base: eb605105f265329d84b5c09bd4d75c1ace44a2ec
Triggering #67 Candidate: 804fd60343b081e5e055ba87f68e7939b106bb19
Parent blocker: #67 GENERIC-YTDLP-BILIBILI-REAL Attempt 6
Preferred worker: cloud-codex
Eligible environment after publication: env:cloud
Accepted runtime/security authorities: #60, #79, #83, #85, #95 / ADR 0007, R008
Downstream: #67 next Attempt only after #97 Final Acceptance
Freshness policy: dependency-aware
```

> #97 owns only the inherited broker IPC/wire protocol required to carry an already R008-accepted bounded response between Gateway and the sandboxed generic-ytdlp worker. It does not own real-site compatibility, HTTP egress policy, sandbox authority, media-format support or production enablement.

## Trigger / Evidence

#67 Attempt 6 reached the accepted real path on the Ubuntu ARM64 target and proved:

```text
runtime_cache: offline-hit
formal Bilibili direct/no-proxy: 2xx
ARM64 sandbox: PASS
#85 close_range=ENOSYS fallback: PASS
R008 broker traffic: reached
#95 response Secret containment: accepted-path
broker_status_class: 2xx
broker_error_code: n/a
broker_request_count: 1
process_error: BROKER_PROTOCOL
reproduction: 2/2
ResolvedMedia: not reached
```

Therefore the new blocker is after an R008 response is produced and before the brokered worker completes extraction. It is not a Bilibili format result and does not authorize weakening any earlier boundary.

Current code facts at the triggering Candidate:

- `BrokerProcessRunner` maps an unexpected broker frame read or broker response write failure to `ProcessError::BrokerProtocol`;
- the inherited protocol uses a fixed 4-byte big-endian length prefix and a bounded frame;
- the Python worker uses the same length-prefix model and reconstructs the broker response before passing it to yt-dlp;
- `BrokerResponse.body` is binary HTTP body data represented inside the current serialized response envelope.

## Goal

Determine the exact protocol failure layer and implement the smallest bounded correction so an R008-accepted response can cross the inherited fd-3 IPC boundary and be consumed by the worker.

Required decomposition:

```text
R008 BrokerResponse
→ response serialization
→ frame-size admission
→ Rust write to fd 3
→ Python length read
→ Python envelope decode
→ body/header reconstruction
→ yt-dlp RequestHandler continuation
```

The implementation must identify the failed edge rather than treating `BROKER_PROTOCOL` as a generic reason to enlarge limits.

## Required hypothesis test

At minimum prove or disprove this hypothesis with deterministic synthetic data:

```text
H1:
R008-accepted binary body
→ current response serialization expands body representation
→ serialized wire envelope exceeds current frame bound
→ Rust response write fails
→ BrokerProcessRunner reports BROKER_PROTOCOL
```

If H1 is false, identify the actual read/write/decode/reconstruction failure with a bounded non-secret diagnostic classification and fix only that protocol-local cause.

## Protocol authority / boundedness

The Task may change internal broker wire representation or a fixed wire-envelope bound only when all of the following hold:

1. R008 HTTP body/header/count/value limits are unchanged.
2. The IPC envelope remains fixed and fail-closed, never caller-configurable.
3. Any wire bound is explicitly derived from the maximum admitted HTTP response envelope plus fixed protocol overhead, or the representation is changed so the existing bound is sufficient.
4. Malformed length, zero length, impossible length, truncated frame and oversize frame fail closed.
5. No alternate socket, streaming tunnel, CONNECT path, shared filesystem handoff or direct worker network path is introduced.
6. Response Secret material remains contained before it can enter the worker-visible envelope under #95 / ADR 0007.
7. No raw body, Secret or signed URL is added to diagnostics merely to identify the failure.

## Scope

Allowed implementation surfaces are the smallest subset required from:

```text
plugins/generic-ytdlp/src/runtime.rs
plugins/generic-ytdlp/worker/worker.py
plugins/generic-ytdlp/tests/**
.github/workflows/** only for exact-Candidate #97 verification
small protocol-local test helpers / docs if required
```

`gateway-egress/**` may be read as authority. Do not change R008 egress/body/Secret policy. If fixing the protocol genuinely requires changing `gateway-egress` semantics, STOP and report a scope blocker for Coordinator revision.

## Frozen boundaries

- no real Bilibili/site request required or authorized for #97 acceptance;
- do not increase R008 HTTP body/header/count/value limits;
- no DNS/public-IP/address-pinning/TLS/per-hop redirect weakening;
- no response Secret declassification and no Cookie/Auth store/replay;
- no request Cookie/Auth/token authority;
- no direct AF_INET/AF_INET6/alternate AF_UNIX worker authority;
- #83 seccomp/no_new_privs and #85 fd isolation remain unchanged;
- no root/sudo/Target environment changes;
- frozen yt-dlp version and #79 offline-runtime provenance remain unchanged;
- production `GenericYtdlpAdapter::default()` remains `DisabledRunner`;
- no DASH/separate-A/V/remux/FFmpeg/navigation/Browser/Web E2E/performance work;
- no #67 execution and no #68 work inside this Task.

## Claims

```text
C1 — Root-cause localization
A deterministic test identifies the exact BrokerProtocol failure edge represented by #67 Attempt 6, or proves a protocol-equivalent pre-fix failure with the same bounded condition.

C2 — Accepted-response delivery
An R008-compatible synthetic response near the accepted body envelope crosses the broker IPC and is consumed by the Python worker without BROKER_PROTOCOL.

C3 — Wire bound remains fail-closed
Zero/truncated/malformed/impossible/oversize request and response frames remain rejected without unbounded allocation or transport authority.

C4 — No HTTP-policy expansion
R008 body/header/count/value limits and egress semantics are unchanged; any IPC envelope change is derived/fixed protocol overhead, not a new caller payload authority.

C5 — Secret containment preserved
#95 response Secret material never enters the worker-visible wire envelope; request Secret rejection remains before prohibited side effects.

C6 — Sandbox/broker authority preserved
Worker still has no direct network/alternate IPC authority; inherited fd 3 remains the sole HTTP capability under BrokerProcessRunner + R008Broker.

C7 — Lifecycle preserved
Timeout/cancel/read/write failure/worker exit/descendant cleanup remain bounded and do not leak raw stderr/body/Secret material.

C8 — Runtime compatibility regressions
#60/#66 generic-ytdlp deterministic runtime/parser/security regressions remain PASS; frozen yt-dlp identity is unchanged.

C9 — Production boundary
`GenericYtdlpAdapter::default()` remains disabled and #97 does not claim real Bilibili or #68 success.
```

## Verification

All required jobs must assert the exact final Task Candidate SHA.

### J1 — Protocol root cause + accepted envelope

Deterministic Linux hosted verification must include:

- a pre-fix/root-cause reproduction fixture tied to the `BROKER_PROTOCOL` edge;
- small normal response round-trip;
- response with contained Secret headers plus safe public body/header continuation;
- binary/synthetic body sizes around meaningful wire thresholds, including a body near the accepted R008 envelope;
- proof that the fixed response reaches/returns through the actual Rust `BrokerProcessRunner` + Python worker protocol, not merely a standalone encoder.

No real site is needed.

### J2 — Protocol/security negatives

Must include:

- zero-length frame;
- truncated length/payload;
- declared oversize frame;
- malformed envelope/JSON or equivalent chosen codec;
- response envelope over the fixed derived wire bound;
- request Secret negative before egress;
- response Secret sentinel absent from worker-visible serialization;
- timeout/cancellation/broker disconnect/write failure cleanup.

No raw fixture Secret may appear in durable logs.

### J3 — Runtime/workspace regression

At minimum:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
relevant generic-ytdlp runtime-prep tests
worker Python syntax/import bounded verification
core/site architecture guard
static proof production default is DisabledRunner
```

If parallel test resource contention is known, serialization is allowed as a verification detail but may not hide failing tests.

## Success criteria

1. C1-C9 explicitly PASS on one exact final Candidate.
2. The real #67 `BROKER_PROTOCOL` class has a deterministic protocol-level root-cause explanation supported by code/test Evidence, not guesswork.
3. An R008-accepted near-limit synthetic response crosses the actual broker IPC and is consumable by the worker.
4. R008 HTTP/security limits are not expanded.
5. Wire framing remains fixed, bounded and fail-closed.
6. #95 response containment and request Secret rejection remain intact.
7. Sandbox/fd/no-direct-egress authority remains intact.
8. No Bilibili/site request is required to accept #97.
9. Worker posts `[EXECUTION REPORT]` and stops at `status:review`, or posts `[BLOCKER REPORT]` and stops at `status:blocked`; Worker never merges/closes or runs #67.

## Evidence contract

Final report must contain only bounded protocol metadata:

```text
Attempt / worker / environment
Base SHA
Exact Task Candidate SHA
PR
Root-cause classification
Pre-fix reproduction: PASS/FAIL
Wire representation before/after (type only, no payload)
R008 HTTP bound changed: no
Derived/fixed IPC bound or invariant
Small round-trip result
Near-limit round-trip result
Malformed/zero/truncated/oversize results
Response Secret containment result
Request Secret pre-egress result
No-direct-egress/sandbox result
Timeout/cancel/cleanup result
J1/J2/J3 workflow + jobs
Claims C1-C9
Outcome: COMPLETED | BLOCKED
Downstream #67 readiness: yes/no + reason
```

Never publish real response bodies, response Secret header names/values from #67, Cookie/Auth/token, signed URLs, raw worker stderr, media/page payload or credentials.

## Freshness

Semantic authorities:

- `plugins/generic-ytdlp/src/runtime.rs` broker process/framing;
- `plugins/generic-ytdlp/worker/worker.py` broker framing/response reconstruction;
- #60 broker runtime security model;
- R008 + #95/ADR 0007 response containment;
- #83/#85 sandbox/fd authority.

Unrelated product/docs changes do not invalidate protocol Evidence. Any accepted change to these protocol/security authorities before final verification requires dependency-aware freshness classification.

## Out of scope

- real-site/Bilibili verification;
- #67 contract revision or execution;
- R008 HTTP policy changes;
- sandbox/fd redesign;
- new network/IPC transports;
- production enablement;
- media-format/DASH/remux work;
- #68.
