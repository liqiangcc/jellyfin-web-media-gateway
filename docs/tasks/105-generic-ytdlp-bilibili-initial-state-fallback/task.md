# Task — GENERIC-YTDLP-BILIBILI-INITIAL-STATE-FALLBACK

## Metadata

```text
GitHub Issue: #105
Parent Goal / Research Item: #67 GENERIC-YTDLP-BILIBILI-REAL Attempt 10
Task / Research ID: GENERIC-YTDLP-BILIBILI-INITIAL-STATE-FALLBACK
Task kind: implementation + deterministic cross-architecture verification
Base commit: bae985b6f36bd6d735229c218b98fd9de6d59255
Candidate commit: n/a
Session bootstrap prompt: docs/tasks/105-generic-ytdlp-bilibili-initial-state-fallback/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, Python/Rust repository implementation, deterministic fixture design, hosted x86_64 and native hosted ARM64 Actions, bounded security evidence
Hard publication dependencies: #67 Attempt 10 bounded report and #103 Attempt 2 Coordinator ACCEPTANCE / PR #104 merge (satisfied); no unresolved hard dependency
```

## Goal

Using only repository-owned code and deterministic offline fixtures, implement the smallest compatibility mechanism at the generic-ytdlp runtime boundary for the frozen yt-dlp `2026.08.19` BilibiliIE path in which webpage/WBI navigation/view/detail processing has no initial state and no Bangumi redirect. The supported synthetic continuation must produce a valid current muxed `http-file` `ResolvedMedia`. The mechanism must remain bounded, fail closed, and preserve the production `DisabledRunner` default.

## Why / Context

#67 R10 ran exact Candidate `bec606fe0346e60fa5f05f98e27981fca8feffb2` on the accepted low-privilege ARM64 target. The exact harness reached three broker 2xx requests, returned bounded `EXTRACTOR_FAILURE`, produced no current `ResolvedMedia`, and passed J4 cleanup/leak scanning. Durable evidence is [#67 Attempt 10 report](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/67#issuecomment-5447573374).

