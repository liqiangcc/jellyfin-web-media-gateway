# Task — ENV-ARM64-GITHUB-EGRESS-DIAG

## Metadata

```text
GitHub Issue: #89
Task ID: ENV-ARM64-GITHUB-EGRESS-DIAG
Task kind: environment diagnosis + bounded user-level repair + verification
Planning Base: 800202c7beffcd51a7c91c7c1cfef9037389474c
Session Bootstrap: docs/tasks/89-env-arm64-github-egress-diag/prompt.md
Downstream Handoff: docs/tasks/handoffs/ubuntu-arm64.md
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Required capabilities: github-read-write, arm64-target-runtime, interactive-linux-debug, process/network inspection, git/curl/gh diagnostics, user-level configuration, evidence-authoring
Target: ubuntu-arm64-target-phone
Parent blocker: #85 BROKER-FD-ISOLATION-LEGACY-KERNEL-PREP
Parent exact Candidate: 4af64b124af4d1599a87bd211395ee832e9d7e4b
Trigger run/job: 33026189606 / 98367958569
Freshness policy: evidence-bound / environment-only
```

> Issue #89 owns live status, Attempt, owner and result. This `task.md` owns the stable target-network diagnostic contract.

## Trigger Evidence

#85 J1/J2/J3 are PASS on exact Candidate:

```text
4af64b124af4d1599a87bd211395ee832e9d7e4b
```

The J4 `workflow_dispatch` run reached the accepted phone Runner successfully:

```text
run: 33026189606
job: 98367958569
runner: ubuntu-arm64-target-phone
runner version: 2.336.0
Set up job: PASS
actions/checkout@v4 package download: PASS
candidate input validation: PASS
```

`actions/checkout@v4` then failed while Git attempted to fetch the exact Candidate from `github.com`:

```text
attempt 1:
GnuTLS recv error (-110): The TLS connection was non-properly terminated

attempt 2/3:
Failed to connect to github.com port 443
```

The failure occurred before target identity recording, `close_range` probing, #79 offline runtime transfer, or `BrokerProcessRunner` execution.

Therefore this Task owns only the Ubuntu ARM64 target's GitHub access path. It does **not** own #85 fd-isolation implementation or its security semantics.

## Goal

Determine which GitHub access path is actually reliable on the target phone, apply only the smallest justified user-level repair, and produce enough durable Evidence for the Coordinator to safely rerun #85 J4.

The required diagnostic model is:

```text
current inherited/configured network state
        ↓
Direct vs Proxy
        ×
IPv4 vs IPv6 where available
        ×
curl vs gh vs git
        ↓
repeated stability evidence
        ↓
exact Candidate git fetch
        ↓
minimal user-level repair if evidence requires it
        ↓
post-repair repeated proof
        ↓
CHECKOUT_RESUME_ELIGIBLE_FOR_#85: yes | no
```

The Worker must **not** assume `proxy on` or `proxy off` is correct. First prove the current route and compare paths.

## Canonical / Durable Sources

Read before execution:

- `AGENTS.md`
- Issue #89 and all relevant comments
- this `task.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/tasks/handoffs/ubuntu-arm64.md`
- `docs/tasks/63-arm64-functional-environment-readiness/task.md`
- Issue #63 Final Acceptance
- Issue #87 Final Acceptance / current accepted Runner recovery evidence
- Issue #85 current comments, especially J1-J3 PASS and J4 run `33026189606`
- `.github/workflows/broker-fd-isolation.yml` at exact Candidate `4af64b124af4d1599a87bd211395ee832e9d7e4b`
- `docs/runner-execution-architecture.md`
- target-related sections of `docs/security.md`

## Invariants

1. `gateway-runner` remains low privilege. Do not grant root, sudo, ADB, Vault, production Secret or privileged network authority.
2. Do not install system packages or modify Android/kernel settings, firewall, VPN/TUN, routing tables, DNS daemon or system-wide proxy configuration.
3. Do not rotate through public/residential proxies or use access-control bypass techniques.
4. Do not expose `GITHUB_TOKEN`, `GH_TOKEN`, proxy credentials, Authorization headers, cookies or other secrets in Issue comments/log excerpts.
5. Proxy Evidence must be sanitized. Record only whether configured, scheme, port when non-sensitive, and endpoint class such as `loopback`, `LAN`, `Tailscale/private`, or `public`; redact credentials and sensitive hostnames.
6. Do not modify #85 product/runtime/security code, PR #86 implementation, R008, sandbox, fd-isolation semantics, or #67.
7. Do not execute Bilibili/site verification in this Task.
8. Do not change the #85 exact Candidate. This Task diagnoses the environment against `4af64b124af4d1599a87bd211395ee832e9d7e4b`.
9. Do not treat one successful request as proof of stability. Repetition is required because the trigger failure was intermittent/TLS-network shaped.
10. If persistent repair would require privileged/system-wide changes, STOP and report a blocker instead of weakening the boundary.
11. Worker must not execute #85 J4 or #67 automatically. The Coordinator owns parent-task resumption.

