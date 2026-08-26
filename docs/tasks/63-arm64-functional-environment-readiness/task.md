# Task — ENV-ARM64-READY Functional Environment Readiness

## Metadata

```text
GitHub Issue: #63
Task ID: ENV-ARM64-READY
Task kind: verification / environment readiness
Planning Base: 9fb6b25bc7781e1396c4e979454df962de43090d
Functional Baseline: 9fb6b25bc7781e1396c4e979454df962de43090d
Session Bootstrap: docs/tasks/63-arm64-functional-environment-readiness/prompt.md
Downstream Handoff: docs/tasks/handoffs/ubuntu-arm64.md
Preferred worker: ubuntu-arm64
Eligible environment: env:ubuntu-arm64
Required capabilities: github-read-write, arm64-target-runtime, interactive-linux-debug, rust-build, process-control, functional-network-diagnostic, evidence-authoring
Target: Ubuntu ARM64 phone
Existing infrastructure authority: #1 INFRA-001 ACCEPTED; #21 INFRA-002 ACCEPTED
Accepted environment repair: #64 ENV-ARM64-RUST Final Accepted
Downstream functional consumers: #67 GENERIC-YTDLP-BILIBILI-REAL and later real Bilibili Web E2E
Explicitly not owned here: #9 R003 performance/resource verification
Freshness policy: dependency-aware
```

> Issue #63 owns live status, Attempt, owner and result. This `task.md` owns the stable environment-readiness contract.

## Contract Revision R2

Issue #63 Attempt 1 correctly BLOCKED because `gateway-runner` lacked `cargo`/`rustc`. Issue #64 now owns and has completed that provisioning repair.

The earlier #63 Task also referenced #36/#23 as downstream Bilibili consumers. Those Tasks were later closed `not_planned` during product-route re-planning. The current real-site consumer is #67, through the accepted #66/#73 generic-ytdlp route.

Because accepted runtime/security work landed after the original frozen baseline, the functional build smoke is refreshed to the latest accepted runtime merge before the subsequent docs-only planning commits:

```text
Functional Baseline: 9fb6b25bc7781e1396c4e979454df962de43090d
```

This revision does **not** turn #63 into a Bilibili playback Task and does not add performance scope. It only makes the environment smoke representative of the current accepted runtime and removes superseded #36/#23 routing.

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

## Reusable Durable Evidence

Accepted/reusable evidence already established before Attempt 2:

```text
trusted runner smoke:
  run: 32727443950
  job: 98053402258
  runner: ubuntu-arm64-target-phone
  Linux / ARM64 / uid 999 gateway-runner
  isolated workspace / no sudo / temp cleanup PASS

#63 Attempt 1:
  target identity/safety: PASS
  direct public HTTPS: PASS
  frozen Bilibili page direct/no-proxy: HTTP 200
  capacity inventory only: recorded
  blocker: cargo/rustc absent

#64 Final Accepted:
  user-owned cargo/rustup under /home/gateway-runner
  cargo/rustc usable non-interactively
  rustc 1.98.0 >= repository minimum 1.85
  Runner.Listener PATH includes /home/gateway-runner/.cargo/bin
  uid 999 / zero capability sets / no sudo-admin
  runner online/idle/claimable
```

Attempt 2 should re-read current identity/toolchain before the build and should re-check the bounded direct network classification if practical, but it need not pretend the already accepted evidence never existed.

## Canonical Sources

Read before execution:

- `AGENTS.md`
- Issue #63 and relevant comments, including Attempt 1 blocker and Coordinator revision
- Issue #64 Final Acceptance
- `docs/product-roadmap.md`
- `docs/planning-priority.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/runner-execution-architecture.md`
- `docs/security.md` Target Runner sections
- Issue #1 Final Acceptance
- Issue #21 Final Acceptance
- Issue #67 current draft contract
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
8. If a proxy is present, record the route class safely. A proxy-mediated Bilibili result must not be reported as ordinary/direct real-site Evidence.
9. Do not change Gateway/R007/R001/R008 semantics in this Task.
10. Worker does not start #67/#68 or #9 automatically.

## In Scope

- current target identity and privilege/workspace boundary;
- required tool/runtime inventory after #64;
- exact Functional Baseline checkout/build;
- bounded local Gateway start/health/UI-route smoke;
- deterministic stop/cleanup;
- sanitized network/proxy route inventory;
- direct public HTTPS reachability classification;
- bounded direct reachability check of frozen Bilibili sample `BV14V411W7r5` only as host/network eligibility evidence for later #67;
- GitHub Issue Evidence/reporting.

## Out of Scope

- R003 CPU/RSS/load/temperature/throughput measurement;
- 5/30/60-minute checkpoints;
- long-running Direct/Remux/Chromium resource measurement;
- installing or tuning FFmpeg/Chromium;
- real generic-ytdlp extraction of Bilibili;
- claiming #67 ResolvedMedia compatibility Evidence;
- login/authenticated Bilibili;
- TV verification;
- production deployment/service management.

## Claims