The verified upstream wheel remains yt-dlp `2026.08.19`, source commit `3a08beaf031ab68f966401ead017ac81fe8486cf`, SHA256 `86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9`. The official upstream context is [yt-dlp issue 15924](https://github.com/yt-dlp/yt-dlp/issues/15924): the BilibiliIE path performs webpage/WBI/view/detail work, then raises when initial state is absent and there is no Bangumi redirect; the proposed detail-data continuation reaches formats. This Task treats that as bounded design context, not as runtime or public-site evidence.

#103 Attempt 2 already delivered the accepted direct GenericIE normalization and PR #104 merged to `bec606fe0346e60fa5f05f98e27981fca8feffb2`. This Task addresses only the next repository-owned initial-state/detail-data compatibility seam; it must not alter #67 or claim Bilibili fixed.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: The runtime-boundary mechanism, synthetic fixtures, negative cases, and cross-architecture CI prove one bounded implementation contract. No public-site or target-phone evidence belongs in this Task.
```

## Worker Routing Decision

```text
Worker: cloud-codex
Environment: env:cloud
Verification backend: GitHub-hosted x86_64 and native hosted ARM64 Actions
```

## Work Role

### Implementation

The Candidate may modify only the generic-ytdlp runtime compatibility boundary, its deterministic fixtures/tests, and narrowly necessary generic-ytdlp CI wiring. A patched-runtime or overlay mechanism is acceptable only when it is repository-owned, loaded at the runtime-prep boundary, limited to the missing-initial-state/detail-data continuation, and unable to enable direct worker egress or production execution. Do not change the frozen yt-dlp wheel/version/source or copy external upstream code wholesale.

### Verification

Claims to verify:

- C1: A deterministic synthetic webpage/WBI/view/detail/playurl fixture reproduces the missing-initial-state/no-Bangumi-redirect branch and the bounded detail-data continuation yields a current muxed `http-file` `ResolvedMedia`.
- C2: The continuation is narrow and fail-closed: malformed webpage/detail/playurl data, missing required fields, redirect-only input, non-media output, unsupported/separate A/V output, and unexpected worker results never become a false muxed success.
- C3: Secret-bearing headers or fields, cookies, authorization, signed material, raw diagnostics, source/page bodies and media payloads are not exposed or retained in machine output, reports or fixtures.
- C4: The implementation preserves R008 broker-only egress, broker framing/fd authority, accepted sandbox/no-new-privs/capability behavior, cleanup, and the fixed #101 result taxonomy.
- C5: The same deterministic behavior passes on hosted x86_64 and native hosted ARM64 with exact Candidate identity checks; a generic ARM64 runner cannot substitute for native hosted ARM64 evidence.
- C6: `DisabledRunner` remains the production generic-ytdlp default, no Core site branch is added, and no public Bilibili/#67/#68 execution occurs.
- C7: The Candidate makes the bounded #67 failure actionable for a later Coordinator-controlled real-site rerun without claiming that rerun or changing #67.

## Task vs Job Boundary

```text
Issue #105 Task
→ C1–C7
→ cloud-codex Worker
→ J1 hosted x86_64
→ J2 native hosted ARM64
→ J3 security/static boundary checks
→ J4 affected regression set
```

Jobs do not claim Issue #105 or acquire separate lifecycle state.

## Preconditions

- Read `AGENTS.md`, the canonical docs named by `prompt.md`, the complete live history of #105 and relevant #67/#103 comments, and accepted #79/#83/#85/#95/#97/#99/#101/#103 authorities before changing files.
- Confirm #105 is `status:ready + env:cloud + no owner` before claim.
- Use Base commit `bae985b6f36bd6d735229c218b98fd9de6d59255` as the planning baseline and record the final Candidate SHA.
- Use only the verified offline wheel provenance above and repository-owned synthetic fixtures. No package index, source checkout, replacement wheel or public network is permitted.
- Treat #67 Attempt 10 and #103 acceptance as immutable evidence. Do not execute `scripts/generic-ytdlp-real-smoke.sh`, #67, #68, or any public-site request.

## In Scope

- Inspecting the frozen yt-dlp BilibiliIE-compatible runtime path and the generic-ytdlp worker/adapter boundary using local code only.
- One smallest patched-runtime or overlay mechanism at the runtime boundary for missing initial state followed by bounded detail-data continuation.
- Synthetic offline fixtures for webpage, WBI navigation, view, detail and playurl responses, including the successful muxed `http-file` path.
- Deterministic negative fixtures for malformed data, missing fields, redirect input, Secret fields/headers, non-media output, unsupported/separate A/V output, and unexpected worker failure.
- Rust/Python tests, bounded-output leak checks, cleanup checks, and narrowly necessary `generic-ytdlp-prep` workflow assertions.
- Hosted x86_64 and native hosted ARM64 J1–J4 evidence, one candidate PR, one lifecycle report or blocker report, owner release and STOP.

## Out of Scope

- Any public or real-site request, Bilibili verification, `scripts/generic-ytdlp-real-smoke.sh`, #67 execution/mutation, or #68 execution.
- Challenge solving, CAPTCHA, cookie/login/profile/fingerprint, proxy/alternate egress, browser/navigation authority, or access-control bypass.
- Inspecting, logging, storing or reproducing raw stderr, exceptions, page bodies, source URLs, response/request headers, tokens, cookies, signed URLs or media payloads.
- Changing the frozen yt-dlp version/source/wheel, R008/SSRF/DNS/TLS/redirect policy, broker ownership/framing, fd-3 or close-range behavior, sandbox/seccomp/no-new-privs/capabilities, or `DisabledRunner`.
- Core site-specific branches, product playback changes, DASH/remux/FFmpeg/subtitle work, separate A/V support, target-phone proof, TV/device work or performance tuning.
- Treating an upstream issue, a green x86_64 job, or a generic ARM64 job as proof of a real Bilibili fix.

If the supported fixture can only produce separate audio/video, classify it as a bounded result and recommend a later DASH/remux split; do not implement that split here.

## Architecture Invariants

- Core remains site-agnostic and does not understand yt-dlp, Bilibili URLs or extractor diagnostics.
- The worker has no direct socket or alternate egress authority; network fixtures use only the existing broker abstraction.
- Response Secret containment and fixed safe summaries remain enforced by R008/#95/#101.
- The runtime boundary accepts only bounded machine output and current clear `ResolvedMedia` fields; `upstream_access_ref` remains absent.
- Unknown, malformed, unsafe, unsupported and unexpected outputs fail closed.
- Production registration continues to use `Arc::new(DisabledRunner)`; any runtime-prep enablement is explicit to tests/fixtures only.

## Files Expected to Change

- `plugins/generic-ytdlp/worker/worker.py` or one narrowly scoped runtime-boundary overlay module.
- `plugins/generic-ytdlp/src/**` only when required to carry the existing bounded output contract.
- `plugins/generic-ytdlp/tests/**` and deterministic local fixtures.
- `.github/workflows/generic-ytdlp-prep.yml` only for exact architecture/fixture/security CI assertions.
- No `gateway-core`, R008 policy, sandbox/fd authority, site-specific Core branch or production registry change is expected.

## Implementation Requirements

1. Establish the repository-owned cause from local code and synthetic fixtures only; do not infer implementation details from prohibited #67 diagnostics.
2. Keep the patch limited to the missing-initial-state/detail-data continuation. It must require bounded response shape and safe fields before constructing formats.
3. Prove a muxed `http-file` result with current `ResolvedMedia` validation. If the only valid shape is separate A/V, return a bounded unsupported classification and the later split recommendation.
4. Preserve the exact frozen wheel provenance and prevent the overlay from becoming a general extractor or network path.
5. Add deterministic malformed, redirect, Secret and non-media negatives plus existing taxonomy, sandbox, broker and cleanup regressions.
6. After the first coherent in-scope commit, push the worker branch and create/update one focused PR. Post at most one `[EXECUTION CHECKPOINT]` when a durable anchor exists; do not post heartbeat comments.
7. Run J1–J4 and report exact Candidate, PR, Actions jobs and bounded outcomes. Never call the real-site smoke script.

## Verification Plan

### Claims

```text
C1: missing-initial-state synthetic continuation yields muxed http-file ResolvedMedia
C2: malformed/redirect/Secret/non-media/separate-A/V/unexpected negatives fail closed
C3: no raw diagnostics, secrets, URLs, bodies or media payloads cross the boundary
C4: R008, broker, fd, sandbox and cleanup authorities remain unchanged
C5: exact deterministic proof passes on hosted x86_64 and native hosted ARM64
C6: DisabledRunner and no-public-site/Core boundaries remain intact
C7: result is actionable for a later #67 Coordinator rerun without running #67/#68 here
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2,C5,C6 | github-actions | github-hosted-x64 | x86_64 | yes | exact Candidate; frozen offline cache; synthetic fixture/worker/adapter tests; clean-build and generic-ytdlp regressions | run/jobs |
| J2 | C1,C2,C5,C6 | github-actions | native hosted ARM64 | aarch64 | yes | exact Candidate; same offline fixture matrix; native architecture assertion; sandbox/runtime checks | run/jobs |
| J3 | C3,C4,C6 | github-actions | github-hosted-x64 | runner-self | yes | bounded-output leak scan; Core/site boundary; R008/Secret/broker/fd/sandbox/DisabledRunner guards; no network request | run/job |
| J4 | C2,C3,C4,C5,C6 | github-actions | github-hosted-x64 | runner-self | yes | `cargo fmt --all -- --check`; workspace clippy/tests; gateway-egress; runtime/smoke tests; Python compile; shell syntax and cleanup checks | run/jobs |

J1/J2 must use deterministic fixture inputs only. A public-site or real-network request is an automatic scope violation, not a verification failure to work around.

## Execution Plane

```text
Execution plane: github-actions
```

## Runner Selection

```text
J1/J3/J4 → github-hosted-x64
J2 → native hosted ARM64
```

Cloud is the Worker environment, not a substitute for the Actions runners.

## Target verification

```text
Target proof required: no
Why target evidence is not required: #105 proves a repository-owned offline compatibility seam; real ARM64 phone proof remains the separate #67 lifecycle.
```

## Runner Security Constraints

```text
Trusted candidate only: yes
Dedicated low-privilege runner user: required where runtime-prep executes
Vault/profile access: forbidden
Production service mutation: forbidden
Cleanup / timeout requirements: remove fixture/staging artifacts and confirm no worker/sandbox descendants or media-payload candidates remain
```

## Success Criteria

### Task success

1. A focused Candidate/PR implements the missing-initial-state/detail-data continuation at the generic-ytdlp runtime boundary using only repository-owned code.
2. Synthetic offline fixtures prove a muxed `http-file` `ResolvedMedia` and all required negative cases remain bounded/fail closed.
3. J1–J4 pass on hosted x86_64 and native hosted ARM64 with exact Candidate checks and no public request.
4. R008, broker/fd/sandbox/cleanup, `DisabledRunner`, and fixed taxonomy authorities remain intact.

### Verification claim success

```text
C1 PASS when the synthetic webpage/WBI/view/detail/playurl continuation produces one current valid muxed http-file ResolvedMedia.
C2 PASS when malformed, redirect, Secret, non-media, unsupported/separate-A/V and unexpected cases fail closed with fixed bounded results.
C3 PASS when retained outputs and fixtures contain no prohibited diagnostic, secret, URL, body or payload material.
C4 PASS when existing broker/R008/fd/sandbox/cleanup regressions remain green and no alternate egress is introduced.
C5 PASS when J1 and J2 both prove the exact Candidate on their asserted architectures.
C6 PASS when DisabledRunner, Core/site boundary and no-public-site guards remain green.
C7 PASS when the report states only that the Candidate makes the #67 signal actionable for a later rerun, without running or modifying #67/#68.
```

## Evidence Contract

Report only bounded facts: Issue/Attempt, Worker/environment, exact Candidate, PR, Actions run/job URLs, architecture identity, synthetic fixture selector/class, fixed result/process codes, request count/status class only for offline broker fixtures, and cleanup/leak-scan booleans.

The Worker must not publish credentials, cookies, tokens, signed material, full URLs, headers, page bodies, raw stderr, exception text or media payloads. It must post `[EXECUTION REPORT]` for a reviewable Candidate or `[BLOCKER REPORT]` when the bounded cause or required evidence cannot be established, transition to `status:review` or `status:blocked`, release ownership, and STOP. Coordinator review/merge/close is outside this Task.

## Failure / Blocked Handling

- `BLOCKED` if the cause cannot be established without public traffic, prohibited diagnostics, an external dependency, or a security/sandbox/fd/R008 change.
- `FAIL` if the implementation violates the bounded contract or required negative cases are not fail closed.
- A separate A/V-only result is not a failure to repair here; report it as bounded unsupported and recommend a later DASH/remux split.
- A green generic architecture job without native hosted ARM64 evidence is incomplete.
- Do not lower criteria, guess an upstream fix, execute #67/#68, or rewrite this contract during execution.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #67 Attempt 10 bounded `EXTRACTOR_FAILURE` report and R10 evidence boundary.
- #103 Attempt 2 Coordinator ACCEPTANCE and PR #104 merge `bec606fe0346e60fa5f05f98e27981fca8feffb2`.
- #101 fixed bounded worker taxonomy; #99 clean-build binding; #97 broker protocol; #95/R008 Secret containment; #85/#83 sandbox/fd; #79 frozen offline runtime; `DisabledRunner` and current `ResolvedMedia` contracts.

Semantic freshness domains:
- generic-ytdlp worker/runtime overlay and bounded output normalization;
- generic-ytdlp deterministic fixtures/tests and `generic-ytdlp-prep` workflow;
- `site-adapter-api::ResolvedMedia` validation and broker/R008/Secret/sandbox/fd boundaries.

Integration surfaces:
- Cargo workspace/Cargo.lock and feature-gated runtime-prep tests;
- `.github/workflows/generic-ytdlp-prep.yml` architecture matrix and frozen cache setup;
- generic-ytdlp worker/parser/adapter boundary with gateway-egress R008 response handling.

Task-owned surfaces:
- missing-initial-state/detail-data compatibility overlay, deterministic fixtures/tests and narrowly required CI assertions.

Authority/domain → Claim mapping:
- #67/#103/#101 generic-ytdlp outcome and ResolvedMedia seam: C1,C2,C7.
- #95/R008/broker/fd/sandbox authorities: C3,C4,C6.
- #79 frozen wheel/cache and CI architecture matrix: C1,C5,C6.
- `DisabledRunner`/Core site boundary: C4,C6,C7.

Integration verification:
- JI1: exact Candidate checkout and declared J1/J2 fixture matrix with hosted x86_64 and native hosted ARM64 identity assertions.
- JI2: static/security/cleanup guards from J3/J4 proving no public request, no direct worker egress, no prohibited output, and no DisabledRunner/R008/sandbox/fd regression.

Unrelated-main policy:
- Existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because unrelated main advances.

Integration-overlap policy:
- Preserve accepted #79/#83/#85/#95/#97/#99/#101/#103 evidence. Compose with a Coordinator-frozen Integration Base and rerun only mapped JI jobs unless an overlap changes this Task's semantics.

Semantic-authority-change policy:
- Reconcile any changed accepted authority and rerun mapped Claims; broaden only when impact cannot be bounded safely. If the change alters this Scope or security premise, return to draft for formal Contract Revision.

Strict-main reason:
- n/a

## Deliverables

- Repository-owned compatibility implementation and deterministic fixture/tests.
- Candidate commit and focused PR.
- Session bootstrap prompt.
- J1–J4 Actions evidence and bounded lifecycle report.

## Completion Protocol

```text
ready
→ claim / Attempt 1
→ in-progress
→ Candidate + J1–J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ review / blocked
→ release owner
→ STOP
```

Worker must not merge, close, set `status:done`, execute #67/#68, or create a child task.
