# Task — GENERIC-YTDLP-PREP constrained generic yt-dlp plugin contract / parser

## Metadata

```text
GitHub Issue: #46
Parent Goal / Research Item: Phase 1 / generic fallback plugin
Task / Research ID: GENERIC-YTDLP-PREP
Task kind: combined
Base commit: cb3e205e8b4caa4657b00f83f9e32c0f68777f1b
Candidate commit: n/a until Worker execution
Session bootstrap prompt: docs/tasks/46-generic-ytdlp-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, rust-build, rust-test, github-actions-orchestration
Hard publication dependencies: accepted #39 conformance/security boundary
```

Realtime status/owner/branch/PR/Evidence lives only in Issue #46. Attempt history follows `docs/tasks/issue-lifecycle-protocol.md`.

## Goal

Implement the **deterministic PREP layer** for `plugins/generic-ytdlp` as a low-priority SiteAdapter fallback: reusable process-runner boundary, bounded machine-readable output parser/validator, generic locator/resolved-media mapping and accepted #39 conformance integration — **without enabling a production yt-dlp network runtime that can bypass R008**.

```text
SiteAdapterRegistry
→ higher-priority adapter first
→ generic-ytdlp PREP plugin
→ bounded ProcessRunner contract
→ deterministic local fake executable / captured fixture
→ machine-readable parser
→ accepted SourceLocator / ResolvedMedia
→ #39 conformance

production real-network yt-dlp executor
→ NOT enabled by this Task
→ requires later R008-mediated egress/runtime decision
```

## Why / Context

Canonical architecture allows `generic-ytdlp` only as a plugin fallback; Core must never call yt-dlp directly. Accepted R008 also requires source/network access to remain under central EgressPolicy/redirect/DNS validation.

