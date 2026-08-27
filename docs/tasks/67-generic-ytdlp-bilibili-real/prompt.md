# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #67 and all relevant comments, especially Attempt 5 authoritative `[BLOCKER REPORT]` and Coordinator `[SPLIT]` / dependency update
3. `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #95 Final Acceptance / PR #96 / ADR 0007
7. #85 Final Acceptance / PR #86
8. #83 Final Acceptance / PR #84
9. #79 Final Acceptance / PR #82
10. #63 Final Acceptance
11. #73 R2 Final Acceptance
12. #66 Final Acceptance
13. #60 / accepted R008 runtime-security authority
14. #90 accepted trusted exact-source transport Evidence when direct Target Git is unreliable

Claim only if live #67 is:

```text
status:ready
env:ubuntu-arm64
no active owner
```

## Frozen execution

```text
Contract Revision: R6
Attempt: 6
Exact Candidate: 804fd60343b081e5e055ba87f68e7939b106bb19
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh

Accepted #95 response Secret containment authority:
Task Candidate: 0738e1826b17400a92aff483cba4bd37f683e673
Merged main / exact #67 Candidate: 804fd60343b081e5e055ba87f68e7939b106bb19
workflow: 33061040363 PASS

Accepted #85 legacy-kernel authority:
Task Candidate: 9b874c3b4404a776da35fd37d37abe040fb06a2b
Merge: 76b2032410b19ee18cfb14f00317b97f84e3b691
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

## Why Attempt 6

Attempt 5 proved the real target path reaches R008:

```text
runtime_cache: offline-hit
formal Bilibili direct/no-proxy: 2xx
ARM64 sandbox: PASS
#85 ENOSYS fd isolation: PASS
BrokerProcessRunner: PASS
broker_request_count: 1 per run
R008: 4xx / BROKER_RESPONSE_SECRET_REJECTED
reproduction: 2/2
```

#95 was split to fix that independent response-boundary security/compatibility blocker. It is now Final Accepted.

Accepted #95 semantics must remain intact:

```text
request Secret material
→ REJECT before prohibited side effects

origin response Secret headers
→ remain Secret-classified
→ count against existing bounded header budget
→ CONTAIN before BrokerResponse / IPC
→ no cookie/auth store or replay
→ safe status/body/non-Secret headers may continue only when all other R008 checks pass
```

Do not reveal or infer the real origin response Secret header name/value in #67 Evidence.

## Goal

Resume the same verification-only real Bilibili extraction on the first exact Candidate containing #95:

```text
exact Candidate 804fd603...
→ exact #79 bundle / offline cache hit-or-prepare
→ direct/no-proxy Bilibili reachability
→ generic-ytdlp-real-smoke
→ accepted ARM64 ytdlp-sandbox
→ BrokerProcessRunner with #85 ENOSYS fallback
→ R008Broker with #95 response Secret containment
→ yt_dlp.extract_info(download=False)
→ safe PASS / CONDITIONAL PASS / FAIL / BLOCKED
```

Decisive progress is:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
process_error != SPAWN_FAILED
broker_request_count > 0
broker_error_code != BROKER_RESPONSE_SECRET_REJECTED
```

## Critical boundaries

- verification-only; do not modify repository/product/security policy;
- runtime checkout must be exact `804fd60343b081e5e055ba87f68e7939b106bb19`;
- accepted low-privilege Ubuntu ARM64 Target only; no root/sudo/system package install;
- if direct Target Git is unreliable, use only accepted trusted exact-source transport and verify SHA/tree locally; do not change Candidate;
- do not build/resolve yt-dlp from source or package index on Target;
- provision only accepted #79 runtime via authenticated CI-artifact transport or Coordinator/operator-provided exact local copy;
- remove/unset transfer credentials before extraction;
- verify with `python3 scripts/generic-ytdlp-offline-runtime.py verify <bundle>` before trusting it;
- never replace the locked wheel with another same-version wheel;
- formal Bilibili reachability must be direct/no-proxy and bounded;
- artifact/source transfer network is not site Evidence;
- extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority;
- ARM64 sandbox and #85 fd-isolation must remain enabled; never bypass/weaken seccomp/no_new_privs/socket/fd boundaries;
- #95 response Secret containment must remain exactly accepted: do not declassify Set-Cookie/auth/token response material, create cookie/auth state, replay response credentials, weaken header/body/redirect/TLS/SSRF bounds, or expose response Secret material;
- invoke only the accepted harness with `YTDLP_OFFLINE_BUNDLE=<verified path>`; no ad-hoc extractor/global fallback;
- preserve only bounded safe summary fields;
- never durable-log full resolved/signed URL, Cookie/Auth/token, response Secret header name/value, transfer credential, raw worker stderr, page body or media payload;
- verified final cache may persist; staging/process/media payload must not;
- if transfer, trust-anchor/hash, sandbox, `SPAWN_FAILED`, response containment, unsupported format, broker policy/limit or site access fails, classify and report; do not fix inside #67;
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
→ Coordinator plans only the smallest generic media-format capability required by Evidence

R008 / response containment / sandbox / SPAWN_FAILED / site / transfer / trust blocker
→ BLOCKED
→ preserve bounded Evidence

challenge/access behavior
→ compatibility research only
→ no bypass
```
