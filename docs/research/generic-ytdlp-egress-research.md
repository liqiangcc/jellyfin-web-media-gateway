# Generic yt-dlp Egress Research

Issue: #50 `GENERIC-YTDLP-EGRESS-RESEARCH`  
Attempt: 1  
Worker / environment: Web Worker / `env:web-gpt`  
Planning Base: `18dd2b60f21d98436341e056376b26730c392ab7`  
Execution Base (Publication/current main at claim): `6a6c724ae3e8ebafc733fcf1c5fcc1b031a32757`  
Candidate: commit containing this document; the exact immutable SHA is recorded in Issue #50 `[EXECUTION REPORT]`  
Research date: 2026-08-26  
Decision: **CONDITIONALLY SUPPORTED**

## 1. Executive decision

Production real-network `generic-ytdlp` is **not safe to enable from the current #46 CLI subprocess seam**. A normal yt-dlp process owns DNS, connection establishment, proxy selection, TLS handling, redirects, cookies, request handlers and optional child/runtime surfaces. Validating only the initial URL in Gateway therefore is not R008-equivalent.

A bounded architecture is nevertheless plausible without weakening R008, so the product decision is **CONDITIONALLY SUPPORTED**, not `SUPPORTED`:

1. replace the prospective production CLI runner with a **version-pinned dedicated Python worker** owned by the `generic-ytdlp` Site Plugin;
2. give that worker only an **inherited, per-attempt IPC capability** to a Gateway-owned HTTP(S) broker;
3. run the worker and every descendant in a sandbox that **cannot open direct AF_INET/AF_INET6 network connections**;
4. make the Gateway broker the only component that resolves DNS, applies `EgressPolicy`, pins the checked `SocketAddr` set, performs origin TLS, and revalidates every redirect hop;
5. run an anonymous, fixed policy profile: no user Cookie/browser profile/netrc/Authorization input, no caller proxy/config/executable/flags, no arbitrary plugin directories, no remote components, no external downloader, no file/FTP/WebSocket direct transport, and no certificate bypass;
6. retain #46 bounded process output/timeout/kill/reap behavior and extend it to descendants and broker I/O;
7. keep production `GenericYtdlpAdapter::default()` on `DisabledRunner` until an implementation/verification Task proves these conditions on an exact candidate.

The missing evidence is therefore executable enforcement evidence, not permission to relax R008. Until that proof exists, `generic-direct` + explicit Site Plugins + Browser/site-specific fallback remain the production path.

## 2. Authority and live repository seam

### 2.1 Accepted evidence consumed

- Issue #46 Final Acceptance: candidate `29a4c687b2fa7c751f0f97d6ed5809f06fa682e3`, PR #52, merged as `f53d6d4e87aca8d861c18a77a22db5b9a44e4d83`. It accepted deterministic process/parser PREP while explicitly leaving production real-network generic-ytdlp disabled.
- Issue #39 Final Acceptance: candidate `a8811cee5d2752a20537337a041556b0fc579305`, PR #42, merged as `ec6ac75b02658bf4300f9d6f72de16aafa7e6af1`. It established the shared conservative Secret-header classifier used by SiteAdapter and R008 boundaries.
- Issue #14 / R008 Final Acceptance: candidate `d655ad5f01db7bf39e820f3c4425ef48e0faf508`, PR #17, merged as `e8981ada4e9a51b2856cd9206d2e7546bada5eca`. It closed DNS-rebinding/TOCTOU by binding the actual connection to the validated address set and revalidating redirects hop by hop.

Primary repository selectors:

- https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/46
- https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/39
- https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/14
- `plugins/generic-ytdlp/src/lib.rs`
- `gateway-core/src/security.rs`
- `docs/security.md`
- `docs/research/r008-security-boundary.md`
- `docs/site-plugin-architecture.md`
- `docs/implementation-contracts.md`

### 2.2 Current generic-ytdlp seam

Current accepted flow is:

```text
SiteAdapterRegistry
  -> lower-priority generic-ytdlp adapter
  -> ProcessRequest { source_url: Url }
  -> ProcessRunner
  -> bounded machine-output parser
  -> ResolvedMedia
```

