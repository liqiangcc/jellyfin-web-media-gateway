# Task — Generic MediaShapeV1 contract and paired A/V projection

## Metadata

```text
GitHub Issue: #268
Parent Goal / Research Item: #68 Bilibili Web E2E
Research anchor: #265 Final Acceptance (CONDITIONAL PASS), main `32498b845a147a9afa9a231070cfc9622886ccac`
Task / Research ID: R005-MEDIA-CONTRACT
Task kind: implementation
Base commit: 32498b845a147a9afa9a231070cfc9622886ccac
Candidate commit: n/a until Worker attempt
Session bootstrap prompt: docs/tasks/268-r005-media-contract/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive
Hard publication dependencies: #265 Final Acceptance; accepted #237/#240/#243/#248/#251/#255/#257/#259; Coordinator Publication Gate
```

## Goal

Implement a versioned, generic `MediaShapeV1`/track contract that represents muxed, video and audio tracks, pairing, source protocol, codec/container metadata, bounded expiry and opaque server-owned access references. Make the Bilibili plugin interpret paired BrowserObservation candidates into this shape without adding Bilibili semantics to Core. Preserve existing muxed HTTP-file/HLS behavior and Playback/Display authority.

This Task does not claim live Bilibili resolution or playback and does not implement the remux process itself; it produces the contract that a later delivery Task can consume.

## Scope

1. Extend `site-adapter-api` with bounded, versioned track/shape types and conformance validation. Support `http_file`, `hls` and `dash` source protocols plus `muxed`, `video`, `audio` roles and opaque pairing/group identity.
2. Keep legacy direct `ResolvedMedia` construction source-compatible through an explicit compatibility/default path; do not leak shape internals into public Control/Display DTOs.
3. Update the Bilibili plugin to accept a valid paired video+audio observation under its existing generic handoff and return the generic shape; reject incomplete groups, unsupported protection, expired/egress-denied candidates and secret-bearing refs.
4. Update Registry/SourceSession validation and deterministic fixtures so the shape remains site-neutral, bounded and secret-safe. Public projection may expose only bounded role/group/media-generation metadata and Gateway resource identities.
5. Add exact-Candidate hosted x64 Actions for format/clippy/tests, conformance, architecture/site-boundary guards and synthetic paired/muxed/invalid cases. No local build/test/package/install.

## Out of scope

- Server remux/FFmpeg supervisor, MPD/MSE player or HLS player implementation (follow-up Task).
- Live Bilibili requests, login/account/profile/cookie handling, tx-node, phone/TV/VNC/CDP, physical/audible playback or #246 target verification.
- Any Core Bilibili URL/DOM/API branch, open proxy/SSRF relaxation, Cookie/Authorization/signed URL projection, #191/#195 or anonymous #188/#232 reruns.

## Architecture invariants

- Gateway remains PlaybackSession authority; Site Plugin owns Bilibili semantics; Browser Worker emits generic observations.
- Vault is the only Secret owner; Display/Control receive opaque bounded projections only.
- Every future delivery input remains subject to EgressPolicy; this Task must not bypass it.
- SourceLocator remains plugin-owned content identity; short-lived source URLs are not identity.
- Existing muxed HTTP-file/HLS and R007 stale/revision behavior remain compatible.

## Claims

```text
C1: MediaShapeV1 is versioned, bounded and site-neutral, with roles, pairing, source protocol, codec/container metadata, expiry and opaque access refs.
C2: Conformance validation accepts valid muxed and paired video/audio shapes and rejects incomplete, mismatched, expired, unsupported or secret-bearing shapes.
C3: Bilibili interpretation consumes only generic BrowserObservation facts and returns the shape without leaking site URL, Cookie, Authorization, signed query, profile or raw body.
C4: Registry/SourceSession/public projection preserve existing direct playback behavior, PlaybackSession authority and stale/idempotency boundaries.
C5: Exact-Candidate hosted Actions pass format/clippy/tests, architecture/site-boundary guards and synthetic shape fixtures.
C6: No live Bilibili authentication, target navigation, media delivery or playback is claimed; #246 remains independently blocked.
```

## Verification matrix

| Job | Claims | Execution plane | Runner/target | Required |
|---|---|---|---|---|
| J0 | C1,C6 | Coordinator/GitHub read-back | current main and #265 evidence | yes |
| J1 | C1-C5 | GitHub Actions | hosted x64 fmt/clippy/workspace tests | yes |
| J2 | C2-C4 | GitHub Actions | hosted x64 paired/muxed/invalid conformance and secret-boundary fixtures | yes |
| J3 | C3-C5 | GitHub Actions | hosted x64 architecture/site-neutrality/public DTO guards | yes |
| J4 | C5,C6 | Coordinator review | exact Candidate, run/job/artifact provenance | yes |

No target action is permitted. A later remux/delivery Task must consume this accepted contract and retain #246's authorization gate.

## Success criteria

- One focused Candidate/PR implements the contract and plugin projection without production remux code.
- Required hosted jobs pass on the exact Candidate SHA; all artifacts/logs are bounded and secret-free.
- Existing muxed HTTP-file/HLS tests remain green and no concrete-site knowledge enters Core/Browser Worker.
- Worker posts `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, releases ownership and stops; Coordinator alone reviews/merges/accepts.
