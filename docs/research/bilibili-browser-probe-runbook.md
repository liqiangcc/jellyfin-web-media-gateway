# Bilibili browser probe runbook (#166 handoff)

This is the bounded live runbook for verification Task #166. It is only a
research procedure for clean anonymous source portability. It does not prove
Gateway playback, TV playback, ARM64 phone behavior or production `EgressPolicy`
integration. Do not run it in hosted CI and do not use a personal browser,
profile, VNC session or existing Gateway process.

## Admission gate

The Coordinator has accepted the live-capable implementation (#169), the
self-contained target runtime package (#172), the transport diagnostic (#182)
and exact artifact delivery (#185). Before any live request, the Worker must
verify the exact provenance below from GitHub and verify the manifest/digest of
the downloaded bundle. If any field differs, stop with `BLOCKED` and do not
enable live mode.

### Accepted #169 live probe

```text
Candidate SHA: eb6598d473768f544744431c5af792d6de86d59f
Merged main: f9a48dbef6535c4bb38188b050384ac4156187af
Workflow run: 34218848612
J1: 102036943379
J2: 102036943625
J3a: 102036943040
J3: 102037252460
J4: 102036943488
JI1: 102036943403
Bundle artifact: 10052918296
Bundle digest: sha256:c565934ea092f54c67e902c6e3615cb035e7f37c6ec78448e176c925b1ad3a37
Broker evidence: 10052975493 / sha256:75a51e8d05507c82f84670190d33b1f4ad5d0391323d5f78bc39a9ba8b6b8ae4
Sanitized evidence: 10052975884 / sha256:d92d8ec65bafbd91880b4610046ce0416a9a02155553ac4a38eaa6a2f7b66051
```

### Accepted #172 target runtime

```text
Candidate SHA: 989bacdbf7e0fcde053005a45a1ad41b219b8b46
Merged main: 3a2a5fe8900b0ecf728e12461420ca043bdd2702
Workflow run: 34223368033
J1: 102051511536
J2: 102051511433
J3a: 102051511227
J3: 102051889697
J4: 102051511455
JI1: 102051511612
Bundle artifact: 10054697137
Bundle zip digest: sha256:66cae21de2eecb6a7166a703f9679e961a4556a93050d0e5733627fe06dacbc1
Bundle archive SHA: a236f10085ea73c9c783b11335fb81bf979cba805d1f69064ad4f5aca15ed11d
J2 probe evidence: 10054718111 / sha256:f728f264946b3d6cc31133dacaa35415ae4016e124066b5da19a56778b61cd2d
J2 broker evidence: 10054717444 / sha256:0a5f088a140a1c2b5f864ea7029e0648bfcbaccaad8f495d829d8fb9da60260b
Runtime: playwright-core@1.55.0, regular-file package tree, no browser binary and no bin/ installer files
```

The #172 bundle is the accepted target-runtime provenance for the #169 probe.
The current live Attempt must use the exact #182 artifact delivered and
integrity-verified by #185 below. Do not run `npm install`, use a workspace
`node_modules` link, use an npm cache, or install a browser package on tx-node.
The target supplies only its external system Chrome.

### Current live Attempt artifact (#182 / #185)

```text
Candidate SHA: 7de63b231fc4582fc29ceb5a359050f4ffe22fcb
Actions workflow run: 34236219201
J1: 102094492167
J2: 102094492067
J3a: 102094491740
J3: 102095295856
J4: 102094492237
JI1: 102094492032
Bundle artifact: 10060036115
Bundle size: 4,201,531 bytes
Actions upload digest: sha256:53c57ce5c22219753ff8ad7f3fbea6c112958d88f84daa799bc03aa8e3f02259
```

This is the only artifact admitted for the live Attempt. #185 confirmed that
the same archive was copied to a fresh tx-node staging directory, verified for
size, SHA-256, ZIP integrity and manifest identity as `gateway-verify` UID/GID
1001, and then removed with no residue. That accepted staging copy is gone;
the Worker must retrieve and verify artifact `10060036115` again before a new
staging copy. Do not substitute a prior #172 artifact, another Candidate or an
unverified copy.

## Target admission

The accepted read-only preflight recorded in [Issue #166 comment](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/166#issuecomment-5583242133)
showed:

```text
Host: VM-0-11-ubuntu (tx-node)
OS/kernel: Ubuntu 26.04 / Linux 7.0.0-14-generic
Architecture: x86_64
User: gateway-verify / UID 1001 / GID 1001
Node/npm: v22.22.1 / 9.2.0
Chrome: /usr/bin/google-chrome-stable / Google Chrome 152.0.7977.82
Proxy variables: unset
```

Repeat these checks read-only immediately before the run. Execute as
`gateway-verify`, use the downloaded bundle's manifest-verified files, and let
the probe create a fresh mode-700 temporary profile. Do not attach to an
existing Chrome, CDP endpoint, source-runtime profile, Gateway process or VNC
session. Do not pass `--no-sandbox`, `--user-data-dir`, `--proxy-server`,
`--remote-debugging-port`, cookies, Authorization or arbitrary headers. The
probe's own fail-closed broker is the only egress path; explicitly remove
`HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` and lowercase variants from the
process environment.

## Fixed selector and command

The only admitted locator is the plugin-owned opaque selector below. The page
URL is constructed by the plugin; the caller supplies no URL or authority.

```text
bilibili:BV14V411W7r5:part-2
```

After downloading and verifying the artifact and changing into its clean root,
run as `gateway-verify` (the shell wrapping may use the target's existing
privilege handoff):

```sh
env -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY \
  -u http_proxy -u https_proxy -u all_proxy \
  -u NO_PROXY -u no_proxy \
  BILIBILI_PROBE_ALLOW_LIVE=1 \
  CHROME_PATH=/usr/bin/google-chrome-stable \
  CANDIDATE_SHA=7de63b231fc4582fc29ceb5a359050f4ffe22fcb \
  node experiments/bilibili-browser-probe/artifact.mjs verify .

env -u HTTP_PROXY -u HTTPS_PROXY -u ALL_PROXY \
  -u http_proxy -u https_proxy -u all_proxy \
  -u NO_PROXY -u no_proxy \
  BILIBILI_PROBE_ALLOW_LIVE=1 \
  CHROME_PATH=/usr/bin/google-chrome-stable \
  CANDIDATE_SHA=7de63b231fc4582fc29ceb5a359050f4ffe22fcb \
  node experiments/bilibili-browser-probe/probe.mjs \
    --mode live --selector bilibili:BV14V411W7r5:part-2
```

`artifact.mjs verify` must pass before `probe.mjs` starts. The verification and
probe commands must run from the bundle root so `playwright-core` resolves
inside `node_modules/playwright-core`. The Worker may use an equivalent
structured SSH/runuser wrapper that preserves these arguments and the same
low-privilege process; it must record the exact command shape without secrets.

## Diagnostic output (#176)

The #176 diagnostic mode uses the same live command and selector above. If the
browser or broker fails before a sanitized observation is produced, the probe
emits a bounded `diagnostic` object instead of the original error. Its `phase`
is one of `dns_address_policy`, `broker_connect`, `tls_handshake`,
`proxy_response`, `chromium_navigation`, `http_status`, or `unknown`; counters
are capped by the existing session budgets and `status_class` is coarse.

The diagnostic intentionally omits the error message and all URL, host/address,
certificate, header, body, profile, cookie, authorization, and candidate data.
Treat a phase result as an explanation of the blocked layer only. It does not
produce a media candidate or change #166's source-portability result.

## Transport preflight (#182)

The transport diagnostic adds schema version 2 fields `transport_stage` and
`transport_outcome`. The finite stages are `resolve_policy`, `tcp_connect`,
`tls_handshake`, `proxy_response`, `downstream_close`, and `unknown`; outcomes
are `success`, `failure`, and `unknown`. A generic `broker_connect` failure is
therefore retained as `unknown` until a bounded transport boundary is known.

The target-only preflight entry point is
`experiments/bilibili-browser-probe/transport-preflight.mjs`. It performs one
broker CONNECT to the Bilibili plugin-owned public authority and immediately
closes the socket. It accepts no URL, host, selector, proxy, profile, header,
or credential input, and it never opens a page or requests media. Hosted
Actions must build and manifest-verify the exact Candidate artifact before a
single `gateway-verify` tx-node preflight is allowed. An observed stage is
diagnostic evidence only; it does not authorize an egress relay or unlock
Issue #166.

The accepted #182 no-page preflight reached `transport_stage=proxy_response`
with `transport_outcome=success`, `status_class=2xx`, one request, zero
response/metadata bytes and no page navigation, media request, selector or
consumer activity. This is transport admission evidence only. It does not
prove that the page can be observed, that a source is portable, that the
independent consumer can read media, or that any playback path works; the live
Attempt must still produce page/source and post-browser consumer evidence.

Do not click play or trigger full-video preload. The probe observes bounded
response metadata, keeps short-lived candidate descriptors server-side, closes
the browser, and then performs only the independent bounded reads allowed by
the contract.

## Budgets and repetition

Use no more than two clean sessions, each with a fresh profile:

- navigation deadline: 120 seconds;
- observed requests: at most 200 per session;
- actual response bytes: at most 32 MiB per session;
- retained metadata: at most 1 MiB per session;
- independent consumer: at most 8 requests total, 1 MiB per request and 4 MiB total actual bytes, including error responses.

Redirects count as requests. Stop as soon as a limit is touched. Do not rotate
proxies, copy credentials, retry indefinitely or change the selector to
manufacture success.

## Sanitized evidence and result rules

Record only schema-safe fields: requested selector/part-match boolean,
candidate count, role, codec/container when known, status class, Range support,
safe header names, expiry category, request/byte counts, denial decisions,
consumer result and cleanup. Keep candidate URLs, query strings, signed
values, cookies, Authorization, raw DOM/playinfo/HAR and full bodies out of
logs, files, Issue comments and artifacts.

`PASS` requires C0 admission, correct part identity, and an independent
post-browser read. For AV-separated media both video and audio must be
independently readable. A muxed candidate must be independently readable as a
single source. `FAIL` records a bounded reproducible incompatibility such as
blob-only media, browser-only token, wrong part, expiry, 4xx/5xx, private
redirect or non-readable stream. `BLOCKED` is only for missing capability,
artifact/browser/provenance mismatch or a required design change. Unknown
codec/expiry fields remain `unknown`.

Every result must include Task/Claim/Attempt, exact #169/#172 provenance,
runbook/task base, Worker/orchestrator, execution plane/target, OS and version,
network path, command shape, budgets, actual counts and cleanup. Store evidence
only in the designated sanitized report. Clean the browser, broker, child
processes, profile, staging directory and any ephemeral candidate map in a
re-entrant finalizer.
