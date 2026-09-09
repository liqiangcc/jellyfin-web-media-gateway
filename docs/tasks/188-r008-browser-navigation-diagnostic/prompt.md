# Session Bootstrap — R008 browser navigation fast path

Routing: use gpt-5.6-luna with high reasoning. Fast is enabled by the user and may be selected.

You are executing `liqiangcc/jellyfin-web-media-gateway` Issue #188.

## Start

Read Issue #188, its latest comments, and `docs/tasks/188-r008-browser-navigation-diagnostic/task.md`.

Confirm #188 is `status:ready`, `env:cloud`, and owner-free, then claim a new Attempt according to the repository lifecycle protocol.

## Existing accepted anchor

Reuse the already accepted implementation/artifact unless real target evidence requires a focused correction:

```text
Candidate: 4ffcb63ef156d042571c39d0fc15a5b4e30a8b54
PR: #221
Hosted run: 34309034915
Artifact: 10087720720
Artifact size: 4,218,043 bytes
Digest: sha256:a5b9f07eb5ab7d81289539a90574d50a306353d80e4f37f7131e592ab555bcef
Selector: bilibili:BV14V411W7r5:part-2
```

The implementation anchor includes accepted Issue #199 stage markers, Issue #203 post-navigation lifecycle classification, Issue #207 finite navigation-rejection classification, Issue #211 transport/failure-precedence correction, Issue #215 upstream response/socket lifecycle tracing, and Issue #219 bounded marker retention (PR #221 merged to `main` at `46d3a67588f066c8b6a8b9de0e6edf5391aa9ae7`). It records only finite navigation-promise, allowlisted rejection, page/browser lifecycle, process termination, finalizer, upstream response and socket closure markers; no target evidence is reused.

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
