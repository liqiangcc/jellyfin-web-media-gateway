# Task — TX-NODE-BROWSER-EGRESS

## Metadata

```text
GitHub Issue: #176
Parent / blocked dependency: #166 BILIBILI-BROWSER-SOURCE-REAL
Task kind: research
Planning base: 6f4356011c84b693cd150397d5d3e8a344bc51a5
Candidate commit: n/a; Worker owns any implementation Candidate
Task contract: docs/tasks/176-tx-node-browser-egress/task.md
Bootstrap prompt: docs/tasks/176-tx-node-browser-egress/prompt.md
Preferred worker: cloud-codex (gpt-5.6-luna, reasoning high; Fast only if exposed and recorded)
Eligible environment: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, cloud-interactive, interactive-linux-debug, authenticated SSH tx-node
Execution planes: GitHub-hosted x64 Actions for code/build/test; authenticated SSH to tx-node only for the bounded diagnostic
```

This Task explains the current #166 blocker. It must not be used to claim Bilibili source portability, Gateway playback, phone behavior or TV behavior.

## Problem and goal

#166 Attempt 1 verified the accepted #169 live probe and #172 target-runnable package, and verified that tx-node can launch external Chrome as low-privilege `gateway-verify`. The first clean no-proxy navigation then stopped with the sanitized browser error `ERR_SSL_PROTOCOL_ERROR`, before page/source observation. Consequently #166 C1 and C2 were BLOCKED; there is no evidence yet that Bilibili is incompatible or that a portable media candidate exists.

Determine which bounded layer fails:

```text
target DNS/address admission
→ broker CONNECT/TLS establishment
→ Chromium proxy/secure navigation
→ page/source observation
```

Deliver a sanitized diagnostic Candidate and one target diagnostic report that identify the failing phase, or record a reproducible capability blocker. Do not change the egress policy to make the request succeed.

## Authority and decomposition

```text
Implementation authority: GitHub-hosted Actions on the exact Candidate
Target evidence authority: tx-node / VM-0-11-ubuntu / gateway-verify
Downstream Task: #166 remains blocked until its Coordinator revises and republishes the live contract
```

This is a focused diagnostic Task, split from #166 because its implementation/evidence lifecycle is different. It does not replace #166 and does not authorize a second #166 source-portability Attempt by itself.

The accepted dependencies to read back are:

- #169 live probe Candidate `eb6598d473768f544744431c5af792d6de86d59f`, merged main `f9a48dbef6535c4bb38188b050384ac4156187af`, bundle artifact `10052918296`.
- #172 target runtime Candidate `989bacdbf7e0fcde053005a45a1ad41b219b8b46`, merged main `3a2a5fe8900b0ecf728e12461420ca043bdd2702`, bundle artifact `10054697137`, bundle digest `sha256:66cae21de2eecb6a7166a703f9679e961a4556a93050d0e5733627fe06dacbc1`.
- #166 Attempt 1 blocker: `ERR_SSL_PROTOCOL_ERROR` before C1 page/source observation, with no raw URL, body, candidate, cookie or authorization material.

If any accepted dependency, artifact layout, browser path, target identity or security authority differs, stop and report BLOCKED rather than improvising.

## Scope

### Implementation

On a clean branch based on this Task's planning base:

1. Add a small diagnostic seam in the experimental browser probe, or an equivalent standalone harness under `experiments/`, that classifies failures by bounded phase: DNS/address policy, broker connect, TLS handshake, proxy response, Chromium navigation, HTTP status or an explicitly bounded unknown class. Preserve the existing live and synthetic behavior.
2. Emit only schema-safe counters and enums. Never emit a URL/path/query, IP, certificate body, TLS transcript, response body, DOM/playinfo/HAR, candidate URL, Cookie, Authorization, proxy credential or browser profile path.
3. Keep the browser generic and the plugin-owned opaque selector unchanged: only `bilibili:BV14V411W7r5:part-2` is admitted for the target diagnostic. The caller still cannot supply a URL, host, headers, profile, CDP endpoint, proxy or credentials.
4. Preserve TLS certificate verification, public DNS/address checks, redirect/transport policy, request/byte/time limits, service-worker/QUIC controls, no-`--no-sandbox` target rule and cleanup. Do not add a fallback proxy, `--ignore-certificate-errors`, `--no-check-certificate`, private-address exception or direct/open proxy.
5. Add deterministic stubs/fixtures for each phase and leakage/negative tests. Tests must prove that a TLS error is classified without retaining sensitive diagnostics and that success-path metadata remains unchanged.
6. Update only focused experiment documentation/runbook references needed to explain the diagnostic mode and its relationship to #166. Do not alter canonical architecture/security claims or #166's frozen source-portability contract in this Task.

