# Task — Generic separated A/V media delivery decision

## Metadata

```text
GitHub Issue: #265
Parent Goal / Research Item: #68 Bilibili Web E2E
Task / Research ID: R005-MEDIA-SHAPE-DELIVERY
Task kind: research
Base commit: 91cf910e2e280dc48ac5359f74c6bf629f9d590e
Candidate commit: n/a until Worker attempt
Session bootstrap prompt: docs/tasks/265-r005-media-shape-delivery-decision/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: current main; accepted #237/#240/#243/#248/#251/#255/#257/#259; Coordinator Publication Gate
```

## Goal

Using only synthetic, secret-free BrowserObservation fixtures and hosted GitHub Actions, define and choose a generic delivery path for muxed and separated video/audio media that can later carry Bilibili playback through `ResolvedMedia`, Media Gateway and Web Display. Produce a reviewable contract decision and implementation handoff. Do not perform live Bilibili, login, tx-node, phone, TV, VNC or CDP activity.

The Task must classify each result as `PASS`, `CONDITIONAL PASS`, `FAIL` or `BLOCKED`. It must not infer real Bilibili playback from synthetic fixtures or earlier Chrome observation.

## Scope

1. Audit the current `ResolvedMedia`, `ResolvedStream`, `StreamProtocol`, SourceSession capability projection and Web Display media-selection behavior.
2. Define a versioned generic shape for muxed, video and audio tracks, pairing/group identity, protocol, codec/container metadata, bounded expiry and server-owned upstream access references without Bilibili fields.
3. Compare server remux to browser-compatible fMP4/HTTP-file, server remux to HLS, DASH+MSE, and source-browser playback against capability binding, EgressPolicy, cancellation/expiry, stale item/display generation, seek/reconnect, browser compatibility and bounded resource requirements.
4. Select one first implementation route, or classify the decision `CONDITIONAL PASS`/`BLOCKED` with exact missing evidence and a follow-up implementation contract.
5. Update only task/research documentation and bounded synthetic verification harnesses required for the decision. Production Bilibili/plugin behavior is out of scope.

## Out of scope

- Real Bilibili requests, account login, cookies, profiles, signed URL retention, tx-node access, phone/TV deployment, VNC/CDP, or physical/audible playback.
- FFmpeg installation or execution on tx-node; no local compile/test/package/install.
- Site-specific fields, Bilibili URL/DOM/API logic or changes to Core that violate site-agnostic boundaries.
- Reopening/processing #191/#195 or repeating anonymous #188/#232 diagnostics.

## Architecture invariants

- Gateway remains PlaybackSession authority; Site Plugin owns site semantics; Browser Worker stays generic.
- Display never reads Vault. Secrets and signed URLs remain server-owned opaque capabilities.
- Every upstream request, redirect and remux input remains inside EgressPolicy; no open proxy or arbitrary host authority.
- Any future FFmpeg/process path must use structured argv, bounded resources, cancellation, cleanup and no direct unvalidated site URL access.
- Existing muxed HTTP-file/HLS behavior and R007 stale/revision semantics remain compatible.

## Claims

```text
C1: Current unsupported DASH/separated-A/V behavior has direct source-level evidence and is distinct from live-site playback.
C2: A generic versioned media shape represents muxed/video/audio tracks, pairing, protocol, codec/container, expiry and opaque access without Bilibili semantics or Secret leakage.
C3: Candidate delivery routes are compared against capability binding, EgressPolicy, cancellation/expiry, stale item/display generation, seek/reconnect, browser compatibility and bounded resource requirements.
C4: One first implementation route is selected, or the decision is explicitly CONDITIONAL PASS/BLOCKED with exact follow-up evidence and an implementation Task outline.
C5: Hosted synthetic verification is reproducible, exact-Candidate bound and emits no source URL, cookie, Authorization, signed query or raw media/profile artifact.
C6: No real Bilibili authentication, target navigation or playback is claimed; #246 remains independently blocked.
```

## Verification matrix

| Job | Claims | Execution plane | Runner/target | Required |
|---|---|---|---|---|
| J0 | C1,C6 | Coordinator/GitHub read-back | current main and canonical docs | yes |
| J1 | C1,C2,C5 | GitHub Actions | hosted x64 static/schema/secret-boundary checks | yes |
| J2 | C2,C3,C5 | GitHub Actions | hosted x64 synthetic separated-A/V fixture and capability lifecycle checks | yes |
| J3 | C3,C4,C5 | GitHub Actions | hosted x64 bounded browser/media compatibility harness; no live site | yes |
| J4 | C4,C5,C6 | Coordinator review | exact Candidate, Actions artifacts and follow-up handoff | yes |

No target action is permitted. A future implementation Task must be separately published after this decision and must preserve the current #246 authorization gate.

## Success criteria

- Issue history contains one bounded Attempt, exact Candidate and hosted run/job/artifact provenance.
- The current gap and selected route are backed by synthetic/static evidence, with limitations stated.
- The result includes a concrete follow-up implementation contract or an explicit BLOCKED condition; no vague “should work” language.
- No Secret, personal account data, signed URL, raw browser/profile artifact or real-site request enters comments, logs or artifacts.
- Worker reports `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, releases ownership and stops; Coordinator alone decides acceptance and publication of any child implementation Task.
