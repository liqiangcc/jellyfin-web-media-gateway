# Planning Template — #67 Attempt 4

Status: planning-only. This file is **not** the active Task Contract and does not authorize execution.

Planning Base: `2ff12dede1942d975ba1b9db82d8f80b29e7c43a`.

Use only after #83 `GENERIC-YTDLP-SANDBOX-ARM64-PREP` is Final Accepted and merged.

## Trigger

#67 Attempt 3 successfully crossed the earlier runtime-preparation blocker:

```text
offline bundle transfer: PASS
repository trust-anchor verification: PASS
wheel SHA256 verification: PASS
frozen yt-dlp identity: PASS
runtime_cache: offline-prepared
direct public HTTPS: HTTP 200
direct frozen Bilibili page: HTTP 200
```

Attempt 3 then stopped immediately before extractor execution at:

```text
process_error: SANDBOX_UNAVAILABLE
broker_request_count: 0
```

The verified Target is Linux `aarch64`. Static read-back showed the frozen `ytdlp-sandbox` accepted only the x86_64 seccomp audit architecture. Therefore this remains a pre-broker runtime compatibility blocker, not a Bilibili compatibility result.

#83 owns the generic sandbox repair. #67 must not implement that repair itself.

## Already accepted durable identities

These do not need to be redesigned for Attempt 4 unless freshness proves an accepted semantic change:

```text
#79 offline-runtime accepted merge: 290268c3cabe5ac16022b1ae5e4fa7716ee5deae
offline wheel: yt_dlp-2026.8.19-py3-none-any.whl
wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
trust anchor: scripts/generic-ytdlp-offline-runtime.lock.json
manifest schema: 1
yt-dlp version: 2026.08.19
source commit: 3a08beaf031ab68f966401ead017ac81fe8486cf
Target: accepted #63 Ubuntu ARM64 phone / gateway-runner low-privilege class
Harness: scripts/generic-ytdlp-real-smoke.sh
Frozen selector: BV14V411W7r5
Formal site network class: direct / no proxy / no bypass
Extractor network authority: R008Broker + BrokerProcessRunner
```

Attempt 3 intentionally retained the verified final user-owned runtime cache for warm reuse. The transferred CI bundle and staging data were removed.

## Coordinator fields to fill from #83 Final Acceptance

Do not guess these values. Copy exact durable identities only after Coordinator acceptance:

```text
#83 Final Acceptance comment/reference:
#83 accepted Candidate SHA:
#83 PR:
#83 merged main SHA:
#83 x86_64 sandbox Evidence:
#83 hosted ARM64 sandbox Evidence:
#83 AArch64 audit-arch mapping result:
#83 socket/socketpair deny matrix result:
#83 inherited broker-fd usability result:
#83 unsupported-architecture fail-closed result:
#83 DisabledRunner / R008 regression result:
```

Then derive and freeze:

```text
#67 Attempt 4 exact Execution Candidate:
#67 Task Contract revision commit:
#67 prompt / Task Package head:
Candidate -> current-main freshness classification:
Target cache state before execution: offline-hit | cache-missing
```

The #83 implementation checkpoint/PR head is never sufficient authority by itself. #67 may publish only from #83 Final Acceptance + merged main.

## Attempt 4 publication intent

After #83 Final Acceptance, revise the active #67 Issue/task/prompt to `Attempt: 4` and freeze an exact Candidate containing the accepted ARM64 sandbox support plus all previously accepted extraction/offline-runtime/security authorities.

Preferred Target flow:

```text
exact accepted Candidate
→ verify accepted low-privilege ARM64 Target identity
→ verify repository offline-runtime trust anchor
→ reuse verified user-owned runtime cache (`offline-hit`)
→ if cache is absent, provision the exact accepted #79 bundle and perform the same locked offline install (`offline-prepared`)
→ direct/no-proxy public + frozen Bilibili reachability
→ scripts/generic-ytdlp-real-smoke.sh
→ ARM64 ytdlp-sandbox starts fail-closed
→ BrokerProcessRunner
→ R008Broker
→ frozen yt_dlp.extract_info(download=False)
→ safe compatibility result
→ cleanup / leak scan
```

No target-side source build, package-index resolver, global yt-dlp fallback, sandbox bypass, or security-policy weakening is permitted.

## Attempt 4 decisive signals

Attempt 4 must distinguish sandbox closure from actual site compatibility.

