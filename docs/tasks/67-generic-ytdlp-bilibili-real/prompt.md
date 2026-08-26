# Session Bootstrap — GENERIC-YTDLP-BILIBILI-REAL

Execute Issue #67 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #67 and all relevant comments
3. `docs/tasks/67-generic-ytdlp-bilibili-real/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #79 Final Acceptance / PR #82
7. #63 Final Acceptance
8. #73 R2 Final Acceptance
9. #66 Final Acceptance
10. #60 / R008 accepted runtime-security authority
11. `docs/product-roadmap.md`

Claim only if live #67 is:

```text
status:ready
env:ubuntu-arm64
no active owner
```

## Frozen execution

```text
Attempt: 3
Exact Candidate: 290268c3cabe5ac16022b1ae5e4fa7716ee5deae
Target: accepted Ubuntu ARM64 phone / gateway-runner
Sample: BV14V411W7r5
Harness: scripts/generic-ytdlp-real-smoke.sh
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

The CI artifact identifier is not the trust root. Target must verify the supplied bundle against the exact Candidate's repository lock and wheel SHA.

## Goal

Reach the real extractor for the first time on the accepted ARM64 Target and classify the frozen public Bilibili sample:

```text
exact locked offline bundle
→ Target offline verify/install
→ direct/no-proxy Bilibili reachability
→ generic-ytdlp-real-smoke
→ R008Broker
→ BrokerProcessRunner
→ yt_dlp.extract_info(download=False)
→ safe PASS / CONDITIONAL PASS / FAIL / BLOCKED
```

A successful compatibility determination should show:

```text
runtime_cache: offline-prepared | offline-hit
broker_request_count > 0
```

## Critical boundaries

- verification-only; do not modify repository/product/security policy;
- runtime checkout must be exact `290268c3cabe5ac16022b1ae5e4fa7716ee5deae`;
- low-privilege accepted Target only; no root/sudo/system package install;
- do not build/resolve yt-dlp from source or package index on Target;
- provision only the accepted #79 bundle via an authenticated CI-artifact transfer or Coordinator/operator-provided exact local copy;
- after artifact transfer, remove/unset `GH_TOKEN`, `GITHUB_TOKEN` or equivalent transfer credentials before extraction;
- verify bundle with `python3 scripts/generic-ytdlp-offline-runtime.py verify <bundle>` before trusting it;
- never replace the locked wheel with another same-version wheel;
- J2 formal Bilibili reachability must be direct/no-proxy and bounded;
- artifact-transfer network is not Bilibili/site Evidence;
- extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority;
- invoke only the accepted harness using `YTDLP_OFFLINE_BUNDLE=<verified path>`; no ad-hoc yt-dlp CLI/Python/Rust substitute or global fallback;
- preserve only bounded safe summary fields;
- never durable-log full source/resolved/signed URL, Cookie/Auth/token, proxy/transfer credential, raw worker stderr, page body or media payload;
- verified user-owned cache may persist; staging/process/media payload must not;
- if transfer, trust-anchor/hash, unsupported format, broker policy/limit or site access fails, classify and report; do not fix inside this Task;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/production enablement/performance scope;
- normal completion: `[EXECUTION REPORT] -> status:review -> release owner -> STOP`;
- blocker: `[BLOCKER REPORT] -> status:blocked -> release owner -> STOP`;
- never auto-start #68, set done, close Issue or change security limits.

## Key separation

```text
Artifact transfer network
!=
Offline runtime install
!=
Formal Bilibili site Evidence
!=
Extractor network authority
```

Transfer may use GitHub authentication. Offline install consumes only the locked wheel. Formal site Evidence is direct/no-proxy. Extractor traffic must be brokered by R008.