## Allowed Repair Surface

Only after A/B Evidence identifies a concrete cause/path, the Worker may make the smallest reversible **user-level** adjustment needed to stabilize GitHub access, for example:

- remove a stale user-level Git proxy setting;
- add or correct a **GitHub-host-scoped** user-level Git proxy setting when a known local/private proxy is proven stable;
- correct user-owned Runner launch/env files when this can be done safely without killing the active Task and without storing credentials in plaintext;
- correct user-level Git CA/path/config mistakes if evidence proves them and the fix does not weaken TLS verification;
- create a rollback backup of modified user-owned config.

Not allowed:

- `sslVerify=false` or equivalent TLS weakening;
- plaintext proxy credentials in `.gitconfig`/Runner env;
- global public proxy rotation;
- root/system package replacement;
- permanent broad `HTTP_PROXY`/`HTTPS_PROXY` changes without A/B proof;
- forcing a proxy merely to manufacture a PASS;
- changing #85 workflow to hide an environment problem.

If the most likely repair is a different Git build/TLS backend, OS network stack change, VPN/TUN change, or privileged service change, preserve Evidence and recommend a separate bounded Task; do not expand #89 silently.

## Claims

```text
C1 — Target / Runner boundary
The diagnosis is performed on the accepted `ubuntu-arm64-target-phone` environment as the low-privilege `gateway-runner` context, with no production/Vault privilege expansion.

C2 — Current network/proxy authority is known
The Worker records whether proxy variables/configuration exist in the shell, Git, gh-relevant environment and Runner process/launch context, with sanitized values, and distinguishes inherited environment from explicit Git configuration.

C3 — Direct path is characterized
Direct GitHub connectivity is tested without inherited proxy influence and is classified separately for IPv4 and IPv6 where available, using curl, gh and git.

C4 — Proxy path is characterized
If a legitimate existing local/private proxy path is configured or discoverable, the same curl/gh/git tests are executed explicitly through that proxy and compared with Direct. If no legitimate proxy exists, this is recorded as `proxy path unavailable`; no proxy is invented.

C5 — Tool-specific failure layer is isolated
Evidence distinguishes base HTTPS (`curl`), GitHub API / Go TLS (`gh`) and Git smart-HTTP / Git-libcurl-GnuTLS (`git`), including local tool versions/TLS backend indicators sufficient to explain whether the failure is generic networking or Git-specific.

C6 — Exact Candidate Git access is stable after repair/selection
The selected final path can repeatedly fetch `4af64b124af4d1599a87bd211395ee832e9d7e4b` from the repository in fresh temporary Git state without TLS/TCP failure.

C7 — Parent resume decision is evidence-backed
The final report says `CHECKOUT_RESUME_ELIGIBLE_FOR_#85: yes|no` and identifies the exact user-level configuration/path that the next J4 is expected to inherit. Actual #85 `actions/checkout@v4` proof remains owned by the Coordinator rerun of #85 J4.
```

Claim result vocabulary: `PASS | CONDITIONAL PASS | FAIL | BLOCKED | NOT RUN`.

## Verification Procedure

### J0 — Target identity and safe workspace

Record bounded non-secret target information:

```text
uname -a
uname -m
id
whoami
pwd
runner name / current task context
git --version
curl --version
gh --version (if present)
```

Verify:

- architecture is `aarch64`;
- worker is non-root;
- no sudo privilege is required;
- work is outside `/var/lib/web-media-gateway` production state;
- no production Vault/Secret access is introduced.

If `gh` is absent, record that before changing anything. Do not immediately install it. Continue curl/git isolation first. Full PASS should include a usable `gh` comparison; if `gh` cannot be safely made available user-level after a working GitHub path exists, report `CONDITIONAL PASS` or `BLOCKED` with the exact limitation rather than hiding it.

### J1 — Proxy and configuration inventory before network tests

Record **presence and sanitized classification**, not secret values, for both upper/lowercase variables:

```text
HTTP_PROXY / HTTPS_PROXY / ALL_PROXY / NO_PROXY
http_proxy / https_proxy / all_proxy / no_proxy
```

Also inspect safely:

```text
git config --show-origin --get-regexp 'http|https|proxy|ssl' || true
git config --get-urlmatch http.proxy https://github.com/ || true
git config --get-urlmatch http.sslVerify https://github.com/ || true
gh auth status   # summarize only; do not expose tokens
gh config list   # if available and non-secret
```

Inspect the current Runner process/launch context only enough to answer whether proxy variables are inherited by jobs. If reading `/proc/<runner-pid>/environ` or a user-owned launch file, print only proxy variable names and sanitized endpoint classification; never print tokens/credentials or the full environment.

Optionally inventory local proxy processes/listeners such as known `mihomo`, `clash`, `xray`, `sing-box` only as process/listener presence. Do not connect to arbitrary unknown proxies.

### J2 — DNS / IP-family classification

Use tools already available on the target (for example `getent` or Python `socket`) to record whether `github.com` resolves to IPv4 and/or IPv6.

Then test bounded TCP/HTTPS reachability using curl:

```text
Direct IPv4
Direct IPv6 (only when the target has a usable IPv6 route)
```

IPv6 absence is `unavailable`, not automatically a Task failure if IPv4 is stable.

Use bounded timeouts. Do not run throughput benchmarks.

### J3 — Direct A/B baseline: curl / gh / git

For **Direct**, explicitly remove inherited proxy influence for the command and override Git proxy configuration so the test is genuinely direct.

Run a repeated matrix. Minimum recommended repetitions:

```text
curl https://github.com:          5 attempts
gh api /meta:                    5 attempts, if gh available
git ls-remote <repo> HEAD:       5 attempts
```

Record only success/failure, bounded error class and path class. Do not log Authorization headers.

For Git, also record enough local implementation metadata to compare TLS stacks, for example:

```text
which git
git --version
curl --version
ldd $(command -v git) | grep -Ei 'curl|gnutls|ssl' || true
```

If curl/gh are stable but Git alone fails, classify this as a Git/libcurl/TLS-specific path before attempting repair.

### J4 — Existing proxy A/B baseline: curl / gh / git

If J1 finds a legitimate existing local/private proxy endpoint/path, run the same bounded repeated matrix explicitly through it:

```text
curl:        5 attempts
gh api:      5 attempts, if gh available
git ls-remote: 5 attempts
```

Requirements:

- use one known existing endpoint; no rotation;
- keep credentials out of logs and persistent plaintext config;
- distinguish an environment proxy from a Git-specific proxy;
- record `NO_PROXY` effects;
- if proxy is not available, report `PROXY_PATH: unavailable` and do not invent one.

### J5 — Secondary Git-specific diagnostics when needed

Only if the matrix shows `curl/gh PASS` but `git FAIL`, use bounded diagnostics to determine whether Git's HTTP/TLS behavior is the fault domain.

Permitted examples:

```text
compare default Git behavior vs command-scoped HTTP/1.1
inspect user/repo Git proxy and SSL config
inspect Git/libcurl/GnuTLS linkage
fresh temporary repository fetch
```

Do not post raw verbose traces containing auth headers. Summarize/redact locally.

Do not persist `sslVerify=false` or other TLS weakening.

### J6 — Minimal user-level repair

Select repair only from Evidence.

Examples:

```text
Direct stable + stale proxy config exists
→ remove stale user-level GitHub proxy setting