Pre-broker success requires:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
```

The first decisive extractor-progress signal is:

```text
broker_request_count > 0
```

Only after broker traffic occurs may #67 classify the frozen Bilibili sample as PASS / CONDITIONAL PASS / FAIL / a new bounded broker/site blocker.

If Attempt 4 again reports `broker_request_count: 0`, it is still not a Bilibili media-format result unless the harness produced another concrete pre-broker classification.

## Expected J0-J4 shape

### J0 — Exact Candidate / Target / cache identity

Record bounded Evidence only:

```text
exact Candidate SHA
Linux aarch64
uid 999 gateway-runner / non-root / no sudo / effective capabilities zero
Python/Rust bounded versions
runtime cache state: offline-hit | cache-missing
```

If the retained cache exists, verify its provenance against the exact Candidate repository lock before use.

If the cache is missing, only the accepted #79 exact bundle may be provisioned. Transfer is transport, not a trust root.

### J1 — Frozen runtime provenance

Require:

```text
trust anchor present: yes
expected wheel SHA256: 86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9
frozen yt-dlp version/source identity: PASS
runtime_cache: offline-hit | offline-prepared
```

No online dependency resolution on Target.

### J2 — Formal site reachability

Clear all proxy variables and use bounded direct checks.

Require separate classifications for:

```text
public HTTPS
direct frozen Bilibili page
```

Artifact/cache provisioning network is not site Evidence.

### J3 — Accepted real-site smoke

Run only:

```text
YTDLP_OFFLINE_BUNDLE=<verified-path-if-needed> \
  scripts/generic-ytdlp-real-smoke.sh \
  'https://www.bilibili.com/video/BV14V411W7r5/'
```

If warm cache reuse means no bundle path is required by the accepted post-#83 harness, use the exact accepted invocation shape documented by that Candidate; do not invent an ad-hoc extractor command.

Capture only bounded safe fields:

```text
result
plugin
runtime_cache
broker_status_class
broker_error_code
broker_request_count
protocol
stream_count
title_length
process_error
```

### J4 — Safety / cleanup

Require:

- no staging directory;
- no task-owned smoke/worker/sandbox/descendant process;
- no media payload file;
- verified final cache may remain;
- checkout remains exact/unmodified;
- no Vault/profile/Secret state touched;
- leak scan contains no full resolved/signed URL, Cookie/Auth/token, transfer credential, raw worker stderr, page body or media payload.

## Result routing

### PASS

All of the following must hold:

```text
ARM64 sandbox starts with accepted fail-closed semantics
runtime_cache: offline-hit | offline-prepared
direct Bilibili reachability: PASS
broker_request_count > 0
result: PASS
protocol: http-file | hls
stream_count >= 1
security/cleanup/leak boundaries: PASS
```

Then Coordinator may Final Accept #67 and proceed to the already-prepared #68 Publication Gate.

### CONDITIONAL PASS

Only when brokered extraction reaches a valid current `ResolvedMedia` and a bounded non-security condition still permits an explicit #68 path. Coordinator decides acceptance.

### FAIL — actual media-format incompatibility

If accepted runtime + ARM64 sandbox + R008 execute correctly and the normally reachable sample stabilizes at a current-contract incompatibility such as:

```text
UNSUPPORTED_FORMAT
separate audio/video only
```

then #67 has finally produced a real compatibility result.

Do not add DASH/remux/FFmpeg inside #67. Route to a generic media-format capability Task.

### BLOCKED — broker/site/security

Examples after sandbox closure:

```text
BROKER_RESPONSE_TOO_LARGE
other bounded R008 broker policy/limit code
normal direct site access changed
safe Evidence cannot be produced
```

Preserve the bounded code and split evidence-driven repair/research. Do not weaken R008 or use bypass behavior.

### PRE-BROKER BLOCKED

If `broker_request_count = 0`, preserve the concrete new pre-broker code and do not label it Bilibili incompatibility.

## Frozen boundaries

- verification-only on #67; no product/runtime/security implementation changes;
- no sandbox bypass or weakened seccomp/no_new_privs;
- no direct worker socket authority; extractor networking stays inherited broker capability only;
- no root/sudo/system package installation;
- no Target-side git/source/package-index dependency resolution;
- no global/different yt-dlp fallback;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy rotation/access-control bypass;
- formal Bilibili Evidence stays direct/no-proxy;
- no full source/resolved/signed URL or Secret/raw content in durable Evidence;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/production enablement/performance work;
- Worker reports and STOPs; Coordinator decides #68.

## Publication checklist

Before changing #67 from `status:blocked` to `status:ready`:

1. #83 is Final Accepted and merged.
2. Read back exact #83 Candidate, PR, merge SHA and x86_64 + hosted ARM64 Evidence.
3. Confirm #83 preserved `no_new_privs`, socket/socketpair denial, inherited broker IPC, unsupported-arch fail-closed, R008 and `DisabledRunner`.
4. Confirm #79 trust-lock/wheel identity remains accepted and unchanged, or explicitly freshness-review any accepted semantic change.
5. Confirm #63 low-privilege Ubuntu ARM64 Target authority remains applicable.
6. Inspect current Target cache state only as execution preflight; never assume a warm cache is present.
7. Update the **active** #67 `task.md`, `prompt.md` and Issue metadata to Attempt 4; this planning file remains non-executable.
8. Freeze one exact #67 Execution Candidate containing #83 accepted sandbox support.
9. Compare Candidate to current main and classify freshness.
10. Publication Gate records all exact identities and explicitly states that Attempt 3 already proved offline runtime + direct site reachability but did not reach broker traffic.
11. Publish only as `status:ready + env:ubuntu-arm64 + no owner`.
12. Worker executes one Attempt 4 and STOPs; never auto-start #68.

## Stop planning here

Do not use this template as justification to publish #68/#72/Auth/performance work early. The next evidence boundary is still #67 brokered extraction on the physical ARM64 Target.
