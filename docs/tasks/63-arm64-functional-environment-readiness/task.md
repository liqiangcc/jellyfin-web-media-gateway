# Task — ENV-ARM64-READY Functional Environment Readiness

## Metadata

```text
GitHub Issue: #63
Task ID: ENV-ARM64-READY
Task kind: verification / environment readiness
Planning Base: f6f6096c32b01c4cad2cf8d7e717807ecb26e033
Functional Baseline: f6f6096c32b01c4cad2cf8d7e717807ecb26e033
Session Bootstrap: docs/tasks/63-arm64-functional-environment-readiness/prompt.md
Downstream Handoff: docs/tasks/handoffs/ubuntu-arm64.md
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Required capabilities: github-read-write, arm64-target-runtime, interactive-linux-debug, rust-build, process-control, functional-network-diagnostic, evidence-authoring
Target: Ubuntu ARM64 phone
Existing infrastructure authority: #1 INFRA-001 ACCEPTED; #21 INFRA-002 ACCEPTED
Downstream functional consumers: #36 R005-PUBLIC-REAL, #23 R005-PUBLIC, later real Bilibili Web E2E
Explicitly not owned here: #9 R003 performance/resource verification
Freshness policy: dependency-aware
```

> Issue #63 owns live status, Attempt, owner and result. This `task.md` owns the stable environment-readiness contract.

## Goal

Prove that the real Ubuntu ARM64 phone is usable for **functional development and verification** without prematurely benchmarking it:

```text
trusted target identity
→ required functional tool/runtime inventory
→ exact Functional Baseline build
→ isolated Gateway start / HTTP smoke / stop
→ bounded public-network route classification
→ durable Evidence in GitHub
```

This Task answers **“can we use this target for functional work?”** It does not answer **“is the target fast/cool/stable enough for long-running production?”**

## Current durable Evidence

Before publication the Coordinator reran the accepted `target-runner-smoke`:

```text
run: 32727443950
job: 98053402258
runner: ubuntu-arm64-target-phone
runner OS/arch: Linux / ARM64
kernel arch: aarch64
runtime uid: 999 gateway-runner
workspace: /home/gateway-runner/actions-runner/_work/...
sudo: absent
temporary workspace create/cleanup: PASS
production Vault path: not present
```

The job was accepted by the phone and completed the trusted scheduling/security boundary successfully.

Worker must still perform the current interactive checks below; do not treat the historical smoke as proof that every tool/network condition remains unchanged.

## Canonical Sources

Read before execution:

- `AGENTS.md`
- Issue #63 and relevant comments
- `docs/planning-priority.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/runner-execution-architecture.md`
- `docs/security.md` Target Runner sections
- Issue #1 Final Acceptance
- Issue #21 Final Acceptance
- Issue #36 current draft contract
- `gateway-core/Cargo.toml`
- `gateway-core/src/bin/r001-server.rs`

## Invariants

1. `gateway-runner` remains low privilege; do not grant root/sudo/ADB/Vault/production Secret authority.
2. Do not install packages with root/sudo during this Task.
3. Test work stays in an isolated directory under the user's development/Runner workspace, never `/var/lib/web-media-gateway` production state.
4. Do not change CPU governor, thermal policy, Android/kernel settings, firewall or proxy configuration to manufacture a PASS.
5. Do not run sustained performance scenarios. No 30/60-minute soak, CPU/RSS/temperature benchmark, throughput benchmark, remux endurance or transcode benchmark belongs here.
6. Missing `ffmpeg` or Chromium is environment inventory Evidence; do not turn it into #9 performance Evidence.
7. Network tests are public/no-login only. No Cookie, Authorization, CAPTCHA automation, fingerprint spoofing, proxy rotation or access-control bypass.
8. If a proxy is present, record the route class safely. A proxy-mediated Bilibili result must not be reported as ordinary/direct #36 Evidence.
9. Do not change Gateway/R007/R001/R008 semantics in this Task.
10. Worker does not start #36/#23 or a Bilibili E2E automatically.

## In Scope

- current target identity and privilege/workspace boundary;
- required tool/runtime inventory;
- exact Functional Baseline checkout/build;
- bounded local Gateway start/health/UI-route smoke;
- deterministic stop/cleanup;
- sanitized network/proxy route inventory;
- direct public HTTPS reachability classification;
- bounded direct reachability check of frozen Bilibili sample `BV14V411W7r5` only to decide whether this phone/network may be eligible for later #36 execution;
- GitHub Issue Evidence/reporting.

