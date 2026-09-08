# Task — BILIBILI-BROWSER-LIVE-PROBE

## Metadata

```text
GitHub Issue: #169
Parent Goal / Research Item: #68 / R005 real source portability; downstream #166
Task / Research ID: BILIBILI-BROWSER-LIVE-PROBE
Task kind: combined
Base commit: 3511e70c73686d90ec65ca530a5f48be07023586
Candidate commit: n/a; live Candidate is owned by Issue/Attempt
Session bootstrap prompt: docs/tasks/169-bilibili-browser-live-probe/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
Hard publication dependencies: #165 Final Acceptance (satisfied at 3511e70c73686d90ec65ca530a5f48be07023586); no unresolved dependency for publication
```

> GitHub Actions / Runner 是 execution backend，不是会 claim Issue 的 Worker；实时状态、Attempt、owner、PR 和 result summary 只保存在 Issue。

## Goal

在已接受的 #165 offline probe 上增加一个**显式、受限、可由 #166 在 tx-node 执行的 live mode**。live mode 只能从插件拥有的 opaque Bilibili `SourceLocator` 生成公开页面导航并在 clean browser 中观察候选媒体；短期媒体地址只能留在服务端有界内存中，浏览器退出后由独立消费者读取有限字节。synthetic mode 必须继续可运行。该 Task 交付可核验的 hosted artifact 和 runbook 更新，但自身不宣称 Bilibili 兼容、Gateway 播放或 TV 播放。

## Why / Context

#165 已在 merged main `3511e70c73686d90ec65ca530a5f48be07023586` 接受 C1–C3，但其入口故意只运行 synthetic fixture，不能消费 `BV14V411W7r5` 或其它实站 selector。当前 #166 runbook 因此没有可执行的 live entry。若直接发布 #166，Worker 会被迫绕过 artifact 或把离线 fixture 误当实站能力。本 Task 补齐这个明确缺口，再由独立 #166 负责 tx-node clean anonymous 实站 Evidence；不修改 #67 历史，也不把浏览器画面/VNC 作为媒体输出路径。

## Task Decomposition Decision

```text
Verification mode: inline for portable synthetic/contract/artifact claims; separate-task #166 owns live tx-node Evidence
Linked implementation task: this Task (#169)
Linked verification task: #166 BILIBILI-BROWSER-SOURCE-REAL
Decision reason: live target/provenance is an independent Evidence authority and must not be hidden inside the hosted implementation Task; the artifact must nevertheless expose an explicit live entry before #166 can be published.
```

## Worker Routing Decision

```text
Worker/client: cloud-codex
Eligible environment: env:cloud
Portable execution plane: GitHub-hosted Actions, x64
Target execution: none in this Task; #166 uses external-codex/ssh to tx-node
```

Cloud is the coding/orchestration environment, not a Runner. The Worker must use `gpt-5.6-luna` with reasoning high; enable Fast only if the runtime exposes it and record the actual setting. No local or tx-node compilation is permitted.

## Work Role

### Implementation

The Candidate must extend the experimental probe and artifact without registering a production SiteAdapter or adding Bilibili branches to Core:

1. Add an explicit live entry (for example a structured `--mode live --selector bilibili:BV...:part-N` invocation) while keeping synthetic as the default. The caller may supply only the versioned opaque selector and bounded options; it may not supply a URL, CDP endpoint, browser profile, cookies, Authorization, arbitrary headers, proxy, upstream host, or short-lived media URL.
2. Keep Bilibili selector parsing and public page URL construction in `plugins/bilibili/`. Validate BVID/part syntax and preserve part identity. The browser runtime remains generic and receives a plugin-produced navigation descriptor; Core and Control remain unaware of Bilibili URL rules.
3. Run Chromium with a fresh disposable profile and a fail-closed egress broker. The broker owns DNS resolution and connection selection, rejects loopback/private/link-local/multicast/reserved destinations, restricts public Bilibili page/CDN host patterns, revalidates every redirect, and handles or disables every declared transport (including CONNECT/WebSocket and QUIC/WebTransport). A hostname check or CDP observation alone is not sufficient.
4. Observe only bounded, allowlisted response metadata. Candidate URLs, signed query values, cookies, authorization values, raw DOM/playinfo/HAR, and full media bodies must never be exported. Signed locations may exist only in server-side ephemeral memory until browser exit/independent read and must be cleared in `finally` cleanup.
5. Add an independent server-side consumer that reads at most the frozen request/byte budgets after browser exit. It may use only approved non-secret request metadata. Muxed candidates are independently readable; AV-separated candidates require independently readable video and audio. Blob-only, browser-only tokens, secret-header requirements, redirects to private space, 4xx/5xx, expiry, malformed fields, over-budget and cancellation are explicit negative outcomes.
6. Keep the #165 synthetic fixture path and its negative containment cases. Any shared production API or security premise that would need to change is a blocker/Contract Revision, not a permissive shortcut.
7. Update the manifest-addressed artifact and #166 runbook with the exact live entry, runtime admission checks, cleanup commands and provenance fields. The artifact consumer must download and verify the produced bundle in a separate job; it may not run a build-workspace copy.

