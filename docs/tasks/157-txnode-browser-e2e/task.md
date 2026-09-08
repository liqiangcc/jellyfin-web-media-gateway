# Task — TXNODE-BROWSER-E2E Deploy and verify browser Control on tx-node

## Metadata

~~~text
GitHub Issue: #157
Parent Goal: browser end-to-end control verified in the tx-node browser
Task / Research ID: TXNODE-BROWSER-E2E
Task kind: verification
Planning / Evidence Base: 836e220e6ba4e38377a4e40cff677c9549aa7798
Accepted implementation: #154 / PR #156 browser-control-e2e workflow
Session bootstrap prompt: docs/tasks/157-txnode-browser-e2e/prompt.md
Preferred worker: cloud-codex (Fast + gpt-5.6-luna, high reasoning)
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, github-actions-orchestration, ssh-target-diagnostics, interactive-linux-debug, chrome-devtools-mcp, headless-browser-verification
Evidence authority: tx-node process + tx-node Chrome DevTools MCP browser
Hard dependency: #154 ACCEPTED; browser-control-e2e.yml on current main
No dependency: #67/#68 real-site compatibility, phone/ADB, physical TV, Jellyfin
~~~

Issue #157 is the realtime status, owner, Attempt and Evidence authority. This contract is stable; do not copy dynamic run IDs or credentials into it.

## Goal

Deploy the exact GitHub-hosted artifact for a frozen current-main Candidate to tx-node, run Gateway under the dedicated low-privilege gateway-verify identity, and use the browser already running on tx-node through the approved Chrome DevTools MCP connection to verify the product Control → Gateway PlaybackSession → Web Display loop.

The required browser evidence must be produced by the tx-node browser itself. GitHub-hosted Chromium evidence from #154 remains a baseline and cannot substitute for J2 target evidence.

## Target and security preflight

The target is ordinary Linux tx-node, expected to provide:

~~~text
architecture: x86_64
Gateway runtime user: gateway-verify (uid/gid 1001)
browser: Google Chrome with DevTools on loopback 127.0.0.1:9222
Gateway bind: loopback only (127.0.0.1:<port>, no public listener)
~~~

The SSH alias may enter as a provisioning/control identity. It may perform only bounded directory creation, artifact transfer, process launch/inspection and cleanup. The Gateway process and temporary files must be owned by gateway-verify, run without root capabilities, and use NoNewPrivs: 1 where the launcher supports it. If this cannot be proven, stop with BLOCKER REPORT; do not claim a root-run deployment.

Do not expose a public Gateway or CDP listener, copy browser profiles, forward cookies or Authorization, or install persistent credentials. The target must not receive GitHub tokens, SSH keys, Tailscale auth keys, site secrets or Vault data.

## Claims

- C1 — exact artifact deployment: target consumes an artifact produced by GitHub-hosted Actions for one exact Candidate SHA; manifest, per-file SHA-256 and artifact identity are verified before execution; no local or tx-node compilation occurs.
- C2 — target privilege/bind boundary: Gateway runs as gateway-verify uid/gid 1001 with no effective capabilities and NoNewPrivs asserted where supported; listener is loopback-only.
- C3 — tx-node browser Display authority: tx-node Chrome opens an isolated context, loads the real product /display?profile=tv, registers/heartbeats a Display and has no pre-session proof media.
- C4 — tx-node browser Control authority: the same isolated browser opens /control, discovers/selects the live Display and creates a bounded deterministic generic-direct session through the public product API.
- C5 — tx-node command/render loop: browser Control executes play, pause, seek and stop; Web Display receives server-owned same-origin Gateway rendering/media paths bound to authoritative session/item/revision.
- C6 — target refresh/stale/error behavior: Control and Display refresh/reconnect preserve authority and rotate page lease; stale lease/rendering, stale revision, duplicate request ID, missing Display/session and event-resync cases stay bounded.
- C7 — browser evidence provenance: report identifies Orchestrator, Execution Plane, SSH control host, target host class, Gateway uid/gid, browser/CDP mechanism, isolated context, Candidate SHA, run/job/artifact IDs and cleanup.
- C8 — reproducible cleanup: Gateway, tunnel if any, browser context and temporary target files are stopped/removed; same exact artifact can be consumed again without compilation. Unavailable checks are NOT RUN or BLOCKED.
- C9 — scope boundary: this proves ordinary-Linux tx-node browser Control/Display only; it does not claim Bilibili extraction, #67, physical TV/phone, Jellyfin, Native Site Panel or Core Feasibility GO.