The preferred diagnostic path remains broker-owned. If a target-side transport preflight is needed to distinguish a network failure from a broker failure, it must use one plugin-derived public authority, one bounded TLS handshake, no HTTP/media body, no secret/header injection and no arbitrary caller host. It is diagnostic evidence only and must never become a worker egress path.

### Target verification

After the Candidate and required hosted jobs are accepted, run at most one fresh diagnostic session on tx-node:

- repeat read-only admission as `gateway-verify` (UID/GID 1001), Node `v22.22.1`, external Chrome `/usr/bin/google-chrome-stable` `152.0.7977.82`;
- use a clean mode-700 temporary profile and the exact plugin-owned selector;
- remove all proxy environment variables; do not attach to the existing source-runtime Chrome/profile and do not use VNC/CDP;
- use the downloaded, manifest-verified Candidate artifact; no `npm install`, package installation, compilation or build hook on tx-node;
- bound the diagnostic to one session, 120 seconds, 200 requests, 32 MiB response bytes and 1 MiB retained metadata; do not click play or run the independent media consumer;
- retain only phase enum, bounded counters, status class and cleanup result.

A target diagnostic that reaches page/source observation must stop before exporting candidates and must not be converted into a #166 portability result. If the target is unavailable or the phase cannot be classified without weakening a boundary, report BLOCKED.

## Claims and verification matrix

| Claim | PASS condition | Evidence authority |
| --- | --- | --- |
| C0 | Exact accepted dependency/artifact provenance, manifest, target admission and no-install/no-proxy policy are read back before any target request. | GitHub + tx-node read-only checks |
| C1 | Phase classifier and sanitized schema pass deterministic positive/negative tests, including DNS, broker/TLS, Chromium navigation, cleanup and secret-leakage cases. | GitHub-hosted x64 Actions |
| C2 | One approved tx-node diagnostic run records a bounded phase result for the frozen selector, or a concrete reproducible external capability blocker with cleanup. | authenticated SSH / tx-node |
| C3 | Static and integration checks show no weakened egress/TLS/security authority, no caller-controlled URL/profile/proxy/secret input and no production Gateway coupling. | GitHub-hosted Actions |

C2 is not a Bilibili compatibility claim. A phase result such as `tls_handshake_failed` explains why #166 is blocked; it does not imply that a proxy, credentials or a policy exception is allowed.

## Job matrix

| Job | Claims | Plane / runner | Required work |
| --- | --- | --- | --- |
| J1 | C1 | GitHub-hosted x64 | pinned Node runtime; deterministic phase/fixture/leakage tests |
| J2 | C3 | GitHub-hosted x64 | static boundary, secret-marker, URL/profile/proxy input and production-coupling checks |
| J3a | C0 | GitHub-hosted x64 | build and verify an exact Candidate diagnostic artifact; record manifest/digest |
| J3 | C0,C1 | GitHub-hosted x64 | download the artifact into a clean consumer directory and run tests without workspace installs/symlinks |
| J4 | C2 | authenticated SSH / tx-node | one fresh target diagnostic session with the frozen selector, exact external Chrome and sanitized phase evidence |
| JI1 | C1,C3 | GitHub-hosted x64 | focused remote integration/regression checks required by the changed experiment; no local cargo/npm/build |
| J1/J2/J3/JI1 timeout | — | GitHub-hosted | at most 35 minutes each |
| J4 timeout | — | tx-node | at most 5 minutes including cleanup |

