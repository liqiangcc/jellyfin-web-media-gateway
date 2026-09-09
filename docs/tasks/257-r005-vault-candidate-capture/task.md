# Task Contract — R005 Vault-bound candidate capture

- Issue: #257
- Parent Goal / Research Item: #68 Bilibili Web E2E
- Related accepted implementation: #243 Browser Auth Runtime, #251 Gateway auth route, #255 Gateway runtime bootstrap
- Task kind: implementation
- Base: 12a5da5356ba2b809971120aaf1df7dfde67bea3
- Preferred worker: Codex Cloud (env:cloud), gpt-5.6-luna, reasoning high
- Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
- Hard boundary: no live login/target/tx-node/browser/VNC/CDP action; no phone deployment; no #191/#195; no local build/test/install

## Goal

Add the server-owned candidate-capture seam needed after a generic Browser Worker auth attempt reaches a plugin-interpreted CandidateReady state. The seam must create a Vault-owned candidate session from bounded worker-owned session/profile material, return only an opaque SiteSessionRef to trusted server code, and let the existing atomic Vault validation/swap produce AuthenticatedSessionHandoff. Raw Cookie, Authorization, localStorage, profile bytes, profile paths and browser state must never cross HTTP, Site Plugin, Control, Display, ordinary events, logs or artifacts.

This Task closes the repository candidate-acquisition contract only. It does not claim live Bilibili login, target execution, media resolution or Gateway playback.

## In Scope

1. Extend the generic BrowserWorker/Auth runtime with a server-internal, one-shot candidate-capture operation bound to the live attempt, site_id, account_ref, expiry and CandidateReady observation.
2. Add a Vault-owned capture/materialization sink or equivalent private API that creates a candidate session without exposing SecretMaterial or a filesystem path to untrusted callers. Apply finite size/count/time limits, reject empty or malformed material, and keep profile persistence under Vault ownership.
3. If profile data is copied from a Chromium worker profile, enforce destination ownership, no traversal/symlink escape, bounded files/bytes, cleanup on success/failure/cancel/expiry and no profile archive in public artifacts. The worker must not read another account/profile or use a copied personal profile.
4. Make capture idempotent/stale-safe: duplicate request_id returns the same opaque candidate outcome; mismatched request or stale operation/attempt/observation fails closed; capture after cancel/expiry/crash cannot create or swap a candidate.
5. Integrate the captured candidate with BrowserAuthAttempt.accept_candidate or an equivalent trusted server path so Vault.validate_and_swap remains atomic. Invalid/cancelled candidates remove only the new candidate and preserve the previous active session.
6. Add deterministic fake-worker/Vault tests for candidate creation, binding, duplicate/mismatch, invalid observation, atomic swap, old-session preservation, cleanup and secret/debug redaction. Add architecture guards proving no capture DTO or public log contains Secret fields.
7. Add a focused GitHub Actions workflow asserting the exact Candidate SHA and running hosted x64 fmt/clippy/unit/security/architecture checks. No local compilation, test, install or package command.

## Out of Scope

- Production Gateway executable composition (tracked by #255).
- Generic auth UI, remote view/input transport, VNC/CDP, password/QR/CAPTCHA handling or site-specific login semantics.
- Bilibili DOM/API selectors, media extraction, DASH/remux/HLS support, target playback, phone/TV/Jellyfin deployment.
- Changes to Playback authority, Display authority, EgressPolicy exceptions, open proxy behavior or #68/#246 publication state.
- Any direct HTTP endpoint that accepts or returns raw profile/session material.
- #191/#195 and local build/test/install.

## Architecture Invariants

- Session Vault is the sole owner of persistent site Secret/profile material.
- Browser Worker is generic; Site Plugin interprets auth observations and supplies only bounded candidate-ready meaning.
- Core exposes opaque references/capabilities only; Site Plugin never reads Vault directly.
- Candidate validation/swap preserves the old active session on failure and uses existing R007/R008 boundaries.
- Target runners remain low privilege and cannot inherit Vault, SSH, Tailscale, root or production credentials.

## Claims

- C1: A live generic auth attempt can produce a Vault-owned opaque candidate reference through a server-internal capture seam.
- C2: Candidate capture is one-shot, bounded, expiry/cancel/crash-safe and cannot be triggered by stale/mismatched requests.
- C3: Valid candidates atomically swap through SessionVault; invalid/cancelled candidates never replace the previous active session.
- C4: Profile/Secret material stays inside Vault and owned worker runtime; DTOs, events, logs and artifacts remain redacted.
- C5: Hosted x64 deterministic tests and architecture/security guards pass for the exact Candidate SHA.
- C6: Real account login, Bilibili access, target evidence and playback remain BLOCKED/NOT RUN and are owned by #246/#68.

## Verification Job Matrix

| Job | Claims | Execution plane | Runner | Required |
|---|---|---|---|---|
| J1 | C1-C5 | GitHub Actions | hosted x64 | yes |
| J2 | C2-C4 | GitHub Actions | hosted x64 | yes |
| J3 | C5 | GitHub Actions artifact/log guards | hosted x64 | yes |
| J4 | C6 | target | none | no; must remain NOT RUN |

Every required job must assert the exact Candidate SHA and record actual run/job URLs. Separate Implementation Result, Verification Result and Coordinator Decision. Do not count unrelated aggregate workflow failures as this Task evidence.

## Freshness / Integration

Use dependency-aware freshness. Worker reports exact base/Candidate. If #255 changes overlapping auth/runtime files before this Task is accepted, Coordinator must classify integration freshness and reuse this Issue/PR rather than silently creating a duplicate implementation.

## Worker completion

Worker claims only after Publication Gate sets status:ready; then posts [EXECUTION REPORT] or [BLOCKER REPORT], sets status:review, releases ownership and stops. Worker must not merge or close the Issue. Coordinator alone reviews and performs Final Acceptance.

