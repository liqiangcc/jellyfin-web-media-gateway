# Issue #251 Task Contract — Gateway Auth Route

- Task kind: `combined`
- Issue: #251
- Base commit: `373ba8279e6d1f18d9194db954aae2551d244195`
- Eligible environment: `env:cloud`
- Parent goal: #68 Bilibili Web E2E
- Dependencies: accepted #243 Browser Auth Runtime, #240 Browser Observation Bridge, merged #248 Gateway seam

## Goal

Expose the accepted server-side authenticated Browser Worker flow through a bounded Gateway API so an authorized target runner can drive login, hand off a redacted browser observation plus an opaque Vault-owned session reference, and create the normal SourceSession/Playback/Web Display session.

## Scope

Implement a Gateway-owned coordinator/state integration around the generic `BrowserAuthRuntime` and add authenticated API routes for:

1. start an auth attempt for a site/account;
2. poll bounded auth and browser observations;
3. drive only generic navigation/input/cancel operations under `EgressPolicy`;
4. accept a Vault candidate using a generic `BrowserAuthObservation`;
5. bind a server-owned `BrowserObservationHandoff` and invoke the merged #248 authenticated playback seam.

Use typed DTOs with strict bounds, server-owned lookup for attempts, idempotent request handling, stale/expired attempt rejection, cleanup on cancel/expiry, and generic safe error codes. Preserve Origin/CSRF, body-size, SSRF/EgressPolicy, and secret-boundary protections. The HTTP surface must never accept or return Cookie, Authorization, profile bytes/paths, raw media URLs, access references, Vault paths, or arbitrary diagnostic text.

## Required claims

- C1: Gateway owns attempt lifecycle and runtime state; routes are wired through the normal router and cannot expose secrets.
- C2: Generic navigation/input/cancel and redacted event polling enforce attempt ownership, expiry, operation identity, and EgressPolicy.
- C3: Candidate acceptance plus observation handoff reaches the registered SiteAdapter through `ResolveContext` and creates the normal Playback/Web Display response exactly once.
- C4: Deterministic tests cover duplicate request IDs, stale attempt/operation, expiry/cancel cleanup, malformed/oversized DTOs, Origin/CSRF, and secret-safe errors.
- C5: Focused GitHub Actions workflow verifies exact candidate SHA on GitHub-hosted x64 with fmt, clippy, tests, and architecture/secret guards.
- C6: Live Bilibili login/playback remains `BLOCKED` / `NOT RUN`; no target evidence is claimed here.

## Verification

Required evidence is the exact candidate SHA and successful run of `.github/workflows/issue-251-gateway-auth-route.yml`. No local build, test, package, install, phone, TV, VNC, CDP, or live site action is allowed. #191 and #195 are out of scope. #246 remains separately blocked by authorization and target prerequisites.

## Acceptance

Coordinator accepts only when C1–C5 have hosted Actions evidence, C6 is explicitly recorded as blocked/not run, no secret or concrete-site vocabulary crosses generic Core/API DTOs, and the candidate is merged before Issue closure.
