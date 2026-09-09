# Session Bootstrap — R005 Browser Observation Bridge

GitHub Issue: #240
Task Contract: `docs/tasks/240-browser-observation-bridge/task.md`
Expected worker: cloud Codex (`env:cloud`), `gpt-5.6-luna`, reasoning high, Fast enabled

Before claiming, read `AGENTS.md`, Issue #240 and comments, this task contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, and the canonical Browser Worker, security, plugin and playback documents it references. Confirm `status:ready + env:cloud + no active owner`.

Implement only the generic Browser Worker observation bridge. Keep Bilibili semantics in `plugins/bilibili`; never perform live site/login/tx-node/phone/TV/VNC/CDP actions, never touch #191/#195, and never run local build/test/package/install. Use GitHub Actions on the exact Candidate SHA for all verification. Follow the lifecycle protocol and stop after reporting the Attempt.
