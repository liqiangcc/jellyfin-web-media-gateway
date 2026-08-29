# Task Contract — #117 ENV-ARM64-BROWSER-WORKER-DIFFERENTIAL-DIAG

## 1. Identity

- Issue: `#117`
- Task ID: `ENV-ARM64-BROWSER-WORKER-DIFFERENTIAL-DIAG`
- Kind: verification-only / Browser-vs-Worker differential diagnosis
- Preferred Worker: `ubuntu-arm64`
- Eligible environment: `env:ubuntu-arm64`
- Planning Base: `a1e684be1a7835a454f26dbac899e0efa5c487f6`
- Hard dependency: `#116 Final Acceptance`
- Parent Goal: `#67 GENERIC-YTDLP-BILIBILI-REAL`
- Historical reachability authority: `#113`
- Publication state: draft. Do not publish before #116 Final Acceptance and a fresh Publication Gate.

The Git commit containing this contract and its sibling `prompt.md` is the #117 Task Package authority once recorded by the Coordinator in Issue #117.

## 2. Source evidence and question

Accepted/observed inputs are intentionally separated:

### Machine Evidence

- #67 R17 / Attempt 17: exact runtime Candidate `80fb081b129f8f664124b84ddcc9698039e2cfd1`; J0/J1 PASS; public direct/no-proxy `2xx`; unchanged frozen Bilibili sample `4xx`; J3 NOT RUN.
- #113 R2 / Attempt 2: unchanged direct/no-proxy bounded probes `2xx → 4xx → 2xx`; no two consecutive 2xx; `BILIBILI_HOST_ELIGIBLE_FOR_#67_REFRESH=no`; BLOCKED.

### Operator observation

On the same phone, manually opening the video in the normal Android browser is fast.

This is a useful routing hint, but not machine Evidence. It is not permission to remote-debug the normal browser or copy its profile/cookies/cache/account state.

## 3. Hard dependency on #116

#117 may execute only after the Coordinator has Final Accepted #116 and the accepted capability proves:

- a **dedicated**, empty/non-account Android diagnostic browser exists;
- a local-only CDP transport is available;
- the minimal browser-diag MCP exposes only bounded network metadata;
- redaction/data-minimization tests passed;
- no personal/normal browser profile is exposed.

If any of those authorities are missing or stale, #117 is BLOCKED / not publishable.

## 4. Goal

Collect bounded near-simultaneous observations on the **same phone** that explain or classify the difference between:

1. dedicated Android diagnostic-browser navigation to the unchanged frozen public sample; and
2. the canonical #113 Worker direct/no-proxy request path.

The goal is classification, not obtaining 2xx.

The Worker must not change either path merely to produce a preferred status.

## 5. Core principle: observe, do not imitate

The browser and Worker naturally use different HTTP stacks and request identities. This Task does not inspect or clone browser headers to make Worker requests browser-like.

Allowed:

- observe protocol/IP-family/peer/redirect/cache/TLS/status metadata;
- observe whether browser and Worker results correlate with those bounded facts;
- report `UNKNOWN` when evidence does not support a narrower claim.

Forbidden:

- copy browser UA, Referer, headers, cookies, auth, local storage or fingerprint;
- force Worker/browser protocol/family/peer as an experiment;
- alter redirects or cache settings merely to prove a theory;
- increase sampling until a 2xx sequence appears.

Controlled causal tests, if ever justified, require a later independent Coordinator-authorized Task.

## 6. Frozen inputs

- Device: accepted phone used by #67/#113.
- Browser path: the **dedicated diagnostic browser accepted by #116**, never the user's normal browser.
- Worker path: the exact canonical direct/no-proxy reachability request shape frozen by #113.
- Frozen Bilibili selector/page: reuse the exact unchanged public sample authority from #67/#113. Do not substitute another video/page.
- Proxy policy: direct/no-proxy with proxy variables cleared for the Worker path, unchanged from #113.
- Browser: do not inject a proxy or custom request identity.

