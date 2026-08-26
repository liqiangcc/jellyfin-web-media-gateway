# Task — R005-PUBLIC-INTEGRATION

## Metadata

```text
GitHub Issue: #65
Task ID: R005-PUBLIC-INTEGRATION
Task kind: integration-only / verification
Parent: #23 R005-PUBLIC
Real-site child: #36 R005-PUBLIC-REAL
Preserved Task Candidate: eb03c199481191a88897ba6b45252bbaa957a63e
Frozen Integration Base: aae02b505bde65b39c6eab1e5ee441decbe8186a
Reuse branch: worker/issue-23-r005-public
Reuse PR: #37
Preferred worker: cloud-codex
Eligible environment: env:cloud
Freshness policy: integration-only / frozen base
```

> #65 owns only the integration refresh lifecycle. #23 remains the R005-PUBLIC result authority and stays blocked on missing real-site J3. #36 remains the real-site Evidence authority.

## Goal

Reconcile the preserved Bilibili public adapter candidate with the accepted repository runtime through the frozen Integration Base, producing one exact Integration Candidate that can safely be handed to #36.

```text
#23 preserved semantic Candidate eb03c199...
+
main@aae02b...
→ same branch / PR #37
→ Integration Candidate
→ deterministic integration Evidence
→ candidate freeze for #36
```

No real Bilibili request occurs in this Task.

## Why integration is required

The preserved Candidate was based on `ee0789b...`. Since then accepted main has changed in material surfaces relevant to final R005 composition:

- accepted R008/security implementation evolved, including #60 async DNS resolver and Gateway-owned `gateway-egress` broker layer;
- `site-adapter-api` conformance/security surfaces evolved;
- SourceSession/Registry/session preparation was added;
- workspace/Cargo dependency graph changed;
- generic-ytdlp runtime and plugin conformance evolved;
- Web MVP server composition evolved.

This is not proof that the Bilibili implementation is wrong. It means the old exact-SHA Evidence alone is insufficient as the final integration snapshot.

## Authority / preserved semantics

Preserve #23 frozen semantics:

- public/no-login/non-DRM Bilibili sample `BV14V411W7r5`;
- Core may know `site_id` but no Bilibili URL/BVID/page/navigation/cookie/private-API semantics;
- Bilibili semantics stay in `plugins/bilibili-public`;
- `SourceLocator` remains plugin-owned opaque/versioned identity;
- R007 remains playback revision/re-resolve authority;
- accepted R008 remains central Egress/Secret authority;
- no plugin-local egress bypass;
- no login/Cookie/profile/CAPTCHA/fingerprint/access-control bypass.

Preserved Task Candidate:

```text
eb03c199481191a88897ba6b45252bbaa957a63e
```

Its deterministic semantic Evidence may be referenced as historical Evidence, but this Task must not claim the Integration Candidate has re-proven real-site C3/C4. That remains #36.

## Integration procedure

### JI0 — Read-back and classify

Before mutation, verify:

- #23 is open and `status:blocked` waiting #36;
- #65 is claimable;
- PR #37 remains open/draft and branch is `worker/issue-23-r005-public`;
- branch head descends from/preserves `eb03c199...` before integration;
- frozen Integration Base resolves exactly to `aae02b505bde65b39c6eab1e5ee441decbe8186a`.

Record changed semantic/integration surfaces between `eb03c199...` and Integration Base.

### JI1 — Compose without rewriting preserved Candidate

Preferred operation:

```text
git merge --no-ff aae02b505bde65b39c6eab1e5ee441decbe8186a
```

on `worker/issue-23-r005-public`.

Requirements:

- preserve `eb03c199...` as ancestry;
- do not rebase/force-push/rewrite the preserved semantic Candidate;
- keep PR #37 draft;
- do not merge PR #37.

Conflict handling:

- mechanical docs/workspace composition may be resolved with exact explanation;
- if a conflict touches Bilibili task-owned semantic behavior, SiteAdapter contract meaning, R007 semantics, R008 authority, Secret handling, SourceLocator meaning, or requires product redesign, stop and post `[BLOCKER REPORT]` rather than treating it as mechanical integration.

### JI2 — Bilibili / SiteAdapter deterministic integration proof

On the exact Integration Candidate run the current repository tests that prove at minimum:

- `bilibili-public` builds/tests;
- SiteAdapter Registry routing still selects Bilibili plugin for the frozen URL shape;
- SourceLocator version/opaque round-trip and unsupported-version behavior remain valid;
- deterministic four-part fixture navigation remains valid;
- deterministic ResolvedMedia mapping/error classifications remain valid;
- current plugin conformance/Secret boundary passes;
- Core-site-boundary guard passes.

