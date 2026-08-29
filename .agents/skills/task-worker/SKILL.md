---
name: task-worker
description: Claim and execute one published jellyfin-web-media-gateway Task Attempt, then report the result back to its GitHub Issue. Use only when explicitly asked to execute a ready Task; do not publish, review, accept, or close tasks.
---
# Task Worker

Execute exactly one published Task Attempt. Read `AGENTS.md`, live Issue/comments, Task Package, lifecycle/recovery/freshness protocols, and required canonical docs. Higher authority wins.

## Claim

Immediately before claim require open + `status:ready` + eligible environment/capabilities + no active owner + executable Task Package. Claim `status:in-progress`, record active owner, determine a new Attempt number, and read back. Do not execute on an unconfirmed claim.

## Execution

Follow `task.md` exactly. Persist coherent work using the execution-anchor protocol. Do not widen Scope, lower Success Criteria, or reinterpret targets. Freshness/integration behavior remains governed by repository protocols and Coordinator Review.

## Fresh terminal-write authority guard

Claim-time authority is not authority to write terminal state later. Immediately before the first irreversible Worker terminal mutation, freshly read the live Issue and apply `docs/tasks/task-worker-terminal-write-guard.md` (optionally using `scripts/task-worker-terminal-guard.py` for the pure normalized decision).

Proceed only if the Issue is open, `status:in-progress`, current Attempt and owner still match, `status:done` is absent, no durable `[FINAL ACCEPTANCE]` exists, and no newer Coordinator gate/Attempt supersedes this Worker.

On rejection or ambiguity: `STALE_AUTHORITY`; **zero terminal Issue mutations**; do not post a report, change status, release/reassign owner, or reopen; STOP. If authority becomes known stale after an earlier terminal mutation, do not continue later status/owner writes.

## Normal completion

Persist Candidate/Evidence; prepare `[EXECUTION REPORT]` without posting; perform the fresh terminal guard; if authorized post the report, transition to `status:review`, release owner only while authority remains current, read back durable state, STOP. Never set done/close or start the next Attempt.

## Blocked completion

Preserve safe state/anchors and required cleanup; prepare `[BLOCKER REPORT]` without posting; perform the same fresh terminal guard; if authorized post the report, transition to `status:blocked`, release owner only while authority remains current and policy permits, read back, STOP. Never lower security or Success Criteria to avoid BLOCKED.

## Coordinator boundary

Worker outcome is not acceptance. Coordinator alone reviews, Final Accepts, sets `status:done`, and closes. After REVISE, read the revision class and claim a new Attempt before further execution.

## User output

After authorized durable reporting summarize Issue, Attempt, outcome, state, Candidate identities, report posted, and next authority. On guard rejection report `STALE_AUTHORITY` and `Issue mutations: none`.