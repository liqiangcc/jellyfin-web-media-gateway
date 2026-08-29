# Session Bootstrap — Issue #116

You are an **independent Ubuntu ARM64 Worker**, not the Web Coordinator and not the s2 Dispatcher.

Execute Issue #116 using `docs/tasks/116-env-arm64-android-browser-cdp-bridge-prep/task.md` as the canonical contract.

## Entry gate

1. Live-read Issue #116 and this Task Contract from GitHub.
2. Proceed only if Issue #116 is OPEN, has `status:ready`, has `env:ubuntu-arm64`, and has no owner.
3. If the gate is not satisfied, do not claim or execute; report the observed state to the Dispatcher and STOP.
4. If satisfied, claim durably yourself using the repository lifecycle protocol: assign the owner, replace `status:ready` with `status:in-progress`, read back the claim, and write a bounded `[EXECUTION CHECKPOINT]`.

## Role boundary

- You are the Worker. s2 only dispatched you.
- Do not ask s2/Coordinator to run shell steps for you.
- Do not merge/close/mark done.
- Do not publish or execute #117.
- Do not execute #67, #113, #68, yt-dlp, generic-ytdlp, broker/R008/sandbox resolver paths.

## Task objective

Build and prove the smallest safe capability:

`dedicated Android diagnostic browser → local-only CDP transport → minimal repository-owned browser-diag MCP → this Worker`

This Task proves tooling/transport on a neutral HTTPS page only. It does not diagnose Bilibili.

## Mandatory privacy boundary

Never attach to or enumerate the user's normal browser profile/tabs. Never copy browser cookies, login/account state, passwords, local/session storage, cache/history or credentials into the diagnostic browser or Worker.

Do not expose arbitrary CDP commands. Do not log/return page bodies, arbitrary headers, Cookie/Auth/Set-Cookie, full signed/query URLs, tokens, media URLs/payloads or raw DevTools event dumps.

## Execution order

### C0 — Capability audit first

Record only bounded device capability facts. Determine the least-privilege standard route for a **dedicated** debug-capable Android browser to expose CDP locally to the Worker.

If a safe route is unavailable, or requires an operator-only action you cannot safely complete, report BLOCKED. Do not attach the normal browser as a workaround.

### C1 — Dedicated browser

Use an already-present dedicated debug browser if suitable. If installation is required, only use a trusted device package source or operator-supplied official package. Do not fetch/sideload an arbitrary APK.

Keep the diagnostic browser empty and logged out. Record product family/version only; do not publish a full UA/fingerprint string.

### C2 — Local-only CDP bridge

Establish an on-demand transport. It must not listen on public/Tailscale/`0.0.0.0`. Do not leave permanently enabled TCP adbd/CDP. Document deterministic teardown.

### C3/C4 — Minimal MCP implementation

Create the smallest repository-owned implementation satisfying the Task Contract. Prefer a strict allowlist over a generic CDP proxy.

If repository files change:

- create a dedicated short-lived Worker branch;
- keep changes inside the #116 scope;
- add focused tests for allowed fields, host allowlist, bounds/redaction, and no arbitrary CDP primitive;
- commit a Candidate and open a reviewable PR according to repository protocol;
- do not merge it yourself.

### C5 — Neutral proof

Use only a neutral non-sensitive HTTPS page and a configured host allowlist. Prove health, target isolation, navigation, bounded network summary, teardown, and safe-output checks.

Do not use Bilibili as the neutral proof.

### C6 — Cleanup

Stop forwards/listeners and remove temporary proof state. It is acceptable to leave the dedicated diagnostic browser installed and empty if that is explicitly the reusable prerequisite for #117. Never leave the normal browser attached.

## Report

On success, write `[EXECUTION REPORT]` with:

- Worker/environment;
- Candidate/branch/PR if any;
- dedicated-browser isolation result;
- CDP transport class and local-only check;
- MCP tool surface actually implemented;
- neutral proof status class and **field names/bounded enum values only**;
- tests/redaction/cleanup results;
- exact limitations/operator requirements;
- statement that Bilibili/#67/#113/#117/#68 were not executed.

Then transition to `status:review`, release owner, and STOP.

On blocker, write `[BLOCKER REPORT]` with only sanitized missing capability / failed safe boundary, transition to `status:blocked`, release owner, and STOP.

Do not widen scope to solve a blocker.