# Task Contract — #116 ENV-ARM64-ANDROID-BROWSER-CDP-BRIDGE-PREP

## 1. Identity

- Issue: `#116`
- Task ID: `ENV-ARM64-ANDROID-BROWSER-CDP-BRIDGE-PREP`
- Kind: diagnostic capability / environment + tooling prep
- Preferred Worker: `ubuntu-arm64`
- Eligible environment: `env:ubuntu-arm64`
- Planning Base: `6f33434d0eee36ac6b60f2d3741264e61c275b92`
- Parent Goal: `#67 GENERIC-YTDLP-BILIBILI-REAL`
- Downstream diagnostic Task: `#117 ENV-ARM64-BROWSER-WORKER-DIFFERENTIAL-DIAG`
- Publication state: draft until Coordinator Publication Gate.

The Git commit containing this contract and its sibling `prompt.md` is the Task Package authority once the Coordinator records that exact commit in Issue #116.

## 2. Why this Task exists

Current accepted evidence does not justify another compatibility repair or another immediate #67 resolver run:

- #67 R17 / Attempt 17 established exact runtime Candidate `80fb081b129f8f664124b84ddcc9698039e2cfd1` and the frozen runtime, then stopped at J2 because the unchanged frozen Bilibili sample returned `4xx`; J3 was correctly not run.
- #113 R2 / Attempt 2 observed `2xx → 4xx → 2xx` with the unchanged direct/no-proxy bounded probe contract and therefore remained BLOCKED.
- Operator observation says the same video opens quickly in the phone's normal Android browser. Treat that statement as a routing hint only. It is not machine-verification Evidence and does not authorize access to the normal browser profile, cookies, account state, cache or credentials.

The smallest independent next question is therefore not “how do we make curl look like a browser?” It is “can we safely observe the real Android browser network path through a dedicated diagnostic browser without exposing browser state?”

## 3. Goal

Establish a reversible diagnostic path:

```text
Dedicated Android diagnostic browser
→ Android/ADB/CDP transport
→ local-only relay
→ repository-owned minimal browser-diag MCP
→ independent ARM64 Codex Worker
```

The result is a bounded diagnostic capability, not a Bilibili compatibility verdict.

## 4. Separation boundary

This Task has its own lifecycle and success criteria because capability construction is independent from the later real-source comparison.

#116 may:

- audit Android browser/debug capabilities;
- install or configure a dedicated diagnostic browser under the rules below;
- implement the narrow bridge/MCP and focused tests;
- prove the bridge on a neutral HTTPS page;
- produce a Candidate/PR for repository-owned tooling.

#116 must not:

- diagnose the frozen Bilibili sample;
- run #113 probes as evidence;
- run yt-dlp/generic-ytdlp, #67 J3, broker/sandbox/R008 paths or #68;
- attach to the user's normal browser profile;
- copy browser state into a Worker request.

#117 is the only planned consumer of this capability for the Browser-vs-Worker comparison, and remains draft until #116 Final Acceptance.

## 5. Capability audit — C0

Before implementing or installing anything, record only bounded capability facts:

- Android/Termux/Ubuntu execution layers available to the Worker;
- whether a separate debug-capable browser package already exists;
- whether a safe ADB/CDP transport can be established without exposing a globally reachable debug endpoint;
- whether any required one-time operator action is unavoidable (for example, an Android pairing/permission or installing an approved browser package).

Do not enumerate normal browser tabs, profiles, databases, cookies or app-private data.

If a safe dedicated-browser path is unavailable, report BLOCKED with the missing capability and stop. Do not convert the normal browser into the diagnostic target as a workaround.

## 6. Dedicated browser isolation — C1

The diagnostic browser must be logically separate from the user's normal browser state.

Required properties:

- separate browser app/package or a genuinely separate empty diagnostic profile that cannot expose the normal profile;
- no source-site account login;
- no Cookie/Auth/profile import;
- no password-store, local-storage or session-storage import;
- no pre-seeding from the normal browser cache/history;
- browser product family/version may be recorded, but do not publish a full user-agent/fingerprint string.

If installation is necessary:

- use only a trusted package source already configured on the device or an operator-supplied official package;
- do not download/sideload an arbitrary APK from an untrusted URL;
- do not silently replace or modify the user's normal browser.

## 7. CDP transport — C2

Choose the least-privilege working transport discovered in C0. Examples may include Android ADB forwarding, a device-local abstract socket relay, or another standard browser-supported debugging path, but the following invariants are mandatory:

- CDP/DevTools is on-demand, not a permanent globally reachable service;
- bind only to loopback and/or an equivalent local Unix/abstract socket boundary;
- never expose the DevTools endpoint on `0.0.0.0`, a public interface or Tailscale;
- do not permanently enable TCP adbd solely for this Task;
- if a temporary forward/pairing is required, document deterministic teardown;
- do not use root/app-private-data scraping as a shortcut to browser state;
- do not access the user's normal browser DevTools targets.

A successful C2 proof must show that only the dedicated diagnostic browser is reachable through the bridge.

## 8. Minimal MCP surface — C3

Implement a repository-owned MCP with a strict allowlist. Do **not** expose an arbitrary “send CDP command” primitive.

Minimum candidate tools:

- `health`
- `list_targets`
- `open_url`
- `reload`
- `network_capture_start`
- `network_summary`
- `network_capture_stop`

The implementation may reduce this set if the same success criteria can be satisfied with a smaller surface. Any extra tool requires explicit justification in the Worker report.

### 8.1 Target and navigation constraints

- `list_targets` returns only targets from the dedicated diagnostic browser.
- Target identifiers are opaque and bounded.
- `open_url` accepts HTTPS only and must enforce a startup/configured host allowlist; the MCP must not provide a tool that mutates that allowlist at runtime.
- #116 proof uses a neutral non-sensitive HTTPS host only.
- #117 may later configure the exact frozen Bilibili host through its own accepted Task contract.

### 8.2 Allowed `network_summary` fields

A bounded summary may contain only fields needed for path correlation, for example:

- top-level navigation `status_class` (`2xx`, `3xx`, `4xx`, `5xx`, `network-error`, `unknown`);
- bounded redirect count and redirect status classes;
- protocol enum (`http/1.1`, `h2`, `h3`, `other`, `unknown`);
- remote IP family and, if implemented, bounded public remote-peer metadata used only for correlation;
- `connection_reused` boolean/unknown;
- `from_disk_cache` boolean/unknown;
- `from_service_worker` boolean/unknown;
- TLS protocol enum/unknown;
- coarse duration bucket, not high-cardinality raw timing traces;
- diagnostic browser product family/version.

### 8.3 Prohibited MCP output

The MCP must never return or log:

- Cookie, Authorization, Set-Cookie or credential-bearing state;
- password/profile/local-storage/session-storage contents;
- arbitrary request/response headers;
- response/page bodies, DOM dumps or raw DevTools event streams;
- full source/media URLs when they contain query strings, signatures or tokens;
- signed media URLs, tokens or media payload;
- personal-browser targets/titles/history.

Prefer structural status classes and bounded enums over raw strings.

## 9. Repository implementation scope — C4

Implementation may add the smallest coherent repository-owned tooling needed for C2/C3. Expected areas may include:

- a new tool directory such as `tools/browser-diag-mcp/`;
- a narrow Android/Termux bridge/startup helper under `scripts/`;
- unit/integration tests for schema, bounds, host allowlisting and redaction;
- operator/Worker startup notes tightly scoped to this capability.

Do not modify generic-ytdlp, Gateway runtime behavior, Secret/Vault, broker/sandbox policy or Web Display to satisfy this Task.

The Worker must keep capability code separate from any device-specific temporary state.

## 10. Neutral proof — C5

Prove the complete path using one neutral non-sensitive HTTPS page:

```text
dedicated Android browser
→ safe CDP transport
→ browser-diag MCP
→ open neutral page
→ capture bounded navigation metadata
→ stop capture
```

The proof must not rely on Bilibili, cookies, login or the normal browser profile.

Required evidence is bounded to:

- bridge/MCP health PASS;
- dedicated-browser identity/isolation PASS;
- neutral navigation status class;
- allowed network-summary field names and bounded values;
- redaction/bounding tests PASS;
- no public debug listener after teardown.

Do not publish the neutral page body or arbitrary headers.

## 11. Cleanup / persistence — C6

After verification:

- stop temporary CDP/ADB forwarding and bridge listeners;
- remove temporary files/profiles created solely for the proof unless the Candidate explicitly defines a safe reusable diagnostic profile required by #117;
- leave no public DevTools listener;
- do not leave a debug process attached to the normal browser;
- repository code/PR may remain for Coordinator review;
- a dedicated diagnostic browser app may remain installed only if its state is empty/non-account and the Worker clearly records this as the intended reusable #117 prerequisite.

## 12. Success criteria

PASS requires all of the following:

1. dedicated diagnostic browser identity is established without importing/attaching the normal browser profile;
2. safe local-only CDP transport starts and stops deterministically;
3. repository-owned minimal MCP reaches only that diagnostic browser;
4. neutral HTTPS navigation produces allowed bounded network metadata;
5. output schema/allowlist/redaction tests PASS;
6. no prohibited browser/page/auth data appears in tests or Evidence;
7. no public/remote DevTools listener remains after cleanup;
8. implementation Candidate/PR is reviewable and does not widen Gateway/runtime/security authorities;
9. evidence is sufficient for Coordinator to decide whether #117 may be published.

BLOCK when a safe dedicated-browser/CDP route cannot be established with available device capabilities, when required operator-only setup cannot be completed, or when the proposed MCP cannot meet the data-minimization boundary.

## 13. Worker lifecycle

Normal path:

```text
live read-back
→ claim
→ status:in-progress + owner
→ [EXECUTION CHECKPOINT]
→ C0 capability audit
→ implement smallest Candidate
→ tests + neutral proof
→ branch/PR if repository changes
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Blocked path:

```text
live read-back
→ claim
→ bounded capability attempt
→ [BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must never merge, close, mark done, publish #117, execute #67/#113/#68, or expand scope itself.

## 14. Evidence publication boundary

Issue/PR Evidence should contain only bounded facts: package/version, status classes, enum fields, pass/fail checks, repository paths/commits, and sanitized capability errors.

Do not place ADB pairing secrets, auth tokens, browser data paths that reveal personal state, page bodies, headers, cookies, account identifiers or signed URLs in GitHub.
