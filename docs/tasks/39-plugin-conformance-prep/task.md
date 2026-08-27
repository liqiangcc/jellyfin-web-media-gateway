# Task — PLUGIN-CONFORMANCE-PREP SiteAdapter conformance / architecture guards

## Metadata

```text
GitHub Issue: #39
Parent Goal / Research Item: Phase 1 / Plugin Boundary Hardening
Task / Research ID: PLUGIN-CONFORMANCE-PREP
Task kind: combined
Base commit: aebf921b6876616dcf791ecae7d894ea0bb847c7
Candidate commit: n/a until Worker execution
Session bootstrap prompt: docs/tasks/39-plugin-conformance-prep/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, rust-build, rust-test, github-actions-orchestration
Hard publication dependencies: none beyond accepted current generic plugin/security authorities
```

Realtime status/owner/branch/PR/Evidence lives only in Issue #39. Attempt history follows `docs/tasks/issue-lifecycle-protocol.md`.

## Goal

Create a reusable deterministic conformance harness for the accepted generic `SiteAdapter` / `SiteAdapterRegistry` boundary and add bounded Stable Core architecture guards so current/future compile-time plugins can be verified without leaking concrete-site knowledge into Core.

```text
SiteAdapter implementation
→ reusable conformance harness
   → recognize determinism
   → locator ownership/version invariants
   → ResolvedMedia schema + public-header Secret safety
   → Registry conflict semantics
   → generic error/diagnostic safety

Stable Core source
→ architecture guard
→ rejects concrete-site business knowledge
```

## Context / Authority

Canonical inputs:

- `docs/site-plugin-architecture.md`
- `docs/implementation-contracts.md`
- `docs/security.md`
- `docs/mvp-plan.md`
- accepted Issue #14 / R008 security semantics
- accepted Issue #3 / R001 generic-direct/media-path behavior

