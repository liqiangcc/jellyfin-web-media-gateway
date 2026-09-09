# Session Bootstrap — R008 browser navigation fast path

Routing: use gpt-5.6-luna with high reasoning. Fast is disabled by the user; do not enable or select Fast.

You are executing `liqiangcc/jellyfin-web-media-gateway` Issue #188.

## Start

Read Issue #188, its latest comments, and `docs/tasks/188-r008-browser-navigation-diagnostic/task.md`.

Confirm #188 is `status:ready`, `env:cloud`, and owner-free, then claim a new Attempt according to the repository lifecycle protocol.

## Existing accepted anchor

Reuse the already accepted implementation/artifact unless real target evidence requires a focused correction:

```text
Candidate: c95beda94c87f166a1dd5056efb1834bdb79cc10
PR: #201
Hosted run: 34297453241
Artifact: 10083700408
Artifact size: 4,212,729 bytes
Digest: sha256:e0f289cff7852aad60300863d0a7c86f8840d6291a5511e9e70b85095894fbb2
Selector: bilibili:BV14V411W7r5:part-2
```

The implementation anchor includes accepted Issue #199 stage-marker follow-up (PR #201 merged to `main` at `97b27f34dda868e352782fea1e6f9a1aa28be91a`).

## Target rule

Do not require tx-node to be globally clean. The existing owner-controlled `source-runtime` may remain active and is not a blocker.

Start a disposable browser runtime alongside it using only minimal separation:

- fresh temporary Chrome profile;
- separate temporary display/Xvfb if needed;
- separate non-conflicting local CDP/debugging endpoint if needed;
- cleanup only processes/files created by this Attempt.

Do not wait for #191 or #195. Do not create a dedicated UID/GID, cgroup, network namespace, slot descriptor, or general browser-slot platform unless a concrete target failure proves one is necessary.

Use the existing Tailscale SSH control path. Run the probe/browser as `gateway-verify` when practical. If a temporary port/display collides, choose another and continue instead of blocking.

Run one navigation session first; use a second only if useful. A concrete navigation failure class is valid diagnostic evidence, not a blocker.

On completion post `[EXECUTION REPORT]`, move #188 to `status:review`, release ownership, and stop. Use `[BLOCKER REPORT]` only if target access/artifact execution or starting the disposable runtime is genuinely impossible.
