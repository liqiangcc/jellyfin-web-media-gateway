# Task Contract — R005 Bilibili Site Plugin acquisition

- Issue: #237
- Parent: #68 public Web playback
- Research anchor: #235 Final Acceptance; anonymous #232 remains 4xx/no-media
- Task kind: `implementation`
- Preferred worker: Codex Cloud (`env:cloud`), `gpt-5.6-luna`, reasoning high
- Base: `main@6c78a2f85b863e0f9c03ac8b577a07b743e22828`

## Goal

Add the first production Bilibili Site Plugin boundary needed to turn a Bilibili source into a plugin-owned opaque `SourceLocator` and to interpret bounded generic Browser Worker observations into a Secret-safe, standard media result. Keep site URL/DOM/login/media semantics inside `plugins/bilibili`; keep Gateway Core, Browser Worker, PlaybackSession and Display site-agnostic.

This Task closes the contract and deterministic implementation gap only. It does not claim that Bilibili can currently be reached or played on tx-node.

## Scope

1. Add a production `plugins/bilibili` adapter registered through `SiteAdapterRegistry` (or the smallest registry-facing integration required by the existing workspace).
2. Recognize only bounded supported Bilibili source inputs and produce a versioned opaque locator owned by the plugin. Locator payload must contain content identity/part semantics only; never CDN URLs, cookies, Authorization, bearer tokens, profile paths, signed query material or account secrets.
3. Define and implement the minimal versioned browser-observation handoff required by the current APIs. The Browser Worker may emit only generic, bounded, redacted facts; the Bilibili plugin interprets site meaning. Do not add Bilibili selectors, URL rules, Cookie names or login-success rules to `gateway-core/src/browser*.rs`.
4. Define a plugin-owned resolution path that accepts only an explicitly scoped access/capability or server-owned observation handoff. Preserve R008 EgressPolicy and server-side secret injection; `ResolvedMedia.public_headers` must remain free of Cookie/Authorization/bearer material. Short-lived upstream URLs may exist only in bounded server-side state/capability references.
5. Support deterministic conformance fixtures for recognition, locator ownership/versioning, browser-result interpretation, media shape validation, rejection, redaction and registry routing. Use synthetic observations only; no live Bilibili request is required for this Task.
6. Wire only the minimum SourceSession/Playback integration needed to exercise the generic contract with a deterministic fixture. Preserve R007 revision/stale-result semantics and existing generic-direct/ytdlp behavior.

## Out of scope

- Real account login, profile materialization, Vault reads, password/verification/QR handling, or authenticated target proof (future children under #26/#235).
- Any real Bilibili/tx-node request, playback click, media consumer, browser/VNC/CDP observation, phone or TV deployment.
- CAPTCHA/DRM/access-control/region bypass, proxy rotation, open CONNECT proxy, arbitrary URL/host authority, Cookie/header/token smuggling.
- Site-specific branches in Core, Browser Worker, Control, Display or Playback; changing Gateway authority; changing #191/#195.
- Local compile/test/package/install. All required verification runs in GitHub Actions on the exact Candidate SHA.

## Architecture invariants

- Gateway remains `PlaybackSession` authority; Browser Worker is generic runtime; Display never reads Vault.
- Core stores/transports opaque locators and standard results only; it does not parse Bilibili identifiers or DOM/private APIs.
- Site Plugin never reads Vault directly and cannot bypass `EgressPolicy`.
- Generic yt-dlp remains a Site Plugin; no Core fallback is added.
- No Secret enters SourceLocator, logs, normal browser events, Control/Display DTOs, artifacts or Target Runner state.

## Claims and success criteria

- C1 — `PASS`: production Bilibili adapter is registry-routable and recognizes/rejects bounded inputs deterministically; locator version/ownership and no-secret tests pass.
- C2 — `PASS`: browser observation contract is versioned, bounded and generic at the Worker boundary; Bilibili interpretation is isolated to the plugin and rejects malformed/stale/oversized observations.
- C3 — `PASS`: deterministic plugin resolution/standard media handoff validates protocol/stream shape and secret boundary; no raw upstream credential is returned.
- C4 — `PASS`: SourceSession/registry integration preserves existing idempotency, revision and stale-result behavior; existing adapters remain conformant.
- C5 — `PASS`: security/conformance tests cover SSRF/egress admission, sensitive header redaction, opaque locator limits and unsupported/DRM/expiry outcomes.
- C6 — `BLOCKED` unless separately supplied: real Bilibili target reachability, authenticated access and actual Web Display playback are explicitly not part of this implementation Task.

Implementation acceptance requires C1–C5 and all required hosted verification jobs. C6 must remain `BLOCKED`/`NOT RUN`, never be inferred from synthetic fixtures or #232.

## Verification matrix

All jobs run through GitHub Actions; local build/test/install is forbidden.

- Hosted x64: format/lint/unit/conformance/security tests for `site-adapter-api`, `plugins/bilibili`, `gateway-core` and affected workspace crates.
- Hosted x64: architecture check proving no concrete Bilibili import/branch in Core/Browser Worker and no secret-bearing fields in public DTOs.
- Hosted x64: deterministic SourceSession/Playback stale-callback and idempotency regressions covering the new plugin route.
- Target job: none for this Task. A later independently published verification Task must use an exact accepted Candidate/artifact and the #235 legal/account gate before any authenticated or live target action.

Required evidence records exact Candidate SHA, workflow run/job IDs, artifact digests where applicable, and separates Implementation Result, Verification Result and Coordinator Decision.

## Freshness and integration

Use dependency-aware freshness. Rebase/integrate current `main` before final Candidate if required by the worker protocol. Do not silently broaden this Task because #68 or #235 changes; contract changes require Coordinator review and a revised package.

## Worker completion

Claim #237 only after Publication Gate sets `status:ready`. Report one Attempt using the lifecycle protocol, then set `status:review` (or `status:blocked`) and release ownership. Worker must not close the Issue or set `status:done`.
