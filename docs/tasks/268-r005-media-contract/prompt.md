# Session Bootstrap — Generic MediaShapeV1 contract and paired A/V projection

```text
GitHub Issue: #268
Task Contract: docs/tasks/268-r005-media-contract/task.md
Expected worker: cloud
Expected environment: env:cloud
Handoff profile: docs/tasks/handoffs/cloud.md
```

Read `AGENTS.md`, Issue #268 and all comments, this Task Contract, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, `docs/tasks/freshness-integration-protocol.md`, current `main@32498b845a147a9afa9a231070cfc9622886ccac`, accepted #265 evidence, and accepted #237/#240/#243/#248/#251/#255/#257/#259 evidence.

Before claiming, verify the Issue is `status:ready`, `env:cloud`, owner-free, and package files read back from GitHub. Implement only the generic versioned media shape and Bilibili plugin projection in this Task. Do not implement remux/FFmpeg or a browser player yet. No live Bilibili/login/tx-node/phone/TV/VNC/CDP, no #191/#195, and no local build/test/package/install; all required verification must run through GitHub Actions.

Preserve legacy muxed HTTP-file/HLS behavior, PlaybackSession authority, SourceLocator opacity, Vault/EgressPolicy boundaries and site-neutral Core/Browser Worker code. Post one `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, set `status:review` or `status:blocked`, release ownership and stop. Never claim real Bilibili playback or close the Issue.