The important live facts are:

- production registration uses `GenericYtdlpAdapter::default()` with `DisabledRunner` and `runtime_enabled == false`;
- the fixed-argv `CommandProcessRunner` is `#[cfg(test)]`, not a production network executor;
- `ProcessRequest::new()` currently enforces HTTP(S) + a host, but by itself is not the future credential/egress authority;
- the parser accepts bounded HTTP(S) media URLs, rejects Secret-classified public headers, and rejects a subprocess-minted `upstream_access_ref`;
- overflow/timeout paths kill and reap the child, and Debug output redacts the source URL.

Therefore #46 proves a safe **process/parser seam**, not safe real-network egress.

### 2.3 Accepted R008 equivalence target

The current Gateway security path is:

```text
URL
  -> EgressPolicy validation
  -> DNS resolution
  -> reject forbidden/non-public address set
  -> ValidatedTarget { host, checked SocketAddr[] }
  -> reqwest client with redirects disabled + no proxy
  -> resolve_to_addrs(host, checked addresses)
  -> origin TLS/hostname verification
  -> response
  -> if redirect: resolve + validate + pin the next hop again
```

A design is not equivalent if Gateway checks a hostname and then yt-dlp independently re-resolves or follows a redirect outside that authority.

## 3. Current yt-dlp primary-source selectors

The audit uses the latest stable release available on the research date plus current master as a freshness check.

### Stable release selector

- version/tag: `2026.08.19`
- commit: `3a08beaf031ab68f966401ead017ac81fe8486cf`
- release commit timestamp: `2026-08-19T23:31:25Z`
- release: https://github.com/yt-dlp/yt-dlp/releases/tag/2026.08.19
- commit: https://github.com/yt-dlp/yt-dlp/commit/3a08beaf031ab68f966401ead017ac81fe8486cf

### Current-master freshness selector

- master at research time: `66f49765d5a46c7be1c2c414f245c71530c4a2fd`
- commit timestamp: `2026-08-25T16:30:27Z`
- commit: https://github.com/yt-dlp/yt-dlp/commit/66f49765d5a46c7be1c2c414f245c71530c4a2fd
- master was six commits ahead of the stable release during the read-back. The relevant networking files had only small deltas; current-master read-back still shows `YoutubeDL.urlopen() -> _request_director.send`, mutable `RequestDirector.add_handler()`, and `RequestsRH` with `allow_redirects=True`. The post-release networking touches do not remove the security gap identified here. Any later implementation must still re-audit its pinned yt-dlp version rather than treating this document as a forever-current upstream contract.

### Audited upstream source paths at tag `2026.08.19`

- `yt_dlp/YoutubeDL.py` — `urlopen`, `build_request_director`, `_request_director`: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/YoutubeDL.py
- `yt_dlp/networking/common.py` — `RequestDirector`, `RequestHandler`, handler registry: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/networking/common.py
- `yt_dlp/networking/_urllib.py` — HTTP/TLS/proxy/automatic redirect handler: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/networking/_urllib.py
- `yt_dlp/networking/_requests.py` — `RequestsRH`, proxy support, `allow_redirects=True`: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/networking/_requests.py
- `yt_dlp/networking/_websockets.py` — direct WS/WSS socket/TLS path: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/networking/_websockets.py
- `yt_dlp/options.py` — config/plugin/JS/proxy/file URL options: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/options.py
- `yt_dlp/__init__.py` — CLI option materialization, plugin loading, credentials/TLS/proxy/external downloader options: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/__init__.py
- `yt_dlp/plugins.py` — default plugin search locations and explicit warning that plugin API backwards compatibility is not guaranteed: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/plugins.py
- `yt_dlp/utils/_jsruntime.py` — Deno/Node/Bun/QuickJS executable discovery: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/utils/_jsruntime.py
- `yt_dlp/extractor/abematv.py` — an extractor can add a custom `RequestHandler` to the shared request director: https://github.com/yt-dlp/yt-dlp/blob/2026.08.19/yt_dlp/extractor/abematv.py

## 4. What current yt-dlp actually owns

