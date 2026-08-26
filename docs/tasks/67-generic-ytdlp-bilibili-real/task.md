# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

```text
GitHub Issue: #67
Task ID: GENERIC-YTDLP-BILIBILI-REAL
Task kind: verification-only / real public network
Contract Revision: R3
Next Attempt: 3
Exact Execution Candidate: 290268c3cabe5ac16022b1ae5e4fa7716ee5deae
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Accepted extraction upstream: #66 Final Accepted
Accepted harness authority: #73 R2 Final Accepted
Accepted offline runtime authority: #79 Attempt 2 Final Accepted
Accepted target environment: #63 Final Accepted
Accepted security/runtime authority: #60 + R008
Downstream: #68 BILIBILI-WEB-E2E
Freshness policy: dependency-aware / exact Candidate
```

> #67 owns only real-site compatibility Evidence for the frozen public Bilibili sample. It does not implement fixes, weaken security, add DASH/remux, enable production generic-ytdlp, or start #68.

## Trigger / why Attempt 3

Attempts 1 and 2 both established normal direct public/Bilibili reachability, but stopped before extractor traffic:

```text
direct public HTTPS: HTTP 200
direct frozen Bilibili page: HTTP 200
result: BLOCKED
process_error: FROZEN_RUNTIME_SETUP
broker_request_count: 0
```

Those results did not establish Bilibili incompatibility. The blocker was Target-side frozen-runtime preparation.

#79 is now Final Accepted and replaces Target-side `pip git+https` / source acquisition with a repository-locked offline runtime bundle.

Accepted #79 identities:

```text
Accepted Candidate: 3a3de8ee2f9ac8b0e1e312735a9305db7569baef
Merged main / Execution Candidate for #67: 290268c3cabe5ac16022b1ae5e4fa7716ee5deae
Artifact: yt_dlp-2026.8.19-py3-none-any.whl
Wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
Trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
Manifest schema: 1
yt-dlp version: 2026.08.19
Source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
Platform: verified py3-none-any on hosted Linux x86_64 + ARM64
```

Accepted CI transfer convenience from #79:

```text
workflow run: 32956386626
artifact name: generic-ytdlp-offline-runtime-3a3de8ee2f9ac8b0e1e312735a9305db7569baef
artifact id: 9602124791
```

The Actions artifact ID/name is **not** the trust root. It is only one permitted transfer source. Durable trust is the exact repository Candidate + repository-owned lock + wheel SHA above.

## Frozen sample

```text
site: Bilibili
mode: public / no-login / non-DRM
selector: BV14V411W7r5
source: https://www.bilibili.com/video/BV14V411W7r5/
formal site network class: normal direct / no bypass proxy
```

The source URL is Task input. Durable Evidence must not publish full source/resolved/signed media URLs, query tokens, Cookie, Authorization, profile/account state, artifact-transfer credentials, raw worker stderr, page body, or media payload.

## Goal

Determine whether the accepted current generic-ytdlp path can resolve the frozen public Bilibili sample on the accepted Ubuntu ARM64 phone/network to the current first-playback muxed HTTP/HLS `ResolvedMedia` contract.

Required path:

```text
accepted immutable offline bundle
→ transfer/provision to Ubuntu ARM64 Target
→ repository lock + wheel SHA verification
→ non-root offline install/reuse in gateway-runner user cache
→ scripts/generic-ytdlp-real-smoke.sh
→ R008Broker
→ BrokerProcessRunner sandbox
→ frozen yt_dlp.extract_info(download=False)
→ GenericYtdlpAdapter
→ current ResolvedMedia
→ evidence-safe summary only
```

The first decisive Attempt-3 signal is:

```text
runtime_cache: offline-prepared | offline-hit
broker_request_count > 0
```

Only after broker traffic occurs may this Task classify actual Bilibili compatibility.

## Exact Candidate

Execute product/runtime code exactly at:

```text
290268c3cabe5ac16022b1ae5e4fa7716ee5deae
```

This merge contains accepted #66 extraction, #73 safe real-site harness/R008 separation, and #79 repository-locked offline-runtime consumer path.

Task/prompt documentation may be newer than the execution Candidate. Do not silently substitute moving `main` for runtime execution. If an accepted semantic change later touches generic-ytdlp/R008/SiteAdapter output authority before claim, return to Coordinator freshness review.

## Host / environment authority

Use the Final Accepted #63 Ubuntu ARM64 phone environment.

Re-read and verify:

- Linux ARM64/aarch64;
- low-privilege `gateway-runner` uid999, non-root/no-sudo/no-admin;
- Python 3.12, pip, git, curl available;
- user Rust toolchain available;
- direct/no-proxy public HTTPS and frozen Bilibili page previously HTTP 200;
- FFmpeg/Chromium/Node are not required for this extraction-only Task.

No root/sudo/system package installation is permitted.

## J0 — Exact identity + bundle provisioning

Record only bounded safe Evidence:

```text
UTC time
uname -m
uid / privilege class
exact checkout SHA
python3 version
cargo/rustc bounded versions
bundle transfer class
```

Requirements:

1. checkout equals `290268c3cabe5ac16022b1ae5e4fa7716ee5deae`;
2. execution user matches accepted low-privilege Target class;
3. obtain the exact #79 offline bundle without rebuilding it on Target;
4. permitted transfer shapes are:
   - authenticated download of the accepted #79 CI artifact from run `32956386626`, or
   - Coordinator/operator-provided local copy of that exact bundle;
5. transfer credentials/tokens are transport-only and must not enter durable Evidence;
6. after transfer, unset/remove any `GH_TOKEN`, `GITHUB_TOKEN` or equivalent transfer credential from the extraction command environment;
7. do not use Target-side git/source dependency resolution to create the wheel.

If no permitted method can place the exact bundle on Target, report BLOCKED as `OFFLINE_BUNDLE_TRANSFER` before extraction. Do not fall back to online source installation.

## J1 — Repository trust-anchor + offline runtime verification

Before any real-site extraction, verify the supplied bundle using the exact Candidate repository code:

```text
python3 scripts/generic-ytdlp-offline-runtime.py verify "$YTDLP_OFFLINE_BUNDLE"
```

Then exercise the offline install/reuse path, directly or through the smoke harness.

Required Evidence:

```text
trust anchor present: yes
expected wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
bundle verification: PASS
runtime provenance: yt-dlp 2026.08.19 / source commit class accepted
runtime cache: offline-prepared | offline-hit
```

Rules:

- Target install must use the supplied wheel only;
- package index/source network is not required for install;
- helper remains non-root/user-owned/atomic/fail-closed;
- no global/system yt-dlp fallback;
- lock/manifest/hash mismatch is BLOCKED and must not be repaired locally;
- no replacement wheel merely because it reports the same yt-dlp version.

## J2 — Direct/no-bypass site reachability

Independently re-confirm formal site network class before extraction:

- clear `HTTP_PROXY/HTTPS_PROXY/ALL_PROXY` and lowercase equivalents for these reachability checks;
- use `curl --noproxy '*'` with strict bounded timeouts;
- record only public HTTPS status class and frozen Bilibili page status/error class.

Rules:

- no Cookie/Authorization/login;
- no proxy rotation/residential proxy;
- no fingerprint spoofing;
- no CAPTCHA/challenge automation;
- no access-control bypass;
- artifact-transfer routing is not Bilibili/site Evidence.

If the sample is no longer normally reachable on the accepted direct route, report BLOCKED.

## J3 — Accepted real-site smoke

Run only the accepted repository harness with the verified offline bundle:

```text
YTDLP_OFFLINE_BUNDLE=<verified-bundle-path> \
  scripts/generic-ytdlp-real-smoke.sh 'https://www.bilibili.com/video/BV14V411W7r5/'
```

Do not replace it with ad-hoc Python/Rust/yt-dlp CLI code.

Network authority separation is now:

```text
Artifact transfer network
!=
Offline runtime install
!=
Formal Bilibili direct reachability
!=
Extractor network authority
```

- artifact transfer may use GitHub authentication before execution;
- runtime install uses the already-supplied locked wheel and no package/source resolver;
- formal site Evidence is J2 direct/no-proxy;
- extractor HTTP(S) remains R008Broker + BrokerProcessRunner authority;
- `BrokerProcessRunner` remains the only extractor network capability path.

Capture only safe harness fields when present:

```text
result
plugin
runtime_cache: offline-prepared | offline-hit
broker_status_class
broker_error_code
broker_request_count
protocol
stream_count
title_length
process_error
```

Never preserve transfer credentials, setup logs/raw stderr, or full source/resolved URLs.

## J4 — Post-run safety / cleanup

Verify:

- no cache staging directory remains;
- final verified user-owned cache may remain for warm reuse;
- no task-owned smoke/worker/sandbox/descendant process remains;
- no media payload/file was downloaded;
- repository checkout remains exact/unmodified;
- no production Vault/profile/Secret state was touched;
- safe-output leak scan contains no full resolved URL, signed query, Cookie/Authorization/token/account/profile/transfer credential data.

## Result semantics

### PASS

All must hold:

- exact Candidate and accepted #79 trust anchor are used;
- exact offline bundle verifies and runtime cache becomes `offline-prepared` or `offline-hit`;
- J2 direct/no-proxy frozen sample is normally reachable;
- harness reaches brokered extraction and returns `result: PASS`;
- `broker_request_count > 0`;
- protocol is `http-file` or `hls`;
- at least one current-contract accepted muxed stream is represented;
- no security/Secret/policy violation;
- J4 cleanup/leak boundary PASS.

PASS means the frozen public Bilibili source is compatible with the accepted generic-ytdlp first-playback resolution contract. It does not prove #68 browser playback.

### CONDITIONAL PASS