Direct Git unstable + existing local/private proxy stable for Git
→ configure only the smallest GitHub-scoped user-level Git proxy path

Direct curl/gh stable + Git HTTP/2-specific failure isolated
→ prove command-scoped mitigation first; if persistence would require broad/global policy, report rather than silently broadening scope
```

Before modifying a user-owned config file, preserve a rollback copy or record the previous exact non-secret config state.

After repair, repeat the relevant matrix. A repair that merely changes one failure into another is not PASS.

### J7 — Exact Candidate repeated fetch proof

Using the final selected path/configuration, create fresh temporary Git state and fetch exactly:

```text
repo: https://github.com/liqiangcc/jellyfin-web-media-gateway.git
Candidate: 4af64b124af4d1599a87bd211395ee832e9d7e4b
```

Minimum:

```text
3 consecutive fresh exact-SHA fetch/checkout attempts
```

Each attempt must verify:

```text
git rev-parse HEAD == 4af64b124af4d1599a87bd211395ee832e9d7e4b
```

Cleanup the temporary repositories afterward.

If the final selected path uses a persistent user-level Git setting, verify the setting is visible to a fresh non-interactive shell consistent with the Runner user.

### J8 — Parent resume classification

Do **not** execute #85 J4 from this Worker.

Final report must state:

```text
FINAL_GITHUB_PATH:
  direct | git-specific-proxy | runner-proxy | unresolved

