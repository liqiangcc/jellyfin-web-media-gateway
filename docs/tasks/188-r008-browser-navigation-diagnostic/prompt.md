# Session Bootstrap — R008 browser navigation fast path

Routing: use gpt-5.6-luna with high reasoning. Fast is disabled by the user; do not enable or select Fast.

You are executing `liqiangcc/jellyfin-web-media-gateway` Issue #188.

## Start

Read Issue #188, its latest comments, and `docs/tasks/188-r008-browser-navigation-diagnostic/task.md`.

Confirm #188 is `status:ready`, `env:cloud`, and owner-free, then claim a new Attempt according to the repository lifecycle protocol.

## Existing accepted anchor

Reuse the already accepted implementation/artifact unless real target evidence requires a focused correction:

```text
Candidate: c8ff3f9e1b5f468e2b68d3f274da82eba0b9a76f
PR: #209
Hosted run: 34302593582
Artifact: 10085488784
Artifact size: 4,215,814 bytes
Digest: sha256:9a074b534a4067e307d1e0108dfa66f6f296e6baeaf52e096ab8b01ee6957cb6
Selector: bilibili:BV14V411W7r5:part-2
```

The implementation anchor includes accepted Issue #199 stage markers, Issue #203 post-navigation lifecycle classification, and Issue #207 finite navigation-rejection classification (PR #209 merged to `main` at `c3bfad69565423e4cae82b47143344e10bd89f19`). It records only finite navigation-promise, allowlisted rejection, page/browser lifecycle, process termination and finalizer markers; no target evidence is reused.

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
