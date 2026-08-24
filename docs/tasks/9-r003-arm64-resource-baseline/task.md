# Task — R003-TARGET Ubuntu ARM64 Resource Baseline

## Metadata

```text
GitHub Issue: #9
Parent Goal / Research Item: R003 / P0 Ubuntu ARM64 Resource Baseline
Task / Research ID: R003-TARGET
Task kind: verification
Planning base commit: 2f3ec8dd279b62d7f2e6c1f73ecb7f1a37f0c649
Session bootstrap prompt: docs/tasks/9-r003-arm64-resource-baseline/prompt.md
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Preferred worker: web
Eligible worker environments after publication: env:web-gpt
Required capabilities: github-read-write, actions-dispatch, actions-log-artifact-read, metrics-analysis, target-proof-review
Linked implementation task: Issue #8 / R003-PREP
Execution plane: github-actions
Runner class: ubuntu-arm64-self-hosted
Target: ubuntu-arm64-phone
Required runner labels: self-hosted, linux, ARM64, ubuntu-arm64, target-device
Infrastructure dependency: Issue #1 / INFRA-001 ACCEPTED
Hard publication dependencies: Issue #8 Coordinator-accepted/merged trusted harness+target workflow; a specific stable R001 candidate/deployment approved for target proof; target Runner available
```

> The Web Worker owns this Issue and orchestrates GitHub Actions. The phone Runner is the execution backend/target and does not claim the Issue.

## Goal

Measure the real Ubuntu ARM64 phone under canonical R003 scenarios and produce a trusted resource-feasibility classification:

```text
PASS
CONDITIONAL PASS
FAIL
BLOCKED
```

The result must answer whether the phone can remain a low-power, long-running Gateway for the intended Direct/Remux-first media strategy, and what concrete limits are required for media path, concurrency, bitrate or browser-worker use.

This is target Evidence. Generic hosted ARM64, desktop machines, theoretical estimates or short smoke tests cannot replace it.

## Hypothesis

The Ubuntu ARM64 phone can sustain the primary Gateway media paths with stable CPU/RSS/temperature behavior and sufficient throughput for long-running use, while expensive paths can be bounded or marked unsupported rather than hidden behind default software transcoding.

## Why / Context

Canonical R003 requires target measurements for:

- Idle;
- typical 1080p Direct HTTP/HLS Proxy;
- optional 4K Direct Proxy where meaningful;
- FFmpeg Remux;
- a bounded software-transcode boundary measurement;
- Chromium idle/page-load baseline;
- 5/30/60 minute checkpoints and cold-vs-steady thermal behavior.