Only if brokered extraction reaches a valid current `ResolvedMedia` but a bounded non-security condition still permits an explicit #68 path. State the condition; Coordinator decides acceptance.

### FAIL

Use when site is normally reachable and accepted runtime executes correctly, but the sample cannot be represented by the current first-playback contract, for example stable `UNSUPPORTED_FORMAT` because only separate audio/video is usable.

Do not add DASH/remux/FFmpeg inside #67.

### BLOCKED

Examples:

- `OFFLINE_BUNDLE_TRANSFER` — exact accepted bundle cannot be provisioned;
- `FROZEN_RUNTIME_VERIFY` — lock/manifest/hash/provenance fails closed;
- direct site no longer normally reachable;
- R008 policy/limit prevents compatibility determination;
- safe Evidence cannot be produced.

If a bounded broker code such as `BROKER_RESPONSE_TOO_LARGE` is emitted, preserve only the safe code and request/status counts. Do not change R008 limits in this Task.

## Claims

```text
R1 — Exact accepted runtime
#79 accepted offline runtime identity is used from exact Candidate 290268c... and verifies against repository lock.

R2 — Target build independence
Target does not build/resolve frozen yt-dlp from source or package index; it consumes the exact locked bundle.

R3 — Normal-network public accessibility
Frozen Bilibili sample is reachable on accepted direct/no-bypass route independently of artifact transfer.

R4 — Brokered extraction integrity
Real extractor HTTP(S) remains under R008Broker/BrokerProcessRunner authority.

R5 — Current ResolvedMedia compatibility
Safe result establishes whether the sample maps to current muxed HTTP/HLS first-playback contract.

R6 — Secret/evidence boundary
No Secret, signed media URL, raw page/media payload, proxy/transfer credential or profile/account state enters durable Evidence.

R7 — Cleanup / target safety
No staging/process/media payload persists; verified final cache may persist; low-privilege target boundaries remain unchanged.
```

## Success criteria

1. J0-J4 execute or a concrete safe blocker is preserved.
2. Exact Candidate, repository trust anchor, wheel SHA, yt-dlp version and source identity are verified.
3. No Target-side source build/network dependency resolution occurs.
4. Direct/no-proxy Bilibili reachability is classified separately from artifact transfer.
5. Brokered extractor traffic demonstrably reaches R008 (`broker_request_count > 0`) unless a pre-broker blocker is explicitly classified.
6. Safe result is sufficient to classify PASS / CONDITIONAL PASS / FAIL / BLOCKED.
7. R1-R7 are explicitly reported.
8. No implementation/security-policy modification occurs.
9. Worker reports, releases ownership and STOPs; it does not execute #68.

## Evidence contract

`[EXECUTION REPORT]` or `[BLOCKER REPORT]` must include only bounded Evidence:

```text
Attempt / worker / environment
UTC time
Host class / arch / runtime uid privilege class
Exact Candidate SHA
Frozen selector: BV14V411W7r5
Offline bundle transfer class: accepted CI artifact | operator-provided exact copy | blocked
Repository trust anchor result
Accepted wheel SHA verification: pass/fail
Frozen yt-dlp version/source identity: pass/fail
runtime_cache: offline-prepared | offline-hit | blocked
formal site network class: direct/no-proxy | blocked
Direct public HTTPS status class
Direct Bilibili page status class
Harness result
protocol: http-file | hls | n/a
stream_count
safe title length/hash if emitted
broker_status_class
broker_error_code
broker_request_count
process_error
staging/process cleanup result
safe-output leak scan
Claims R1-R7
Overall: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Downstream #68 readiness: yes/no + reason
```

Never publish artifact-transfer credentials, full resolved media URLs, signed query parameters, Cookie, Authorization, tokens, profile/account state, setup logs, raw worker stderr, page body, or media payload.

## Freshness

Semantic authorities:

- exact accepted #79 merge `290268c3cabe5ac16022b1ae5e4fa7716ee5deae`;
- `scripts/generic-ytdlp-offline-runtime.py` + repository lock;
- `scripts/generic-ytdlp-real-smoke.sh`;
- `plugins/generic-ytdlp/**`;
- `gateway-egress/**` / R008;
- `site-adapter-api/**` only if an accepted change materially alters extraction output/conformance before claim.

#75 may proceed independently. Planning/doc changes are normally `UNRELATED`. Any later accepted semantic change in the authorities above requires Coordinator re-freeze before execution.

## Out of scope

- code changes/fixes;
- R008 policy/limit weakening;
- Cookie/login/profile/auth/access-control bypass;
- DASH/separate A/V composition/remux/FFmpeg;
- Bilibili navigation/multipart (#72);
- Browser Worker/Native Panel;
- Web Display/control E2E (#68);
- production generic-ytdlp enablement;
- performance/capacity/thermal/soak (#9).

## Completion protocol

```text
status:ready
→ claim / Attempt 3
→ status:in-progress
→ J0-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
```

Worker cannot set `status:done`, close #67, execute #68, or modify product/security policy.