### 4.1 Request routing

`YoutubeDL.urlopen()` converts URL-embedded basic-auth userinfo into an `Authorization` header, sanitizes the URL, cleans headers/proxies, then calls `_request_director.send(req)`. The default cached request director is built from **all registered request handlers**.

`RequestDirector.send()` chooses a supporting handler and calls its `send()`. A handler is documented as processing the request “from start to finish”. This is a useful interception seam, but it is an upstream Python-internal surface rather than a Gateway security boundary.

### 4.2 Redirects

The default transports do not expose redirects to Gateway:

- `UrllibRH` installs yt-dlp's `RedirectHandler`, which automatically follows 301/302/303/307/308;
- `RequestsRH._send()` calls `session.request(... allow_redirects=True ...)`.

Consequently an initial-URL check or a standard outbound proxy does not reproduce R008 per-hop revalidation.

### 4.3 DNS, proxy and TLS

The built-in handlers own socket creation/DNS and support direct or HTTP/SOCKS proxy paths. `YoutubeDL.build_request_director()` passes cookie jar, proxy configuration, source address, TLS verification controls, client certificate inputs, file-URL enablement and impersonation into handlers.

`RequestsRH` disables `requests` environment trust only because yt-dlp has already loaded proxy configuration itself. `UrllibRH` can handle HTTP, HTTPS, data and FTP, and may enable file URLs. `WebsocketsRH` directly establishes WS/WSS connections and can use SOCKS.

That surface is broader than the accepted `public_web` HTTP(S) R008 boundary.

### 4.4 Config, plugins, credentials and child/runtime surfaces

The stable CLI exposes or materializes:

- Cookie file / cookies-from-browser / netrc / usernames / passwords;
- `--no-check-certificate`, legacy TLS and client certificates;
- HTTP/HTTPS/SOCKS proxies and geo-verification proxy;
- custom headers;
- external downloader and external downloader args;
- default plugin-directory search and custom plugin directories;
- JS runtimes, with Deno enabled by CLI default;
- optional remote components;
- file URLs when explicitly enabled.

The CLI loads plugins from configured/default locations before constructing `YoutubeDL`. `plugins.py` explicitly says backwards compatibility is not guaranteed for the plugin-system API. JS runtime lookup can discover executables from configured paths, Python script directories, cwd/PATH.

The current #46 fixed argv does **not** make these upstream surfaces an R008 boundary. Production must not be enabled by merely moving `CommandProcessRunner` out of `#[cfg(test)]`.

## 5. Threat and data-flow model

### 5.1 Unsafe current/naive path

```text
untrusted source URL
  -> Gateway validates only initial URL
  -> yt-dlp CLI process
       -> local config/plugin discovery
       -> extractor
       -> RequestDirector
       -> urllib / requests / curl_cffi / websocket / proxy / child runtime
       -> subprocess DNS
       -> direct socket
       -> automatic redirects
       -> origin
```

Bypass classes:

- DNS rebinding/TOCTOU after Gateway validation;
- redirect to loopback/private/link-local/metadata/reserved target;
- proxy or SOCKS path outside Gateway policy;
- direct WebSocket/alternate transport;
- URL userinfo becoming `Authorization`;
- Cookie/profile/netrc/config/headers injecting credential authority;
- plugin or child executable adding a new network path;
- TLS verification weakening through yt-dlp options;
- descendant process surviving the parent lifecycle.

### 5.2 Conditionally acceptable path

```text
caller source URL
  -> generic-ytdlp Site Plugin validates request shape
  -> dedicated version-pinned Python worker
       [network namespace/seccomp: no AF_INET/AF_INET6]
       [no CLI config/plugin discovery]
       [fixed anonymous options]
       [only inherited per-attempt IPC capability]
       -> yt-dlp extraction request
       -> broker RequestHandler / adapter
       -> capability IPC
  -> Gateway-owned scoped HTTP(S) broker
       -> reject non-http(s), URL userinfo and forbidden request authority
       -> anonymous mode rejects worker-originated Secret-classified request headers
       -> Core/Vault owns any future credential injection
       -> EgressPolicy::PublicWeb
       -> DNS resolve + public-address classification
       -> checked SocketAddr[]
       -> pinned direct origin connection
       -> normal TLS hostname/certificate verification
       -> redirects disabled at transport
       -> each Location is normalized and fully revalidated/pinned before next hop
       -> bounded response back over IPC
  -> worker emits bounded machine metadata
  -> #46 parser
  -> ResolvedMedia
```

