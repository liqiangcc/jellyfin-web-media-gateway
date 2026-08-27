# Session Bootstrap — BILIBILI-WEB-E2E

Execute Issue #68 using the repository Worker protocol **only after Coordinator Publication Gate PASS**.

## Read first

1. `AGENTS.md`
2. Issue #68 and relevant comments
3. `docs/tasks/68-bilibili-web-e2e/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. #67 Final Acceptance and accepted real Bilibili Evidence
6. #49 Final Acceptance
7. #44 / #45 / #47 accepted product authorities
8. #60 / #66 / #73 / #79 accepted generic-ytdlp runtime/security authorities as applicable
9. `docs/product-roadmap.md`

Do **not** claim while Issue #68 remains `status:draft`.

Claim only when live Issue #68 is explicitly published as:

```text
status:ready
env:cloud
no active owner
```

## Publication-frozen fields

Before claim, re-read the Issue/Task Contract and verify Coordinator has filled/frozen:

```text
exact Planning/Evidence Base
exact Execution Candidate
#67 Final Acceptance reference
frozen selector: BV14V411W7r5
accepted source protocol: http-file | hls
accepted first-playback stream shape
real-source Evidence routing/environment
```

If any required field remains unresolved, or #67 is not Final Accepted PASS, do not claim; report the publication inconsistency.

## Goal

Prove the first real Bilibili Web playback closure through the accepted product path:

```text
/control
→ frozen public Bilibili URL
→ SiteAdapterRegistry / generic-ytdlp
→ accepted #67 ResolvedMedia shape
→ SourceSession
→ Gateway same-origin media
→ /display?profile=tv <video>
→ play / pause / seek / stop
→ refresh / reconnect
```

## Critical boundaries

- use product/public routes and accepted service APIs only;
- no direct `ResolvedMedia` injection, session-store mutation, proof-only seed path or ad-hoc yt-dlp CLI/Python substitute;
- reproduce only the exact media protocol/shape accepted by #67;
- R008 remains extractor/upstream network authority;
- browser receives Gateway-safe same-origin media paths, not arbitrary upstream Secret authority;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy bypass;
- no full resolved/signed URL, Cookie/Auth/token, raw worker stderr, page/media payload or lease token in durable Evidence;
- no Bilibili navigation/#72, Auth, BrowserWorker/Native Panel, DASH/remux/FFmpeg, physical-TV certification, performance or production-enable scope;
- preserve #44/#45/#47/#49/R007 semantics;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never set `status:done`, close #68, start #72, or weaken security/runtime policy.
