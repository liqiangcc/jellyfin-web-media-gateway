# Task — R006-RUNTIME-FUNCTIONAL-PREP

## Metadata

```text
GitHub Issue: #75
Task ID: R006-RUNTIME-FUNCTIONAL-PREP
Task kind: implementation + deterministic/hosted functional verification
Planning Base: 826d02c22105ee1877ae79706d2cb03112f995a9
Preferred worker: cloud-codex
Eligible environment: env:cloud
Parent umbrella: #27 R006-DESIGN
Accepted foundation: #33 R006-CONTRACT-PREP Final Accepted
Accepted security authority: #14/R008
Performance authority: #9 R003-TARGET (explicitly not required here)
Freshness policy: dependency-aware
```

> #75 proves that the accepted generic BrowserWorker/BrowserEvent contracts can drive a real Chromium process in a bounded hosted environment. It does not claim phone performance, real-site semantics, login success or production placement.

## Context

#33 Final Acceptance established target-neutral contracts in `gateway-core/src/browser.rs`:

```text
BrowserWorker
BrowserCommand / BrowserEvent
BrowserSession / BrowserStatus
R008NavigationPolicy
ProfileAttachmentRef
NativePanelSession / short-lived token
FakeBrowserWorker
```

The current implementation is deterministic/in-memory only. #27 has been replanned so functional Chromium viability does not hard-depend on #9 resource/thermal/soak Evidence.

## Goal

Add the smallest real Chromium-backed BrowserWorker runtime that proves the generic contract can execute against an actual browser process while preserving security, lifecycle and site-neutral boundaries.

Target shape:

```text
Gateway generic BrowserWorker contract
→ ChromiumBrowserWorker / equivalent runtime adapter
→ bounded Chromium process
→ direct browser automation channel (CDP or equivalent)
→ generic navigation/input/status/events
→ deterministic close/crash/timeout cleanup
```

No concrete-site interpretation is added.

## Runtime implementation requirements

### A. Real Chromium process

Use a real installed Chromium/Google Chrome binary discovered from a bounded allowlist such as:

```text
google-chrome-stable
google-chrome
chromium
chromium-browser
```

Requirements:

- caller cannot provide an arbitrary executable path/argv;
- runtime owns a bounded, explicit launch argument set;
- use a Task-owned temporary user-data-dir/profile when no Vault attachment is involved;
- browser must be headless for hosted verification unless a stronger deterministic reason exists;
- disable or avoid features that would grant unrelated filesystem/device/network authority where practical;
- process group/child cleanup is deterministic;
- raw Chromium stdout/stderr is bounded/sanitized and must not become a Secret/content log channel.

If no allowed browser is available in the required hosted verification environment, report BLOCKED rather than downloading/running an unpinned arbitrary browser.

### B. BrowserWorker contract seam

Implement the accepted `BrowserWorker` trait or the smallest current-authority equivalent.

Because current BrowserSession/ID constructors are intentionally private to Core, minimal trusted-runtime constructors/factories may be added to `gateway-core/src/browser.rs` only when required for an external/runtime implementation.

Rules:

- do not weaken opaque ID/token/profile boundaries;
- do not expose raw token/profile values through Debug/logging;
- do not add site identifiers/DOM selectors/login rules;
- FakeBrowserWorker remains valid and existing #33 conformance tests continue passing.

### C. Browser automation channel

Use a direct generic browser control channel such as Chrome DevTools Protocol (CDP) or equivalent.

Minimum supported functional operations:

- open session;
- navigate to an allowed public URL after `R008NavigationPolicy` authorization;
- observe generic loading/URL/title/ready facts and emit versioned BrowserEvents;
- send at least bounded key/text/pointer/submit input through generic BrowserInput mapping or explicitly classify unsupported input kinds;
- query status;
- poll ordered events;
- close session;
- detect browser process exit/crash;
- enforce timeout/cancel/cleanup.

