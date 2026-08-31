# Session Bootstrap — ENV-BILIBILI-REACHABILITY-OBS-PREP

Execute Issue #128 as a cloud PREP Worker.

## Start gate

1. Read live Issue #128, this `task.md`, `AGENTS.md`, `.agents/skills/task-worker/SKILL.md`, `docs/tasks/issue-lifecycle-protocol.md`, and #113 Attempt 2/3 Coordinator Reviews.
2. Claim only if live #128 is OPEN + `status:ready + env:cloud + no owner`.
3. This is PREP-only: do **not** make a Bilibili/public-site HTTP request.

## Goal

Implement the smallest stdlib-only privacy-safe observation transform described by `task.md` so a future bounded reachability check can correlate status class with opaque same-run endpoint/address-family observations without changing request behavior.

## Non-negotiable boundaries

- helper performs no HTTP, DNS, socket, subprocess, curl or routing operation;
- no raw IP/address is emitted in normal output;
- normal runtime generates a fresh in-memory salt and never emits it;
- deterministic tests may inject a fixed salt only through imported testable functions, not via a production CLI option;
- no proxy/login/Cookie/Auth/UA/fingerprint/Referer/header/DNS-pinning/IPv4/IPv6 forcing/endpoint steering/bypass work;
- no product/media/browser/site/security runtime code;
- no #113/#67/#68 execution.

## Verification

Run the deterministic offline matrix in `task.md`, Python compile/static checks, targeted secret-pattern scanning when available, and a static check showing the helper does not import/use network/subprocess modules.

Before every terminal Issue mutation, follow the fresh terminal-write authority guard now in `task-worker/SKILL.md`.

Normal completion:
`prepare report → fresh guard → [EXECUTION REPORT] → fresh guard → status:review → fresh guard(expected=review) → release owner → STOP`.