Actions are the build/test authority. The Worker must not run `cargo build/check/clippy/test`, frontend builds, `npm install` or any dependency-compiling command in the workspace, on WSL/Windows, or on tx-node.

## Evidence contract

Every Attempt must include:

```text
Task / Claims / Attempt: #176 / C0,C1,C2,C3 / N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high (+ actual Fast availability)
Execution plane: GitHub-hosted Actions and, for J4, authenticated SSH
Runner / Target: hosted x64; tx-node VM-0-11-ubuntu / gateway-verify
OS / architecture / versions: exact observed values
Base / Candidate: exact SHAs; dependency SHAs and artifact IDs/digests
Workflow / run / job: exact IDs for J1/J2/J3a/J3/JI1 and target command
Selector: site_id + opaque selector only; no page URL
Phase: one sanitized enum and bounded request/byte counters
Network path: browser → experimental fail-closed broker → public site; no proxy
Cleanup: browser, broker, child process, temporary profile and staging removed
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Evidence must be append-only on the Issue or a sanitized artifact. Never persist raw URLs, query strings, IP addresses, signed locations, TLS certificates/transcripts, headers beyond allowlisted names, cookies, authorization, DOM, HAR, response bodies, account/profile data or target credentials. A missing phase due to a capability limit is BLOCKED, not PASS.

## Architecture and security boundaries

- Gateway remains the `PlaybackSession` authority; this experiment is not a playback path.
- Site semantics remain plugin-owned; Core, Playback, Display and Control receive no Bilibili-specific branch.
- The broker remains the only browser egress path for the diagnostic. Certificate verification and public-address checks stay enabled.
- No private-network/SSRF exception, open proxy, proxy rotation, profile reuse, VNC/CDP attach, Cookie/Auth injection or TLS verification bypass.
- No production Gateway/Vault/service/phone/TV mutation.
- No change to #166's frozen selector, budgets or source-portability success criteria.

## Out of scope

- Bilibili source portability, media candidate capture or independent consumer reads; #166 owns those Claims.
- Production SiteAdapter/Registry, SourceLocator/ResolvedMedia, Playback/Display/Control, remux/transcode, MSE, login/DRM/CAPTCHA bypass or browser UI work.
- Proxy rotation, copied credentials, existing Chrome/source-runtime profile, VNC/CDP, phone/TV/Jellyfin deployment.
- Local compilation, package installation or target build.
- Broad network scans, arbitrary-host probes, raw TLS/HTTP captures and long-running retries.

## Freshness and lifecycle

Freshness policy: dependency-aware.

Semantic authorities: #169 and #172 accepted artifacts above; `AGENTS.md`; `docs/security.md`; R008 egress/secret contracts; browser/site-plugin contracts; #166 task/runbook and blocker report.

A change to artifact layout, browser/broker/TLS authority, selector, target image or budgets requires Coordinator contract revision and a new Publication Gate before target execution. A main-branch advance alone does not invalidate a Candidate; semantic changes must be reconciled and mapped Claims rerun.

The Worker must follow `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md`, and `docs/tasks/freshness-integration-protocol.md`: claim only from `status:ready`, create Attempt N, use fresh terminal-write guards, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to review/blocked, release owner and stop. The Worker cannot close this Issue.

Coordinator review must distinguish implementation result, C0–C3 verification results, this Task decision and the parent #166/R005 gate. If C0–C3 pass, the Coordinator may accept #176, then revise/re-publish #166 with the new diagnostic Candidate. This Task never closes #166 or #68 by itself.

## Success criteria

1. C0, C1, C2 and C3 have exact, sanitized evidence on one Candidate; all build/test evidence came from GitHub-hosted Actions.
2. The diagnostic identifies the failing layer or records a concrete capability blocker without weakening security boundaries.
3. The target runtime remains disposable and clean; no phone, TV, VNC, production mutation or secret export occurs.
4. The Coordinator accepts the diagnostic before #166 is revised. A successful diagnostic does not itself unlock Bilibili media/plugin/playback implementation.
