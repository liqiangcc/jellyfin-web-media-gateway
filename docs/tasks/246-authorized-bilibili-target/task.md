# Task — Authorized Bilibili target playback verification

## Metadata

```text
GitHub Issue: #246
Parent Goal / Research Item: #68 Bilibili Web E2E
Task / Research ID: R005-TARGET-AUTHORIZED-PLAYBACK
Task kind: verification
Base commit: 289f4357558b211ebb9be68b7f6fb8b731006324
Candidate commit: accepted main/runtime; exact target Candidate recorded before execution
Session bootstrap prompt: docs/tasks/246-authorized-bilibili-target/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, cloud-interactive, lan-access, tv-browser, manual-observation
Hard publication dependencies: #237, #240, #243, #248, #251, #255, #257 and #259 Final Acceptance; #235 authorization/account gate; Coordinator target publication gate
```

## Goal

On one fresh low-privilege tx-node browser slot, prove that an explicitly authorized Bilibili source can travel through the accepted plugin, Vault-bound Browser Worker auth runtime, server-owned media handoff, PlaybackSession and Gateway same-origin Web Display, with bounded play/pause/seek/stop and sanitized evidence. This Task must produce a real PASS, FAIL or BLOCKED result and must never infer playback from navigation or synthetic fixtures.

## Preconditions / hard blocker

The Task stays `status:blocked` until the Issue contains all of the following, recorded by the Coordinator or authorized operator:

1. Written authorization for this exact automation, account, source content and target host.
2. A dedicated disposable Bilibili test account owned/approved for automation; no personal account and no copied browser profile.
3. An approved interactive authentication channel owned by the Auth Mode design, with timeout/cancellation and no raw VNC/CDP/personal-browser control.
4. Permission to use the tx-node target slot and retain only sanitized bounded evidence.
5. Fresh low-privilege target identity with no Vault, SSH, Tailscale auth key, root/ADB or production credential.
6. Exact accepted implementation Candidates and current hosted regression evidence from #240, #243, #248, #251, #255, #257 and #259; the current integration base is `289f4357558b211ebb9be68b7f6fb8b731006324`.

Absence of any item is `BLOCKED`, not a reason to attempt anonymous login or bypass policy.

## In Scope

- Coordinator read-back of authorization and target identity.
- One fresh target session using the approved authenticated Browser Worker flow and one fixed opaque Bilibili locator.
- Observe generic auth/session/resource lifecycle, Gateway PlaybackSession revision, same-origin Web Display media request, and bounded play/pause/seek/stop behavior.
- Capture only sanitized stage/status/result facts, media kind/protocol/byte counters and timestamps; no URLs with signatures, cookies, Authorization, page bodies, screenshots containing account data, profile archives or raw browser logs.
- Cleanup verification: no process/profile/staging residue after stop/cancel/failure.
- Classify claims with `PASS`, `CONDITIONAL PASS`, `FAIL` or `BLOCKED`; separate target Verification Result from Coordinator Parent Goal decision.

## Out of Scope

- Phone/Android/TV deployment, physical remote/audible proof, VNC, CDP, copied profile, manual Cookie/token injection, CAPTCHA/DRM/paywall/region/access-control bypass, proxy rotation, open proxy or SSRF relaxation.
- Anonymous reruns of closed #188/#232 diagnostics, #191/#195, or changes to production code.
- Local build/test/package/install; implementation verification remains GitHub Actions evidence from accepted Candidates.

## Architecture invariants

- Gateway owns PlaybackSession; Site Plugin owns Bilibili URL/login/media semantics; Core remains site-agnostic.
- Vault is the sole Secret owner. Worker/Plugin/Display/Control receive opaque refs/capabilities, never Cookie/Authorization/profile material.
- R008 EgressPolicy governs every destination and redirect; authentication never widens egress.
- Display Adapter does not read Vault; Native Panel failure cannot stop Web Display.
- Target runner is isolated, low privilege and disposable.

## Claims

```text
C1: Authorized account/profile handoff completes through the accepted generic auth runtime without Secret leakage.
C2: Bilibili plugin resolves the fixed opaque locator using server-owned observations and produces a valid media candidate under R008.
C3: PlaybackSession commits the candidate with correct revision/generation and Gateway Web Display receives same-origin media.
C4: Target browser observes bounded play/pause/seek/stop (or a precise media failure) and cleanup leaves no target residue.
C5: Evidence is sanitized, exact-target provenance is recorded, and any missing prerequisite/result is classified BLOCKED/FAIL rather than guessed.
C6: Parent #68 playback acceptance is a separate Coordinator Gate; this Task's result alone does not close the parent.
```

## Verification matrix

| Job | Claims | Execution plane | Runner/target | Required |
|---|---|---|---|---|
| J0 | C1,C5 | Web/GitHub read-back | Coordinator | yes before target |
| J1 | C1-C4 | approved target execution | tx-node isolated browser slot | yes after J0 |
| J2 | C5 | target cleanup/read-back | tx-node low-privilege identity | yes |
| J3 | implementation freshness | GitHub Actions | hosted x64, exact current integration Candidate `289f4357558b211ebb9be68b7f6fb8b731006324` | yes before target |

No target action is permitted while `status:blocked`; a Worker may claim only after publication gate changes it to `status:ready` and records all authorization prerequisites.

## Success criteria

- Issue history contains authorization read-back, exact target/runner provenance and one bounded Attempt.
- J1/J2 produce sanitized evidence with a clear PASS/FAIL/BLOCKED classification.
- No secret, personal account data, signed URL or raw browser/profile artifact enters Issue comments, logs or artifacts.
- Worker reports `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, releases ownership and stops; Coordinator alone decides parent acceptance and closes this Issue.
