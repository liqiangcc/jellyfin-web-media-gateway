# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #67 and all relevant comments
3. `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #85 Final Acceptance / PR #86
7. #83 Final Acceptance / PR #84
8. #79 Final Acceptance / PR #82
9. #63 Final Acceptance
10. #73 R2 Final Acceptance
11. #66 Final Acceptance
12. #60 / R008 accepted runtime-security authority
13. #90 accepted trusted exact-source transport Evidence when direct Target Git is unreliable

Claim only if live #67 is:

```text
status:ready
env:ubuntu-arm64
no active owner
```

## Frozen execution

```text
Contract Revision: R5
Attempt: 5
Exact Candidate: 76b2032410b19ee18cfb14f00317b97f84e3b691
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh

Accepted #85 legacy-kernel authority:
Task Candidate: 9b874c3b4404a776da35fd37d37abe040fb06a2b
Merged main / exact #67 Candidate: 76b2032410b19ee18cfb14f00317b97f84e3b691
J4 Linux 4.19 ARM64: workflow 33045463590 / job 98428782004 PASS

Accepted #79 offline runtime:
wheel: yt_dlp-2026.8.19-py3-none-any.whl
Wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
Trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
yt-dlp: 2026.08.19 @ 3a08beaf031ab68f966401ead017ac81fe8486cf
```

Accepted CI transfer convenience:

```text
#79 workflow run: 32956386626
artifact name: generic-ytdlp-offline-runtime-3a3de8ee2f9ac8b0e1e312735a9305db7569baef
artifact id: 9602124791
```

Artifact/source-bundle transport is not site Evidence and is not a trust root. Verify exact Candidate SHA/tree and #79 repository lock + wheel SHA/provenance before execution.

## Goal

Clear the prior Linux 4.19 `SPAWN_FAILED` blocker with the accepted #85 implementation and reach real brokered extraction:

```text
exact Candidate 76b203...
→ exact #79 bundle / offline cache hit-or-prepare
→ direct/no-proxy Bilibili reachability
→ generic-ytdlp-real-smoke
→ accepted ARM64 ytdlp-sandbox
→ BrokerProcessRunner with #85 ENOSYS fallback
→ R008Broker
→ yt_dlp.extract_info(download=False)
→ safe PASS / CONDITIONAL PASS / FAIL / BLOCKED
```

Decisive progress is:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
broker_request_count > 0
```

## Critical boundaries

- verification-only; do not modify repository/product/security policy;
- runtime checkout must be exact `76b2032410b19ee18cfb14f00317b97f84e3b691`;
- accepted low-privilege Ubuntu ARM64 Target only; no root/sudo/system package install;
- if direct Target Git is unreliable, use only an accepted trusted exact-source transport and verify SHA/tree locally; do not change Candidate;
- do not build/resolve yt-dlp from source or package index on Target;
- provision only accepted #79 runtime via authenticated CI-artifact transport or Coordinator/operator-provided exact local copy;
- remove/unset transfer credentials before extraction;
- verify with `python3 scripts/generic-ytdlp-offline-runtime.py verify <bundle>` before trusting it;
- never replace the locked wheel with another same-version wheel;
- formal Bilibili reachability must be direct/no-proxy and bounded;
- artifact/source transfer network is not site Evidence;
- extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority;
- ARM64 sandbox and #85 fd-isolation must remain enabled; never bypass/weaken seccomp/no_new_privs/socket/fd boundaries;
- invoke only the accepted harness with `YTDLP_OFFLINE_BUNDLE=<verified path>`; no ad-hoc extractor/global fallback;
- preserve only bounded safe summary fields;
- never durable-log full resolved/signed URL, Cookie/Auth/token, transfer credential, raw worker stderr, page body or media payload;
- verified final cache may persist; staging/process/media payload must not;
- if transfer, trust-anchor/hash, sandbox, `SPAWN_FAILED`, unsupported format, broker policy/limit or site access fails, classify and report; do not fix inside #67;
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
→ Coordinator plans smallest generic media-format/DASH-remux capability

R008 / sandbox / SPAWN_FAILED / site / transfer / trust blocker
→ BLOCKED
→ preserve bounded Evidence

challenge/access behavior
→ compatibility research only
→ no bypass
```
