# Task — Policy-bound separated A/V remux delivery

## Metadata

```text
GitHub Issue: #271
Parent Goal / Research Item: #68 Bilibili Web E2E
Research anchors: #265 (CONDITIONAL PASS), #268 (accepted), main `2bf8ce4d27ce937a2649bccaa0fd3ffbf1bead10`
Task kind: implementation
Base commit: 2bf8ce4d27ce937a2649bccaa0fd3ffbf1bead10
Candidate commit: n/a until Worker attempt
Session bootstrap prompt: docs/tasks/271-r005-media-delivery/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: #265 and #268 accepted; #246 authorization gate remains independent
```

## Goal

Implement a generic Gateway-owned remux delivery supervisor consuming validated paired `MediaShapeV1` tracks and exposing a bounded browser-compatible fMP4/HTTP-file capability. Preserve direct muxed/HLS behavior and PlaybackSession authority.

## Scope

1. Define delivery request/result contracts bound to session, item, revision, media generation and track group.
2. Validate paired tracks and server-owned capability ownership before starting; reject stale, expired, incomplete, unsupported or secret-bearing inputs.
3. Run FFmpeg/remux using structured argv or controlled stdin/broker inputs. FFmpeg must never receive an unvalidated site URL or display/Vault secret.
4. Enforce cancellation, timeout, cleanup, output limits, expiry refresh/error classification and stale-generation CAS.
5. Project only a short-lived Gateway HTTP-file capability to Web Display; upstream URLs, headers, access refs and Vault material remain server-side.
6. Add hosted x64 synthetic fMP4 fixture tests and security/architecture guards. No local build/test/package/install.

## Out of scope

- Live Bilibili/login/account/profile, tx-node, phone/TV/VNC/CDP, physical playback or #246 target verification.
- DASH/MSE or HLS player libraries, site-specific URL/DOM/API logic, open proxy or SSRF relaxation.
- #191/#195 and anonymous #188/#232 reruns.

## Architecture invariants

- Gateway remains PlaybackSession authority; Site Plugin remains site-semantic owner; Core consumes generic shape only.
- EgressPolicy is mandatory for every upstream input; Vault is the only Secret owner.
- FFmpeg receives only controlled broker/stdin inputs and structured arguments; no shell concatenation.
- Display receives only a short-lived Gateway resource capability.
- Stale callbacks and generations cannot overwrite newer playback state.

## Claims

```text
C1: Delivery accepts only validated MediaShapeV1 paired tracks and enforces session/item/revision/generation binding.
C2: EgressPolicy and server-owned capability boundaries hold; no URL/header/Cookie/Authorization/Vault material reaches FFmpeg argv, public DTOs, logs or artifacts.
C3: Timeout, cancellation, expiry, stale generation and cleanup are bounded and cannot corrupt PlaybackSession state.
C4: Synthetic hosted x64 fMP4/HTTP-file fixtures prove bounded output and browser resource projection while direct muxed/HLS regressions remain green.
C5: No live Bilibili or target playback is claimed; #246 remains independently blocked.
```

## Verification matrix

| Job | Claims | Execution plane | Runner/target | Required |
|---|---|---|---|---|
| J0 | C1,C5 | Coordinator/GitHub read-back | main + #265/#268 evidence | yes |
| J1 | C1-C4 | GitHub Actions | hosted x64 fmt/clippy/workspace tests | yes |
| J2 | C1-C3 | GitHub Actions | synthetic paired tracks, stale/expiry/cancel/cleanup matrix | yes |
| J3 | C2,C4 | GitHub Actions | FFmpeg argv/stdin isolation, output bounds and public projection guards | yes |
| J4 | C4,C5 | GitHub Actions | synthetic browser HTTP-file resource smoke; no real site | yes |
| J5 | C1-C5 | Coordinator review | exact Candidate, run/job/artifact provenance | yes |

## Success criteria

- Focused Candidate/PR adds the generic remux delivery path without site-specific Core branches.
- Required hosted jobs pass on the exact Candidate SHA with bounded, secret-free artifacts.
- Direct muxed/HLS and R007 stale/revision behavior remain green.
- Worker posts `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, releases ownership and stops; Coordinator alone reviews/merges/accepts.