Use argv/structured APIs for Chromium and network helpers. Do not concatenate shell commands or expose an open proxy. Do not enable live mode in hosted CI; hosted CI uses deterministic synthetic fixtures only.

### Verification

Claims to verify on the exact Candidate:

- C1: plugin-owned opaque selector parsing, public navigation descriptor, observation schema and Secret/URL boundary are bounded and covered by positive/negative tests.
- C2: real Chromium synthetic fixtures demonstrate fail-closed egress ownership for DNS/address policy, redirects, CONNECT/WebSocket, worker/service-worker paths, QUIC/WebTransport handling, cancellation and profile cleanup; no direct browser escape is admitted.
- C3: the explicit live entry is artifact-addressed, rejects caller-controlled URL/profile/headers, and its independent consumer can read bounded muxed and AV-separated fixture candidates after browser exit while preserving negative outcomes.
- C4: the downloaded artifact, manifest provenance and remote workspace regressions pass on GitHub-hosted Actions. No live Bilibili request is made by this Task.

## Task vs Job Boundary

```text
Task
→ C1–C4
→ cloud-codex Worker
→ GitHub-hosted x64 jobs J1–J4/JI1
→ exact Candidate artifact/Evidence
→ Coordinator Review
→ #166 external tx-node live verification
```

Jobs prove the Candidate claims; they do not claim Issue state, run a live site, or replace #166 target Evidence.

## Preconditions

- #165 Final Acceptance is complete and merged at `3511e70c73686d90ec65ca530a5f48be07023586`; its offline probe and artifact identity are the starting point.
- `AGENTS.md`, all required startup documents, `docs/research/bilibili-browser-acquisition.md`, `docs/research/bilibili-browser-probe-runbook.md`, `docs/site-plugin-architecture.md`, `docs/security.md`, Browser/ResolvedMedia contracts, and R008 contracts are read before coding.
- GitHub Actions hosted x64 is available for Node/browser and workspace checks.
- No phone, TV, VNC, existing browser profile, production Gateway, Vault, or live Bilibili access is needed for this Task.

## In Scope

- Experimental browser probe/runtime and Bilibili plugin interpretation under `experiments/bilibili-browser-probe/` and `plugins/bilibili/`.
- Browser egress broker, bounded observation/independent-consumer fixtures, explicit live-selector admission and sanitized evidence schema.
- Hosted workflow, downloaded-artifact consumer, manifest/provenance, and the bounded #166 runbook.
- Contract, security-negative and lifecycle tests required by C1–C4.

## Out of Scope

- Any production SiteAdapter registration, Registry wiring, Core/Playback/Display/Control code, ResolvedMedia shape change, remux/transcode, MSE player, login, DRM/permission bypass or Native Panel.
- Live Bilibili requests in GitHub Actions or by the Worker; target live requests belong only to #166.
- Reusing any personal browser, cookies/localStorage, SSH/Vault secret, existing Gateway/VNC/Chrome, or external proxy.
- Phone deployment, physical TV, autoplay/audible UX, or declaring #68/#67 PASS.

## Architecture Invariants

- Gateway remains `PlaybackSession` authority; this experimental probe is not a playback path.
- Browser is generic runtime; Bilibili DOM/URL/part semantics stay in the plugin.
- `SourceLocator` is versioned opaque content identity; CDN/HLS/signed URLs are ephemeral and never Control/Display state.
- Site Plugin does not read Vault or bypass `EgressPolicy`; no open proxy or arbitrary private-network access.
- No browser profile, Cookie, Authorization, signed URL, raw DOM/playinfo, HAR or media body is exported.
- Native Panel/Jellyfin/phone failures are irrelevant to this Task and must not be used as evidence.

## Files Expected to Change

- `experiments/bilibili-browser-probe/`
- `plugins/bilibili/` experimental interpretation only
- `.github/workflows/bilibili-browser-probe.yml` or a focused successor workflow
- `docs/research/bilibili-browser-probe-runbook.md`
- Focused research/README updates only when needed to keep the accepted offline/live boundary truthful

Production Core, Playback, Display, Control and Vault code are task-owned exclusions. If a design needs them, stop and return a blocker/Contract Revision.

