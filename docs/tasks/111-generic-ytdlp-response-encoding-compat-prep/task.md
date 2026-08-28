# Task — GENERIC-YTDLP-RESPONSE-ENCODING-COMPAT-PREP

## Metadata

```text
GitHub Issue: #111
Task ID: GENERIC-YTDLP-RESPONSE-ENCODING-COMPAT-PREP
Task kind: implementation + deterministic compatibility/security verification
Parent Goal: #67 GENERIC-YTDLP-BILIBILI-REAL / R13 Attempt 13
Planning Base: 57d03a6875f957805c4dcb3dc09a139e65548fee
Runtime authority under repair: af65b2e2fec4cd3b3303db19415890f4052aa026
Preferred worker: cloud-codex
Eligible environment: env:cloud
Execution plane: GitHub Actions
Downstream: #67 next Contract Revision / real ARM64 Attempt
Freshness policy: dependency-aware
Publication state: non-executable until Coordinator Publication Gate
```

## Trigger Evidence

#67 R13 executed exact Candidate `af65b2e2fec4cd3b3303db19415890f4052aa026` on the accepted low-privilege Ubuntu ARM64 phone target. The accepted runtime/security path completed and returned:

```text
J0 PASS
J1 PASS
J2 PASS
J3 COMPLETED
runtime_cache: offline-hit
broker_status_class: 2xx
broker_error_code: n/a
broker_request_count: 4
process_error: UNSUPPORTED_FORMAT
unsupported_stage: FALLBACK_WEBPAGE
fallback_reason: RESPONSE_ENCODING
protocol: n/a
stream_count: 0
J4 PASS
Overall: FAIL
```

This is a valid compatibility FAIL, not an infrastructure/security/site-reachability blocker.

The exact real response body and headers were intentionally not retained, so this Evidence does **not** prove gzip, br, deflate, a particular charset, anti-bot content, DASH, separate A/V, or any other specific site content.

Source-first repository read-back of the accepted Candidate shows the repository-owned seam:

```text
R008 BrokerResponse body bytes
→ BrokerRH Response
→ _fallback_response_body()
→ bounded response.read(...)
→ body.decode("utf-8")
→ UnicodeDecodeError => RESPONSE_ENCODING
```

There is no existing deterministic compatibility layer between bounded response bytes and the strict UTF-8/JSON/HTML fallback admission.

## Goal

Repair only the bounded, repository-owned fallback response byte-to-text/content-coding compatibility seam exposed by R13.

The Candidate must define a **closed and bounded admitted response-encoding normalization contract** before the existing UTF-8 / JSON / HTML fallback admission, using deterministic offline fixtures only.

This Task is a concrete repair Task. Do not add another reason-only diagnostic layer.

## Core Contract

Preserve the existing top-level and stage/reason semantics:

```text
UNSUPPORTED_FORMAT
+ unsupported_stage
+ fallback_reason
```

`RESPONSE_ENCODING` remains the bounded fail-closed result when response bytes cannot be safely normalized into the repository's admitted UTF-8 text contract.

The implementation may normalize only explicitly admitted response encodings/content-codings proven by deterministic fixtures. Unknown, malformed, ambiguous, nested beyond the admitted bound, truncated, or expansion-abusive encodings must fail closed.

The Worker owns the exact implementation layer, but body bytes and response metadata must remain internally consistent after normalization. Do not silently reinterpret arbitrary bytes or delete evidence-significant metadata without a tested invariant.

## Required Bounds

1. Preserve the accepted fallback text limit: `96 KiB` maximum normalized text body.
2. Enforce the bound **after** any admitted decoding/decompression/normalization, so compressed or otherwise encoded input cannot expand past the accepted limit.
3. Preserve the existing broker/R008 body/frame bounds; do not enlarge them speculatively.
4. Bound decoder state, nesting/count of content-codings, and all intermediate allocation according to repository-owned constants.
5. No unbounded streaming, temporary media/page payload retention, or arbitrary codec/plugin loading.
6. Unsupported or malformed response encoding must map to an existing closed failure result, not arbitrary diagnostic text.

## Failure Semantics

Preserve these distinctions:

```text
response.read() failure        -> RESPONSE_READ
encoded bytes cannot normalize -> RESPONSE_ENCODING
normalized UTF-8 invalid JSON   -> RESPONSE_JSON
normalized text non-HTML        -> WEBPAGE_NOT_HTML
body exceeds admitted bound     -> RESPONSE_BODY_TOO_LARGE
```

Do not collapse these into one generic failure.

## Deterministic Fixture Matrix

Required offline fixtures must include at least:

1. identity UTF-8 webpage success;
2. identity UTF-8 JSON API success;
3. one fixture for every response content-coding/encoding admitted by the implementation;
4. malformed/truncated form for every admitted encoding;
5. unknown/unsupported encoding;
6. encoded payload whose normalized output exceeds 96 KiB;
7. encoded payload that normalizes within the bound but then fails JSON admission;
8. encoded payload that normalizes within the bound but then fails webpage HTML admission;
9. response-read failure remains `RESPONSE_READ`;
10. Secret-bearing response headers remain contained under #95 and do not become decoder inputs unless explicitly public/admitted by the existing boundary;
11. full deterministic #109 positive fallback traversal still reaches current muxed `http-file` ResolvedMedia;
12. existing #109 full stage→reason negative matrix remains PASS.

No public network is permitted in these fixtures.

## Security / Architecture Invariants

1. Preserve #79 frozen runtime provenance and bundle authority.
2. Preserve #83 ARM64 sandbox and #85 legacy-kernel fd isolation.
3. Preserve #95 response Secret containment and request Secret rejection.
4. Preserve #97 broker framing and existing R008 DNS/pinning/TLS/redirect/body/time/cancel authority.
5. Preserve #99 clean-build sibling binding.
6. Preserve #101 top-level failure taxonomy.
7. Preserve #103 ResolvedMedia compatibility contract.
8. Preserve #105 normal-extract-first narrow fallback admission.
9. Preserve #107 closed `unsupported_stage` and #109 closed stage-scoped `fallback_reason` semantics.
10. Preserve production `DisabledRunner` and broker-only egress.
11. No raw response body/header/page diagnostics in durable output.
12. No Cookie/login/profile/fingerprint/CAPTCHA/proxy/access-control bypass.
13. No DASH/separate-A/V/remux/FFmpeg/transcoding/navigation/Browser/Web-E2E/performance work.
14. No public Bilibili or other real-site request.
15. No #67 or #68 execution in this Task.

## Claims

```text
C1: accepted #101/#107/#109 error semantics remain closed and unchanged
C2: bounded response byte-to-text normalization exists before UTF-8/JSON/HTML fallback admission
C3: only a closed explicitly admitted encoding/content-coding set is accepted
C4: normalized output is capped at the accepted 96 KiB fallback text bound
C5: malformed/truncated/unknown/expansion-abusive inputs fail closed
C6: RESPONSE_READ / RESPONSE_ENCODING / RESPONSE_JSON / WEBPAGE_NOT_HTML / RESPONSE_BODY_TOO_LARGE remain distinct
C7: identity response behavior remains unchanged
C8: #109 positive full fallback traversal still reaches current muxed http-file ResolvedMedia
C9: #109 stage→reason negative/forged matrices remain PASS
C10: #95/R008/broker/sandbox/fd/DisabledRunner boundaries remain intact
C11: hosted x86_64 and native hosted ARM64 behavior is equivalent
C12: no real-site request, #67/#68 execution, raw diagnostics or speculative media capability is introduced
```

## Job Matrix

### J1 — hosted x86_64 deterministic compatibility

Run unit/integration fixtures for identity + admitted encodings, malformed/truncated/unknown forms, normalized-size bounds, JSON/HTML continuation and full #109 positive/negative matrix.

### J2 — native hosted ARM64 deterministic compatibility

Run the same authoritative compatibility matrix on native ARM64. Emulation-only evidence is insufficient.

### J3 — security / fail-closed matrix

Prove Secret containment, request-policy rejection, broker-only egress, malformed/unknown encoding fail-closed behavior, expansion-bound enforcement, no raw diagnostic leakage, and production `DisabledRunner` preservation.

### J4 — regressions / workspace

Run the accepted #79/#83/#85/#95/#97/#99/#101/#103/#105/#107/#109 relevant regression suites and workspace checks on the exact Candidate.

## Success Criteria

PASS requires all C1-C12 plus J1-J4 PASS on one exact Candidate.

The Execution Report must include:

```text
Candidate
PR
Actions run(s)
final admitted response-encoding/content-coding allowlist
normalization layer / invariant
normalized output bound
identity fixture result
admitted-encoding fixture matrix
malformed/unknown/expansion matrix
full #109 positive traversal
#109 stage→reason regression matrix
J1/J2/J3/J4
C1-C12
x86_64 identity
native ARM64 identity
safe-output sentinel scan
freshness
Overall
#67 rerun readiness
```

Do not include raw response/header/body data, decoded site content, signed URLs, Secret material or media payload.

## Worker Lifecycle

```text
status:ready
→ claim / Attempt 1
→ status:in-progress
→ exact Candidate + Draft PR + J1-J4
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review | status:blocked
→ release owner
→ STOP
```

Worker must not merge, close, set `status:done`, execute #67/#68, or create another Task.