## In Scope

- Freeze a Candidate SHA from current origin/main and trigger/read GitHub Actions browser-control-e2e.yml with that exact SHA.
- Download runtime artifact and verify manifest, SHA-256 and run/artifact identity before transfer.
- Transfer only the verified archive/fixture to a private gateway-verify directory on tx-node.
- Launch artifact-backed r001-server as uid/gid 1001, loopback-only, with bounded timeout and cleanup.
- Use tx-node Chrome DevTools MCP in a named isolated context for the full product browser loop and redacted snapshots/DOM/network counters.
- Run target-side stale/error/security checks without exposing secrets.
- Remove temporary files/processes and record final target state.

## Out of Scope

- Source code or product semantic changes unless a separately reviewed defect is found.
- Any local/Codex/tx-node cargo build, cargo test, cargo run, FFmpeg/Chromium compilation or binary-producing command outside GitHub-hosted Actions.
- Running Gateway as root, granting capabilities, disabling NoNewPrivs, changing SSH host policy, or public-binding Gateway/CDP.
- Phone/Android/ADB, physical TV/audible autoplay, Jellyfin, site login, Native Site Panel or real Bilibili/generic-ytdlp extraction.
- Uploading browser profiles, cookies, Authorization headers, Vault data, signed URLs, raw upstream headers or private credentials.
- Treating direct Bilibili clicks or a successful Chrome page load as Gateway Control/Playback evidence.

## Architecture Invariants

1. Gateway/R007 remains the only PlaybackSession command/revision/item/media/display/handoff authority.
2. Control remains View + Intent; Chrome DevTools MCP, DOM, storage and events are not state authority.
3. Source creation uses accepted #44 bounded DTO and SiteAdapterRegistry; no concrete-site or yt-dlp branch is introduced.
4. Display receives only server-owned Gateway-safe same-origin rendering paths.
5. R008 Egress/Secret/SSRF and open-proxy boundaries remain unchanged.
6. Gateway and browser remain loopback-only; any SSH forwarding is narrow, temporary and recorded.
7. Target workdir is separated from Gateway Vault/production runtime and holds no long-lived credentials.
8. Native Site Panel, Jellyfin and physical-device failures remain independent.

## Required execution jobs

### J0 — candidate and target preflight

- Read Issue #157, all comments, this contract, lifecycle/recovery/freshness protocols, accepted #154 package and browser-control-e2e.yml.
- Confirm status:ready, env:cloud, unowned and freeze current origin/main.
- On tx-node record non-secret hostname class, uname, OS, Chrome version/DevTools loopback, gateway-verify uid/gid, workdir ownership, port conflicts, capability and NoNewPrivs facts.
- Prove Chrome DevTools MCP reaches tx-node loopback by opening a temporary isolated page to http://127.0.0.1:9222/json/version or another non-secret marker, then close it. If browser and Gateway cannot be related without public bind, stop BLOCKED.

### J1 — remote build and artifact admission

- Freeze BASE_SHA from origin/main and record it before dispatch.
- Trigger browser-control-e2e.yml using workflow_dispatch with candidate_sha=BASE_SHA, or reuse only a run whose head SHA/input exactly matches BASE_SHA and whose artifact is unexpired.
- Require the exact run and hosted J1/J2/J3/J4 success.
- Download only the runtime artifact named for BASE_SHA; verify manifest, file hashes and run/artifact identity; record digest without printing protected contents.
- Build, test and artifact generation happen only on GitHub-hosted runner.

### J2 — tx-node browser Control/Display loop

