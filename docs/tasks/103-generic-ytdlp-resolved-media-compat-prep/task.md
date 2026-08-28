# Task — GENERIC-YTDLP-RESOLVED-MEDIA-COMPAT-PREP

## Metadata

```text
GitHub Issue: #103
Parent Goal / Research Item: #67 GENERIC-YTDLP-BILIBILI-REAL Attempt 9
Task / Research ID: GENERIC-YTDLP-RESOLVED-MEDIA-COMPAT-PREP
Task kind: implementation + deterministic cross-architecture verification
Base commit: 9ef7e36f5edeb686c9bd6200d62163410d81a6c7
Candidate commit: n/a
Session bootstrap prompt: docs/tasks/103-generic-ytdlp-resolved-media-compat-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, Rust/Python repository implementation, deterministic fixture design, GitHub Actions workflow/log inspection, security evidence authoring
Hard publication dependencies: #67 Attempt 9 final bounded report and R9 publication (satisfied); no new code dependency
```

> GitHub Issue #103 owns live status, Attempt, assignee, branch, PR, Actions result and result summary. This file owns the stable implementation contract. It does not reopen, mutate or rerun #67.

## Goal

Using only deterministic offline fixtures and repository-owned code, identify and repair the smallest generic-ytdlp worker/adapter compatibility seam that can make #67 Attempt 9's bounded `EXTRACTOR_FAILURE` actionable. The accepted supported path must produce the current `ResolvedMedia` contract, while malformed, unsafe, unsupported and unexpected outcomes remain fixed, bounded classifications. The change must be proved on hosted x86_64 and native hosted ARM64 without any public-site request.

If no repository-owned compatibility cause can be isolated without inspecting prohibited diagnostics or changing the accepted security boundary, report `BLOCKED` with the bounded reason and stop; do not guess an external-site cause or broaden this Task.

## Why / Context

#67 Attempt 9 on exact Candidate `c2834fd046cbf29a3602e9f13ae5153217c6c886` reached the accepted low-privilege ARM64 sandbox and R008 broker path. It recorded three broker requests with 2xx status classes, then produced fixed `process_error: EXTRACTOR_FAILURE`, with no current `ResolvedMedia`. The durable report explicitly forbids raw stderr, URLs, headers, tokens, cookies, page bodies and media-payload inspection, and requires any repair to be separately classified.