## 7. Diagnostic window — D0

Start one bounded diagnostic window.

Record only coarse, privacy-safe environment facts needed to interpret the pairings:

- diagnostic run identifier;
- coarse UTC time buckets;
- active Android network/interface class if available (for example Wi-Fi/cellular/unknown), without SSID/BSSID/account identifiers;
- current IP-family availability (`ipv4`, `ipv6`, `dual`, `unknown`);
- DNS answer family/count or bounded correlation tokens, not arbitrary resolver dumps;
- no route/DNS/network mutation.

Do not publish local/private device IPs unless already part of accepted infrastructure authority and strictly needed. Public remote-peer metadata should preferably be represented by bounded correlation tokens.

## 8. Paired observation design — D1/D2

Default maximum: **3 browser navigations + 3 Worker probes**, arranged as up to 3 near-simultaneous pairs.

A pair should occur within one small diagnostic window without changing device network configuration between the browser and Worker observation.

Stop early if enough Evidence exists for the Task's classification, but never exceed the maximum merely to wait for 2xx.

### D1 — Dedicated-browser observation

Using only the accepted #116 MCP:

- navigate the exact frozen public page;
- collect top-level bounded navigation metadata;
- no arbitrary CDP event dump;
- no response body/header capture;
- do not inspect page/account state.

Allowed fields per observation are limited to:

- status class;
- redirect count and bounded redirect status classes;
- HTTP protocol enum;
- remote IP family;
- bounded remote-peer correlation token if #116 supports it;
- connection reuse boolean/unknown;
- disk-cache boolean/unknown;
- service-worker boolean/unknown;
- TLS protocol enum/unknown;
- coarse duration bucket;
- diagnostic browser product family/version.

### D2 — Worker observation

Run the canonical #113 direct/no-proxy probe shape unchanged.

Additional write-out metadata is allowed only when collecting it does **not** alter the request:

- status class;
- HTTP protocol/version;
- remote IP family;
- bounded remote-peer correlation token;
- redirect count if the canonical request semantics already permit that measurement;
- coarse total-duration bucket.

Do not add browser-like headers, Cookie/Auth, new redirect behavior, retry policy, protocol forcing, IP-family forcing or proxy settings.

## 9. Correlation matrix — D3

Produce one bounded matrix with one row per pair. Example schema:

| Pair | Browser status | Worker status | Browser proto | Worker proto | Browser family | Worker family | Peer correlation | Browser cache/SW | Redirect correlation | Duration buckets |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |

Use bounded enums/tokens, not raw headers, bodies or query-bearing URLs.

The matrix is Evidence. Narrative speculation is not.

## 10. Allowed classifications — D4

Choose one primary classification and zero or more supporting correlations from this fixed vocabulary:

- `BROWSER_CLEAN_STABLE_WORKER_UNSTABLE`
- `BOTH_PATHS_UNSTABLE`
- `BOTH_PATHS_STABLE`
- `IP_FAMILY_CORRELATION`
- `CDN_PEER_CORRELATION`
- `HTTP_PROTOCOL_CORRELATION`
- `REDIRECT_PATH_CORRELATION`
- `BROWSER_CACHE_OR_SERVICE_WORKER_OBSERVED`
- `DEDICATED_BROWSER_ALSO_UNSTABLE`
- `NORMAL_BROWSER_STATE_HINT_ONLY`
- `NO_OBSERVED_PATH_CORRELATION`
- `UNKNOWN`

Definitions:

### `BROWSER_CLEAN_STABLE_WORKER_UNSTABLE`

The dedicated, logged-out browser is consistently successful within the bounded sample while the unchanged Worker path is not.

This still does not authorize browser-state cloning. It only proves a reproducible stack/path differential worthy of a later controlled diagnosis.

### `BOTH_PATHS_UNSTABLE`

