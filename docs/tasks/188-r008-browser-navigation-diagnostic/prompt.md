# Session Bootstrap — R008 browser navigation fast path

Routing: use gpt-5.6-luna with high reasoning. Fast is enabled by the user and may be selected.

You are executing `liqiangcc/jellyfin-web-media-gateway` Issue #188.

## Start

Read Issue #188, its latest comments, and `docs/tasks/188-r008-browser-navigation-diagnostic/task.md`.

Confirm #188 is `status:ready`, `env:cloud`, and owner-free, then claim a new Attempt according to the repository lifecycle protocol.

## Existing accepted anchor

Reuse the already accepted implementation/artifact unless real target evidence requires a focused correction:

```text
Candidate: 8246c9f79db64b36b00c0b9a938569b28c55bc29
PR: #213
Hosted run: 34305340293
Artifact: 10086463816
Artifact size: 4,215,888 bytes
Digest: sha256:3bed203c49a1d5edf815633e390147236cb814083ead242bd049077c87af1bde
Selector: bilibili:BV14V411W7r5:part-2
```

The implementation anchor includes accepted Issue #199 stage markers, Issue #203 post-navigation lifecycle classification, Issue #207 finite navigation-rejection classification, and Issue #211 transport/failure-precedence correction (PR #213 merged to `main` at `06dc9b63f05a8c4c9e316d277c627549a4119c03`). It records only finite navigation-promise, allowlisted rejection, page/browser lifecycle, process termination and finalizer markers; no target evidence is reused.

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