The runtime must not evaluate site-specific selectors or interpret login/media state.

### D. R008/navigation boundary

The accepted `R008NavigationPolicy` remains the pre-navigation authorization boundary.

Requirements:

- caller-provided URL is validated before browser navigation begins;
- private/loopback/metadata/reserved targets rejected by R008 remain denied;
- redirects/final browser URL observations must not be treated as authorization to access private targets; if the runtime cannot enforce equivalent per-navigation/redirect policy yet, the Task must fail closed on unsupported redirect behavior and document the bounded limitation;
- no caller proxy configuration, proxy rotation or bypass authority;
- do not weaken R008 to make Chromium work.

Hosted fixture servers used for deterministic browser interaction are allowed only if routed through an explicitly controlled test-only policy/harness that cannot become production `public_web` behavior.

### E. Native Panel functional seam

This Task does **not** need to implement a full user-facing remote desktop/panel UI. It must only prove the accepted NativePanel control/session boundary can be attached to a real BrowserWorker lifecycle without becoming a second authority.

At minimum prove one of:

1. real BrowserWorker session can be wrapped by existing NativePanel session/control-token logic and bounded input reaches that worker; or
2. if current #33 implementation is structurally coupled to FakeBrowserWorker, introduce the smallest generic panel coordinator seam needed for both fake and real workers.

Rules:

- deny-by-default PanelPermissions remain unchanged;
- no clipboard/file upload/audio permission grant in this Task;
- panel disconnect/crash cannot stop or mutate an unrelated PlaybackSession;
- no browser profile download/exposure.

## Primary file ownership

Expected #75 ownership is primarily:

```text
gateway-core/src/browser.rs                     # minimal trusted runtime seam only
gateway-core/src/browser_chromium.rs or equivalent new runtime module
browser-runtime-specific tests / hosted scripts/workflows
workspace/Cargo dependency changes required only by Chromium/CDP runtime
```

To preserve parallelism with #71 SITE-NAVIGATION-PREP:

- do **not** modify `site-adapter-api/**`;
- do not add SiteAdapter navigation/previous/next types;
- do not modify `gateway-core/src/source_session.rs` or navigation command semantics;
- avoid `gateway-core/src/playback.rs` changes.

If a hard dependency on those surfaces appears, BLOCK/SPLIT rather than crossing ownership silently.

## Architecture / security invariants

1. Browser Worker is generic runtime only; no Bilibili/YouTube/site selector/API/login-success logic.
2. Site Plugin remains responsible for interpreting BrowserEvent into Source/Account/NativePanel state.
3. Browser Worker is not Playback authority and cannot mutate PlaybackSession.
4. Vault/profile Secret ownership remains outside this Task; no raw Cookie/profile DB handling.
5. R008 remains network policy authority; no browser proxy/bypass authority.
6. Panel control token/permissions remain short-lived/opaque/deny-by-default.
7. Raw input text, password/code-like values, DOM content and page body are not normal logs/artifacts.
8. No DRM capture/media extraction workaround.
9. No performance/capacity/phone-placement claim; #9 remains authoritative.
10. No real-site login or Bilibili semantic Evidence.

## Claims

```text
B1 — Real Chromium lifecycle
A real allowed Chromium/Chrome process can be launched, observed and deterministically closed through the accepted runtime adapter.

B2 — Generic BrowserWorker behavior
The real runtime implements the accepted open/navigate/status/events/input/close contract without site-specific knowledge.

B3 — R008 navigation boundary
Navigation authorization remains R008-controlled and forbidden/private target attempts fail closed without caller proxy/bypass authority.

B4 — Event/order safety
Real browser facts are converted into bounded versioned BrowserEvents with monotonic per-session sequence; no raw page/input Secret leakage.

B5 — Failure/lifecycle safety
Timeout, cancel, browser crash/exit and close leave no Task-owned Chromium/helper process or temporary profile behind.

B6 — Native Panel functional compatibility
The accepted NativePanel session/control-token boundary can drive bounded input against a real BrowserWorker session without weakening permissions or becoming Playback authority.

B7 — Existing contract/security preserved
#33 fake-worker/conformance, R008 and affected workspace/security regressions remain passing; no production/browser placement policy is claimed.
```

