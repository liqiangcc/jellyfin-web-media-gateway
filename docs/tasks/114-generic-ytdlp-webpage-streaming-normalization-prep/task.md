# #114 GENERIC-YTDLP-WEBPAGE-STREAMING-NORMALIZATION-PREP

## Identity

- Issue: #114
- Task ID: `GENERIC-YTDLP-WEBPAGE-STREAMING-NORMALIZATION-PREP`
- Kind: implementation + deterministic compatibility/security verification
- Parent: #67 R16 Attempt 16
- Planning Base: `d3103e9679ad1f11ba51048a1354f04832555cf3`
- Preferred Worker: `cloud-codex`
- Environment: `env:cloud`

## Trigger

#67 R16 executed exact Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306` on the accepted low-privilege ARM64 target and returned:

```text
J0 PASS
J1 PASS
J2 PASS
broker_status_class: 2xx
broker_request_count: 4
process_error: UNSUPPORTED_FORMAT
unsupported_stage: FALLBACK_WEBPAGE
fallback_reason: RESPONSE_BODY_TOO_LARGE
protocol: n/a
stream_count: 0
J4 PASS
Overall: FAIL
```

This is a compatibility FAIL, not an environment/security/runtime/site-reachability blocker.

## Source-first facts

At exact Candidate `942a0a18...`:

```text
MAX_BODY = 96 * 1024
MAX_FALLBACK_TEXT_BYTES = MAX_BODY
```

The broker continues to reject raw bodies above `MAX_BODY`.

`_fallback_response_body()` reads at most `MAX_FALLBACK_TEXT_BYTES + 1` and maps both raw read overflow and post-normalization `_ResponseBodyTooLarge` to `RESPONSE_BODY_TOO_LARGE`.

#111 added bounded `identity | gzip | deflate` normalization and strict UTF-8 decoding. Post-normalization text is still capped at the same 96 KiB.

For `FALLBACK_WEBPAGE`, the normalized webpage is consumed only to preserve the existing decisions:

```text
contains <html
contains __initial_state__
contains bangumi
```

After those decisions, the full webpage text is not consumed by later NAV/VIEW/DETAIL/PLAYURL phases.

## Goal

Implement a webpage-only bounded streaming normalization/scan path:

```text
raw broker body <= 96 KiB
→ admitted content-coding normalization
→ strict incremental UTF-8 decode
→ bounded streaming marker scan
→ existing HTML / initial-state / Bangumi decisions
→ existing fallback state machine
```

Do not widen raw R008/broker authority and do not widen all JSON fallback bodies.

## Required design constraints

1. Raw authority unchanged
   - `MAX_BODY` unchanged.
   - broker/R008 response/frame limits unchanged.
   - no new direct network path.

2. JSON authority unchanged
   - NAV/VIEW/DETAIL/PLAYURL `_fallback_json()` and its 96 KiB body authority remain unchanged.

3. Webpage-only scan
   - Do not require retention of the full normalized webpage.
   - Use incremental gzip/deflate/identity processing and incremental strict UTF-8 decoding.
   - Marker matching must remain correct when a marker crosses decompression/decode chunk boundaries.

4. Bounded expansion/work
   - Add one explicit fixed normalized webpage scan ceiling.
   - It must be `> 96 KiB` and `<= 1 MiB`.
   - The Worker must document the selected value and rationale in code/tests/Execution Report.
   - Crossing the ceiling fails closed as `RESPONSE_BODY_TOO_LARGE`.

5. Preserve #111 coding semantics
   - admitted content codings remain exactly `identity | gzip | deflate`;
   - charset remains UTF-8 only;
   - malformed/truncated/unknown/ambiguous/nested/trailing coding fails closed as `RESPONSE_ENCODING`;
   - expansion abuse fails closed.

6. Preserve reason semantics
   - `RESPONSE_READ`
   - `RESPONSE_ENCODING`
   - `RESPONSE_BODY_TOO_LARGE`
   - `WEBPAGE_NOT_HTML`
   - `WEBPAGE_BANGUMI`
   remain distinct and closed.

7. Preserve security/runtime authority
   - #79/#83/#85/#95/#97/#99/#101/#103/#105/#107/#109/#111;
   - R008/Secret/broker/sandbox/fd boundaries;
   - production `DisabledRunner`.

## Required deterministic Evidence

### E1 — oversized normalized webpage positive

At least one deterministic fixture per admitted compressed coding where:

```text
raw body <= 96 KiB
normalized webpage > 96 KiB
normalized webpage <= selected scan ceiling
```

and the scanner preserves the expected existing webpage admission result.

### E2 — chunk-boundary correctness

Deterministic tests split each marker across input/decode boundaries:

- `<html`
- `__initial_state__`
- `bangumi`

Expected result must match unsplit input.

### E3 — closed negative matrix

- malformed gzip/deflate
- truncated coding
- trailing bytes / concatenated members
- unknown or ambiguous coding
- unsupported charset
- normalized output above scan ceiling

must fail closed with the correct existing reason.

### E4 — JSON bound regression

A JSON fallback body exceeding the existing 96 KiB limit remains rejected; #114 must not change JSON limits.

### E5 — parent compatibility regressions

- #109 full positive fallback traversal reaches current muxed `http-file` ResolvedMedia.
- #109 stage/reason forged/negative matrix PASS.
- #111 encoding/normalization matrix PASS.

### E6 — platform/security regressions

Equivalent required suite on:

- hosted x86_64
- native hosted ARM64

plus existing R008/Secret/broker/sandbox/fd/DisabledRunner regressions and sentinel/leak scan.

## Prohibitions

- no public Bilibili/site request;
- no #67/#68 execution;
- no raw site body/header diagnostics;
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/access-control bypass;
- no DASH/remux/FFmpeg/media-shape work;
- no R008/broker raw limit increase;
- no global fallback JSON limit increase;
- Worker must not merge, close, or mark done.

## Claims

- C1: webpage-only streaming normalization exists and full normalized webpage retention is unnecessary.
- C2: raw broker/R008 and JSON fallback limits remain unchanged.
- C3: selected scan ceiling is explicit, bounded and tested.
- C4: marker detection is chunk-boundary correct.
- C5: #111 coding/UTF-8/fail-closed semantics are preserved.
- C6: reason taxonomy remains closed and accurate.
- C7: #109 positive traversal remains valid.
- C8: hosted x86_64 + native ARM64 + security regressions PASS.

## Lifecycle

```text
status:ready
→ Worker claim
→ implementation + deterministic tests
→ [EXECUTION REPORT] | [BLOCKER REPORT]
→ status:review | status:blocked
→ release owner
→ STOP
```

Coordinator alone reviews, merges, Final Accepts and routes #67 afterward.