```text
C1 — Target identity / safety
The current phone environment is the expected Linux ARM64 target and the functional worker operates without weakening the accepted low-privilege/workspace/Vault boundary.

C2 — Functional toolchain inventory
The required source-build tools, including #64-provisioned cargo/rustc, are available non-interactively and exact versions/optional gaps are recorded.

C3 — Gateway functional baseline
The exact frozen Functional Baseline can be checked out/built and the Gateway can start on an isolated loopback test port, serve bounded health/UI routes, stop, and leave no Task-owned process behind.

C4 — Public-network route classification
The target's direct public HTTPS route and any configured proxy path are distinguished. The frozen Bilibili page receives a bounded direct/no-proxy reachability classification without Cookie/login/bypass behavior.

C5 — Downstream environment decision
Evidence is sufficient to say whether the phone is READY, READY_WITH_GAPS or NOT_READY for functional work and separately whether the host/network is eligible for later #67 real-site verification.
```

Research-style claim results use only `PASS | CONDITIONAL PASS | FAIL | BLOCKED` in reports.

## Verification Procedure

### J0 — Current identity / security read-back

Record bounded non-secret identity, architecture, workspace and current Runner state. Confirm runtime work is not root and does not overlap production state. Reference the accepted runner smoke and #64 Final Acceptance.

### J1 — Functional tool/runtime inventory

Record `command -v` plus bounded version output for required current tools:

```text
bash
git
curl
python3
cargo
rustc
```

Inventory optional/later runtimes without installing them:

```text
ffmpeg
chromium | chromium-browser | google-chrome | google-chrome-stable
node
```

Also record bounded disk/memory availability as **capacity inventory only**, not a performance result.

### J2 — Exact Functional Baseline build/start/stop smoke

Use an isolated worktree/directory and checkout exactly:

```text
9fb6b25bc7781e1396c4e979454df962de43090d
```

Verify checkout identity before build.

Build:

```bash
cargo build -p gateway-core --bin r001-server
```

Start only an isolated loopback test instance, expected:

```text
R001_BIND_ADDR=127.0.0.1
R001_PORT=18789
```

Test bounded local routes:

```text
/healthz
/
/control
/display?profile=tv
```

Requirements:

- no production Vault/Secret/profile configuration;
- no root/sudo;
- no public/LAN bind;
- bounded non-secret diagnostics only;
- stop the exact test process and verify no Task-owned Gateway process remains;
- cleanup the isolated worktree/runtime.

If the exact baseline cannot build/start because of a concrete target/toolchain condition, preserve Evidence and report it; do not patch product code inside this verification Task.

### J3 — Network / proxy / Bilibili-host eligibility classification

Record sanitized proxy metadata only. Check bounded direct/no-proxy public HTTPS and the unchanged frozen public Bilibili page:

```text
https://www.bilibili.com/video/BV14V411W7r5/
```

Rules:

- no Cookie or Authorization;
- no login;
- no fingerprint spoofing;
- no CAPTCHA/challenge automation;
- no residential/proxy rotation;
- no use of the local proxy to turn a direct-site failure into PASS;
- no media payload or signed media URL retention.

Record only route/status/error classes, including:

```text
direct public HTTPS: reachable | blocked
direct Bilibili sample: bounded HTTP/error class
configured/default proxy present: yes/no + sanitized endpoint class
BILIBILI_HOST_ELIGIBLE_FOR_#67: yes | no
```

This J3 proves only host/network eligibility. It is **not** #67 generic-ytdlp `ResolvedMedia` Evidence.

## Result Criteria

### PASS

- C1 target/safety boundary passes;
- all required-now tools are available;
- exact Functional Baseline builds, starts, serves bounded local routes, stops and cleans up;
- direct public HTTPS is usable;
- Bilibili host eligibility is explicitly classified.

Bilibili eligibility may be `no` while the general environment Task is still PASS; in that case #67 must use another permitted host/network.

### CONDITIONAL PASS

The target is usable for functional work with a bounded optional/later environment gap that does not require weakening security, for example Chromium/FFmpeg missing. State which downstream work is affected.

### FAIL

The target environment violates accepted safety boundaries or cannot perform the frozen basic functional path without architectural/security weakening.

### BLOCKED

Examples include target unreachability, required toolchain no longer available, checkout/build unavailable, cleanup cannot be guaranteed, or required GitHub reporting unavailable.

## Success Criteria

1. Current target identity/safety Evidence is consistent with accepted runner boundaries.
2. Required-now and optional-later tool inventory is explicit.
3. Exact Functional Baseline SHA `9fb6b25...` is verified before build.
4. Gateway local build/start/route/stop/cleanup smoke is complete or a concrete blocker is preserved.
5. Proxy/direct network classes are not conflated.
6. Direct Bilibili sample reachability is classified without bypass behavior.
7. `BILIBILI_HOST_ELIGIBLE_FOR_#67` is explicit and evidence-backed.
8. No sustained performance result is produced or inferred.
9. Worker posts standard `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, updates status accordingly, releases ownership and stops.

## Freshness Contract

Policy: `dependency-aware`.

Frozen Functional Baseline for Attempt 2 is:

```text
9fb6b25bc7781e1396c4e979454df962de43090d
```

This is the accepted runtime merge containing #66. Subsequent main movement through #73 Task Package / Roadmap documentation is unrelated unless accepted runtime/security code changes again before exact execution.

Coordinator Review should classify later movement as:

- `UNRELATED` for docs/tasks/planning only;
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
BILIBILI_HOST_ELIGIBLE_FOR_#67: yes | no
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

Worker must not set `status:done`, close #63, execute #67/#68, run #9 performance scenarios, install packages with privilege, or automatically start another Task.