The sandbox is not the policy engine. It is the **escape-prevention backstop**: if an extractor, optional dependency, future upstream change or child runtime bypasses the intended request hook, direct network access fails closed instead of escaping R008.

## 6. Required future architecture boundary

This section freezes the only architecture that this research considers eligible for a later implementation Task.

### A. Dedicated worker, not generic CLI authority

- Pin an exact yt-dlp version/hash in the runtime image/package.
- Invoke a repository-owned Python worker module with fixed inputs; do not pass caller argv.
- Use the Python API directly so ordinary yt-dlp CLI config loading and `_real_main()` plugin-directory loading are outside the execution path.
- Start with a minimal fixed option set equivalent to metadata-only/simulated extraction.
- No caller-supplied executable, PATH, config file, plugin directory, proxy, cookies, profile, netrc, username/password, headers, cert options, extractor args, external downloader or postprocessor execution authority.

### B. Gateway broker is the network authority

The worker receives a capability-scoped IPC channel, not a general listening proxy endpoint.

The broker:

- accepts only structured HTTP(S) requests needed by the extraction attempt;
- never exposes CONNECT or a generic raw-tunnel API;
- uses only `EgressScope::PublicWeb` for generic-ytdlp;
- in the initial anonymous profile, rejects worker/extractor-supplied Secret-classified request headers such as `Authorization`, `Cookie`, proxy-auth/API/token families and Basic/Bearer values; future credential injection, if ever accepted, must be broker/Core capability-owned rather than worker-authored;
- performs DNS and connects only to the validated R008 address set;
- keeps automatic redirect disabled at the underlying client and revalidates every redirect target;
- performs end-to-end TLS to the origin with hostname/certificate verification; there is no TLS MITM;
- has bounded request/response sizes, timeouts and cancellation;
- applies logging/redaction rules before diagnostics leave the broker.

### C. Secret boundary

Initial production scope is anonymous extraction only.

- Reject URL userinfo before entering the worker.
- No caller Cookie, browser profile, netrc, Authorization, proxy credential or free-form Secret header.
- The worker/extractor itself is also not credential authority in anonymous mode: if it tries to originate a Secret-classified request header, the broker rejects that request.
- No persistent or broker-owned anonymous cookie jar is admitted by this initial decision. If one is proposed later, it must keep raw `Cookie`/`Set-Cookie` out of worker/public plugin output/logs and receive separate security review.
- Any extractor attempt that requires user/account Secret remains unsupported by generic-ytdlp and falls back to an explicit Site Plugin/Browser flow.
- Future authenticated generic-ytdlp, if ever proposed, requires a separate Core/Vault capability design and is not implied by this decision.

### D. Escape prevention

The worker and every descendant must inherit a sandbox that blocks direct internet sockets. The intended implementation is a network namespace or equivalent OS control with no routable interface plus an inherited Unix-domain/socketpair IPC FD; an equivalent seccomp/capability design is acceptable if evidence proves the same property.

The initial supported surface must also disable/fail closed for:

- default/custom yt-dlp plugins;
- remote components;
- direct WS/WSS, FTP, file/data network escape semantics;
- curl_cffi/browser impersonation as a direct transport;
- external downloaders;
- arbitrary JS runtime discovery/execution.

A later Task may separately admit an allowlisted compute-only JS runtime only if the descendant inherits the same no-direct-network/no-secret boundary and its executable/version/path is fixed. Compatibility loss is preferable to bypass.

### E. Lifecycle

Retain #46 stdout/stderr/output caps, timeout, kill and reap behavior. Extend cancellation to the process group/cgroup and broker request, and prove that descendants cannot remain network-capable or outlive the attempt. Broker and worker errors must not log source URLs, Secret headers/cookies, signed URLs or response bodies by default.

