# Session Bootstrap — Issue #146

Read the latest live package revision before claim: it now includes verified artifact/layout admission and fresh-consumer execution requirements. Older package links are historical.

Before following any older Issue entry, read current AGENTS.md §4.1 and the latest task.md linked from the live Issue. All builds require GitHub-hosted Actions; target/workspace compile paths in earlier package revisions are superseded.

GitHub Issue: https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/146
Task Contract: `docs/tasks/146-non-phone-execution-prep/task.md`
Expected Worker: Codex Cloud with capabilities required by task.md
Eligible environment after publication: `env:cloud`
Handoff profile: `docs/tasks/handoffs/cloud.md`

This file is navigation only. Read live Issue and all relevant comments, this package's task.md, AGENTS.md, referenced canonical docs, issue-lifecycle-protocol.md, task-worker-terminal-write-guard.md, execution-anchor-recovery-protocol.md and freshness-integration-protocol.md before claim.

Fetch the package from the ref linked by the live Issue if the local checkout is older. Confirm ready + eligible environment + required capabilities + no owner and re-read after claiming a new Attempt. Reuse durable previous Candidate/PR when applicable. Never infer current authority from old phone prompts or this chat.

Use `.agents/skills/task-worker/SKILL.md` for exactly one Attempt. Publish report, re-read terminal authority, transition to review/blocked, release owner, and STOP. Worker cannot merge/accept/close or automatically start another Task. Full-route coordination is documented separately in `docs/tasks/handoffs/non-phone-delivery.md`.
