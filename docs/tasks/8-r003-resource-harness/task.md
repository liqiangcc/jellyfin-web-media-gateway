# Task — R003-PREP ARM64 Resource Measurement Harness

## Metadata

```text
GitHub Issue: #8
Parent Goal / Research Item: R003 / P0 Ubuntu ARM64 Resource Baseline
Task / Research ID: R003-PREP
Task kind: implementation
Planning / integration base commit: 2b0a1a0ea95753ff416e41759b7c33823be1b9e0
Session bootstrap prompt: docs/tasks/8-r003-resource-harness/prompt.md
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Preferred worker: cloud
Eligible worker environments after publication: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, workflow-authoring, metrics-harness-authoring
Linked verification task: Issue #9 / R003-TARGET
Hard publication dependency: satisfied — Issue #3 / R001 Final Acceptance; accepted Candidate 42c92db2a380895ec3909cdc9afa847478150eb0, merged to main as 2b0a1a0ea95753ff416e41759b7c33823be1b9e0
Infrastructure state: Issue #1 / INFRA-001 ACCEPTED; Ubuntu ARM64 Target Runner exists
```

> Live status, owner, candidate and results belong in Issue #8. This Task prepares the trusted measurement mechanism; it does not classify R003.

## Goal

Build and review the reproducible resource-measurement harness and trusted target workflow required for Issue #9 to measure the Ubuntu ARM64 phone without weakening the self-hosted Runner security boundary.

The harness must support the canonical R003 scenarios:

- Idle Gateway;
- 1080p Direct HTTP/HLS Proxy;
- optional 4K Direct Proxy when source/network support it;
- FFmpeg Remux using a real copy/remux scenario;
- a short, bounded software-transcode boundary measurement without making transcode a default capability;
- Chromium idle/page-load baseline for later R006 context.

Issue #8 proves that the harness, workflow, artifact schema and scenario controls are trustworthy and reproducible. Real phone CPU/RSS/temperature/throughput conclusions belong only to Issue #9.

## Why / Context

R003 is P0 because a phone that cannot sustain Idle/Direct/Remux within a reasonable low-power envelope invalidates the intended hardware/media strategy.

Target measurements must execute on the accepted Ubuntu ARM64 phone Runner, but unreviewed PR workflow changes must not automatically gain target-device shell authority. Therefore R003 separates:

```text
Issue #8
→ implement/review harness + target workflow
→ merge trusted measurement control plane

Issue #9
→ dispatch trusted workflow
→ execute approved candidate on target phone
→ collect real metrics
→ classify R003
```

The split is based on trust/Evidence lifecycle, not merely on architecture or environment.

## Task Decomposition Decision

```text
Verification mode: separate-task
Linked implementation task: Issue #8 (this Task)
Linked verification task: Issue #9
Decision reason: target-runner workflow definitions must become trusted repository state before they execute candidate code on the phone; the long-running target Evidence has a distinct lifecycle and research result.
```

## Hard Dependency

The R001 publication dependency is satisfied by Issue #3 Final Acceptance:

```text
Accepted R001 Candidate: 42c92db2a380895ec3909cdc9afa847478150eb0
Merged main commit:        2b0a1a0ea95753ff416e41759b7c33823be1b9e0
```

This accepted path/interface is sufficient to define how to build/start the Gateway test instance, bind deterministic media, drive Direct Proxy traffic, identify the test process/ports and clean up. Normal integration with newer `main` is allowed; if later changes invalidate these assumptions, the Worker must report the concrete interface conflict rather than redefining R001/R007 contracts.

## Worker Routing Decision

```text
Harness / workflow implementation / repository integration
→ cloud-codex
→ env:cloud

Portable harness/workflow verification
→ GitHub Actions
→ GitHub-hosted x64 (required), hosted ARM64 optional

Phone-specific metrics proof after #8 acceptance
→ Issue #9 orchestration
→ GitHub Actions
→ Ubuntu ARM64 self-hosted Target Runner
```

