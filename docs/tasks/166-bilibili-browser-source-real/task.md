# Task — BILIBILI-BROWSER-SOURCE-REAL

## Metadata

```text
GitHub Issue: #166
Parent Goal / Research Item: #68 / R005 real source portability
Task ID: BILIBILI-BROWSER-SOURCE-REAL
Task kind: verification
Base commit: 3a2a5fe8900b0ecf728e12461420ca043bdd2702
Candidate commit: n/a; live Evidence is owned by Issue/Attempt
Session bootstrap prompt: docs/tasks/166-bilibili-browser-source-real/prompt.md
Preferred worker: cloud-codex with authenticated SSH tx-node
Eligible worker environments: env:cloud after publication gate
Required capabilities: github-read-write, repository-static-analysis, cloud-interactive, interactive-linux-debug, authenticated SSH tx-node
Hard publication dependencies: #169 Final Acceptance and #172 Final Acceptance; exact provenance/target/runbook freeze below
```

> This is a separate target verification Task. The Worker does not compile locally or on tx-node, does not deploy a phone/TV, and does not modify Gateway/production state. GitHub Actions are the build authority; tx-node is only the live browser evidence target.

## Goal / Decomposition / Routing

Use the accepted bounded browser probe to determine whether one public Bilibili page can expose a source that an independent server-side consumer can read after the clean browser exits. This is a compatibility/research result, not a Gateway playback path.

```text
Orchestrator: cloud-codex (gpt-5.6-luna, reasoning high; Fast only if exposed and recorded)
Implementation/artifact authority: GitHub-hosted Actions
Execution plane for this Task: authenticated SSH to tx-node
Target: tx-node / VM-0-11-ubuntu / ordinary low-privilege Linux user gateway-verify
Target architecture: x86_64 (browser capability evidence only; not ARM64 phone proof)
```

The live network/target Evidence authority is deliberately separate from #169/#172 hosted implementation evidence. Do not split this Task by runner or device, and do not infer a live result from the hosted synthetic jobs.

## Frozen Publication Preconditions

The following values were independently accepted before this revision. The Worker must verify them again from GitHub and the downloaded artifact before any live request.

### Live-capable probe (#169)

```text
Candidate SHA: eb6598d473768f544744431c5af792d6de86d59f
Merged main: f9a48dbef6535c4bb38188b050384ac4156187af
Workflow run: 34218848612
Required jobs:
  J1 102036943379
  J2 102036943625
  J3a 102036943040
  J3 102037252460
  J4 102036943488
  JI1 102036943403
Bundle artifact: ID 10052918296
Bundle digest: sha256:c565934ea092f54c67e902c6e3615cb035e7f37c6ec78448e176c925b1ad3a37
Broker evidence: ID 10052975493 / sha256:75a51e8d05507c82f84670190d33b1f4ad5d0391323d5f78bc39a9ba8b6b8ae4
Sanitized evidence: ID 10052975884 / sha256:d92d8ec65bafbd91880b4610046ce0416a9a02155553ac4a38eaa6a2f7b66051
```

### Target-runnable package (#172)

```text
Candidate SHA: 989bacdbf7e0fcde053005a45a1ad41b219b8b46
Merged main: 3a2a5fe8900b0ecf728e12461420ca043bdd2702
Workflow run: 34223368033
Required jobs:
  J1 102051511536
  J2 102051511433
  J3a 102051511227
  J3 102051889697
  J4 102051511455
  JI1 102051511612
Bundle artifact: ID 10054697137
Bundle zip digest: sha256:66cae21de2eecb6a7166a703f9679e961a4556a93050d0e5733627fe06dacbc1
Bundle archive SHA recorded by J3a: a236f10085ea73c9c783b11335fb81bf979cba805d1f69064ad4f5aca15ed11d
J2 probe evidence: ID 10054718111 / sha256:f728f264946b3d6cc31133dacaa35415ae4016e124066b5da19a56778b61cd2d
J2 broker evidence: ID 10054717444 / sha256:0a5f088a140a1c2b5f864ea7029e0648bfcbaccaad8f495d829d8fb9da60260b
Runtime: playwright-core@1.55.0, 321 manifest-inventoried regular files, no browser binary and no bin/ installer files
```

