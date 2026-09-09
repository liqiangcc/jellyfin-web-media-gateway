# Session Bootstrap — R008 browser navigation fast path

You are executing `liqiangcc/jellyfin-web-media-gateway` Issue #188.

## Start

Read Issue #188, its latest comments, and `docs/tasks/188-r008-browser-navigation-diagnostic/task.md`.

Confirm #188 is `status:ready`, `env:cloud`, and owner-free, then claim a new Attempt according to the repository lifecycle protocol.

## Existing accepted anchor

Reuse the already accepted implementation/artifact unless real target evidence requires a focused correction:

```text
Candidate: 77712ba7acdfd4083bc8e014db20a0a30d070e8c
PR: #190
Hosted run: 34250631237
Artifact: 10065870674
Digest: sha256:8a290032265a03553d654a20379b451b9ed07d013ec2c7dc76a33e229ab84a39
Selector: bilibili:BV14V411W7r5:part-2
```

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
