# Planning Template — #67 Attempt 3

Status: planning-only. This file is **not** the active Task Contract and does not authorize execution.

Use only after #79 `GENERIC-YTDLP-OFFLINE-RUNTIME-PREP` is Final Accepted.

## Trigger

#67 Attempts 1 and 2 both reached the accepted Ubuntu ARM64 target and proved direct public/Bilibili HTTP 200, but stopped before extraction at:

```text
process_error: FROZEN_RUNTIME_SETUP
broker_request_count: 0
```

#79 is intended to replace target-side cold source acquisition with an immutable verified offline runtime bundle.

## Coordinator fields to fill from #79 Final Acceptance

Do not guess any value. At #79 acceptance, copy exact durable identities:

```text
#79 Final Acceptance comment/reference:
#79 accepted Candidate SHA:
#79 merged main SHA:
offline artifact format:
offline artifact bounded identity/filename:
offline artifact SHA256:
manifest schema version:
yt-dlp version: 2026.08.19
source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
platform compatibility: linux-aarch64 | platform-neutral verified
ARM64 offline-consume Evidence:
Target provisioning/transfer method:
```

## Attempt 3 publication intent

After #79 acceptance, revise the active #67 Task Contract/Issue to freeze a new exact Candidate containing the accepted offline-runtime consumer path.

Target flow must become:

```text
accepted immutable bundle
→ transfer/provision to Ubuntu ARM64 target
→ verify manifest + SHA256
→ non-root offline install/reuse into gateway-runner user-owned cache
→ setup/package network unavailable
→ scripts/generic-ytdlp-real-smoke.sh
→ R008Broker
→ BrokerProcessRunner
→ frozen yt_dlp.extract_info(download=False)
→ safe result summary
```

## Attempt 3 success signal

The first key distinction from Attempts 1/2 is:

```text
frozen runtime provenance verification: PASS
runtime_cache: offline verified/hit
broker_request_count > 0
```

Only after broker traffic occurs may #67 classify actual Bilibili compatibility.

Result routing remains:

```text
PASS
→ muxed http-file/HLS accepted ResolvedMedia
→ Coordinator Final Acceptance
→ publish #68

FAIL: UNSUPPORTED_FORMAT / separate A/V
→ create generic media-format/DASH-remux capability Task
→ do not mutate #67

BLOCKED: R008/site/network/policy
→ preserve bounded code
→ evidence-driven repair/research

challenge/access behavior
→ compatibility research
→ no bypass
```

## Frozen boundaries

- verification-only; no code/product/security changes on the Target;
- no root/sudo/system package installation;
- no target-side git/source dependency resolution;
- no global/different yt-dlp fallback;
- formal Bilibili reachability remains direct/no-proxy;
- extractor traffic remains R008Broker-only;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy rotation/access-control bypass;
- no full source/resolved/signed URL, Cookie/Auth/token, raw worker stderr, page/media payload or artifact transfer credential in durable Evidence;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/performance scope;
- Worker reports and STOPs; Coordinator decides #68.

## Publication checklist

Before changing #67 from blocked/draft to ready:

1. #79 Final Accepted and merged.
2. Read back #79 artifact manifest/hash/platform Evidence.
3. Confirm target can receive the exact artifact without rebuilding it.
4. Update #67 active Task Contract and prompt from this template; do not treat this file as executable authority.
5. Freeze the exact new #67 Candidate.
6. Compare Candidate/current main and classify freshness.
7. Confirm #63 target identity remains applicable.
8. Publication Gate comment records all exact identities.
9. Publish only `status:ready + env:ubuntu-arm64 + no owner`.
10. Worker executes one Attempt and STOPs.