The #172 package is a derivative of #169 for target runtime closure. It must be copied to a clean target directory and verified from its manifest; no npm install, package manager cache, workspace `node_modules` link or fallback dependency is allowed on tx-node.

### Target admission

Read-only preflight was recorded in the Issue history ([comment](https://github.com/liqiangcc/jellyfin-web-media-gateway/issues/166#issuecomment-5583242133)); the Worker must repeat the non-mutating checks and record the actual values in its Evidence:

```text
Host: VM-0-11-ubuntu (tx-node)
OS/kernel: Ubuntu 26.04 / Linux 7.0.0-14-generic
Architecture: x86_64
Execution user: gateway-verify / UID 1001 / GID 1001
Node: v22.22.1
npm: 9.2.0
Browser: /usr/bin/google-chrome-stable / Google Chrome 152.0.7977.82
Proxy environment: HTTP(S)/ALL proxy variables unset; use no proxy bypass
Existing browser/profile: never attach or reuse any source-runtime/Gateway/Chrome process or profile
```

The target process must run as `gateway-verify` with a fresh mode-700 profile created by the artifact. Do not pass `--no-sandbox`, `--user-data-dir`, `--proxy-server`, `--remote-debugging-port`, cookies, Authorization, CDP endpoints or arbitrary headers. The probe's own fail-closed broker is the only permitted egress path. The target must have no production Gateway/Vault mutation and no VNC interaction.

The frozen selector and entry are:

```text
bilibili:BV14V411W7r5:part-2
BILIBILI_PROBE_ALLOW_LIVE=1 CHROME_PATH=/usr/bin/google-chrome-stable \
  node experiments/bilibili-browser-probe/probe.mjs \
  --mode live --selector bilibili:BV14V411W7r5:part-2
```

Run it as the low-privilege user from the verified downloaded artifact, with proxy variables explicitly removed. No other selector, URL, profile, proxy, header or option is admitted. The exact no-install command and cleanup procedure are frozen in `docs/research/bilibili-browser-probe-runbook.md` at the current base.

## In Scope / Steps

1. Read the required canonical startup documents, the full Issue history, #169/#172 acceptance comments, the exact task/runbook and lifecycle/freshness protocols.
2. From GitHub, verify both accepted provenance sets, the bundle zip/manifest/digests, the target runtime package version and the runbook commit. Stop `BLOCKED` before live traffic if any value differs.
3. Perform the target admission checks above as read-only diagnostics. Confirm the low-privilege user can launch the external Chrome without `--no-sandbox`; do not install packages or alter target services.
4. Use at most two independent clean sessions. Each session uses only the frozen opaque selector and plugin-generated page navigation. Record whether the requested part is preserved and only bounded sanitized candidate metadata.
5. After each browser exits, run the independent server-side consumer from the same process boundary with approved request metadata only. A muxed candidate must be readable; AV-separated media requires both video and audio reads. Count actual response bytes, including errors, and stop at limits.
6. Store only sanitized evidence: selector/part match, candidate counts, role, codec/container when known, status class, Range support, safe header names, expiry category, denial decisions, byte/request counts, cleanup and portability result. Never write a candidate URL, query/token, cookie, Authorization value, raw DOM/playinfo/HAR or full body.
7. Clean every browser, broker, child process, temporary profile and staging directory in a re-entrant finalizer. Report the exact target command, environment and observed result; do not alter #67 or start implementation work.

## Verification Matrix / Success Criteria

| Job | Claim / PASS condition | Plane / Host | Required Evidence |
| --- | --- | --- | --- |
| J0 | C0: exact #169/#172 artifact, runbook, version, low privilege, clean profile, no proxy, broker/transport policy, budgets and cleanup admission all pass before live traffic | authenticated SSH / tx-node | read-only preflight + exact artifact/manifest/digest checks |
| J1 | C1: two clean anonymous sessions preserve the requested part and expose at least one bounded candidate set | authenticated SSH / tx-node | two sanitized session records, actual request/byte counts and network path |
| J2 | C2: independent post-browser consumer reads at least one portable candidate (both streams when AV-separated), with no Secret, private redirect or budget violation | authenticated SSH / tx-node | consumer status classes/limited byte counts, lifecycle and negative outcomes |

Task success requires complete sanitized Evidence and a Coordinator decision for C0/C1/C2. A complete reproducible negative result may be accepted as research delivery, but downstream source/plugin/playback implementation is unlocked only when C0/C1/C2 are all `PASS`. Unknown codec/expiry fields remain `unknown`; they do not become guessed PASS evidence.

`FAIL` is a reproducible site/protocol incompatibility (wrong part, blob-only source, non-readable stream, expiry, 4xx/5xx, private redirect, secret-header requirement or bounded cancellation). `BLOCKED` is reserved for missing artifact/browser/permission/egress capability or a required design change. Do not retry through proxy rotation, copied credentials or expanded budgets.

## Frozen Budgets / Negative Boundaries

- Maximum two clean sessions.
- Per session: navigation 120 seconds, 200 observed requests, 32 MiB actual response bytes, retained metadata 1 MiB. Redirects count as requests; stop on first touched limit.
- Independent consumer: maximum 8 requests total, 1 MiB per request, 4 MiB total actual bytes including error responses; stop on first limit.
- Do not click play or trigger full-video preload. Login, DRM/permission challenges, browser-only `blob:`/token material, or a private redirect are negative portability results.
- No profile reuse, cookies, Vault/SSH secrets, VNC, Gateway or production service mutation. No phone or physical TV deployment.

## Out of Scope / Architecture

Do not implement a production Bilibili adapter, Registry wiring, remux/transcode, MSE player, login/DRM bypass, Playback/Display/Control changes, or phone/TV workflow. Core remains site-agnostic; the browser is a generic runtime; plugin-owned selector/navigation and the experimental broker remain bounded. This Task cannot claim Gateway/TV playback or close #68.

If the live result requires changing #169/#172 browser semantics, R008 policy or a production API, stop with a `[BLOCKER REPORT]` and return the change to a new implementation/contract Task. Never weaken a negative assertion in this verification Task.

## Freshness / Integration Contract

Freshness policy: dependency-aware.

Semantic authorities:

- #169 Final Acceptance at `f9a48dbef6535c4bb38188b050384ac4156187af`;
- #172 Final Acceptance at `3a2a5fe8900b0ecf728e12461420ca043bdd2702`;
- `AGENTS.md`, `docs/security.md`, `docs/implementation-contracts.md`, Browser/Site Plugin/R008 contracts;
- this exact selector, target admission and runbook freeze.

Task-owned surfaces: sanitized target Evidence and the Issue Attempt history only. No production code or artifact changes are owned here.

Main changes unrelated to the frozen browser/package/runbook semantics do not automatically stale exact target Evidence. Any change to the artifact layout, browser/broker/security authority, selector/sample, target image or budgets requires the Coordinator to return this Issue to `status:draft`, revise the contract, and repeat Publication Gate before further live traffic.

## Evidence / Completion Protocol

Every Attempt report and Coordinator Review must distinguish implementation result, Claim result and Coordinator decision and record:

```text
Task / Claim / Attempt: #166 / C0,C1,C2 / N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high (+ actual Fast availability)
Execution plane: authenticated SSH
Runner / Target: tx-node / VM-0-11-ubuntu / x86_64 / gateway-verify UID 1001
OS, Node/npm, Chrome: exact observed versions
Network path: target browser → experimental fail-closed broker → public Bilibili; no proxy
Base / dependency Candidate SHAs: exact #169/#172 values; runbook/task base
Workflow/run/job/artifact: exact accepted IDs/digests
Commands / selector / budgets: exact values and actual use
Sanitized Evidence: no URL/token/cookie/Auth/raw body/DOM/HAR
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Follow `docs/tasks/issue-lifecycle-protocol.md`: read fresh Issue authority before every terminal mutation; claim only from `status:ready`, post `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, transition to review/blocked, release ownership and stop. Worker cannot set `done` or close the Issue. Coordinator must read all history, task, runbook, Evidence and required provenance, post `[COORDINATOR REVIEW]`, and only after `[FINAL ACCEPTANCE]` set `status:done` and close.

## Worker / Build Constraints

Worker: cloud-codex, `gpt-5.6-luna`, reasoning high; enable Fast only if exposed and report actual availability. No local compilation, test-binary build or npm install; all build/test/package provenance is already from GitHub-hosted Actions. SSH is limited to read-only admission and the bounded live probe/consumer under `gateway-verify`. Do not install `playwright-core` on tx-node. Do not deploy or configure a phone, TV, VNC or production Gateway.
