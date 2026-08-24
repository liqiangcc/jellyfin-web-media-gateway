# Task — Web-only Core Feasibility Review

## Metadata

```text
GitHub Issue: #22
Parent Goal / Research Item: Web-only Core P0 Feasibility Gate
Task / Research ID: CORE-FEASIBILITY-REVIEW
Task kind: research / verification synthesis
Planning base commit: 044a990d108804e991227a6e447ff026a765f28d
Candidate commit: n/a (review output/doc changes belong to the live Issue/Attempt)
Session bootstrap prompt: docs/tasks/22-core-feasibility-review/prompt.md
Downstream handoff profile: docs/tasks/handoffs/web-gpt.md
Preferred worker: web
Eligible worker environments after publication: env:web-gpt
Required capabilities: github-read-write, repository-static-analysis, evidence-synthesis, actions-log-artifact-read, canonical-doc-review, research-gate-decision
Hard publication dependencies: Issue #2/R007 Final Acceptance; Issue #3/R001 Final Acceptance; Issue #7/R002-TV Final Acceptance; Issue #9/R003-TARGET Final Acceptance; Issue #14/R008 Final Acceptance
Explicit non-blocking work: R004/#15/#16, R005, R006
```

> Live status, owner, Attempt, exact accepted child Evidence, review output and final Gate result belong in Issue #22.
>
> This Task is a parent feasibility decision. It consumes accepted child Evidence; it does not rerun target experiments or rewrite child Research results.

## Goal

Aggregate the accepted P0 Evidence for R007, R001, R002, R003 and R008, test that the combined evidence is mutually compatible with the canonical Web-only Core product/architecture, and produce one explicit parent Gate decision:

```text
GO
CONDITIONAL GO
NO-GO
```

The Gate must also record the repository research recommendation:

```text
Continue | Change | Defer | Drop
```

and apply any mandatory canonical-document changes required by accepted Evidence before the Gate can be finally accepted.

## Why / Context

`docs/mvp-plan.md` defines the P0 Core evidence aggregation order:

```text
R007
→ R001
→ R002
→ R003
→ R008
→ Core Feasibility Review
```

and requires, before the Web-only Core can be marked technically feasible:

- R001 PASS;
- R002 PASS or product-acceptable CONDITIONAL PASS;
- R003 PASS or clearly limited CONDITIONAL PASS;
- R007 concurrency/re-resolve/handoff authority closure;
- R008 baseline security validation.

The child Tasks intentionally have independent lifecycles and Evidence Authorities. This Task exists to decide the **combined parent proposition**, not to treat several individually green checks as automatically equivalent to a coherent product architecture.

## Task Decomposition Decision

```text
Verification mode: none (evidence synthesis)
Linked implementation task: n/a
Linked verification tasks: #2, #3, #7, #9, #14 as accepted evidence inputs
Decision reason: all required runtime/device/CI Evidence is produced by child Tasks with their own Evidence Authorities. Re-running those experiments here would blur ownership and create stale/duplicate Evidence.
```

If a required child result is missing, reopened or contradicted, Issue #22 is not executable/publishable; do not simulate the missing proof inside this Task.

## Worker Routing Decision

```text
Evidence aggregation / canonical review / decision draft
→ web
→ env:web-gpt

New automated target jobs
→ none

Phone target / physical TV
→ never from this Task
```

The Web Worker prepares the evidence matrix, contradiction analysis, Gate recommendation and any required canonical-doc patch. Coordinator retains final review/acceptance authority under the normal Issue lifecycle.

## Canonical Sources to Read

At minimum:

- `AGENTS.md`
- Issue #22 and all comments
- `docs/requirements.md`
- `docs/architecture.md`
- `docs/implementation-contracts.md`
- `docs/security.md`
- `docs/mvp-plan.md`
- `docs/technical-feasibility-validation.md`
- `docs/runner-execution-architecture.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- Final Acceptance + accepted Evidence for Issue #2 / R007
- Final Acceptance + accepted Evidence for Issue #3 / R001
- Final Acceptance + accepted Evidence for Issue #7 / R002-TV
- Final Acceptance + accepted Evidence for Issue #9 / R003-TARGET
- Final Acceptance + accepted Evidence for Issue #14 / R008
- relevant durable research docs produced by those Tasks

Dynamic live GitHub state overrides historical planning text. If a supposedly accepted child Task has been reopened or has new contradictory Evidence, stop and report the Gate blocker instead of relying on an old PASS.

## Evidence Authority Boundary

The Gate may consume a child result only when:

```text
child Task has Coordinator Final Acceptance
+ exact accepted result/evidence is identifiable
+ no later Coordinator reopen/contradiction invalidates it
+ the claim actually contributes to this Gate
```

The Gate must preserve distinctions between:

```text
child Task execution success
child Research result
parent Gate decision
```

Examples:

- a correctly executed R002-TV Task may be accepted even if its Research result is FAIL; the Gate must consume FAIL as FAIL;
- a correctly executed R003 Task may be accepted with a CONDITIONAL PASS and explicit bitrate/concurrency/remux limits; the Gate must decide whether those limits are compatible with the intended Core;
- R008 may defer absent R006 Browser Worker runtime without weakening the P0 security result for instantiated Core surfaces.

## Required Input Matrix

### R007 — Playback authority

Required parent fact:

- command CAS / request-id / item refresh / display generation / handoff authority is closed and no current integrated Evidence contradicts it.

### R001 — Media path

Required parent fact:

- the accepted Web-only Media Gateway path works for the frozen P0 media scope without exposing upstream Secret or becoming an open proxy.

Limitations such as unsupported DASH/remux/transcode or browser-native HLS scope must remain explicit and be reconciled with R003/product behavior rather than silently forgotten.

### R002 — TV browser UX

Required parent fact:

- real target-TV Evidence is `PASS`, or `CONDITIONAL PASS` whose required initialization/interaction model is explicitly acceptable for the intended product.

If the real result is FAIL because every new playback requires TV interaction or only muted autoplay is reliable, the current Web-only TV experience does not satisfy the Gate.

### R003 — Ubuntu ARM64 target feasibility

Required parent fact:

- real phone Evidence is `PASS`, or `CONDITIONAL PASS` with bounded restrictions that preserve the intended low-power Direct/Remux-first Gateway value.

Missing required target Evidence is not a conditional result. A hardware/media strategy that requires architecture change is not silently accepted.

### R008 — Security baseline

Required parent fact:

- the final integrated P0 surface preserves the accepted Egress/Secret/process/trusted-runner boundaries; no P0 spike succeeds only by disabling SSRF controls, leaking Secret material or trusting unreviewed target execution.

## Claims

```text
C1 — Evidence completeness
All five required P0 child inputs have current Coordinator Final Acceptance and exact durable Evidence references; no required child is silently substituted by theory, hosted simulation or an optional adapter.

C2 — Cross-result consistency
R007/R001/R002/R003/R008 conclusions do not contain an unresolved contradiction that makes the integrated Web-only Core assumptions incoherent.

C3 — User-experience feasibility
The accepted R002 real-TV result satisfies the intended low-interaction audible remote-play product proposition, either directly or under an explicitly product-acceptable one-time/bounded condition.

C4 — Target/hardware feasibility
The accepted R003 phone result preserves a viable low-power long-running Gateway envelope for required Core paths, with every material concurrency/bitrate/remux/browser limit explicit.

C5 — Security feasibility
The accepted R008 result applies to the final integrated P0 surfaces and no accepted P0 result depends on violating canonical Egress/Secret/trusted-execution boundaries.

C6 — Stable Core boundary
No Gate decision requires Jellyfin/R004, a concrete source-site/R005, or Browser Worker/R006 to become mandatory Core dependencies, and no child result forces concrete site/Jellyfin knowledge into Playback/Media Core without a canonical architecture change.

C7 — Canonical impact closure
Every accepted Evidence-driven change to product limits, architecture, contracts, security or implementation sequence is either already represented in canonical docs or is patched by this Task before final Gate acceptance.

