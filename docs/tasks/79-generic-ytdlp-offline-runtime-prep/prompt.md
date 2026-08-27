# Session Bootstrap — GENERIC-YTDLP-OFFLINE-RUNTIME-PREP

Execute Issue #79 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #79 and relevant comments
3. `docs/tasks/79-generic-ytdlp-offline-runtime-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. #67 latest BLOCKER REPORT
6. #73 R2 Final Acceptance / accepted PR #77
7. #60 / #66 accepted generic-ytdlp runtime-security authority
8. `docs/product-roadmap.md`

Claim only if live #79 remains:

```text
status:ready
env:cloud
no active owner
```

## Frozen task

```text
Planning Base: e8d292ebaa58e66f8ad737c9ddb643b9d8aacfaf
Frozen yt-dlp: 2026.08.19
Frozen source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
Downstream: #67 Attempt 3
```

## Goal

Build one repository-owned immutable offline runtime bundle with manifest + SHA256, then prove it can be consumed without setup/package network on GitHub-hosted Linux x86_64 and ARM64.

The normal Target path must become:

```text
receive exact bundle
→ verify manifest/hash/provenance
→ non-root offline install into user-owned cache
→ verify/reuse
→ later real extractor runtime via R008Broker
```

Target must not run `git` or network dependency resolution/build from source during normal runtime preparation.

## Critical boundaries

- no real Bilibili/site request;
- do not modify R008 DNS/public-IP/pinning/TLS/redirect/Secret/body-limit authority;
- production `GenericYtdlpAdapter::default()` remains `DisabledRunner`;
- no global/system yt-dlp fallback;
- no caller-selected source/version/ref/URL/executable/argv/proxy authority;
- exact frozen identity only;
- artifact metadata must not contain Secret/proxy/user-path/site data;
- prefer one platform-neutral wheel/bundle only if verified; otherwise use an explicit architecture matrix;
- GitHub-hosted x86_64 and ARM64 validation must consume the same canonical bundle identity where declared compatible;
- offline consume tests must make setup/package-index network unavailable;
- no root/sudo/system package installation;
- no #67/#68/#72/Browser/Auth/DASH/remux/performance execution;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never auto-start #67 Attempt 3, set done, close Issue, or weaken security policy.