## Implementation Requirements

1. Preserve the exact #165 synthetic behavior and artifact compatibility unless the manifest version is intentionally bumped with a documented migration.
2. Define a versioned selector DTO/validation in the Bilibili plugin. It must reject arbitrary absolute URLs, URL-like strings, `blob:`, `file:`, Cookie/Authorization/token-bearing input, malformed BVIDs and invalid parts. Page URL generation is plugin-owned and deterministic.
3. Make live mode opt-in and fail closed. No mode may accept caller-supplied upstream host, headers, browser profile, CDP endpoint or proxy. Unknown mode/selector/options must terminate before launching Chromium.
4. Implement egress policy at the connection/broker layer: public-address resolution and pinning, host allowlist, redirect-hop revalidation, bounded CONNECT handling, no QUIC/WebTransport direct route, worker/service-worker coverage, request/response budgets and re-entrant cleanup. A CDP `authorize_url` event stream alone cannot satisfy C2.
5. Keep ephemeral candidate descriptors in a server-only map with bounded lifetime and size. Export only schema version, selector/part match, candidate counts, role, codec/container when known, status class, Range support, safe header names, expiry category, byte/request counts, allow/deny decisions and cleanup. Unknown values remain `unknown`.
6. Consumer reads must count actual bytes including error responses, stop at the first limit, and never retry by rotating proxies or copying credentials. It must distinguish muxed versus AV-separated portability and report browser-only/blob-only outcomes.
7. Add deterministic fixtures/tests for allowed public-looking hosts, private/redirect/DNS failures, worker/SW/WebSocket/CONNECT/QUIC paths, secret leakage, malformed/oversized observations, timeout/cancellation and browser/profile cleanup. Do not weaken a negative assertion to make CI green.
8. Produce a manifest with exact Candidate SHA, Node/Playwright/browser versions, platform, entry, file digests/sizes and frozen limits. A separate downloaded-artifact job must verify and run the consumer. All required compile/test/lint work runs remotely in GitHub Actions.
9. Update the #166 runbook only to reference the new live-capable artifact and its admission contract. Keep the fixed selector/sample, target isolation and budgets explicit; #166 must remain unpublished until this Candidate is accepted.

## Verification Plan

### Claims

```text
C1: opaque selector/plugin interpretation and sanitized observation boundary are correct and bounded.
C2: Chromium egress is fail-closed across declared transports and lifecycle paths in synthetic fixtures.
C3: live-mode admission plus independent post-browser consumer is portable and bounded on deterministic fixtures.
C4: exact artifact provenance and remote integration are reproducible from a downloaded bundle.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1 | github-actions | github-hosted-x64 | synthetic Node fixtures | yes | pinned Node/Playwright; `node --test` contract/leakage/selector tests | exact run/job, sanitized output |
| J2 | C2 | github-actions | github-hosted-x64 | isolated hosted Chromium fixtures | yes | probe containment with broker, DNS/private/redirect/worker/SW/CONNECT/QUIC, timeout/cleanup negatives | enforcement matrix and redacted evidence |
| J3a | C3,C4 | github-actions | github-hosted-x64 | artifact producer | yes | build/verify manifest-addressed bundle from exact SHA | artifact ID/digest/manifest |
| J3 | C3,C4 | github-actions | github-hosted-x64 | downloaded artifact consumer | yes | `actions/download-artifact@v4`, verify manifest, run live-admission + muxed/AV fixture consumer without compilation | consumer run and byte/request counts |
| J4 | C1,C2,C3 | github-actions | github-hosted-x64 | static boundary | yes | scan for arbitrary URL/CDP/profile/header input, secret export, open-proxy and production Core coupling | exact Candidate scan result |
| JI1 | C1–C4 | github-actions | github-hosted-x64 | workspace | yes | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace --all-targets` | run/job |

J1–J4 timeout ≤15 minutes; JI1 timeout ≤35 minutes. No target-phone or TV job is part of this Task. #166 later owns J0/J1/J2 external-codex/ssh evidence on tx-node.

### Execution Plane

```text
Execution plane: github-actions
Runner: GitHub-hosted ubuntu-latest x64
Target: deterministic synthetic fixtures; no live site
```

### Interactive debugging

```text
WSL / Windows / Ubuntu ARM64 external Codex required: no
Reason: this Task is portable implementation and hosted verification; tx-node live browser access is independently owned by #166.
```

### Target verification

```text
Target proof required: no
Target: n/a
Why target evidence is required: #166 owns target-specific anonymous Bilibili proof after this artifact is accepted.
```

### Runner Security Constraints

