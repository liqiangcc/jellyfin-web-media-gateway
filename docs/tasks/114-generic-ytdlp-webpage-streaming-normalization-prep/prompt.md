# Worker prompt — Issue #114

You are the Cloud Worker for `GENERIC-YTDLP-WEBPAGE-STREAMING-NORMALIZATION-PREP`.

Read first:

1. GitHub live Issue #114.
2. `docs/tasks/114-generic-ytdlp-webpage-streaming-normalization-prep/task.md`.
3. `docs/tasks/handoffs/cloud.md`.
4. Exact trigger code at parent Candidate `942a0a1843f8f207332ac646f12ffe6ab5017306`, especially:
   - `plugins/generic-ytdlp/worker/worker.py`
   - generic-ytdlp normalization/fallback tests
   - #109 and #111 Task/Evidence docs.

Do not claim until live state is exactly `OPEN + status:ready + env:cloud + no owner` and the Task Package read-back matches Issue #114.

## Goal

Repair only the repository-owned webpage normalized-text overflow seam exposed by #67 R16:

```text
UNSUPPORTED_FORMAT
+ FALLBACK_WEBPAGE
+ RESPONSE_BODY_TOO_LARGE
```

Implement bounded streaming webpage normalization/scan without widening raw R008/broker authority or JSON fallback limits.

## Hard boundaries

- `MAX_BODY` raw broker/R008 ceiling stays unchanged.
- NAV/VIEW/DETAIL/PLAYURL JSON 96 KiB authority stays unchanged.
- admitted content-coding remains exactly identity/gzip/deflate.
- UTF-8 only.
- malformed/truncated/unknown/ambiguous/nested/trailing coding remains fail-closed.
- add one explicit normalized webpage scan ceiling `>96 KiB` and `<=1 MiB`; document the chosen value/rationale.
- do not retain an arbitrarily larger full normalized webpage.
- preserve existing HTML / `__initial_state__` / Bangumi semantics.
- marker matching must work across chunk boundaries.
- preserve `RESPONSE_READ`, `RESPONSE_ENCODING`, `RESPONSE_BODY_TOO_LARGE`, `WEBPAGE_NOT_HTML`, `WEBPAGE_BANGUMI` separation.
- preserve #79/#83/#85/#95/#97/#99/#101/#103/#105/#107/#109/#111, R008/Secret/broker/sandbox/fd and `DisabledRunner`.
- no real site, no #67, no #68.
- no Cookie/login/profile/fingerprint/CAPTCHA/proxy/bypass.
- no DASH/remux/FFmpeg.

## Required verification

Run deterministic fixtures/tests proving:

- compressed raw <=96 KiB with normalized webpage >96 KiB succeeds when within the selected scan ceiling;
- `<html`, `__initial_state__`, `bangumi` are detected correctly across chunk boundaries;
- malformed/truncated/trailing/unknown/ambiguous coding fails closed;
- scan-ceiling expansion abuse returns `RESPONSE_BODY_TOO_LARGE`;
- JSON body >96 KiB is still rejected;
- #109 positive full traversal still reaches current muxed http-file ResolvedMedia;
- #109 negative/forged taxonomy remains PASS;
- #111 normalization matrix remains PASS;
- hosted x86_64 + native hosted ARM64 and required security/runtime regressions PASS;
- leak/sentinel scan PASS.

## Delivery

Produce a bounded `[EXECUTION REPORT]` with:

- base/Candidate SHA;
- branch and PR;
- exact selected scan ceiling;
- C1-C8 result;
- hosted x86_64 / native ARM64 workflow run/job Evidence;
- files changed;
- tests run;
- explicit confirmation raw broker/R008 and JSON bounds did not change;
- limitations: no real-site claim.

Then set `status:review`, release owner, and STOP.

If blocked, post `[BLOCKER REPORT]`, set `status:blocked`, release owner, and STOP.

Do not merge/close/done the Issue and do not resume #67/#68 yourself.