## 7. Redirect, DNS and TLS reasoning

### DNS / rebinding

A standard subprocess or proxy configuration is insufficient if yt-dlp re-resolves the target. Under the selected conditional architecture, only Gateway resolves public hosts. The broker connects through the same checked-address pinning concept accepted by R008; the sandbox prevents the worker from doing a second authoritative lookup/connect path.

### Redirects

A generic HTTP proxy is not enough for HTTPS: after CONNECT, redirect responses are encrypted end-to-end between client and origin. Observing every Location would require either client cooperation or TLS interception. Broad MITM is excluded.

The selected design uses client cooperation at the yt-dlp request layer **plus** a Gateway broker. The broker itself follows no transport redirects. Every Location is handled as a new policy decision with fresh host/DNS/address validation before the next origin connection.

### TLS

The broker, not a MITM proxy, is the HTTP client to the origin. It preserves the original hostname for Host/SNI/certificate verification while pinning the actual socket to the validated address set, matching the accepted R008 model. `--no-check-certificate`, arbitrary client certificates and caller TLS flags are not exposed.

## 8. Alternatives matrix

| Family | Security against R008 | Complexity | Compatibility/value | Decision |
| --- | --- | --- | --- | --- |
| 1. Gateway HTTP(S) broker/proxy + subprocess sandbox | **Good only as structured broker**, not ordinary CONNECT/SOCKS. With broker-owned redirects/DNS/TLS + no-direct-network sandbox it can be R008-equivalent. | High | Retains much of yt-dlp extractor value but requires adapter/protocol work. | **Selected as part of combined design** |
| 2. Network/process namespace forcing mediator | Strong escape prevention, but namespace alone does not provide URL/redirect/Secret semantics. | Medium-high / platform-specific | Transparent to upstream until a protocol needs direct transport. | **Required backstop, insufficient alone** |
| 3. Python/API RequestHandler hook | Best place to expose logical HTTP(S) requests before transport. Upstream internals are mutable and extractors can add handlers, so hook alone is not a hard boundary. | Medium-high | High compatibility for ordinary extractor HTTP(S). | **Selected with broker + sandbox** |
| 4. Plugin-owned alternative extraction engine with controllable client | Security easiest because all HTTP can directly use Core-scoped client. | Very high per-site/product effort | Loses yt-dlp's broad extractor ecosystem; duplicates explicit Site Plugin work. | Use for explicit high-value sites, not generic replacement now |
| 5. DEFER/DROP generic real network | Safest and lowest complexity. | Low | Loses optional long-tail fallback but `generic-direct`, explicit plugins and Browser path remain. | **Current runtime fallback until conditions are proved** |

Why not standard proxy only: CONNECT hides HTTPS redirects unless TLS is intercepted; SOCKS provides even less HTTP visibility. Why not namespace only: it can force a destination path but cannot by itself enforce R008 URL semantics, redirect revalidation, TLS hostname intent or Secret policy. Why not hook only: an upstream/internal bypass or child runtime would still have direct egress. The three-layer combination is intentional defense in depth.

## 9. Claims C1-C9