- Trusted source candidate only; pull-request workflows must verify exact Candidate SHA.
- Hosted jobs use no long-lived site credentials and no browser profile from a developer.
- The experimental broker must deny private/reserved destinations and non-declared transports by default.
- Every browser, broker, fixture server, child process and temporary directory is cleaned in `finally` with bounded timeout.
- No job may mutate production services, phone state, VNC state or a self-hosted target.

## Success Criteria

### Task success

1. C1, C2, C3 and C4 are PASS on one exact Candidate SHA with reviewable PR and sanitized evidence.
2. Synthetic mode remains green and no production Core/adapter semantics are changed.
3. The downloaded artifact exposes a documented, explicit live-selector entry that rejects arbitrary caller-controlled authority and is ready for #166 target admission review.
4. #166 runbook points to the accepted live-capable artifact/runbook identity without claiming live-site results.

### Verification claim success

```text
C1 PASS when selector/plugin, schema, budget and sensitive-input positive/negative tests pass on the exact Candidate.
C2 PASS when hosted Chromium evidence demonstrates all declared browser exits are brokered or explicitly disabled, private/redirect/DNS/worker/SW/WebSocket/CONNECT/QUIC paths are denied or constrained, and cleanup is complete.
C3 PASS when the downloaded artifact's explicit live entry is bounded and its independent consumer reads both muxed and AV-separated fixture candidates after browser exit, while blob-only/secret/4xx/expired/over-budget/cancel cases remain negative.
C4 PASS when manifest provenance, exact SHA, downloaded-consumer execution and JI1 remote checks all pass; no local compile is used.
```

A PASS here is an implementation/artifact result only. It is not #166 C0/C1/C2, not a Bilibili compatibility verdict, and not #68 product playback acceptance.

## Evidence Contract

Every Attempt report and Coordinator Review must record:

```text
Role: implementation | combined
Task / Claim: #169 / C1..C4
Attempt: N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high (+ actual Fast availability)
Execution plane: github-actions
Runner class / image: github-hosted x64 / ubuntu-latest
Target host/device: synthetic fixture only
OS / architecture: actual Action runtime / x86_64
Relevant versions: exact Node, Playwright, Chromium and tool versions
Network path: loopback fixture → fail-closed broker; no live Bilibili
Base / Candidate commit: exact SHAs
Workflow / run / job: exact IDs and URLs
Commands / steps: exact selectors and bounded commands
Duration / repetitions: actual values
Metrics / artifact: request/byte counts, sanitized evidence, manifest/digests
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Distinguish implementation result, each verification Claim, Coordinator decision, and parent #68/R005 gate. Worker must post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, set `status:review`/`status:blocked`, release ownership and stop. Coordinator must perform the Review/Freshness/Integration/Final Acceptance protocol; Worker may not close the Issue.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #165 accepted experimental observation/artifact boundary at merged `3511e70c73686d90ec65ca530a5f48be07023586`
- `docs/security.md`, R008 Egress/Secret contracts, `docs/implementation-contracts.md`
- `docs/site-plugin-architecture.md`, `docs/architecture.md`, Browser lifecycle contracts
- `docs/research/bilibili-browser-acquisition.md` and the fixed #166 runbook/sample contract

Semantic freshness domains:
- `plugins/bilibili/` selector and observation interpretation
- `experiments/bilibili-browser-probe/` browser lifecycle, broker, consumer and artifact manifest
- workflow browser/runtime admission and runbook live-entry contract
- R008 DNS/address/redirect/Secret/transport boundary

Integration surfaces:
- `.github/workflows/bilibili-browser-probe.yml`
- Node/Playwright dependency and artifact layout
- runbook/research references; no Cargo production API is intentionally touched

Task-owned surfaces:
- experimental probe/plugin files, focused workflow, artifact manifest/consumer and runbook updates

Authority/domain → Claim mapping:
- #165 experiment/schema → C1,C3,C4
- R008/security/browser transport → C2,C3
- plugin boundary/SourceLocator → C1,C3
- runbook/sample/budget contract → C3,C4

Integration verification:
- JI1: exact Candidate hosted `probe-integration` workflow job with remote fmt/clippy/workspace tests

Unrelated-main policy:
- existing exact-Candidate semantic Evidence remains valid; no rebase/full rerun solely because main advances

Integration-overlap policy:
- preserve semantic Evidence; compose with Coordinator-frozen current main and run only JI1 unless a conflict changes Task-owned semantics

Semantic-authority-change policy:
- reconcile accepted authority and rerun mapped Claims; if live-entry security impact cannot be bounded, return to Contract Revision instead of weakening the broker

Strict-main reason:
- n/a