Use current repository test/workflow entrypoints rather than reviving obsolete commands when equivalent current selectors exist.

### JI3 — R008 / SourceSession / workspace integration proof

On the exact Integration Candidate prove:

- accepted current R008/security regression passes, including public/private target and Secret authority;
- Bilibili plugin does not obtain Vault/Secret/direct private egress authority;
- current SourceSession/Registry preparation composes without concrete Bilibili knowledge in Core;
- current workspace builds/tests cleanly;
- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace --all-targets` or current equivalent required suite;
- `git diff --check`.

If repository workflows already split these claims, record exact workflow/run/job IDs.

### JI4 — Candidate handoff readiness

Record:

```text
Preserved Task Candidate: eb03c199...
Integration Base: aae02b...
Integration Candidate: <new SHA>
PR: #37 open/draft
ancestry preserved: yes/no
real-site J3 executed: MUST be no
production/login behavior changed: MUST be no
```

The Integration Candidate must be stable/pushed and suitable for Coordinator freeze in #36.

## Claims

```text
I1 — Preserved ancestry
The Integration Candidate contains eb03c199... as ancestry and the frozen Integration Base without rewriting the preserved Bilibili semantic commit history.

I2 — Semantic preservation
Integration does not redesign Bilibili public semantics, SourceLocator meaning, R007 authority, R008 authority, or Secret boundaries.

I3 — Current contract composition
Bilibili deterministic tests, current SiteAdapter/plugin conformance, SourceSession/Registry composition and Core-site-boundary checks pass on the exact Integration Candidate.

I4 — Current security/workspace composition
Current R008/security and workspace fmt/clippy/test regressions pass on the exact Integration Candidate.

I5 — Safe downstream handoff
PR #37 remains draft/unmerged; no real-site J3 or production/login claim is made; exact Integration Candidate is ready for #36.
```

## Verification result

This integration Task uses:

```text
PASS | FAIL | BLOCKED
```

PASS means I1-I5 are accepted by Evidence. It does **not** mean #23/R005-PUBLIC PASS.

## Success Criteria

1. Same branch/PR #37 is reused.
2. `eb03c199...` ancestry is preserved.
3. Frozen Integration Base `aae02b...` is included.
4. Any conflict is either demonstrably mechanical or reported as semantic blocker.
5. JI2/JI3 exact-Integration-Candidate Evidence passes.
6. No real-site Bilibili request, login, Cookie/profile, bypass or production enablement occurs.
7. PR #37 remains draft and unmerged.
8. Worker posts `[EXECUTION REPORT]`, sets #65 `status:review`, releases owner and stops.

## Evidence Contract

Report:

```text
Attempt:
Worker/environment:
Preserved Task Candidate:
Integration Base:
Pre-integration PR #37 head:
Integration method:
Conflicts / resolutions:
Integration Candidate:
Ancestry proof:
JI2 commands/workflows/runs/jobs:
JI3 commands/workflows/runs/jobs:
Bilibili deterministic result:
SiteAdapter/plugin-conformance result:
Core-site-boundary result:
R008/security result:
SourceSession/Registry result:
Workspace fmt/clippy/test result:
PR #37 state/head:
Real-site J3: NOT RUN
Production/login status: unchanged / disabled
Claims I1-I5:
Secret/sensitive-data scan:
Downstream recommendation for #36:
```

## Freshness / Integration Contract

This Task freezes Integration Base `aae02b505bde65b39c6eab1e5ee441decbe8186a`.

Main changes after materialization are handled as follows:

- task/package/planning docs unrelated to Bilibili/SiteAdapter/R008/workspace composition: `UNRELATED`, no new integration required;
- accepted semantic change to SiteAdapter/R008/SourceSession/plugin authority before completion: stop and Coordinator reclassifies/revises;
- workspace-only change after the frozen slot that must be included for correctness: Coordinator may revise Integration Base before Worker claim or open another bounded integration slot after this Task.

## Out of Scope

- real Bilibili J3;
- changing frozen sample;
- publishing/closing #36;
- unblocking/final accepting #23;
- merging PR #37;
- Bilibili login/auth;
- Native Site Panel/Browser Worker;
- generic-ytdlp production enablement;
- Web Display real Bilibili E2E.

## Completion Protocol

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ JI0-JI4
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

If semantic conflict or required integration cannot be completed safely:

```text
[BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must not execute #36/#23 next, merge PR #37, set #65 done, or close #65.