| Claim | Attempt 1 research result | Analysis / required proof |
| --- | --- | --- |
| C1 Network authority | **CONDITIONAL PASS** | Logical HTTP(S) can be mediated at yt-dlp's request layer, but production acceptance requires the broker to be the sole permitted network path and sandbox evidence proving bypass fails closed. |
| C2 DNS pinning / rebinding | **CONDITIONAL PASS** | Gateway broker can reuse accepted R008 resolution + checked-address pinning. Worker must have no independent routable socket path. |
| C3 Redirect revalidation | **CONDITIONAL PASS** | Default yt-dlp redirects are unacceptable. Selected broker must disable transport redirects and apply R008 to every Location hop. |
| C4 Proxy/open-proxy boundary | **CONDITIONAL PASS** | Use inherited per-attempt IPC capability, no CONNECT/public listener/configured-local scope, and sandbox direct-deny. |
| C5 TLS | **CONDITIONAL PASS** | Broker connects directly to origin with normal hostname/SNI/certificate verification + pinned addresses; no MITM and no no-check-certificate option. |
| C6 Secret/account authority | **CONDITIONAL PASS** | Initial scope anonymous only; reject URL userinfo, caller credentials/Secret headers, and worker/extractor-originated Secret-classified headers. User-account auth remains explicit Site Plugin/Browser territory. |
| C7 alternate transport/config/process escape | **CONDITIONAL PASS** | Python API wrapper avoids CLI config/plugin load; fixed package/options; direct WS/FTP/file/curl-cffi/external downloader/runtime escape disabled and OS network sandbox is mandatory. |
| C8 lifecycle/diagnostics | **CONDITIONAL PASS** | Reuse #46 caps/timeout/kill/reap and add descendant + broker cancellation proof and Secret-safe diagnostics. |
| C9 product value | **PASS** | The long-tail yt-dlp value justifies a bounded future implementation experiment, but not at the cost of R008. Fallback remains generic-direct + explicit Site Plugins + Browser/site-specific runtime. |

`CONDITIONAL PASS` here is architecture research, **not runtime verification** and not permission to enable production networking.

## 10. Missing evidence / future implementation gate

A later implementation Task may be materialized only after #50 is Final Accepted by the Coordinator. That Task must not begin by enabling the current CLI runner. It must build the bounded worker/broker/sandbox design above and produce exact-candidate evidence for at least:

1. all broker HTTP(S) connections use the accepted R008 validated/pinned address set;
2. redirect-to-loopback/private/link-local/metadata/reserved targets is denied at every hop;
3. DNS answer changes after validation cannot alter the actual connected address;
4. direct AF_INET/AF_INET6 connection attempts from Python, a custom handler and an allowed child are denied;
5. the worker cannot reach the internet by changing proxy/config/env/plugin/runtime/external-downloader state;
6. TLS hostname/certificate verification remains enabled and no MITM CA is introduced;
7. caller URL userinfo, Cookie/profile/netrc/Auth/proxy credentials, arbitrary headers/argv, and worker/extractor-generated Secret-classified request headers are rejected in the anonymous profile;
8. crash/timeout/cancel/oversized output kills/reaps descendants and cancels broker work without Secret-bearing diagnostics;
9. a small, explicit compatibility corpus shows enough value over `generic-direct` to justify keeping the feature.

If any of C1-C8 cannot be proved without weakening R008, the implementation must remain disabled and the product decision should be revised to `DEFER` or `DROP`, not weaken the invariant.

J4 runtime prototype was **not used in Attempt 1**. The current upstream source is sufficient to reject the naive CLI/proxy designs and to define a plausible conditional architecture; runtime proof belongs to the later implementation/verification gate and cannot convert this research Task into production enablement.

## 11. Canonical and product implications

No canonical security invariant needs to change for this research result.

- `docs/security.md` / R008 remain authoritative and unchanged.
- `generic-ytdlp` production networking remains disabled.
- `generic-direct` remains the preferred generic path when the input is already a direct media URL.
- Explicit Site Plugins remain the path for site-specific authentication, Secret ownership and high-value integrations.
- Browser/site-specific runtime remains a valid fallback for flows whose behavior cannot fit the anonymous brokered extractor boundary.

Only after a future exact-candidate implementation proves the conditions should canonical implementation/site-plugin docs be updated to describe the admitted runtime path.

## 12. Freshness and stale-source risk

This decision is tied to Task Planning Base `18dd2b60f21d98436341e056376b26730c392ab7`, Attempt 1 Execution Base/current main `6a6c724ae3e8ebafc733fcf1c5fcc1b031a32757`, and yt-dlp stable `2026.08.19@3a08beaf031ab68f966401ead017ac81fe8486cf`, with upstream master freshness checked at `66f49765d5a46c7be1c2c414f245c71530c4a2fd` on 2026-08-26.

Because RequestHandler/plugin/runtime internals are upstream implementation details and the plugin API explicitly carries no backwards-compatibility guarantee, **every future yt-dlp version bump is a security-relevant freshness event**. The later implementation must pin a version and re-run source/escape/egress verification before upgrading.