- Create a fresh named isolated Chrome DevTools MCP context such as txnode-e2e-shortsha.
- Launch artifact-backed Gateway as gateway-verify uid/gid 1001 using a private fixture and loopback port. Record PID ownership, effective capabilities and NoNewPrivs before browser access.
- Open tx-node browser Display at the loopback Gateway /display?profile=tv; wait for registration/heartbeat and verify no source before a session.
- Open Control; select live Display; submit deterministic generic-direct source through product form; wait for opaque session route and Connected state.
- Verify Display receives a same-origin Gateway rendering/media path and requests Gateway media.
- Through tx-node browser Control execute play, pause, seek and stop; observe state transitions and Display rendering.
- Refresh Control and Display in the same isolated context; verify session continuity, lease rotation and server-owned reconstruction.
- Exercise stale lease/rendering, stale expected revision, duplicate request ID, missing Display/session and event-resync with redacted evidence.
- A Bilibili page may be opened in a separate diagnostic context only after the product loop; it is never C1–C9 evidence.

### J3 — target security and isolation

- Confirm Gateway listener is loopback-only and no public CDP/Gateway port was opened.
- Confirm target process/files are not root-owned and have no effective capabilities or secret environment.
- Scan redacted logs/DOM/storage for Cookie, Authorization, Bearer, Vault, profile, signed query, raw upstream header and arbitrary local path markers.
- Confirm target workdir contains no credentials, browser profile, SSH key or token.
- Confirm product path did not call proof-only seed/store/media injection routes.

### J4 — cleanup and repeatability

- Stop Gateway, close isolated browser context, close temporary SSH forward and remove target artifact directory.
- Verify no process/listener/artifact remains except pre-existing Chrome DevTools endpoint.
- Reconsume same verified artifact once without compiling; record PASS or explicit limitation.
- Post complete EXECUTION REPORT with exact values, release ownership, set status:review and stop. Never merge or close Issue #157.

## Freshness / integration contract

- Freshness policy: strict-main. Deployed artifact must be built and admitted for exact current origin/main frozen at J1; newer main requires a new run/admission.
- Semantic authorities: accepted #154 workflow, #44 SourceSession, #45 Display lease, #47 Control/R007, #48 Display UX and R008 Egress/Secret.
- Semantic freshness domains: route composition, PlaybackSession revision/item/display authority, Display lease/rendering capability, browser security and target privilege/bind.
- Integration surfaces: browser-control-e2e.yml, R001 runtime, target launcher/workdir and Chrome DevTools MCP topology.
- Target evidence is not replaced by hosted baseline evidence. Current main changes after target execution require Coordinator classification.
- If target/browser topology needs a contract/security change, stop BLOCKED; do not patch around it.

## Evidence contract

Every report must distinguish:

~~~text
Implementation Result (deployment/commands performed)
!= Verification Claim Result (C1–C9)
!= Coordinator Task Decision
!= Parent Goal / Research Gate Decision
~~~

Record Candidate/base SHA, exact workflow run, runtime artifact ID/manifest/digest, each required job ID/conclusion, Orchestrator/Execution Plane/SSH control host/Runner/Target, tx-node process uid/gid/capabilities/NoNewPrivs/bind, Chrome MCP mechanism and isolated context, redacted browser route/action/state, cleanup and repeat-consumption result.

Never record cookies, Authorization/Bearer values, lease tokens, signed URLs, raw upstream headers, private browser profile paths, SSH keys or GitHub credentials.

## Success criteria

The Task is complete only when:

1. J0 proves Chrome MCP is the tx-node browser and target has acceptable low-privilege runtime.
2. J1 produces hosted exact-main Candidate artifact with required jobs PASS and verified digest.
3. J2 passes on tx-node Chrome MCP: Display registration, Control selection/session creation, Gateway rendering, play/pause/seek/stop and refresh/reconnect.
4. J3 target security/isolation and stale/error evidence passes without secret leakage.
5. J4 cleanup and same-artifact repeatability are recorded.
6. No phone/physical TV/local compilation occurred.
7. Worker posts EXECUTION REPORT, releases ownership and stops; Coordinator independently reviews, then posts FINAL ACCEPTANCE, sets status:done and closes Issue.

## Worker stop conditions

Post BLOCKER REPORT, set status:blocked, release ownership and stop if:

- Issue is not status:ready + env:cloud or another owner is active;
- exact-main artifact cannot be built/admitted on GitHub-hosted Actions;
- Chrome MCP cannot reach tx-node loopback without public binding;
- Gateway can only run as root or with capabilities/secret/profile exposure;
- target port/workdir conflicts cannot be safely isolated;
- any required step needs a new architecture/security contract or real-site result.