A normal yt-dlp subprocess owns its own HTTP stack. Wiring it directly to arbitrary public URLs now would create an EgressPolicy bypass. Therefore this PREP Task deliberately separates deterministic plugin/process/parser mechanics from any later real-network executor. This is a safety/architecture boundary, not a lowered success criterion.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: later GENERIC-YTDLP-RUNTIME task may be created only after an accepted R008-mediated subprocess/network strategy exists
Decision reason: current Claims are deterministic repository/process-contract claims; real external-site/network behavior is intentionally excluded.
```

## Worker Routing Decision

```text
implementation: cloud-codex / env:cloud
verification: GitHub Actions / github-hosted ubuntu-latest x64
```

No phone, TV, real site, account, Browser Worker or production yt-dlp network request is required.

## Claims

- C1 — all yt-dlp executable flags/output/process knowledge remains inside `plugins/generic-ytdlp` (or a plugin-owned helper); Stable Core contains no yt-dlp fallback branch.
- C2 — Registry recognition priority is explicit and lower than accepted `generic-direct` for direct MP4/HLS; registration order is not routing authority.
- C3 — a bounded process-runner contract uses executable + argument vector, never shell interpolation, and applies timeout/stdout/stderr/cleanup limits.
- C4 — required verification uses only deterministic local fake executable and/or committed sanitized fixtures; no real network is performed.
- C5 — production real-network executor is absent, disabled-by-default or otherwise unreachable from the accepted Registry runtime in this Task; a caller cannot use PREP APIs to bypass R008.
- C6 — parser accepts a narrow machine-readable schema and deterministically rejects malformed/oversized/unsupported/DRM/Secret-bearing output.
- C7 — SourceLocator remains versioned/plugin-owned/opaque to Core and generic-ytdlp passes accepted #39 ownership/version conformance checks.
- C8 — stderr/process diagnostics are bounded/redacted; raw process stderr, sensitive URL query, Cookie/Authorization/profile path or arbitrary CLI flags never enter Control/Evidence.
- C9 — accepted #39 reusable conformance/architecture guards plus affected R001/R008 regressions stay green.

## Preconditions

- `main@cb3e205e8b4caa4657b00f83f9e32c0f68777f1b` contains accepted #39 reusable conformance and shared Secret classifier.
- `generic-direct` currently recognizes direct MP4/M4V/HLS at explicit priority 10; generic-ytdlp fallback must not steal those inputs.
- Worker must integrate live accepted main before final exact-Candidate Evidence if main advances.

## In Scope

- new `plugins/generic-ytdlp` workspace crate/skeleton;
- low-priority generic recognition rules for otherwise-unclaimed bounded `http/https` input;
- plugin-owned versioned SourceLocator;
- `ProcessRunner`/executor abstraction or equivalent;
- deterministic fake/local executable runner used by required tests;
- argument-vector invocation, timeout, stdout/stderr cap, exit classification and process cleanup;
- bounded JSON/machine-readable parser;
- mapping to current accepted `ResolvedMedia` / `ResolvedStream` types only;
- accepted shared Secret-header classifier and conformance harness;
- architecture regression proving Core does not invoke/import yt-dlp;
- exact-SHA GitHub Actions.

## Out of Scope

- enabling a real production yt-dlp subprocess to fetch arbitrary URLs;
- weakening/bypassing R008 via subprocess networking, proxy tricks or unvalidated redirects;
- Bilibili #23 substitute or real-site acceptance;
- login/Cookie DB/browser profile import;
- CAPTCHA/DRM/paywall/region/access-control bypass;
- Browser Worker/Native Panel;
- phone/TV proof;
- runtime plugin IPC/marketplace;
- importing #23 blocked API additions such as navigation/DASH/expiry merely to fit yt-dlp output.

## Architecture Invariants

1. Stable Core never shells out to yt-dlp and contains no yt-dlp-specific output/flag/site logic.
2. `generic-ytdlp` is a plugin fallback, not Core fallback.
3. Direct media recognized by higher priority `generic-direct` stays with that adapter.
4. Required process execution is local/deterministic and network-free.
5. A real-network executor is not production-reachable until a later accepted R008-mediated strategy exists.
6. No shell interpolation; executable and arguments are structured values.
7. Caller cannot inject arbitrary yt-dlp CLI flags, Cookie/profile paths or Egress scope.
8. Parsed public headers/diagnostics consume accepted #39/R008 Secret semantics.
9. Current accepted SiteAdapter API is authority; blocked #23 branch is not.

## Files Expected to Change

Likely:

- workspace `Cargo.toml` / `Cargo.lock`;
- `plugins/generic-ytdlp/Cargo.toml`;
- `plugins/generic-ytdlp/src/*`;
- deterministic sanitized fixtures / fake executable under test-only paths;
- bounded architecture/process guard tests;
- `.github/workflows/*generic-ytdlp-prep*`.

Do not modify Stable Core to add a yt-dlp code path.

## Implementation Requirements

1. Register/instantiate generic-ytdlp only through normal `SiteAdapterRegistry` integration/test surfaces.
2. Recognition must be deterministic, site-neutral, `http/https` only and lower-priority than generic-direct direct-media matches.
3. Define a narrow executor API taking a structured request; no caller-supplied free-form argument list.
4. Actual test process invocation must use `Command`/argv or equivalent, never `sh -c`/shell string construction.
5. Apply deterministic timeout and stdout/stderr byte caps; kill/reap process on timeout/overflow/error.
6. Required tests must prove no network access. A fake executable/script may emit fixture JSON locally but must not fetch external URLs.
7. Parse only fields required by the current accepted ResolvedMedia schema; ignore/reject unknown dangerous inputs rather than growing Core types from yt-dlp output.
8. Reject output containing unsupported protection, invalid URL scheme, Secret-bearing public headers or oversized/unbounded structures.
9. Raw stderr is diagnostic input only; public error is stable/bounded and redacted.
10. Production registry/runtime must not wire a real yt-dlp network executor in this Candidate. If a production-facing constructor exists, it must remain explicitly disabled/unconfigured and fail closed.
11. Add an architecture regression ensuring `gateway-core/src` does not reference yt-dlp executable/process details.
12. Reuse accepted #39 conformance harness; do not duplicate a private conformance suite.
13. Final Candidate integrates live accepted main and required jobs assert exact final SHA.

## Verification Plan

| Job | Claims | Execution plane | Runner | Required | Intent |
|---|---|---|---|---|---|
| J1 | C1-C4,C6,C7 | GitHub Actions | ubuntu-latest | yes | fake-process/fixture parser + generic-ytdlp conformance + priority contracts |
| J2 | C3-C5,C8 | GitHub Actions | ubuntu-latest | yes | shell/arg injection, timeout, overflow, malformed JSON, Secret, disabled-real-runtime negatives |
| J3 | C1-C9 | GitHub Actions | ubuntu-latest | yes | workspace + #39/R001/R008/architecture regressions |

```text
Target proof required: no
External real-network smoke: forbidden as required Evidence in this Task
Interactive external debugging: no by default
```

## Success Criteria

1. C1-C9 have exact-Candidate executable Evidence.
2. generic-ytdlp passes the accepted reusable conformance harness with deterministic fixtures.
3. generic-direct retains higher-priority direct MP4/HLS ownership.
4. Process runner proves argv/no-shell, timeout/output caps and cleanup.
5. Malformed/Secret/unsupported output is rejected deterministically.
6. No production path can invoke real-network yt-dlp from this Candidate.
7. Core contains no yt-dlp process/output/site-specific fallback logic.
8. J1/J2/J3 pass on exact final Candidate and Candidate is in a reviewable PR.

## Evidence Contract

`[EXECUTION REPORT]` must include:

```text
Attempt:
Base commit:
Candidate commit:
PR:
Plugin/process runner/parser locations:
Recognition priority result:
Fake-process/no-network proof:
Arg/shell/timeout/output-cap result:
Parser/Secret negative result:
Real-network runtime disabled proof:
#39 conformance result:
Core architecture guard result:
Claims C1-C9:
J1/J2/J3 run + job IDs:
Exact-Candidate assertion:
Affected R001/R008 regressions:
Limitations:
Result: COMPLETED | BLOCKED
```

No real Cookie/token/profile, sensitive full URL, raw unredacted stderr or production Secret may appear in Evidence.

## Failure / Blocked Handling

- parser/process/test defects remain same-Task revision work;
- if making the plugin functional would require direct subprocess networking outside R008, keep runtime disabled and report that as an expected limitation, not permission to bypass security;
- if canonical SiteAdapter API cannot represent a deterministic PREP result without importing blocked #23 types, report Coordinator architecture conflict;
- missing exact-SHA GitHub Actions Evidence is BLOCKED.

## Deliverables

- deterministic generic-ytdlp PREP plugin/process/parser implementation;
- conformance/security/architecture tests;
- exact-Candidate PR/Evidence;
- `docs/tasks/46-generic-ytdlp-prep/prompt.md`.

## Completion Protocol

```text
claim → status:in-progress → Attempt N
→ candidate + exact-SHA J1/J2/J3
→ [EXECUTION REPORT] → status:review → release owner → STOP
```

Blocker path: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.
Worker never sets `status:done`, closes #46, runs a real-site acceptance campaign, enables an R008-bypassing runtime, merges its own PR, or automatically starts another Task.