## Out of Scope

- R003 CPU/RSS/load/temperature/throughput measurement;
- 5/30/60-minute checkpoints;
- long-running Direct/Remux/Chromium resource measurement;
- installing or tuning FFmpeg/Chromium;
- merging the Bilibili plugin;
- claiming #36 J3 C3/C4 ResolvedMedia/navigation Evidence;
- login/authenticated Bilibili;
- TV verification;
- production deployment/service management.

## Claims

```text
C1 — Target identity / safety
The current phone environment is the expected Linux ARM64 target and the functional worker operates without weakening the accepted low-privilege/workspace/Vault boundary.

C2 — Functional toolchain inventory
The Task records exact availability/version of required functional tools and optional later runtimes without installing or silently substituting them.

C3 — Gateway functional baseline
The exact frozen Functional Baseline can be checked out/built and the Gateway can start on an isolated loopback test port, serve bounded health/UI routes, stop, and leave no Task-owned process behind.

C4 — Public-network route classification
The target's direct public HTTPS route and any configured proxy path are distinguished. The frozen Bilibili page receives a bounded direct/no-proxy reachability classification without Cookie/login/bypass behavior.

C5 — Downstream environment decision
Evidence is sufficient to say whether the phone is READY, READY_WITH_GAPS or NOT_READY for functional work and separately whether it is eligible as a normal-network host candidate for #36.
```

Research-style claim results use only `PASS | CONDITIONAL PASS | FAIL | BLOCKED` in reports.

## Verification Procedure

### J0 — Current identity / security read-back

Record:

```bash
date -u
uname -a
uname -m
id
id -u
pwd
printf 'HOME=%s\n' "$HOME"
printf 'TMPDIR=%s\n' "${TMPDIR:-}"
```

Verify:

- architecture is `aarch64`/ARM64;
- runtime work is not executed as root;
- current work directory does not overlap `/var/lib/web-media-gateway`;
- do not print credential files or token values.

Reference the fresh trusted smoke run/job in the final report.

### J1 — Functional tool/runtime inventory

Record `command -v` plus bounded version output for:

Required for the current source-build functional path:

```text
bash
git
curl
python3
cargo
rustc
```

Inventory for later functional/compatibility work:

```text
ffmpeg
chromium | chromium-browser | google-chrome | google-chrome-stable
node
```

Also record bounded disk/memory availability as **capacity inventory only**, not a performance result:

```bash
df -h "$HOME" /tmp 2>/dev/null || true
free -h 2>/dev/null || true
```

Do not install missing tools in this Attempt. Missing required-now tools produce a concrete blocker/gap. Missing optional-later tools are recorded for downstream planning.

### J2 — Exact Functional Baseline build/start/stop smoke

Use an isolated worktree/directory. Fetch/checkout exactly:

```text
f6f6096c32b01c4cad2cf8d7e717807ecb26e033
```

Verify checkout identity before build.

Build the current Gateway server entry:

```bash
cargo build -p gateway-core --bin r001-server
```

Start only a loopback test instance using a non-production port, expected:

```text
R001_BIND_ADDR=127.0.0.1
R001_PORT=18789
```

Use a bounded process lifetime and test only local product routes such as:

```text
/healthz
/
/control
/display?profile=tv
```

Requirements:

- no production Vault/Secret/profile configuration;
- no root/sudo;
- no public/LAN bind in this Task;
- retain only bounded non-secret diagnostics;
- stop the exact test process and verify no Task-owned Gateway process remains;
- cleanup the isolated worktree/runtime created by this Task.

If the frozen baseline cannot build/start because of a concrete target/toolchain condition, preserve Evidence and report it; do not patch product code inside this verification Task.

### J3 — Network / proxy / Bilibili-host eligibility classification

First record only sanitized route metadata. For proxy environment variables (`HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY`), report presence plus scheme/host/port with credentials/query redacted. Do not copy secret-bearing proxy URLs into GitHub.

Check a bounded direct/no-proxy public HTTPS request using `curl --noproxy '*'` and strict connect/overall timeouts.

Then perform a bounded **reachability-only** direct/no-proxy request to the frozen public Bilibili page:

```text
https://www.bilibili.com/video/BV14V411W7r5/
```