C8 — Phase transition integrity
The final result states what may proceed next, what remains deferred/optional, what limits carry forward, and whether Phase 1 can begin without implying R004/R005/R006 are already proven.
```

## In Scope

- read and independently verify accepted child Issue/PR/run/artifact/research references;
- build a concise parent Evidence matrix;
- compare child limitations against requirements/architecture/contracts/security/MVP;
- identify cross-Task contradictions or stale assumptions;
- classify each Gate claim C1-C8;
- produce durable `docs/research/core-feasibility-review.md` (or equivalent explicitly linked from Issue #22);
- when required by Evidence, make the minimal coherent canonical-doc updates across `requirements.md`, `architecture.md`, `implementation-contracts.md`, `mvp-plan.md`, `security.md` and/or ADRs;
- state `GO | CONDITIONAL GO | NO-GO` and `Continue | Change | Defer | Drop`;
- define exact downstream entry/limitations after the Gate.

## Out of Scope

- rerunning R002 physical-TV tests;
- rerunning R003 target-phone soak/resource tests;
- implementing/fixing R001/R007/R008 child defects inside the Gate;
- using R004 Jellyfin to rescue a failed Web-only Core hypothesis;
- implementing R005 real-site support;
- implementing or proving R006 Browser Worker/Native Site Panel;
- inventing new thresholds after seeing results;
- weakening child Success Criteria or rewriting accepted child results;
- full production readiness, release readiness, user auth/RBAC or all Phase-1+ product features.

## Architecture Invariants

- Gateway Core remains PlaybackSession authority.
- Core reaches concrete source behavior through SiteAdapter/Registry boundaries, not site special cases.
- Jellyfin remains optional DisplayAdapter and cannot become required for Web-only Core Gate success.
- EgressPolicy/Secret boundaries remain mandatory, not post-Gate hardening.
- Target Runner is an Evidence execution backend, not production authority or an untrusted PR runner.
- Optional/deferred components are not reported as PASS merely because the Core Gate proceeds.
- Evidence-driven canonical changes must be explicit and ordered according to repository authority; a research review cannot silently override canonical architecture in prose.

## Files Expected to Change

Expected:

- `docs/research/core-feasibility-review.md`

Conditional, only when accepted Evidence requires a real canonical update:

- `docs/requirements.md`
- `docs/architecture.md`
- `docs/implementation-contracts.md`
- `docs/mvp-plan.md`
- `docs/security.md`
- `docs/adr/*`

Avoid unrelated implementation/refactor changes.

## Implementation Requirements

N/A for product code.

For review/document work:

1. Build the child Evidence matrix from live GitHub Final Acceptance, not from memory or old task bodies.
2. Record exact child Issue, accepted candidate/PR where applicable, required run/job/artifact/device Evidence and child Research result.
3. Record every material limitation even when the child result is PASS.
4. Explicitly compare R002 UX conditions and R003 resource limits against current product requirements; do not call them acceptable without explaining why.
5. Explicitly check R008 freshness against the integrated P0 surfaces accepted by the Gate.
6. Check whether R001 limitations and R003 measured media strategy are compatible (for example Direct/Remux-first vs unsupported default transcode).
7. Check that R007 authority remains the only Playback concurrency/handoff authority used by accepted P0 work.
8. Separate Core-blocking conclusions from optional R004/R005/R006 roadmap decisions.
9. Apply required canonical changes atomically/coherently when Evidence invalidates an assumption. Do not patch only one doc and leave known contradictions elsewhere.
10. Record unresolved uncertainty as a Gate blocker or NO-GO input, not optimistic prose.

## Verification Plan

### Execution Plane

```text
Execution plane: none (GitHub evidence review + repository document synthesis)
New runtime jobs: none
Target proof: none
```

No new job is required merely to restate already accepted child Evidence. If review discovers that a child claim actually lacks required proof, return the Gate to blocked/draft routing for that child rather than creating an ad-hoc substitute Job here.

### Claims

C1-C8 above are verified by traceable accepted Evidence plus canonical cross-checks.

### Evidence Matrix Requirement

The durable review must contain, for each child:

```text
Research Item / Issue
Final Acceptance reference
Accepted candidate / merge SHA when applicable
Required run/job/artifact/device Evidence references
Research result
Material limitations / conditions
Parent-Gate implication
```

and a separate cross-cutting matrix for:

```text
Playback authority
Media protocol/capability boundary
TV interaction model
Target resource/thermal envelope
Security/Egress/Secret boundary
Optional adapter/plugin boundaries
```

## Gate Decision Rules

### GO

Use `GO` only when all of the following are true:

1. C1-C8 are satisfied;
2. R001 is accepted PASS;
3. R007 authority is accepted and not contradicted;
4. R002 is PASS, with no product-significant condition required for the intended normal TV flow;
5. R003 is PASS for the required Core target envelope;
6. R008 is accepted for the integrated P0 surface;
7. no mandatory canonical contradiction or Gate-blocking change remains unresolved.

Recommended research decision is normally `Continue`.

### CONDITIONAL GO

Use `CONDITIONAL GO` only when:

1. all required child Evidence is complete and trusted;
2. no required child is FAIL;
3. R002 and/or R003 is an accepted `CONDITIONAL PASS` whose condition is bounded, explicit and product/architecture-compatible;
4. the conditions do not remove the essential Web-only Core value proposition;
5. every material condition is written into the appropriate canonical docs and carried into downstream work;
6. R008 remains accepted and no security boundary is waived.

Examples of potentially acceptable conditions include one initialization interaction after browser/TV restart or explicit measured media/concurrency limits, provided the accepted Evidence and product goals support them.

Recommended research decision may be `Continue` with explicit limits or `Change` when a bounded canonical adjustment is required before continuing.

### NO-GO

Use `NO-GO` for the current Web-only Core architecture/product proposition when any of the following applies:

- R001 required media path is FAIL;
- R002 real-TV result is FAIL for the intended low-interaction product proposition;
- R003 target result is FAIL for the intended low-power Gateway hardware/media strategy;
- R008 required security boundary is FAIL or the Core only works by weakening it;
- accepted child results require mutually incompatible architecture/product assumptions;
- the required fix is an architecture/product re-plan rather than a bounded condition.

`NO-GO` does **not** automatically mean abandon the entire project. The paired research recommendation must say whether the correct next action is `Change`, `Defer` or `Drop` and identify the minimal failed assumption to revisit.

### BLOCKED / not publishable

Do not treat missing required child Final Acceptance as `CONDITIONAL GO` or `NO-GO`. Before execution it is a Publication Gate blocker. If discovered during an Attempt, report `BLOCKED` and route the missing/reopened child Evidence explicitly.

## Success Criteria

### Task success

1. All required child inputs are live-read and traceable to Final Acceptance.
2. C1-C8 receive explicit evidence-backed results.
3. `docs/research/core-feasibility-review.md` contains the complete parent Evidence and limitation matrix.
4. Cross-Task contradictions are resolved, routed or cause an explicit NO-GO/BLOCKED result; none are hidden.
5. Gate result is exactly one of `GO | CONDITIONAL GO | NO-GO` when complete.
6. Research recommendation is explicitly `Continue | Change | Defer | Drop` with rationale.
7. Any mandatory Evidence-driven canonical updates are committed coherently before final Gate acceptance.
8. The result explicitly states what it does **not** prove about R004/R005/R006 or production readiness.
9. Downstream Phase-1 entry conditions and carried limitations are explicit when the result permits continuation.
10. No new runtime/device Evidence is fabricated or substituted inside this Task.

## Evidence Contract

Each Attempt must report:

```text
Attempt:
Worker / Environment:
Review base / candidate doc commit:
Required child Final Acceptance references:
R007 result / limitations:
R001 result / limitations:
R002 result / limitations:
R003 result / limitations:
R008 result / limitations:
Cross-cutting contradictions found/resolved:
Canonical docs changed:
Claims C1-C8:
Gate result: GO | CONDITIONAL GO | NO-GO | BLOCKED
Research recommendation: Continue | Change | Defer | Drop | n/a when blocked
Durable review doc:
Unresolved limitations:
Suggested downstream entry:
```

Do not copy Secret-bearing logs or sensitive URLs into the review document. Reference durable child Evidence instead.

## Failure / Blocked Handling

Return `BLOCKED` when required accepted Evidence is missing, reopened, stale in a way the child authority has not resolved, or cannot be read/verified.

Return `NO-GO` rather than lowering criteria when complete trusted Evidence disproves the current Core proposition.

If review identifies a new independent architecture-repair scope, Coordinator may SPLIT a child Task. Issue #22 remains blocked/draft until the required new Evidence returns; do not implement the architecture repair opportunistically inside this synthesis Task unless the Contract is formally revised.

## Deliverables

- Issue #22 durable Attempt/Review history;
- `docs/research/core-feasibility-review.md`;
- minimal Evidence-driven canonical patches when required;
- explicit Gate decision and research recommendation;
- Phase-1 entry/limit summary when applicable.

## Publication Gate

Keep Issue #22 at `status:draft` until all of the following are true:

```text
Issue #2 Final Acceptance current
Issue #3 Final Acceptance current
Issue #7 Final Acceptance current
Issue #9 Final Acceptance current
Issue #14 Final Acceptance current
no required child reopened/contradicted
Task/prompt/Issue links read back
preferred worker/env/capabilities still correct
```

R004/#15/#16, R005 and R006 are **not** publication dependencies for the Web-only Core Gate.

After publication:

```text
status:ready + env:web-gpt + no active owner
→ Worker may claim
```

## Completion Protocol

Worker:

```text
claim
→ Attempt N
→ read accepted child Evidence
→ synthesize matrices / canonical impact / Gate result
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Coordinator then independently reviews the Gate synthesis. Worker must not self-close #22 or mark `status:done`.