Both dedicated browser and Worker fluctuate in the bounded sample. This supports external/site/edge/network instability more than a Worker-only explanation, but does not establish exact cause.

### Correlation classifications

`IP_FAMILY_CORRELATION`, `CDN_PEER_CORRELATION`, `HTTP_PROTOCOL_CORRELATION`, and `REDIRECT_PATH_CORRELATION` may be asserted only when the bounded matrix shows a repeatable association. They are correlations, not causal proofs.

### `BROWSER_CACHE_OR_SERVICE_WORKER_OBSERVED`

Use when a successful browser navigation is reported as served from disk cache and/or service worker. This is important because manual browser speed would then not directly prove equivalent live network reachability.

Do not disable cache/service worker in this Task to test causality.

### `NORMAL_BROWSER_STATE_HINT_ONLY`

Use only when the clean diagnostic browser does not reproduce the operator's normal-browser experience. The difference remains an operator-state/profile hint only. Do **not** attach to the normal browser to investigate it.

### `UNKNOWN`

Use when the bounded evidence is valid but insufficient for a narrower conclusion. `UNKNOWN` is an acceptable successful diagnostic classification; do not widen scope to avoid it.

## 11. Security / privacy boundaries

- Dedicated #116 diagnostic browser only; never normal/personal browser profile/tabs.
- No Cookie/Auth/login/profile/password/local-storage/session-storage extraction or replay.
- No response/page body, arbitrary headers, raw CDP stream, full signed/query URL, token or media payload retained or published.
- No proxy/rotation, custom browser impersonation, UA/Referer/header spoofing, fingerprint emulation, CAPTCHA/challenge or access-control bypass.
- No network-path forcing merely to obtain 2xx.
- No yt-dlp/generic-ytdlp/R008/broker/sandbox resolver execution; no #67 J3 and no #68.
- Do not modify #116 tooling during this verification Task. A defect in #116 is a blocker/child-task candidate, not in-scope implementation.

## 12. Success criteria

PASS requires:

1. #116 Final Accepted capability is used without widening it;
2. bounded paired observations are collected on the same phone under unchanged request policies;
3. at most 3 browser + 3 Worker observations are used;
4. a privacy-safe correlation matrix is produced;
5. one primary allowed classification is selected, including `UNKNOWN` if appropriate;
6. no prohibited content/state is retained or published;
7. cleanup/safe-output checks pass;
8. Evidence is sufficient for Coordinator to decide the smallest next action.

PASS does **not** mean:

- Bilibili is now reliably reachable;
- Worker must be changed to imitate a browser;
- #114 is proven or disproven;
- #67 may automatically run;
- #68 is ready.

BLOCK if #116 is unavailable/broken, if the dedicated browser cannot safely navigate, or if the paired measurements cannot be completed without violating boundaries.

## 13. Coordinator decision map after #117

The Worker does not create child Tasks, but the report should make the following Coordinator choices possible:

```text
clean browser stable + Worker unstable
→ potential controlled transport/path child, source-first

both paths unstable
→ external/site/edge instability; no product repair

cache/service-worker observed
→ manual-browser-speed evidence is not equivalent to live reachability

normal-browser observation differs from clean browser
→ browser-state hint only; do not inspect personal profile by default

specific peer/family/protocol/redirect correlation
→ possible smallest independent controlled-test child

UNKNOWN
→ preserve blocker; do not guess
```

No automatic #67 publication follows from #117 alone. A fresh reachability gate is still a Coordinator decision.

## 14. Worker lifecycle

Normal path:

```text
live read-back
→ verify #116 Final Acceptance
→ claim
→ status:in-progress + owner
→ [EXECUTION CHECKPOINT]
→ D0-D4 bounded paired diagnosis
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Blocked path:

```text
claim
→ safe bounded blocker evidence
→ [BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must never modify the repository, merge/close/mark done, publish another Task, execute #67/#113/#68, or inspect the normal browser.