Rules:

- no Cookie or Authorization;
- no login;
- no browser/device fingerprint spoofing;
- no CAPTCHA/challenge automation;
- no residential/proxy rotation;
- no use of the local proxy to turn a direct-site failure into PASS;
- do not retain media payloads or signed media URLs.

Record only status/error class and route class, for example:

```text
direct public HTTPS: reachable | blocked
direct Bilibili sample: HTTP 2xx/3xx | HTTP 412 | DNS error | timeout | other bounded status
configured/default proxy present: yes/no + sanitized endpoint class
BILIBILI_HOST_ELIGIBLE_FOR_#36: yes | no
```

`BILIBILI_HOST_ELIGIBLE_FOR_#36=yes` requires the unchanged sample to be normally retrievable through the direct/non-bypass route. A 412/challenge/proxy-only result is `no` and is not a failure of the Site Plugin.

This J3 is only a Publication-Gate prerequisite check for #36; it is **not** #36's real `ResolvedMedia`/navigation Evidence.

## Result Criteria

### PASS

- C1 target/safety boundary passes;
- all required-now tools for the frozen source-build path are available;
- exact Functional Baseline builds, starts, serves bounded local routes, stops and cleans up;
- direct public HTTPS is usable;
- Bilibili host eligibility is explicitly classified.

Bilibili eligibility may be `no` while the general environment Task is still PASS; in that case #36 must use another permitted host/network.

### CONDITIONAL PASS

The target is usable for functional work with a bounded, explicit environment gap that does not require weakening security, for example an optional later runtime such as Chromium/FFmpeg is missing. The condition must state which downstream work is affected.

### FAIL

The target's current environment fundamentally violates accepted safety boundaries or cannot perform the frozen basic functional path in a way that requires architectural/security weakening rather than a normal environment fix.

### BLOCKED

Examples:

- target no longer reachable;
- required source-build tool is absent and provisioning must be handled by a separate approved environment operation;
- checkout/build cannot proceed because network/storage/toolchain is unavailable;
- safe cleanup cannot be guaranteed;
- required GitHub write/report capability is unavailable.

## Success Criteria

1. Current target identity/safety Evidence is recorded and consistent with accepted runner boundaries.
2. Required-now and optional-later tool inventory is explicit with versions/absence.
3. Exact Functional Baseline SHA is verified before build.
4. Gateway local build/start/route/stop/cleanup smoke is complete or a concrete blocker is preserved.
5. Proxy/direct network classes are not conflated.
6. Direct Bilibili sample reachability is classified without bypass behavior.
7. `BILIBILI_HOST_ELIGIBLE_FOR_#36` is explicit and evidence-backed.
8. No sustained performance result is produced or inferred.
9. Worker posts standard `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, updates status accordingly, releases ownership and stops.

## Freshness Contract

Policy: `dependency-aware`.

Frozen Functional Baseline for this Task is:

```text
f6f6096c32b01c4cad2cf8d7e717807ecb26e033
```

Later `main` movement does not automatically invalidate environment Evidence.

Coordinator Review should classify freshness as:

- `UNRELATED` when main changed only in docs/tasks or unrelated plugin work;
- `INTEGRATION_OVERLAP` if accepted changes alter Gateway build/start/runtime requirements relevant to this smoke;
- `SEMANTIC_AUTHORITY` / `CONTRACT_INVALIDATING` only when accepted architecture/security authority changes the environment boundary itself.

Worker must not silently substitute moving `main` for the frozen baseline.

## Evidence Contract

Final report must include:

```text
Attempt:
Worker/environment:
Target identity:
Fresh target-runner-smoke run/job:
Functional Baseline SHA:
Checkout SHA verified:
Required-now tool inventory:
Optional-later tool inventory:
Disk/memory inventory (non-performance):
Gateway build result:
Gateway start/bind/port:
/healthz result:
/ result:
/control result:
/display result:
Gateway stop/cleanup result:
Proxy variables sanitized classification:
Direct public HTTPS result:
Direct Bilibili sample result:
BILIBILI_HOST_ELIGIBLE_FOR_#36: yes | no
Claims C1-C5:
Environment result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Downstream gaps:
Secret/sensitive-data scan:
```

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ J0-J3
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

If blocked:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must not set `status:done`, close #63, execute #36/#23, run #9 performance scenarios, install packages with privilege, or automatically start another Task.