Issue #23 / draft PR #37 is **not accepted authority**. Its Bilibili Candidate may contain useful future implementation work, but this Task must not silently freeze #23-only API additions such as `NavigationContext`, `ResolveContext`, DASH/expiry fields or site-specific error additions merely because they exist on that branch.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: required Claims are deterministic compile-time/API/static-analysis contracts and standard GitHub-hosted regressions; no independent device/real-site Evidence Authority is required.
```

## Worker Routing

```text
implementation: cloud-codex / env:cloud
verification: GitHub Actions / github-hosted ubuntu-latest x64
```

No phone, TV, self-hosted target, real site, Chromium or Jellyfin runtime is required.

## Claims

- C1 — reusable conformance harness exists and can be invoked by plugin crates using deterministic fixtures.
- C2 — recognize behavior is deterministic and returned locator `site_id/plugin_id` ownership agrees with the adapter/registry contract.
- C3 — locator version is explicit; foreign plugin/site ownership and unsupported/invalid locator versions fail deterministically without Core decoding opaque payload semantics.
- C4 — accepted `ResolvedMedia` output schema is validated and public headers/output cannot carry Cookie/Authorization/bearer-style Secret material; reuse R008 helpers where practical.
- C5 — Registry behavior is deterministic for duplicate registration, explicit priority independent of registration order, equal-priority ambiguity, no-match and plugin-not-found/foreign locator routing.
- C6 — generic Core-facing errors/diagnostics remain bounded/non-secret and do not encode concrete-site business facts.
- C7 — Stable Core architecture guard detects concrete-site leakage while being scoped narrowly enough not to flag docs/tests/plugins merely for mentioning sites.
- C8 — accepted `generic-direct` passes the same conformance harness and existing R001 behavior remains green.
- C9 — harness remains usable by a future integrated #23/Bilibili plugin without importing/merging/accepting #23 in this Task.

## Preconditions

- Base: `main@aebf921b6876616dcf791ecae7d894ea0bb847c7`.
- Worker must re-read live main before claim and integrate newer accepted main before final exact-Candidate verification if needed.
- #23 remains blocked; PR #37 remains preservation-only.

## In Scope

- reusable conformance helper/module/crate suitable for compile-time SiteAdapter implementations;
- deterministic fake adapter fixtures;
- accepted current `SourceLocator`, `ResolvedMedia`, `SiteAdapter`, `SiteAdapterRegistry`, `AdapterError` behavior only;
- output validation helpers that are generic and site-neutral;
- Registry conflict/priority/ambiguity/ownership tests;
- Secret/public-header sentinel tests consistent with R008;
- bounded Stable Core static/architecture guard over production Core surfaces;
- apply harness to `plugins/generic-direct`;
- GitHub Actions exact-Candidate verification.

## Out of Scope

- `generic-ytdlp` implementation;
- merging or modifying PR #37 / #23 Bilibili Candidate;
- real Bilibili/network smoke;
- freezing #23-only navigation/resolve-context/DASH/expiry/error API additions;
- Browser Worker, login, Vault runtime, Native Panel;
- R007 Playback changes;
- process-plugin IPC, hot update or plugin marketplace.

## Architecture Invariants

1. Stable Core depends on generic SiteAdapter contracts, never concrete plugin business logic.
2. Core may know `site_id`, `plugin_id`, capability/health/version metadata and opaque locator, but not site URL/ID/Cookie/DOM/private API semantics.
3. Registry priority/conflict behavior is explicit and deterministic; registration order must not become hidden business routing authority.
4. `SourceLocator.opaque_payload` remains plugin-owned opaque data; conformance tests must not teach Core how to parse it.
5. Public `ResolvedMedia` fields/headers remain Secret-free; R008 remains central security authority.
6. A conformance harness can test generic contract shape but cannot manufacture real-site R005 acceptance.
7. Architecture guard is scoped to Stable Core production surfaces and must not prohibit site-specific code inside `plugins/<site>/`.

## Files Expected to Change

Likely:

- `site-adapter-api/src/lib.rs` only where accepted generic validation/test hooks are needed;
- `site-adapter-api/tests/*` or a reusable `site-adapter-api` conformance module;
- `plugins/generic-direct/*` tests/dev-dependencies;
- bounded architecture/static guard script/test;
- `.github/workflows/*plugin-conformance*`;
- root workspace metadata only when strictly required.

Do not edit canonical architecture merely to fit implementation shortcuts.

## Implementation Requirements

1. Design the harness around accepted interfaces on live main; do not copy API shape from blocked #23.
2. Provide deterministic fixtures that exercise matched/no-match/priority/ambiguity/duplicate/foreign-locator cases.
3. Validate locator ownership: returned locator must identify the adapter/plugin that produced it; Registry resolve must not route foreign locator to an unrelated adapter.
4. Add explicit unsupported/invalid locator-version test without inventing migration semantics not yet canonical.
5. Validate accepted `ResolvedMedia` fields structurally and reject Secret-bearing public header material using shared/generic security logic where feasible.
6. Add diagnostic sentinel tests so generic error formatting cannot emit fake Cookie/Authorization/bearer secrets introduced by test fixtures.
7. Architecture guard must scan only defined Stable Core production surfaces and use an explicit deny vocabulary/pattern strategy with regression fixtures proving both detection and boundedness.
8. `generic-direct` must invoke/use the reusable harness rather than duplicate a private one-off suite.
9. Do not make a network request in required conformance jobs.
10. Final Candidate integrates current accepted main and all required jobs run on the exact final SHA.

## Verification Plan

| Job | Claims | Execution plane | Runner | Required | Selector / intent |
|---|---|---|---|---|---|
| J1 | C1-C5,C8 | GitHub Actions | ubuntu-latest | yes | fmt/clippy + conformance harness + generic-direct deterministic tests |
| J2 | C4,C6,C7,C9 | GitHub Actions | ubuntu-latest | yes | Secret/error negatives + architecture guard positive/negative fixtures |
| J3 | C8,C9 + regressions | GitHub Actions | ubuntu-latest | yes | workspace/all-targets + affected R001/R008/plugin regressions |

```text
Target proof required: no
Interactive external debugging: no by default
```

## Success Criteria

1. C1-C9 have explicit exact-Candidate Evidence.
2. At least one reusable harness entry is consumed by `generic-direct` and deterministic fake adapters.
3. Registry conflict/priority/ownership/version negatives are executable and deterministic.
4. Secret-bearing public headers/output are rejected or classified by accepted generic/R008-compatible logic.
5. Stable Core architecture guard catches concrete-site leakage without scanning plugins/docs as forbidden Core code.
6. No #23-only API extension is promoted solely from the blocked branch.
7. J1/J2/J3 are green on exact final Candidate.
8. Candidate is in a reviewable PR; Worker stops at `status:review`.

## Evidence Contract

`[EXECUTION REPORT]` must include:

```text
Attempt:
Base commit:
Candidate commit:
PR:
Harness location/API:
Generic-direct conformance result:
Registry conflict/ownership/version result:
Secret/error diagnostic result:
Architecture guard scope + positive/negative fixture result:
Claims C1-C9:
J1/J2/J3 run + job IDs:
Exact-Candidate assertion:
Affected R001/R008 regressions:
Explicit statement that #23/PR #37 was not merged or accepted:
Limitations:
Result: COMPLETED | BLOCKED
```

No Secret, Cookie, Authorization, production account data or full sensitive URL may appear in Evidence.

## Failure / Blocked Handling

- implementation/test defects remain same-Task revision work;
- genuine contradiction between accepted generic API and canonical Site Plugin architecture must be reported to Coordinator, not resolved by importing blocked #23 API;
- inability to run required GitHub Actions exact-SHA Evidence is BLOCKED;
- phone/TV/real-site unavailability is not a blocker.

## Deliverables

- reusable conformance implementation/tests;
- Stable Core architecture guard;
- exact-Candidate workflow Evidence;
- Candidate commit + PR;
- `docs/tasks/39-plugin-conformance-prep/prompt.md` bootstrap.

## Completion Protocol

Worker follows `docs/tasks/issue-lifecycle-protocol.md`:

```text
claim → status:in-progress → Attempt N
→ candidate + exact-SHA J1/J2/J3
→ [EXECUTION REPORT] → status:review → release owner → STOP
```

Blocker path: `[BLOCKER REPORT] → status:blocked → release owner → STOP`.
Worker never sets `status:done`, closes #39, merges its own PR, starts another Task, or silently changes this Contract.