# Task — Diagnose tx-node Chromium navigation failure and premature termination

## Metadata

```text
GitHub Issue: #188
Parent Goal: #68 / R008
Task ID: R008-BROWSER-NAV-TERMINATION
Task kind: combined target diagnostic
Preferred worker: cloud-codex
Environment: env:cloud
Requested model: gpt-5.6-luna, reasoning high
Fast: enabled by user; may be selected
Target: tx-node / gateway-verify
Contract revision base: 46d3a67588f066c8b6a8b9de0e6edf5391aa9ae7
```

## Goal

Run the already-built #188 browser diagnostic on tx-node as quickly as possible and obtain real navigation evidence for the main playback path.

The existing owner-controlled `source-runtime` may remain running. Its presence is **not** a blocker.

This Task does not require a host-wide clean browser state, a separately provisioned browser platform, a new UID/GID, cgroup, network namespace, slot descriptor, or an independent readiness Task.

## Accepted implementation / artifact anchor

Reuse the accepted #219 marker-retention diagnostic implementation unless an actual target failure proves a code correction is required:

```text
Candidate: 4ffcb63ef156d042571c39d0fc15a5b4e30a8b54
PR: #221
Hosted run: 34309034915
Artifact: 10087720720
Artifact size: 4,218,043 bytes
Artifact digest: sha256:a5b9f07eb5ab7d81289539a90574d50a306353d80e4f37f7131e592ab555bcef
Selector: bilibili:BV14V411W7r5:part-2
```

The hosted implementation gate already passed. Do not rebuild or change code before target execution unless the target result demonstrates that a focused correction is necessary.

The accepted implementation includes the #199 stage-marker follow-up, the #203 post-navigation lifecycle diagnostic, the #207 finite navigation-rejection classifier, the #211 transport/failure-precedence correction, the #215 upstream response/socket lifecycle trace, and the #219 bounded marker-retention correction. Issue #219 was accepted after PR #221 merged to `main` at `46d3a67588f066c8b6a8b9de0e6edf5391aa9ae7`; its exact Candidate `4ffcb63ef156d042571c39d0fc15a5b4e30a8b54`, hosted run `34309034915`, and artifact `10087720720` are the anchor above. The accepted follow-ups add finite lifecycle outcomes for fulfilled/rejected navigation, timeout/abort, page/browser termination, and process error/signal termination, allowlisted navigation reasons including the explicit generic rejection class, a guard preventing late opaque callbacks from overwriting successful transport, bounded upstream response start/body start/body completion/socket close/timeout/abort/error markers with finite error classes, and retention of those markers under bounded broker noise by evicting only broker request noise. #199, #203, #207, #211, #215 and #219 produced no target evidence; this revised #188 Task remains responsible for the next target navigation result.

## Fast-path target contract

The Worker may use the existing authenticated Tailscale SSH control path to tx-node. Root/control-plane access may be used when necessary to create or launch the temporary test runtime, but the browser/probe itself should run as the existing `gateway-verify` identity when practical.

Only the following runtime separation is required:

1. **Fresh profile** — create a new temporary Chrome profile for this Attempt. Never reuse the existing `source-runtime` profile.
2. **Separate display** — use a new temporary Xvfb/display when a display is needed. Do not attach to the existing source display.
3. **Separate browser-control endpoint** — if Chrome requires remote debugging/CDP, allocate a different local port/socket for this Attempt. Do not attach to the existing source-runtime CDP endpoint.
4. **Process ownership by launch identity/PID** — record the processes started by this Attempt and clean up those processes only. Do not use broad host-wide `pkill`/wildcard cleanup.

No additional isolation is required for this milestone. In particular, do **not** block execution on dedicated UID/GID creation, systemd units, cgroups, resource budgets, network namespaces, separate DNS/routes, slot descriptors, or formal multi-slot ownership machinery.

## Existing source-runtime rule

The Worker is allowed to observe enough host-level process/listener facts to avoid collisions and select unused temporary resources.

The Worker must not stop or reconfigure the existing `source-runtime` merely to make the host look clean. Existing Chrome, persistent profile, remote-debugging listener, proxy/session processes, or source acquisition services are compatible with this Task as long as the new Attempt does not reuse their profile/display/CDP resources.

If a port/display/profile collision occurs, choose another temporary value and continue.

## Execution sequence

1. Re-read Issue #188 and this Contract. Confirm the accepted Candidate/artifact identity above.
2. Connect to tx-node through the existing control path and confirm Chrome/Node and `gateway-verify` are available.
3. Create a fresh temporary working/profile directory.
4. Start a temporary Xvfb/display if needed.
5. Start the temporary browser/probe runtime using the fresh profile and a non-conflicting local debugging/control endpoint when required.
6. Run the accepted #188 diagnostic with the fixed selector `bilibili:BV14V411W7r5:part-2`.
7. Run one session first. A second session is allowed only when it helps distinguish a transient failure from a repeatable result.
8. Record the bounded diagnostic result and the actual failure/success boundary.
9. Clean up only the temporary browser/Xvfb/processes/files created by this Attempt.
10. Post `[EXECUTION REPORT]` or `[BLOCKER REPORT]` with the concrete result and next product-level action.

## What is no longer a blocker

The following conditions must **not** block #188:

- existing `source-chrome.service`
- existing `source-mcp-gateway.service`
- existing `source-xvfb.service`
- an existing persistent Chrome profile owned by source-runtime
- an existing source-runtime remote-debugging listener
- an existing source-runtime proxy/session process
- absence of #191 clean-slot PASS
- absence of #195 isolated-slot provisioning

## Scope

In scope:

- tx-node target admission
- starting a disposable browser runtime alongside source-runtime
- the existing #188 Chromium navigation diagnostic
- at most two target sessions
- minimal collision avoidance and Attempt-owned cleanup
- a focused code correction only when target evidence requires it

Out of scope for this milestone:

- building a general browser-slot platform
- dedicated users/cgroups/namespaces/resource schedulers
- production-hardening of the temporary runtime
- unrelated architecture/security refactors
- broad cleanup of source-runtime
- production Gateway mutation

## Success criteria

`PASS` when all of the following are true:

1. A fresh temporary browser profile/runtime is started without reusing the existing source-runtime profile/display/CDP endpoint.
2. The #188 accepted diagnostic artifact actually runs on tx-node.
3. At least one bounded navigation result is produced, whether success or a concrete diagnostic failure class.
4. Attempt-created processes/files are cleaned up without stopping the existing source-runtime.
5. The result gives a concrete next step toward #68 rather than another infrastructure-only prerequisite.

A navigation failure is still useful evidence and does not by itself make this Task `BLOCKED`.

Use `BLOCKED` only when the Worker cannot actually start/run the disposable runtime with available host primitives or cannot access the target/artifact at all.

## Relationship to #191 and #195

#191 and #195 are no longer prerequisites for #188 under this fast-path contract.

The clean-slot and fully isolated-slot designs may be revisited later if productionization or repeated interference demonstrates a real need. They must not delay the current main-function bring-up.

## Completion

The Worker must post the exact target result and cleanup state, move #188 to `status:review` on completed target execution or `status:blocked` only for a genuine execution blocker, release ownership, and stop.