Codex Cloud is the Worker/orchestrator, not the Target Runner. Issue #8 must not execute unreviewed heavy target workloads on the phone.

## Work Role

### Implementation

Implement a small, auditable measurement system, expected to include repository equivalents of:

```text
scripts/r003/
  collect-metrics.*
  run-scenario.*
  summarize.*

.github/workflows/
  r003-target-resource.yml
```

Exact file names are implementation-defined.

The implementation must:

1. collect timestamped process/system CPU, RSS, load and network throughput;
2. collect target temperature/thermal-zone data from low-privilege readable sources when available;
3. record battery/charging state, other obvious high-load processes and network path when observable without privilege escalation;
4. record scenario start/end, sample interval, media bitrate/source type, startup latency and process exit/error;
5. support continuous checkpoints at 5/30/60 minutes rather than requiring three unrelated runs;
6. produce machine-readable raw evidence (CSV/JSON/JSONL or equivalent) plus a concise Markdown/text summary;
7. preserve raw samples needed to review trend/slope rather than only reporting averages;
8. expose preflight capability detection for FFmpeg, Chromium, thermal sources and required media/Gateway commands;
9. never silently install packages or elevate privilege on the Target Runner;
10. clean test processes, temp files and ports on success, failure, timeout and cancellation.

### Scenario integrity

- **Idle**: Gateway test instance started, no playback requests; record CPU/RSS/load/temp and context-switch/wakeup proxy data when available.
- **1080p Direct Proxy**: drive a real representative 1080p stream through the R001 Gateway path; record startup, throughput and resource trend.
- **4K Direct Proxy**: optional only when source/network/consumer path can sustain a meaningful 4K test; otherwise Issue #9 records N/A with reason.
- **Remux**: real FFmpeg stream-copy/remux path with separate audio/video or equivalent container incompatibility; do not substitute a no-op process.
- **Software Transcode Boundary**: short bounded experiment only; never promote video transcode to default support because the process starts successfully.
- **Chromium Baseline**: idle + representative page-load baseline when a real target Chromium runtime is present; missing runtime must be explicit, not simulated by hosted Chromium.

## Trusted Target Workflow Requirements

The target workflow is part of the security boundary and must satisfy all of the following:

```text
trigger: workflow_dispatch only (or equivalently trusted manual gate)
runs-on: self-hosted + linux + ARM64 + ubuntu-arm64 + target-device
permissions: contents: read unless a narrower documented exception is required
concurrency: target resource experiment serialized
pull_request automatic trigger: forbidden
candidate: explicit full 40-hex SHA
scenario: enum/strictly validated input, not arbitrary shell
length/duration: bounded/validated
shell interpolation of untrusted input: forbidden
production Gateway/Vault mutation: forbidden
```

The workflow must distinguish two identities:

```text
Harness / workflow SHA
= trusted measurement/control implementation

Candidate SHA
= the R001/Gateway code being measured
```

Both must be recorded in logs/artifacts. If separate checkouts are used, the trusted harness must not be silently replaced by files from the candidate checkout.

Candidate execution on the phone is permitted only after Coordinator explicitly approves that candidate for target proof.

## Target Runner Security Constraints

- final job user remains the dedicated low-privilege Target Runner account;
- no sudo/root requirement in normal R003 execution;
- no production Vault, real browser profile, source-site Cookie/token, Jellyfin API key, SSH private key, Tailscale auth key or host credential;
- test runtime/workspace remains separate from `/var/lib/web-media-gateway/` production state;
- use separate test ports/runtime paths;
- one heavy target job at a time;
- every heavy scenario has timeout/cleanup;
- do not disable thermal safeguards or force frequencies/governors to manufacture benchmark results.

## Verification Claims for R003-PREP