## Verification

### J1 — Real Chromium hosted smoke

GitHub-hosted Ubuntu on exact Candidate.

Preflight:

- discover an allowed preinstalled Chrome/Chromium binary;
- record browser name/version only;
- no arbitrary installer fallback.

Prove:

- open real session;
- navigate to a deterministic allowed fixture/public-safe page;
- observe ordered NavigationStarted/Loading/NavigationChanged/Ready or equivalent generic events;
- query open status;
- close and verify process/profile cleanup.

### J2 — Input + Native Panel functional seam

Prove against a deterministic page:

- bounded generic input reaches the real browser session;
- event/result classification is generic;
- NativePanel token/session boundary can forward allowed input;
- expired/wrong token/session/permission failure remains denied;
- input value/page body does not appear in durable debug/output evidence.

### J3 — R008 / failure matrix

Prove:

- public allowed navigation path;
- loopback/private/metadata/reserved denial through accepted policy or an equivalent deterministic policy fixture;
- browser crash/kill detection;
- timeout/cancel;
- stale session operation after close rejected;
- no caller proxy/executable/argv override;
- cleanup no stale process/profile.

### J4 — Workspace/regressions

Run exact-Candidate fmt/clippy/test + #33 Browser contract tests + R008 + architecture/security guards.

All required Jobs assert exact Candidate SHA.

## Success Criteria

1. B1-B7 PASS on one exact Candidate.
2. At least one real preinstalled Chromium/Chrome binary is exercised in hosted Evidence.
3. Generic open/navigation/event/input/close path works without site-specific interpretation.
4. R008/private-target and proxy boundaries remain fail-closed.
5. crash/timeout/cancel/close cleanup is deterministic.
6. NativePanel accepted control boundary interoperates with the real worker without permission expansion.
7. No performance/phone viability claim is made.
8. Worker reports and STOPs; it does not start real-site/Auth/NativePanel-site work.

## Evidence Contract

`[EXECUTION REPORT]` must include:

```text
Attempt / worker / environment
Base SHA
Candidate SHA / PR
Browser binary class/version
Runtime module / process launch shape
Automation channel class (CDP/equivalent)
Open/navigate/event/status result
Input result (no raw input value)
NativePanel functional seam result
R008 allowed/denied classifications
Proxy/executable authority denial
Crash/timeout/cancel result
Process/profile cleanup proof
Secret/content leak scan
J1-J4 run/job IDs
Claims B1-B7
Real-site execution: NOT RUN
Performance/capacity claim: NONE
Downstream readiness for future Auth/NativePanel work
```

## Freshness

Semantic authorities:

- `gateway-core/src/browser.rs` / #33;
- R008 navigation/security surfaces;
- Browser runtime-specific modules/dependencies.

#71 SiteAdapter/navigation changes are normally `UNRELATED` if they do not touch Browser/R008 authority. If both candidates touch a shared workspace manifest mechanically, integrate after both deterministic Claims pass; do not merge semantic changes from one Task into the other Worker scope.

## Out of Scope

- Bilibili/real-site DOM interpretation;
- source-site login success or authenticated resolve;
- persistent Vault profile materialization;
- full visual Native Panel/WebRTC/remote desktop UI;
- clipboard/file upload/audio permissions;
- media extraction/DRM capture;
- phone/TV Chromium Evidence;
- CPU/RSS/temperature/soak/performance placement decision (#9);
- SiteAdapter Navigation (#71/#72).

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ J1-J4
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker cannot set status:done, close #75, start Auth/real-site/phone performance tasks, or merge its own PR.