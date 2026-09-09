# Task Contract — R005 Gateway runtime bootstrap

- Issue: #255
- Parent Goal / Research Item: #68 Bilibili Web E2E
- Related accepted implementation: #237, #240, #243, #248, #251
- Task kind: implementation
- Base: bef3fcf7cd265e9a9b22ebaa5de0c7d1e0d72332
- Preferred worker: Codex Cloud (env:cloud), gpt-5.6-luna, reasoning high
- Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
- Hard boundary: no live Bilibili/login/tx-node/browser/VNC/CDP action; #246 remains separately blocked; no phone deployment; no #191/#195; no local build/test/install

## Goal

Make the accepted Bilibili Site Plugin and Gateway authenticated-control seam reachable from one explicitly named, non-proof Gateway executable composition root. The executable must register the Bilibili adapter through SiteAdapterRegistry, configure an HTTP authority, and initialize only a non-secret, deployment-owned Bilibili site_id/account_ref/label entry so the existing POST /api/v1/auth/attempts route can start a bounded attempt in a real packaged process. Build and package this executable only through GitHub Actions for later target consumption.

This Task closes executable composition and artifact provenance. It does not create Vault candidate captures, implement the interactive auth UI, claim live Bilibili access, or prove media playback.

## In Scope

1. Add or designate a production-oriented Gateway runtime binary/composition root separate from the r001-server proof harness. It may reuse existing GatewayService routes but must not seed proof sessions or inject fixture media.
2. Register BilibiliAdapter and the already supported safe generic adapters only from the composition root. Do not add Bilibili branches to gateway-core business logic.
3. Add bounded deployment configuration for bind address/port, HTTP authority, and the non-secret Bilibili account registration tuple. Reject missing/invalid configuration fail-closed; never accept account secrets, Cookie, Authorization, profile paths or arbitrary plugin IDs from HTTP.
4. Keep default bind private/loopback. Do not create an open proxy, broaden EgressPolicy, or expose VNC/CDP. Preserve GatewayService::configure_auth_account as the only server-side account registration path.
5. Add deterministic hosted tests for registry routing, bounded secret-free account configuration, auth-start reachability through the fake/runtime seam, rejection of an unregistered account, and separation from the proof harness.
6. Add a focused GitHub Actions workflow that asserts the exact Candidate SHA, runs fmt/clippy/unit/architecture/secret guards on hosted x64, and produces a bounded packaged executable/artifact manifest suitable for later compile-free target consumption. No local compilation or target compilation is allowed.

## Out of Scope

- Vault candidate capture/profile persistence from a Browser Worker session (follow-up Task).
- Generic Browser Auth Experience/UI, remote view/input transport, or interactive login.
- Real account authorization, passwords, QR/CAPTCHA/verification input, live Bilibili requests, tx-node target execution, VNC/CDP, phone/TV/Jellyfin deployment.
- DASH/A-V/remux/HLS playback changes; media-shape changes must wait for authorized target evidence.
- Replacing auth_route.rs storage with the follow-up registry, changing Playback authority, changing #68 publication state, or processing #191/#195.
- Any local build/test/install/package command. Required verification is GitHub Actions only.

## Architecture Invariants

- Gateway remains PlaybackSession authority; the executable is only a composition root.
- Core remains site-agnostic; all Bilibili URL/DOM/login/media semantics stay in plugins/bilibili.
- Site Plugin does not read Vault directly; Vault remains the sole Secret owner.
- Browser Worker events and all public DTOs remain bounded and secret-free.
- Every outbound destination remains subject to EgressPolicy; authentication does not widen egress.
- Target runners never inherit Vault, production Secret, root, ADB, SSH or Tailscale authority.
- r001-server remains a deterministic media-path proof harness and must not silently become the production runtime.

## Claims

- C1: The production composition root registers Bilibili and supported generic adapters through SiteAdapterRegistry; Core has no concrete-site branch.
- C2: Deployment configuration is bounded, private-by-default and secret-free; only server-owned non-secret account refs can initialize Vault account records.
- C3: A deterministic fake/runtime test proves an authorized registered account can reach the auth-start route, while unregistered accounts fail closed.
- C4: The runtime binary/package is reproducibly tied to an exact Candidate SHA and hosted x64 build/artifact manifest.
- C5: Existing R007/R008/auth/playback/security regressions and architecture guards pass for the exact Candidate.
- C6: Live Bilibili authentication, media resolution and Gateway Web Display playback are BLOCKED/NOT RUN in this Task and remain owned by #246/#68.

## Verification Job Matrix

| Job | Claims | Execution plane | Runner | Required |
|---|---|---|---|---|
| J1 | C1-C3,C5 | GitHub Actions | hosted x64 | yes |
| J2 | C2,C4 | GitHub Actions | hosted x64 | yes |
| J3 | C4 | GitHub Actions artifact consumer | hosted x64 | yes |
| J4 | C6 | target | none | no; must remain NOT RUN |

Each required job must assert the exact Candidate SHA. Evidence must record actual run/job URLs, artifact name/digest/manifest and separate Implementation Result, Verification Result and Coordinator Decision. Do not count unrelated aggregate workflow failures as this Task's required evidence.

## Freshness / Integration

Use dependency-aware freshness. Worker must report exact observed main/base and Candidate. If main changes before review, Coordinator classifies freshness; do not silently change #246 or #68 contracts. Reuse this Issue/PR for revisions.

## Worker completion

Worker claims only after Publication Gate sets status:ready, then posts one [EXECUTION REPORT] or [BLOCKER REPORT], sets the Issue to status:review, releases ownership and stops. Worker must not merge or close the Issue. Coordinator alone reviews, merges and performs Final Acceptance.