DIRECT_IPV4:
DIRECT_IPV6:
PROXY_PATH:
CURL_STABILITY:
GH_STABILITY:
GIT_LS_REMOTE_STABILITY:
EXACT_SHA_FETCH_3X:
RUNNER_INHERITS_REQUIRED_CONFIG: yes | no | n/a
CHECKOUT_RESUME_ELIGIBLE_FOR_#85: yes | no
```

`CHECKOUT_RESUME_ELIGIBLE_FOR_#85=yes` means the child Environment Task has established a stable prerequisite. It does **not** claim #85 J4 PASS. Coordinator must rerun #85 J4 to obtain actual `actions/checkout@v4`, `close_range=ENOSYS`, offline runtime and BrokerProcessRunner evidence.

## Result Criteria

### PASS

All of the following:

- C1-C7 accepted by evidence;
- proxy/current route authority is explicitly known;
- Direct and legitimate existing Proxy paths are not conflated;
- curl/gh/git comparison is complete enough to isolate the failure layer;
- selected final GitHub path is stable under repetition;
- exact Candidate fresh fetch succeeds 3 consecutive times;
- any repair is user-level, minimal, reversible and does not weaken TLS/security;
- `CHECKOUT_RESUME_ELIGIBLE_FOR_#85=yes`.

### CONDITIONAL PASS

The exact Candidate Git path is stable and #85 can safely be rerun, but a bounded optional diagnostic dimension remains unavailable, such as IPv6 or `gh`, and that gap does not obscure the proven Git failure/repair cause. State the gap explicitly.

### FAIL

A proposed repair requires weakening TLS/security, using privileged/system-wide changes, storing secrets unsafely, or the target environment violates accepted Runner boundaries.

### BLOCKED

Examples:

- target Runner/interactive environment unavailable;
- neither Direct nor any legitimate existing proxy can reach GitHub reliably;
- required repair is outside the allowed user-level surface;
- exact Candidate cannot be fetched stably;
- evidence cannot distinguish the selected final route;
- safe rollback/cleanup cannot be guaranteed.

## Success Criteria

1. Trigger run/job and exact #85 Candidate are read back correctly.
2. Current target proxy/network configuration is captured safely and sanitized.
3. Direct IPv4 and IPv6 availability are explicitly classified.
4. Direct curl/gh/git results are compared with repetition.
5. Existing legitimate Proxy curl/gh/git results are compared with repetition when such a proxy exists.
6. The failure is classified as generic network, IP-family, proxy inheritance, Git/libcurl/TLS-specific, or another concrete layer; avoid vague `network unstable` when evidence is more specific.
7. Any applied repair is minimal, user-level, reversible and evidence-driven.
8. Exact Candidate `4af64b...` is fetched/checked out successfully from fresh state 3 consecutive times on the final selected path.
9. No proxy credentials/tokens/secrets are exposed or persisted unsafely.
10. No #85/#67 code or semantic change occurs.
11. Final report includes `CHECKOUT_RESUME_ELIGIBLE_FOR_#85` and the exact expected inherited configuration for the Coordinator rerun.
12. Worker posts standard `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, updates status accordingly, releases active ownership and STOPS.

## Freshness Contract

This is an environment-only Task bound to:

```text
Planning Base: 800202c7beffcd51a7c91c7c1cfef9037389474c
#85 Candidate: 4af64b124af4d1599a87bd211395ee832e9d7e4b
Trigger run/job: 33026189606 / 98367958569
```

Repository main movement is normally `UNRELATED` unless it changes:

- Target Runner security/privilege authority;
- required GitHub host/path;
- #85 exact Candidate identity;
- or the parent resume contract.

The Worker must not substitute moving main for the exact Candidate fetch proof.

## Evidence Contract

Final report must include at least:

```text
Attempt:
Worker/environment:
Target/runner identity:
Planning Base:
#85 exact Candidate:
Trigger run/job:
Initial proxy env classification:
Initial Git proxy/config classification:
Runner proxy inheritance classification:
DNS IPv4/IPv6 classification:
Direct curl 5x result:
Direct gh api 5x result:
Direct git ls-remote 5x result:
Proxy path classification:
Proxy curl 5x result:
Proxy gh api 5x result:
Proxy git ls-remote 5x result:
Git/TLS implementation notes:
Root-cause classification:
Repair applied:
Rollback information:
Post-repair repeated matrix:
Exact SHA fresh fetch/checkout 3x:
FINAL_GITHUB_PATH:
RUNNER_INHERITS_REQUIRED_CONFIG:
CHECKOUT_RESUME_ELIGIBLE_FOR_#85:
Claims C1-C7:
Secrets/sensitive-data check:
Cleanup result:
Suggested Coordinator action:
```

Use `n/a` / `unavailable` when a path legitimately does not exist; do not fabricate a proxy or IPv6 route.

## Completion Protocol

Normal:

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ J0-J8
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Blocked:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must not set `status:done`, close #89, execute #85 J4, execute #67, modify PR #86 product/security semantics, or automatically start another Task.
