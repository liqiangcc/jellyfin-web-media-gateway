# Bilibili browser probe runbook (#166 handoff)

This is a bounded live runbook for the later #166 verification Task. #165 and
#169 ship the offline harness plus the explicit live-selector entry; do not run
this procedure in hosted CI and do not use it alone to claim Bilibili
compatibility.

## Admission

The Coordinator must freeze the exact accepted #165 Candidate SHA, GitHub
artifact ID/digest, workflow run, artifact manifest, low-privilege Linux user,
browser executable/version, and command before publishing #166. The run must
be on a disposable ordinary Linux process with a fresh temporary Chromium
profile. It must not reuse a personal profile, cookies, Authorization headers,
Vault data, an existing browser, or an existing Gateway service.

The run is permitted only when the offline containment Claim C2 is accepted.
If the artifact manifest, browser admission, or egress policy cannot be
verified, stop with `BLOCKED`; do not enable a live mode.

## Frozen budgets

Use at most two clean sessions. Each session has a 120 second navigation
deadline, at most 200 observed network requests, at most 32 MiB of actual
response bytes, and at most 1 MiB of retained metadata. Redirects count as
requests. Once a byte or request limit is touched, stop the session.

The independent server-side consumer may make at most eight requests total,
one MiB per request and four MiB total. Count actual bytes, including error
responses. Stop on the first over-budget response. Do not click play or cause
the site to preload the complete video; the experiment is about bounded source
acquisition.

## Procedure

1. Verify the downloaded artifact against its manifest and the Coordinator's
   exact artifact digest. Verify the candidate SHA and browser version before
   starting Chromium.
2. Start the downloaded probe with the selected public content selector and no
   caller supplied URL, headers, profile, proxy, or CDP endpoint. The Bilibili
   plugin constructs the page navigation descriptor; use its broker and
   central egress policy. The explicit entry is:

   ```text
   BILIBILI_PROBE_ALLOW_LIVE=1 node experiments/bilibili-browser-probe/probe.mjs \
     --mode live --selector bilibili:BV14V411W7r5:part-2
   ```

   Record only sanitized schema output. Any unknown option or URL-like
   selector must terminate before Chromium starts.
3. Confirm the selected part matches the requested opaque locator. Record
   candidate count, role, codec/container, status class, Range support, safe
   header names, expiry category, request/byte budgets, and denial decisions.
4. Close the browser and temporary profile. In a separate process, retrieve
   only the bounded media bytes using the server-side descriptor. A browser
   `blob:` URL, a browser-only token, a Cookie/Authorization requirement, a
   private redirect, or a 4xx/5xx response is a failed portability result.
5. Repeat once with a fresh profile if the first session is a clean, bounded
   run. Do not rotate proxies, copy cookies, retry indefinitely, or change the
   selector to manufacture success.
6. Store the sanitized evidence with Task/Claim/Attempt, exact Candidate SHA,
   workflow/job/artifact identity, execution plane, runner/target, browser and
   OS version, network path, actual budget use, and cleanup result. Never store
   raw DOM, playinfo, HAR, signed URLs, cookies, authorization values, or full
   response bodies.

## Result interpretation

`PASS` requires both clean sessions (or an explicitly documented single-run
acceptance by the Coordinator), correct part identity, an independent read
after browser exit, and no secret or boundary violation. `FAIL` records a
reproducible protocol/site incompatibility such as blob-only media, separate
streams that are not independently readable, wrong part, expiry, or denied
responses. `BLOCKED` is reserved for missing artifact/browser/egress capability
or a required design change. A negative run is useful evidence and must remain
append-only; it does not justify weakening the budgets or security boundary.
