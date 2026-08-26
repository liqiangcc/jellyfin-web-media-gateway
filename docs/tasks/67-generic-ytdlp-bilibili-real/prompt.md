# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #67 and all relevant comments
3. `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #83 Final Acceptance / PR #84
7. #79 Final Acceptance / PR #82
8. #63 Final Acceptance
9. #73 R2 Final Acceptance
10. #66 Final Acceptance
11. #60 / R008 accepted runtime-security authority
12. `docs/product-roadmap.md`

Claim only if live #67 is:

```text
status:ready
env:ubuntu-arm64
no active owner
```

## Frozen execution

```text
Attempt: 4
Exact Candidate: c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh

Accepted ARM64 sandbox:
#83 Candidate: a26995dd96d4765185b5c7c428c19ad2b56ba854
#83 merged main: c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe
#83 ARM64 Evidence: workflow 32961265996 / J2 98154583068 PASS

Offline wheel: yt_dlp-2026.8.19-py3-none-any.whl
Wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
Trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
Frozen yt-dlp: 2026.08.19 @ 3a08beaf031ab68f966401ead017ac81fe8486cf
```

Accepted CI transfer convenience:

```text
#79 workflow run: 32956386626
artifact name: generic-ytdlp-offline-runtime-3a3de8ee2f9ac8b0e1e312735a9305db7569baef
artifact id: 9602124791
```

The artifact ID is transport only. Trust the bundle only after repository lock + wheel SHA + provenance verification.

Attempt 3 retained the verified final cache but removed the transferred bundle. The accepted harness requires `YTDLP_OFFLINE_BUNDLE` even for a warm cache hit, so provision the exact bundle again (or use an operator-provided exact local copy) and expect `runtime_cache: offline-hit` when the retained cache is valid.

## Goal

Clear the prior ARM64 sandbox blocker and reach real brokered extraction:

```text
exact #79 bundle
→ verify bundle / offline cache hit-or-prepare
→ direct/no-proxy Bilibili reachability
→ generic-ytdlp-real-smoke
→ accepted ARM64 ytdlp-sandbox
→ BrokerProcessRunner
→ R008Broker
→ yt_dlp.extract_info(download=False)
→ safe PASS / CONDITIONAL PASS / FAIL / BLOCKED
```

Decisive progress is:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
broker_request_count > 0
```

## Critical boundaries

- verification-only; do not modify repository/product/security policy;
- runtime checkout must be exact `c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe`;
- accepted low-privilege Ubuntu ARM64 Target only; no root/sudo/system package install;
- do not build/resolve yt-dlp from source or package index on Target;
- provision only the accepted #79 bundle via authenticated CI-artifact transport or Coordinator/operator-provided exact local copy;
- remove/unset artifact-transfer credentials before extraction;
- verify with `python3 scripts/generic-ytdlp-offline-runtime.py verify <bundle>` before trusting it;
- never replace the locked wheel with another same-version wheel;
- J2 formal Bilibili reachability must be direct/no-proxy and bounded;
- artifact-transfer network is not site Evidence;
- extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority;
- ARM64 sandbox must remain enabled; never bypass or weaken seccomp/no_new_privs/socket denial;
- invoke only the accepted harness with `YTDLP_OFFLINE_BUNDLE=<verified path>`; no ad-hoc extractor/global fallback;
- preserve only bounded safe summary fields;
- never durable-log full resolved/signed URL, Cookie/Auth/token, transfer credential, raw worker stderr, page body or media payload;
- verified final cache may persist; staging/process/media payload must not;
- if transfer, trust-anchor/hash, sandbox, unsupported format, broker policy/limit or site access fails, classify and report; do not fix inside #67;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/production enablement/performance work;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never auto-start #68, set done, close Issue or change security limits.

## Result routing

```text
PASS + broker_request_count > 0 + protocol http-file|hls
→ Coordinator review
→ possible #67 Final Acceptance
→ then #68 publication

UNSUPPORTED_FORMAT / separate A/V after real brokered extraction
→ FAIL
→ Coordinator plans generic media-format/DASH-remux capability

R008 / sandbox / site / transfer / trust blocker
→ BLOCKED
→ preserve bounded Evidence

challenge/access behavior
→ compatibility research only
→ no bypass
```