R003 is P0. If ordinary Idle or Direct/Remux paths are fundamentally incompatible with the low-power target, hardware/media architecture must be reviewed now rather than deferred to future optimization.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: Issue #8
Linked verification task: Issue #9 (this Task)
Decision reason: the trusted measurement control plane must be reviewed/merged before target execution; continuous phone Evidence has a distinct lifecycle and research classification.
```

## Preconditions / Publication Gate

Before Issue #9 may become `status:ready`:

1. Issue #1 / INFRA-001 remains accepted and the target runner is usable.
2. Issue #8 has Coordinator Final Acceptance; its harness/workflow is in trusted repository state.
3. Coordinator records the exact harness/workflow SHA to be used.
4. Coordinator records the exact R001/Gateway Candidate SHA or deployed test SHA being measured.
5. The R001 candidate has a documented build/start/stop/test-media entry suitable for target proof.
6. The target workflow can isolate the test runtime from production state and use non-production ports/paths.
7. Required test media is legal, non-DRM and suitable for repeatable measurement.

Runtime capability preflight may discover FFmpeg, Chromium or thermal metric gaps. Those are legitimate target findings. They must not be bypassed with sudo/root inside the target job.

## Claims

```text
C1 Idle: Gateway idle resource usage is stable enough for long-running operation; no persistent high CPU/resource growth or unexplained high-frequency polling is observed.
C2 Direct-1080p: A representative 1080p Direct Proxy path sustains playback/throughput for a continuous 60-minute run with 5/30/60 checkpoints and without unbounded RSS growth or uncontrolled thermal escalation.
C3 Direct-4K: When the source/network/consumer setup supports meaningful 4K testing, Direct Proxy remains primarily I/O/network bound or its real limitation is documented; otherwise explicit N/A with reason.
C4 Remux: A real stream-copy/remux workload has measured startup/CPU/RSS/temp/stability and is classified usable, usable-with-limits, unsupported, or blocked by missing runtime.
C5 Transcode boundary: A short bounded software-video-transcode experiment provides enough data to justify whether transcode stays non-default/unsupported; test completion does not imply product support.
C6 Chromium baseline: A real target Chromium idle/page-load resource baseline is recorded when runtime is available; absence is explicit and cannot be substituted by hosted Chromium.
C7 Thermal/resource integrity: raw samples show cold-start vs steady-state behavior, throttling/thermal warnings when exposed, charging state and competing-load context; abort/cleanup leaves no experiment process/resource leak.
C8 R003 decision: final PASS/CONDITIONAL PASS/FAIL/BLOCKED and concrete limits follow the frozen criteria without post-hoc weakening.
```

## Required Scenarios / Jobs

### J0 — Target preflight

Required before heavy runs.

Record:

- target runner name/labels;
- OS/kernel/architecture;
- job uid/groups;
- harness/workflow SHA;
- measured Candidate SHA;
- CPU count/model/frequency info when readable;
- memory total/available;
- thermal-zone sources and trip/throttle signals when readable;
- charging/battery state when readable;
- network interface/path;
- obvious competing high-load processes;
- FFmpeg version/availability;
- Chromium version/availability;
- Gateway build/start prerequisites;
- available disk/workspace.

If temperature cannot be observed through any acceptable low-privilege source, R003 cannot silently claim thermal PASS. Report the gap and let Coordinator decide `BLOCK`/`SPLIT` for a safe read-only metrics capability.

### J1 — Idle continuous baseline

Run the approved Gateway test instance with no playback traffic for a continuous **60 minutes**, recording checkpoints at:

```text
5 min
30 min
60 min
```

Record at minimum process/system CPU, RSS, load, temperature and network/background activity. Context-switch or other wakeup proxy metrics should be recorded when the low-privilege platform exposes them.

### J2 — 1080p Direct Proxy continuous baseline

Use a representative legal 1080p media source through the approved R001 Media Gateway path.

Continuous run: **60 minutes**, with 5/30/60 checkpoints.

Record:

- startup latency;
- source/stream bitrate when known;
- delivered throughput;
- CPU/RSS/load/temp trend;
- errors/reconnects;
- dropped/starved transfer symptoms when observable;
- cleanup after end/abort.

The measurement must drive real bytes through the Gateway. A local no-op loop or direct upstream download that bypasses the Gateway does not satisfy C2.

### J3 — 4K Direct Proxy

Execute only when a meaningful 4K source/network/consumer path is available.

If executed, record the same major metrics and whether CPU or I/O/network becomes the limiting factor. If unavailable, record `N/A` plus the exact missing condition. Do not fabricate 4K by upscaling or synthetic CPU load.

### J4 — FFmpeg Remux

Use a real remux/stream-copy scenario, preferably separate audio/video or a container/codec packaging case that requires remux but not video re-encode.

Target a continuous 60-minute run with 5/30/60 checkpoints when the media/runtime supports it.

Record startup latency, CPU/RSS/temp/throughput/errors and whether the workload remains stable.

If FFmpeg is absent, incompatible or inaccessible to the low-privilege runner, record a concrete capability blocker. Do not install with target-job sudo/root.

### J5 — Software Transcode Boundary

This is a **bounded diagnostic**, not a product capability test.

Use a controlled video-transcode sample for a short fixed window (default 5 minutes or shorter if thermal/throttling safety requires). Record CPU/RSS/temp/throughput and any throttling/overload signal.

Stop immediately when platform thermal protection, critical trip proximity, severe instability or test timeout indicates risk. Do not disable thermal controls or tune governors.

The expected decision may be “software transcode remains unsupported/default-off”. That is a valid R003 result.

### J6 — Chromium baseline

When a real target Chromium runtime exists, record at least:

- idle browser worker baseline;
- representative page load baseline;
- CPU/RSS/temp for a bounded observation window.

If Chromium is not installed/approved on the target, record the gap explicitly. Hosted Chromium is not target evidence.

## Continuous Duration Rule

For J1/J2/J4, the 5/30/60 evidence must come from a continuous run when the claim is “sustained stability”.

```text
60-minute continuous run with checkpoints
!=
three unrelated 5/30/60 jobs
```

Matrix/sharding must not be used to fake continuous soak.

A failed/aborted 60-minute run is valid Evidence and must be preserved.

## Sampling / Artifact Requirements

Use the accepted Issue #8 harness. Raw evidence must include timestamped samples sufficient to inspect trend, not only averages.

Recommended artifact structure (equivalent formats allowed):

```text
r003/<scenario>/metadata.json
r003/<scenario>/samples.csv|jsonl
r003/<scenario>/summary.md
r003/<scenario>/stderr-or-events.log
```

Metadata must identify:

```text
Harness/workflow SHA
Candidate SHA
Scenario
Start/end UTC
Sample interval
Runner name/labels
OS/kernel/arch
Charging state
Network path
Media source type/bitrate (without sensitive signed URL)
FFmpeg/Chromium version when used
Other high-load processes
Abort/timeout reason if any
```

## Interpretation Rules

Canonical docs intentionally do not invent absolute CPU/temperature thresholds before measurement. Therefore R003 must preserve raw data and use observable stability/limit evidence rather than moving the goalposts after the test.

### Stable / acceptable evidence includes

- no monotonic/unbounded RSS growth across the sustained run;
- no sustained CPU saturation that starves the required media throughput;
- temperature reaches a stable operating region or otherwise does not show continuing uncontrolled escalation through the final observation window;
- no platform-reported thermal emergency/forced stop/throttling severe enough to break the media path;
- target delivers the required stream throughput without repeated stalls attributable to Gateway resource exhaustion;
- processes/temporary resources are cleaned after the scenario.

### Risk / failure evidence includes

- Idle stays materially busy because of avoidable polling/work rather than becoming quiescent;
- 1080p Direct cannot sustain required throughput on the target;
- RSS grows cumulatively with duration/bytes without bounded ownership;
- temperature continues climbing without a sustainable plateau and/or platform throttling causes path instability;
- ordinary Direct or Remux repeatedly crashes/gets killed/causes unrecoverable target instability.

The Worker must report the observed values/trends. Coordinator decides final acceptance/result based on frozen criteria and Evidence.

## R003 Result Classification

### PASS

At minimum:

- Idle is stable/light enough for long-running service based on measured trend;
- 1080p Direct Proxy completes the continuous target run without resource leak/thermal runaway/throughput failure;
- Remux is measured and usable within the documented target envelope;
- required temperature data is available and does not reveal an unresolved thermal blocker;
- software-transcode boundary is explicitly classified (often default-off/unsupported is acceptable);
- concrete concurrency/bitrate/browser-worker limits, if needed, are documented.

### CONDITIONAL PASS

The phone remains viable for the intended Gateway with explicit restrictions, for example:

- Direct Proxy is stable but Remux is only viable below documented bitrate/concurrency limits;
- Remux must be disabled/limited while Direct remains healthy;
- Chromium baseline shows only a tightly limited Browser Worker count is acceptable;
- another measured optional path is excluded without invalidating the Direct-first core.

The condition must be evidence-backed and product/architecture-significant. Missing required Evidence alone is not a CONDITIONAL PASS.

### FAIL

Examples:

- Idle behavior itself is incompatible with low-power long-running use;
- representative 1080p Direct Proxy cannot sustain the required workload;
- ordinary Direct/required Remux shows uncontrolled thermal/resource behavior that makes the phone unsuitable under reasonable limits;
- the hardware/media strategy requires architectural change rather than a bounded feature limit.

### BLOCKED

Examples:

- target Runner unavailable;
- trusted harness/workflow unavailable;
- approved R001 candidate cannot run on the target;
- required temperature source inaccessible and no safe approved measurement path exists;
- FFmpeg runtime missing when Remux claim remains required and no Coordinator-approved provisioning path exists;
- artifact/log path cannot retain the required evidence.

Do not convert missing data into PASS.

## Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Duration | Evidence |
|---|---|---|---|---|---|---|---|
| J0 | C7 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | yes | preflight | metadata/log |
| J1 | C1,C7 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | yes | 60m continuous | samples+summary |
| J2 | C2,C7 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | yes | 60m continuous | samples+summary |
| J3 | C3 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | conditional | bounded/meaningful | samples+summary or N/A |
| J4 | C4,C7 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | required for full PASS | 60m continuous | samples+summary |
| J5 | C5,C7 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | yes | short bounded | samples+summary |
| J6 | C6 | github-actions | ubuntu-arm64-self-hosted | ubuntu-arm64-phone | required when runtime available | bounded | samples+summary or explicit gap |

Target jobs must be serialized and use the trusted target workflow from Issue #8.

## Security / Isolation

- target workflow definition comes from trusted main state accepted in Issue #8;
- dispatch identifies an explicit approved full Candidate SHA;
- no automatic `pull_request` target execution;
- no arbitrary shell command input;
- job user remains non-root/no-sudo;
- no production Vault/profile/site/Jellyfin Secrets;
- test instance uses test runtime paths/ports;
- heavy scenario timeout and cleanup are mandatory;
- record competing workload rather than killing unrelated processes to make numbers look better;
- do not change CPU governors, thermal controls or host tuning to manufacture PASS.

## Task Success Criteria

The Verification Task itself is complete when:

1. exact harness/workflow SHA and measured candidate SHA are recorded;
2. J0 is complete and capability gaps are explicit;
3. required feasible target scenarios have raw artifacts and summaries;
4. J1/J2 continuous 60-minute evidence exists unless a preserved failure/blocker terminates the test earlier;
5. Remux receives a real result or a concrete blocker that Coordinator can route—never an assumption;
6. software-transcode boundary receives a bounded real result;
7. Chromium target baseline is measured when runtime is present, otherwise gap is explicit;
8. cleanup/abort behavior is recorded;
9. final R003 `PASS | CONDITIONAL PASS | FAIL | BLOCKED` is assigned using the frozen classification above;
10. concrete limits and `Continue | Change | Defer | Drop` recommendation are recorded.

Coordinator can ACCEPT a correctly executed R003 verification whose research result is `FAIL`; Task acceptance means the Evidence is trusted, not that the hardware hypothesis succeeded.

## Evidence Contract

The Issue #9 `[EXECUTION REPORT]` must include:

```text
Attempt:
Harness/workflow SHA:
Measured candidate SHA:
Actions run/job IDs for J0-J6:
Runner name/labels:
OS/kernel/arch:
Charging state:
Network path:
Sample interval:
Scenario durations:
Idle 5/30/60 summary:
1080p Direct 5/30/60 summary:
4K result or N/A:
Remux 5/30/60 summary / blocker:
Transcode-boundary summary:
Chromium baseline / gap:
Thermal/throttling observations:
RSS/CPU trend observations:
Throughput/startup observations:
Cleanup result:
Artifact references:
Claims C1-C8:
R003 result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Decision: Continue | Change | Defer | Drop
Limitations:
```

Do not put sensitive media URLs, Cookies, Authorization or account data into logs/artifacts.

## Out of Scope

- changing R001 implementation to improve numbers within the same verification Attempt;
- changing R007 Playback semantics;
- production service benchmarking without explicit deployment scope;
- R002 physical TV UX;
- full R006 Browser Worker product decision;
- Jellyfin;
- CPU governor/root tuning;
- treating software video transcode as an expected default feature.

If Evidence reveals an implementation bug in the measured candidate, report it and return to Coordinator; do not silently patch target code during the measurement and continue under the same Candidate SHA.

## Completion Protocol

Web Worker uses GitHub Actions as execution plane, posts one standard `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, moves to `status:review`/`status:blocked`, releases ownership, and stops. Only Coordinator performs research-result acceptance, Task Final Acceptance and closure.