```text
C1: Metrics harness emits timestamped raw CPU/RSS/load/network/temp-or-explicit-unavailable data plus reproducible summary.
C2: Scenario driver has deterministic start/stop/cleanup and bounded duration controls.
C3: Trusted target workflow cannot be automatically triggered by an untrusted PR and validates candidate/scenario inputs before execution.
C4: Workflow records both harness/workflow SHA and measured Candidate SHA.
C5: Hosted tests prove parser/aggregation/trend calculations and failure cleanup without claiming phone metrics.
C6: Harness preflight detects FFmpeg/Chromium/thermal capability gaps and reports them explicitly instead of using privilege escalation or fake substitutes.
C7: Issue #9 can run the canonical 5/30/60 target experiments without inventing new measurement semantics.
```

## Verification Plan

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Evidence |
|---|---|---|---|---|---|---|
| J1 | C1,C2,C5,C6 | github-actions | github-hosted-x64 | runner-self | yes | unit/integration tests over synthetic/proc-like fixtures + summary artifacts |
| J2 | C1-C6 | github-actions | github-hosted-x64 | workflow/static test target | yes | workflow/input/security/schema validation + cleanup/failure tests |
| J3 | C1,C2,C5 | github-actions | github-hosted-arm64 | generic Linux ARM64 | optional | portable harness compatibility only |

Issue #8 does not need to execute a heavy phone benchmark. The target workflow must be merged/reviewed before Issue #9 uses it.

## Success Criteria

1. The accepted Coordinator-approved R001 candidate/interface is the integration base.
2. Metrics harness produces raw samples + reviewable summary for required metric categories.
3. 5/30/60 continuous-checkpoint support exists and cannot be faked by sharded unrelated jobs.
4. Scenario start/stop/cancel cleanup is deterministic and tested.
5. Target workflow obeys the trusted-candidate/manual-dispatch/low-privilege security boundary.
6. Harness/workflow SHA and measured candidate SHA are separately recorded.
7. FFmpeg/Chromium/temperature capability preflight is explicit and produces actionable BLOCKED/available state.
8. J1/J2 required hosted checks pass on the final Issue #8 candidate SHA.
9. Issue #9 has a complete, frozen Evidence schema and does not need to redefine measurement semantics.
10. No R003 target-resource result is claimed by Issue #8.

## Evidence Contract

Each Attempt records:

```text
Attempt:
Issue #8 candidate/PR:
R001 integration candidate:
Hosted workflow/run/jobs:
Metrics schema version:
Sample interval controls:
Duration/checkpoint controls:
Scenario selectors:
Cleanup/failure tests:
Target workflow trust checks:
Harness SHA recording behavior:
Candidate SHA recording behavior:
FFmpeg/Chromium/thermal preflight behavior:
Claim results C1-C7:
```

No Secret or sensitive production path content may enter artifacts.

## Failure / Blocked Handling

BLOCKED when:

- the accepted R001 media path/interface proves insufficient and proceeding would require redefining R001/R007 contracts;
- target workflow security constraints cannot be satisfied without unsafe privilege/PR execution;
- a required repository/Actions capability for harness construction is unavailable.

FAIL when the harness/workflow itself cannot reproducibly measure and cleanup the required scenarios within the frozen boundaries.

Missing FFmpeg/Chromium/thermal access discovered by preflight is an explicit target capability gap for Coordinator routing; do not hide it or install with sudo from the target job.

## Out of Scope

- declaring R003 PASS/FAIL;
- changing R001 media semantics;
- changing R007 Playback authority;
- production Gateway deployment;
- CPU governor/root tuning;
- real account/site Secret use;
- R002 TV behavior;
- R004 Jellyfin;
- turning software transcode into default capability.

## Deliverables

- reviewed metrics/scenario harness;
- trusted target workflow candidate/PR;
- hosted J1/J2 evidence;
- frozen artifact/summary schema;
- explicit target preflight capability reporting;
- executable instructions/entry for Issue #9.

## Completion Protocol

Worker follows `docs/tasks/issue-lifecycle-protocol.md`: one Attempt → `[EXECUTION REPORT]` → `status:review` → release owner → STOP. Coordinator ACCEPT/merge means the measurement control plane is trusted and ready for target verification; it does not mean R003 passed.