The repository already contains a bounded worker, machine-output parser, `ResolvedMedia` contract, muxed/HLS offline fixtures and fixed failure taxonomy. This Task closes only a repository-owned compatibility gap demonstrable through those interfaces. It must not reinterpret the real-site report as proof of a specific external extractor defect.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: The implementation and its deterministic parser/worker/security checks form one bounded contract. Hosted x86_64 and native hosted ARM64 are verification Jobs for the same Candidate, not separate business Tasks. No phone or public-site evidence is in scope.
```

## Worker Routing Decision

```text
Worker: cloud-codex
Environment: env:cloud
Verification backend: GitHub-hosted x86_64 and native hosted ARM64 Actions
```

## Work Role

### Implementation

The Candidate must contain the smallest repository-owned correction required by the deterministic evidence. It may touch only the generic-ytdlp compatibility boundary, its fixtures/tests, and narrowly necessary CI wiring. It must not expose or retain raw process diagnostics. The production registry must remain fail-closed with `DisabledRunner`.

### Verification

Claims to verify:

- C1: A deterministic offline fixture exercises the relevant post-broker extractor/adapter path and yields a valid current `ResolvedMedia` for the supported bounded `http-file` and/or HLS shape, or proves the exact safe bounded classification for an intentionally unsupported shape.
- C2: The fixed #101 taxonomy remains the only externally reported extractor/process outcome; malformed, unknown, nonzero, timeout, policy, broker, secret and unsafe outputs remain fail-closed and bounded.
- C3: No raw stderr, source URL, response header, token, cookie, signed material, request body or media payload crosses or is retained by the result/report path.
- C4: R008 response-secret containment, broker framing/fd authority, accepted sandbox behavior and cleanup remain unchanged.
- C5: The same deterministic behavior passes on hosted x86_64 and native hosted ARM64; architecture-specific assumptions are rejected rather than silently substituted.
- C6: `DisabledRunner` remains the production default and no public-site verification is performed.
- C7: The resulting Candidate makes the #67 bounded signal actionable for a later Coordinator-controlled rerun without executing or modifying #67 in this Task.

## Task vs Job Boundary

```text
Issue #103 Task
→ C1–C7
→ cloud-codex Worker
→ J1 hosted x86_64
→ J2 native hosted ARM64
→ J3 security/static boundary checks
→ J4 full affected regression set
```

Jobs do not claim Issue #103 or acquire separate lifecycle state.

## Preconditions

- Read `AGENTS.md`, the canonical docs named by `prompt.md`, this Task, the complete live history of #103 and #67, and accepted #79/#83/#85/#95/#97/#99/#101 evidence before changing files.
- Confirm #103 is `status:ready + env:cloud + no owner` before claim.
- Use current `origin/main` as the planning base and record the exact Candidate SHA.
- Treat #67 Attempt 9's exact Candidate and bounded report as immutable evidence; do not execute #67, its real smoke script, or any public-site request.
- No phone, credential, cookie, profile, proxy, production Vault or public network access is required or permitted.

## In Scope

- Inspecting the generic-ytdlp worker, machine-output parser, `ResolvedMedia` validation and existing deterministic broker fixtures.
- The minimal repository-owned normalization/compatibility correction demonstrated by an offline fixture; supported output is limited to the current clear `http-file`/HLS `ResolvedMedia` contract.
- Deterministic fixtures that exercise both the corrected supported path and bounded negative paths without network access or real media.
- Rust/Python tests, static guards and narrowly necessary `generic-ytdlp-prep` workflow adjustments.
- Hosted x86_64 and native hosted ARM64 clean-build/test evidence, plus bounded security and cleanup evidence.
- A candidate branch/PR, one lifecycle execution report or blocker report, bounded Actions links/results, and release/STOP according to the lifecycle protocol.

## Out of Scope

- Any public or real-site request, Bilibili verification, #67 execution, #67 mutation, or #68 execution.
- Inspecting, logging, storing or reproducing raw stderr, extractor URLs, response/request headers, tokens, cookies, signed URLs, page bodies or media payloads.
- Guessing or fixing a third-party yt-dlp/Bilibili defect, changing the frozen yt-dlp version/source, adding credentials, cookies, login/profile/proxy/crawler/browser authority or direct egress.
- Changes to R008/SSRF/DNS/pinning/TLS/redirect policy, broker ownership, fd-3 framing, sandbox/seccomp/no-new-privs/capability policy, `close_range` fallback, or ambient-fd rules.
- Enabling the production generic-ytdlp executor, changing `DisabledRunner`, changing Core's site boundary, DASH, remux, FFmpeg, subtitles, TV/device work or performance tuning.
- Treating a green x86_64 job as ARM64 proof, or substituting a generic ARM64 runner for native hosted ARM64.

## Architecture Invariants

- Core remains site-agnostic and does not understand yt-dlp, concrete site URLs or extractor diagnostics.
- Only the inherited R008 broker is a network authority; the worker has no direct socket or alternate egress authority.
- Response Secret material is contained/rejected according to R008/#95; no Secret or raw diagnostic is exposed in bounded summaries.
- The worker/parser boundary accepts only bounded machine output and current clear `ResolvedMedia` fields (`http-file` or HLS, public non-secret headers, no upstream access reference).
- Unknown, malformed, unsafe, unsupported and unexpected outcomes fail closed with fixed classifications.
- Production registration continues to use `DisabledRunner`; runtime-prep constructors remain explicit test/verification seams.
- Existing accepted #79/#83/#85/#95/#97/#99/#101 authorities and R008 remain preserved.

## Files Expected to Change

- `plugins/generic-ytdlp/worker/worker.py` only if the deterministic compatibility seam is there;
- `plugins/generic-ytdlp/src/**` only for the bounded adapter/parser contract;
- `plugins/generic-ytdlp/tests/**` and local fixtures;
- `.github/workflows/generic-ytdlp-prep.yml` only if required to make J1–J4 deterministic and architecture-explicit;
- no `gateway-core` or security-policy changes are expected.

## Implementation Requirements

1. First establish the repository-owned cause using code paths and deterministic fixtures only. Do not infer it from prohibited #67 diagnostics.
2. Implement the smallest compatible normalization that produces current `ResolvedMedia` for the supported fixture. Preserve title, clear protection, bounded stream count, allowed protocol, URL/header validation and `upstream_access_ref: None` rules.
3. Add a regression fixture for the previously un-actionable bounded condition and retain negative tests for unsupported format, malformed/unknown output, policy rejection, broker failure, secret headers and unexpected worker failure.
4. Keep all externally visible summaries machine-readable and fixed; never print or persist raw subprocess stderr or extractor diagnostics.
5. Run the declared J1–J4 checks and verify exact Candidate identity in every Actions job. Do not call `scripts/generic-ytdlp-real-smoke.sh`.
6. After the first coherent in-scope commit, push the recoverable branch and create/update one focused PR as allowed by the lifecycle protocol. Post one `[EXECUTION CHECKPOINT]` only when useful; do not use heartbeat comments.

## Verification Job Matrix

```text
J1 hosted-x86_64:
  ubuntu-latest; exact Candidate checkout; x86_64 assertion; frozen offline runtime cache;
  clean-build binding; parser/worker/adapter tests; generic-ytdlp and gateway-egress regressions.

J2 native-hosted-arm64:
  ubuntu-24.04-arm; exact Candidate checkout; aarch64 assertion; frozen offline runtime cache;
  same deterministic compatibility and negative tests; sandbox binary/architecture checks.

J3 security-static:
  Core/site boundary, no direct worker egress, bounded-output leak scans, R008/Secret guards,
  DisabledRunner and accepted sandbox/fd/framing authority checks; no network request.

J4 affected-regression:
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --all-targets
  cargo test -p gateway-egress --all-targets
  cargo test -p generic-ytdlp --features runtime-prep --test smoke -- --nocapture
  cargo test -p generic-ytdlp --features runtime-prep --test runtime -- --nocapture
  bash -n scripts/generic-ytdlp-real-smoke.sh
```

J1/J2 must use deterministic fixture inputs only. A public-site or real-network request is an automatic scope violation, not a failed verification to work around.

## Evidence / Reporting Contract

Report only bounded facts: Issue/Attempt, Worker/environment, exact Candidate, PR, Actions run/job URLs, architecture identity, fixture selector, pass/fail/block classification, fixed `process_error`/result codes, request count/status class only where generated by offline fixtures, and cleanup/leak-scan booleans. Do not include prohibited diagnostics or payloads.

The Worker must post `[EXECUTION REPORT]` when C1–C7 and J1–J4 pass, or `[BLOCKER REPORT]` when the bounded implementation cause cannot be established or a required check is blocked. It must update the Issue lifecycle to `status:review` only for a reviewable Candidate, release ownership, and STOP. Coordinator review/merge/close is outside this Worker Task.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #67 Attempt 9 final bounded report and R9 publication;
- #101 accepted fixed bounded extractor-failure taxonomy, Candidate `c2834fd046cbf29a3602e9f13ae5153217c6c886`;
- #99 accepted clean-build sibling sandbox binding, Candidate `cd95db5f0becb875455789f168b92c44a96a5260`;
- #97 accepted broker protocol, Candidate `d9c038547ed2df695571f8dd4f732bdcdd4d5c19`;
- #95/R008 accepted response Secret containment, Candidate `804fd60343b081e5e055ba87f68e7939b106bb19`;
- #85/#83/#79 accepted fd/sandbox/runtime authorities and `DisabledRunner` contract;
- `site-adapter-api::ResolvedMedia` and current generic-ytdlp parser/worker contracts;
- `AGENTS.md`, `docs/implementation-contracts.md`, `docs/security.md`, `docs/runner-execution-architecture.md`.

Semantic freshness domains:
- `plugins/generic-ytdlp/worker/worker.py` extraction and bounded error mapping;
- `plugins/generic-ytdlp/src/lib.rs`, `src/smoke.rs`, runtime runner and `site-adapter-api::ResolvedMedia`;
- `plugins/generic-ytdlp/tests/**` deterministic broker/parser/security fixtures;
- generic-ytdlp workflow, Cargo workspace and gateway-egress/R008 integration boundary.

Integration surfaces:
- Cargo workspace/Cargo.lock and feature-gated runtime-prep test surface;
- `.github/workflows/generic-ytdlp-prep.yml` J1/J2 architecture routing;
- `gateway-egress` R008 broker response boundary and `site-adapter-api` ResolvedMedia validation.

Task-owned surfaces:
- generic-ytdlp worker/adapter compatibility normalization, deterministic fixtures/tests and narrowly required workflow assertions.

Authority/domain → Claim mapping:
- #67 Attempt 9 and #101 taxonomy: C1,C2,C7;
- #95/R008 and gateway-egress: C3,C4;
- #79/#83/#85/#99 sandbox/fd/build authorities: C4,C5,C6;
- site-adapter-api / generic-ytdlp contracts: C1,C2,C6;
- J1/J2 workflow and workspace: C5,C6.

Integration verification:
- JI1: run the exact generic-ytdlp workflow test matrix at the Candidate on hosted x86_64 and native hosted ARM64, including workspace and gateway-egress regressions.
- JI2: run static guards proving no Core yt-dlp/process fallback, no worker direct egress, no R008/DisabledRunner/sandbox authority regression, and no prohibited output strings in retained summaries.

Unrelated-main policy:
- Existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because unrelated main advanced.

Integration-overlap policy:
- Preserve accepted #79/#83/#85/#95/#97/#99/#101 semantic Evidence. Compose this Candidate with the Coordinator-frozen Integration Base and rerun only mapped JI jobs unless an overlap changes this Task's semantics.

Semantic-authority-change policy:
- Reconcile any changed accepted authority and rerun its mapped Claims; broaden verification only when impact cannot be bounded safely.

Strict-main reason:
- n/a

## Stop Conditions

Stop and report `BLOCKED` if the only proposed fix requires real-site traffic, raw diagnostics, credentials, policy/sandbox/fd weakening, changing a frozen authority, changing #67, or an unbounded external dependency. Do not substitute a guessed compatibility fix.
