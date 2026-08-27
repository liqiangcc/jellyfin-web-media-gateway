# Task — GENERIC-YTDLP-BOUNDED-EXTRACTOR-FAILURE-PREP

## Metadata

```text
GitHub Issue: #101
Task ID: GENERIC-YTDLP-BOUNDED-EXTRACTOR-FAILURE-PREP
Task kind: implementation + deterministic security verification
Planning Base: 951e73d2f66d8f26a87535d61acd6c0dfd74800e
Parent Evidence: #67 R8 / Attempt 8
Preferred worker: cloud-codex
Eligible environment: env:cloud
Accepted authorities: #79 / #83 / #85 / #95 / #97 / #99 / R008
Downstream: #67 next Contract Revision / Attempt
Freshness policy: dependency-aware
```

## Trigger Evidence

#67 Attempt 8 proved on the accepted low-privilege Linux 4.19 ARM64 Target:

```text
exact Candidate: cd95db5f0becb875455789f168b92c44a96a5260
runtime_cache: offline-hit
direct/no-proxy frozen Bilibili page: 2xx
SANDBOX_UNAVAILABLE: cleared
SPAWN_FAILED: cleared
BROKER_RESPONSE_SECRET_REJECTED: cleared
BROKER_PROTOCOL: cleared
broker_status_class: 2xx
broker_request_count: 3
process_error: NONZERO_EXIT
ResolvedMedia: not reached
cleanup / safe-output boundary: PASS
```

The runtime currently consumes and discards bounded worker stderr, then maps
every unsuccessful worker exit to `NONZERO_EXIT`. That preserves containment
but cannot distinguish a deliberate policy rejection, an expected extractor or
site failure, unsupported media, and a worker defect.

## Goal

Define and implement a fixed, bounded, machine-readable failure envelope owned
by the repository worker. The real-smoke summary must classify accepted worker
failures without exposing exception text or stderr.

The result must make the next #67 Attempt able to answer at least:

```text
did repository request policy reject the extractor request?
did the broker reject or fail a request?
did yt-dlp report a bounded extractor/site failure?
did the extracted result violate the current muxed HTTP/HLS contract?
did an unexpected worker failure occur?
```

No public or real-site request is allowed in this Task.

## Security invariants

1. Raw stderr, traceback, exception text, source URL, response body, headers,
   signed media URL/query data and credentials never cross the worker boundary.
2. Error values form a small closed enum. No origin-, extractor- or
   exception-provided string may be serialized as a diagnostic.
3. Cookie/Auth/profile/proxy/login state remains prohibited; response Secret
   containment and request Secret rejection remain unchanged.
4. The worker retains no direct socket authority; R008 remains the only public
   network authority.
5. Body/frame/time/process/fd/sandbox bounds remain fail closed.
6. Unknown exceptions do not become successful output and do not disclose
   details.
7. Production `GenericYtdlpAdapter::default()` remains `DisabledRunner`.

## Implementation requirements

### A. Closed failure taxonomy

Introduce the smallest stable taxonomy needed to classify the accepted path.
It must include distinct bounded classes for:

- repository request-policy rejection;
- broker request/response failure;
- expected yt-dlp extractor/site failure;
- unsupported current media contract;
- unexpected worker failure.

Names may differ, but they must be fixed repository constants and must not be
derived from error text, URLs, headers or response data.

### B. Safe worker envelope

Expected worker failures must produce a valid bounded JSON envelope on stdout.
The Rust parser and smoke renderer must map only admitted enum values to stable
safe `process_error` codes. Unknown values, malformed JSON, oversized stdout,
stderr overflow, crash and nonzero exit remain fail closed.

Do not turn arbitrary nonzero exits into trusted classifications.

### C. Deterministic classification fixtures

Add fixtures proving each admitted class without public network access. Include
at minimum:

- request Secret/policy rejection before prohibited network traffic;
- broker denial or transport-class failure;
- synthetic yt-dlp expected failure with exception text containing URL,
  query-token and credential sentinels;
- unsupported muxed-format result;
- unexpected exception and malformed/unknown envelope;
- worker crash/nonzero exit remains generic and contained.

All sentinels must be absent from stdout, smoke summary, logs and durable test
evidence.

### D. Cross-architecture and authority regressions

Run equivalent parser/worker/runtime classification tests on hosted x86_64 and
native hosted ARM64. Preserve the complete #83 socket/socketpair denial,
broker-IPC, no_new_privs and seccomp matrix plus #85/#95/#97/#99/R008 tests.

## Claims

```text
C1 closed bounded taxonomy
C2 raw diagnostics and Secret containment
C3 request-policy and broker-failure separation
C4 expected extractor/site failure classification
C5 unsupported-media and unexpected-worker separation
C6 x86_64 / native ARM64 equivalence
C7 accepted sandbox/fd/framing/R008 authorities preserved
C8 #67 blocker prepared for safe re-verification
```

## Verification matrix

### J1 — hosted x86_64 worker/parser taxonomy

- exact Candidate assertion;
- all deterministic failure envelopes and negative parser cases;
- sentinel absence checks;
- focused generic-ytdlp tests.

### J2 — native hosted ARM64 equivalent proof

Runner: `ubuntu-24.04-arm`.

- repeat the same failure taxonomy and containment matrix;
- execute the accepted sandbox/runtime matrix;
- no public-site request.

### J3 — security/static guards

- no exception/stderr/message serialization;
- no Cookie/Auth/profile/proxy capability;
- no direct egress or sandbox bypass;
- production DisabledRunner unchanged;
- fixed enum and bounded parser only.

### J4 — full affected regressions

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
cargo test -p gateway-egress --all-targets
cargo test -p generic-ytdlp --features runtime-prep --test runtime -- --nocapture
```

## Success criteria

1. C1–C8 PASS on one exact Candidate.
2. J1–J4 PASS, including native ARM64 equivalent evidence.
3. Every admitted category is deterministic and Secret-safe; sentinels do not
   appear in any admitted output.
4. Crash, unknown envelope, malformed JSON and arbitrary nonzero exit remain
   generic fail-closed conditions.
5. No real-site request occurs.
6. Worker reports, releases ownership and stops; it does not execute #67.

## Expected files

Primarily:

```text
plugins/generic-ytdlp/worker/worker.py
plugins/generic-ytdlp/src/lib.rs
plugins/generic-ytdlp/src/smoke.rs
plugins/generic-ytdlp/tests/**
.github/workflows/generic-ytdlp-prep.yml
```

Only the smallest necessary subset should change.

## Out of scope

- real Bilibili/site extraction;
- changing frozen yt-dlp or the sample;
- relaxing request/response Secret policy;
- Cookie jar, login, profile, CAPTCHA, proxy or access-control bypass;
- DASH/remux/FFmpeg/Browser/Web E2E;
- production generic-ytdlp enablement;
- diagnosis by publishing raw stderr or exception messages.

## Completion protocol

```text
status:ready
→ Worker claim / Attempt 1
→ status:in-progress
→ implementation + J1-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
```

Coordinator alone reviews, merges, closes and republishes #67.
