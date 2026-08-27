# Task — R008 P0 Egress / Secret / Execution Security Baseline

## Metadata

```text
GitHub Issue: #14
Parent Goal / Research Item: R008 / P0 Core Feasibility Security Gate
Task / Research ID: R008-P0-SECURITY
Task kind: combined
Planning / integration base commit: 72434fd3faefc1852f757871a316a28d02abbc87
Candidate commit: n/a (live candidate belongs in Issue)
Session bootstrap prompt: docs/tasks/14-r008-p0-security-baseline/prompt.md
Downstream handoff profile: docs/tasks/handoffs/cloud.md
Preferred worker: cloud
Eligible worker environments after publication: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, rust-build, rust-test, security-test-authoring, failure-injection, workflow-static-analysis
Hard publication dependencies: none; accepted authority/evidence from Issue #1 / INFRA-001, Issue #2 / R007, Issue #3 / R001 must be consumed rather than redefined
```

> Live status, owner, Attempt, branch/PR, candidate SHA, runs and results belong in Issue #14.
>
> R008 is a security feasibility/boundary proof. It may add missing reusable security implementation/tests needed to make the canonical boundary executable, but it must not silently redesign product scope or claim runtime security for components that do not yet exist.

## Goal

Produce the executable **P0 security baseline** required by the Web-only Core Feasibility Gate.

The final accepted R008 candidate must demonstrate, with exact-SHA GitHub Actions Evidence plus accepted upstream Evidence where applicable, that the currently implemented P0 surfaces preserve:

```text
central EgressPolicy / SSRF boundary
+
configured-local-service exception boundary
+
SiteAccess / Vault / Secret isolation
+
ResolvedMedia + Media capability Secret/replay boundary
+
non-secret logs/errors
+
safe structured external-process invocation
+
trusted Target Runner / workflow boundary
```

R008 must also record explicit **deferred security claims** for Browser Worker / Native Site Panel runtime because R006 is not a P0 runtime implementation. A future component cannot be marked secure merely because R008 exists.

## Why / Context

Canonical security rules are not post-MVP hardening. `docs/technical-feasibility-validation.md` defines R008 as a Core-blocking security Research Item, and `docs/security.md` requires the P0 spikes to preserve Egress/Secret boundaries from the beginning.

Accepted earlier work already supplies partial evidence:

- Issue #3 / R001 proves scoped Media Gateway capabilities, no arbitrary `/stream?url=...`, redirect egress revalidation, protected upstream Secret injection, replay rejection and browser-side non-leakage for the R001 media path.
- Issue #2 / R007 proves command/request-id/revision/display-generation stale-event authority behavior.
- Issue #1 / INFRA-001 proves the accepted low-privilege Target Runner deployment boundary.

R008 must **reuse and independently map** that evidence into the R008 claim matrix, then implement/verify the security gaps not yet proven. It must not copy old PASS statements without checking the referenced Issue/run/candidate.

## Task Decomposition Decision

```text
Verification mode: inline
Linked implementation task: n/a
Linked verification task: n/a
Decision reason: the missing P0 security controls and deterministic security tests are ordinary repository work and can be verified on GitHub-hosted runners. Existing target-phone trust facts come from accepted INFRA-001 Evidence; R008 does not need a new phone benchmark or physical-TV Evidence authority.
```

Do not split x64 security tests, workflow trust checks and optional generic ARM64 regression into separate business Tasks. They are Jobs under one R008 claim set.

### P0 concurrency / integration rule

Issue #6 (R002-PREP) and Issue #8 (R003-PREP) may execute in parallel with R008 because neither is a hard publication dependency for the base security primitives.

However:

- the final R008 candidate must integrate **current `main` at the time of its final verification**;
- if an accepted #6/#8 candidate lands before R008 final review and changes HTTP/media/target-workflow security-sensitive surfaces, R008 must rebase/integrate and rerun the affected required Jobs on the new exact Candidate SHA;
- an old R008 run never proves a later security-sensitive candidate automatically;
- the later Core Feasibility Review must run/inspect the accepted security suite on the integrated P0 `main` state.

This is an integration/freshness rule, not a reason to serialize all P0 Tasks.

## Worker Routing Decision

```text
Repository implementation / test authoring / CI authoring / evidence mapping
→ cloud-codex
→ env:cloud

Required deterministic/runtime security evidence
→ GitHub Actions
→ GitHub-hosted x64

Portable architecture regression
→ GitHub-hosted ARM64 optional when useful

Phone runner security facts
→ consume accepted Issue #1 Evidence + static trusted-workflow checks
→ do not schedule ordinary R008 CI on the phone

Physical TV
→ not required for R008
```

Codex Cloud is the Worker/orchestrator, not the Verification Runner or phone target.

## Canonical Sources to Read

Before implementation the Worker must read at least:

- `AGENTS.md`
- Issue #14 and all relevant comments
- `docs/requirements.md` — FR-18, FR-19, FR-20, FR-21, FR-22 and relevant acceptance criteria
- `docs/architecture.md` — Site Plugin, Media Gateway, Vault, EgressPolicy, API and observability boundaries
- `docs/implementation-contracts.md` — SiteAccessCapability, ResolvedMedia, Playback/Display freshness where reused
- `docs/technical-feasibility-validation.md` — R008
- `docs/security.md`
- `docs/runner-execution-architecture.md`
- `docs/research/r001-media-path.md`
- Issue #1 Final Acceptance / Evidence
- Issue #2 Final Acceptance / Evidence
- Issue #3 Final Acceptance / Attempt 2 Evidence
- `docs/tasks/issue-lifecycle-protocol.md`

If these sources conflict, follow repository authority order and report any real canonical conflict instead of inventing a new security model inside R008.

## Work Role

### Implementation

Build the minimum reusable security implementation/tests needed to turn the canonical P0 boundary into executable proof.

Expected categories, as required by the current codebase:

1. central/public-web EgressPolicy test surface rather than ad-hoc media-only URL checks;
2. explicit configured-local-service policy that cannot be selected by an arbitrary user/plugin URL;
3. scoped site access / capability validation primitives sufficient to prove cross-site/host/expiry isolation without implementing a production Vault;
4. ResolvedMedia/public-header validation and media capability replay regression coverage;
5. redaction/non-leakage checks for errors/loggable diagnostic structures introduced by current P0 code;
6. structured process-invocation guards/tests for any current FFmpeg/Chromium/yt-dlp execution path or reusable helper; no shell-string escape hatch;
7. workflow trust/static checks for any self-hosted target workflow present on the final candidate;
8. durable `docs/research/r008-security-boundary.md` mapping each R008 claim to exact Evidence and explicit deferred boundaries.

Implementation details are codebase-driven. Do not create a fake production Vault, fake Browser Worker or fake local-service integration merely to make a test pass.

### Verification Claims

```text
C1 — Public-web EgressPolicy
All caller/plugin/media-controlled public-web targets reject loopback, private, link-local, metadata, multicast/unspecified/reserved targets as applicable, including IPv4/IPv6 forms. DNS/connection/redirect boundaries are revalidated so a public entry cannot redirect into a forbidden target.

C2 — Configured local service boundary
Private/LAN egress is available only through explicit administrator/deployment configuration for a named local integration. User URL, Site Plugin output, media capability input or Browser Worker input cannot self-declare/upgrade itself into the local-service exception.

C3 — Site access / Vault / cross-site isolation
A Site Plugin does not receive raw Vault access. Scoped SiteAccess capability semantics enforce site/account/allowed-host/expiry boundaries; cross-site or expired access is rejected and redirect host scope is rechecked. No production Cookie jar is required for the fixture.

C4 — ResolvedMedia / media Secret boundary
ResolvedMedia/public headers reject Cookie/Authorization/bearer-like Secret material; Display/public media capability surfaces do not expose upstream credentials; invalid/expired/cross-session/cross-item/cross-resource replay remains rejected before unauthorized upstream access.

C5 — Log / error / artifact non-leakage
Deterministic fixtures prove known Secret values, protected Authorization/Cookie material and unnecessary signed query material do not appear in browser-visible responses, ordinary errors, logs or accepted security artifacts.

C6 — Structured process invocation
Current/reusable external process invocation for FFmpeg, yt-dlp or Chromium uses argv/structured APIs. Shell command construction from URL/title/file/path/user-controlled strings is absent or rejected by executable/static regression checks.

C7 — Playback/display security freshness reuse
Accepted R007 request-id/revision/item/display-generation stale-event protections remain passing on the integrated R008 candidate; R008 does not redefine Playback authority.

C8 — Target Runner / trusted-workflow boundary
Accepted INFRA-001 low-privilege Target Runner evidence remains the target trust authority. Repository target workflows on the final candidate cannot be automatically entered by untrusted PR/fork changes, use explicit trusted candidate identity, minimal permissions, bounded inputs and do not grant Vault/profile/production-Secret access by default.

C9 — Browser Worker / Native Site Panel boundary is not falsely claimed
Current canonical requirements remain explicit: Browser Worker must not access local files/private egress or expose/download profile/Secret data. If no Browser Worker runtime exists in P0, R008 records this as deferred runtime verification for R006 rather than reporting PASS from static prose alone.

C10 — Current P0 HTTP/security surface
For currently implemented externally reachable P0 control/display/media endpoints, relevant Host/Origin/Content-Type/request-size/token checks are either executable and verified or explicitly identified as not-yet-instantiated surface. R008 may not call an absent production Control/API implementation secure merely because a test proof server is safe.
```

## Task vs Existing Evidence Boundary

R008 may accept existing Evidence only when all of these are true:

```text
source Issue is Coordinator-accepted
claim actually overlaps R008
exact candidate/run is identifiable
security property was executed, not merely described
evidence boundary is still applicable to current candidate
```

Existing Evidence does **not** exempt current candidate regression. For example, the R001 protected fixture may establish the original media Secret property, but the final integrated workspace still runs the corresponding security regression suite.

## In Scope

- EgressPolicy/SSRF deterministic matrix for current public-web traffic;
- redirect revalidation and forbidden-target regression;
- configured-local-service exception boundary;
- scoped SiteAccess capability contract/fixtures sufficient for current architecture;
- ResolvedMedia Secret-header schema validation;
- media capability open-proxy/replay regression;
- log/error/artifact redaction checks with deterministic fake secrets;
- structured process invocation guard/test for existing process execution code or reusable wrapper;
- static/automated repository workflow trust checks for self-hosted target jobs;
- regression of accepted R007 security-relevant concurrency/freshness tests;
- Evidence mapping from accepted #1/#2/#3;
- R008 durable research Evidence document;
- current-main integration before final exact-SHA verification.

## Out of Scope

- Gateway user accounts/RBAC/multi-user identity;
- exposing the service publicly;
- production Vault encryption implementation unless a current P0 code path genuinely requires a minimal primitive for the claim;
- real source-site accounts/Cookies/tokens;
- implementing a real Site Browser Worker / Native Site Panel just for R008;
- claiming R006 Browser Worker runtime security PASS;
- Jellyfin Adapter implementation or real Jellyfin API key testing;
- R002 physical TV UX/autoplay;
- R003 resource/thermal performance;
- DRM bypass or protected-media capture;
- root/ADB privilege changes on the target phone;
- arbitrary penetration testing outside the frozen P0 trust model.

## Architecture Invariants

- Gateway Core owns central EgressPolicy; plugins cannot bypass or declare private exceptions.
- `configured_local_service` addresses come from explicit deployment/admin configuration, never arbitrary user/plugin input.
- Site Plugin does not directly read Vault or another site's Session.
- Display Adapter does not read Vault.
- `ResolvedMedia.public_headers` never carries Cookie/Authorization/bearer Secret material.
- Media capability maps server-side to approved upstream resources; no arbitrary caller-selected proxy target.
- every redirect/host scope transition is revalidated.
- external processes use argv/structured APIs, never shell strings built from untrusted values.
- logs/errors/artifacts avoid Cookie, Authorization, API keys, profiles, temporary signed media URLs and sensitive query material.
- Target Runner remains dedicated/low-privilege and untrusted code cannot automatically obtain target shell authority.
- R008 does not create a second Playback authority or weaken accepted R007 semantics.
- absent/deferred components are reported as deferred/unverified, not PASS.

## Files Expected to Change

Exact paths are implementation-defined. Expected categories include:

- `gateway-core/` security/egress/capability validation code and tests as needed;
- `site-adapter-api/` contract validation only if required by current architecture;
- `.github/workflows/` security verification or trust-check additions as needed;
- repository test/static-check scripts if a small deterministic guard is clearer;
- `docs/research/r008-security-boundary.md`;
- minimal canonical status wording only if executable Evidence reveals an actual documentation mismatch.

Avoid unrelated refactors.

## Implementation Requirements

1. Inventory current P0 security-sensitive surfaces before coding; map each canonical minimum test to implemented / accepted-existing-evidence / missing / deferred.
2. Centralize reusable EgressPolicy behavior enough that R001 media logic is not the only place enforcing public-web safety.
3. Cover IPv4/IPv6 forbidden address classes and redirect/host revalidation with deterministic tests; do not depend only on public internet behavior.
4. Model configured-local-service access as a distinct capability/configured target, not a caller-supplied bool or URL escape hatch.
5. Provide a deterministic scoped-site-access fixture with fake credentials/hosts; never use real account data.
6. Preserve R001 media capability and protected-secret behavior; extend tests instead of replacing evidence with weaker unit-only assertions.
7. Add executable/static regression that fails on unsafe shell process invocation in in-scope runtime paths; do not require implementing tools not yet used.
8. Validate target workflows from repository state. A workflow that automatically runs untrusted PR code on `self-hosted ... target-device` is an R008 failure.
9. Keep GitHub Actions token permissions minimal; do not add repository write or Secret access merely to run security tests.
10. Record explicit `NOT IMPLEMENTED / DEFERRED TO R006` for Browser Worker runtime checks that cannot honestly execute.
11. Produce a concise durable evidence record that points to Issue/run/job/candidate instead of copying sensitive logs.
12. If final integration changes the Candidate SHA, rerun all required J1-J3 Evidence on the exact new SHA.

## Verification Plan

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1-C7,C10 | github-actions | github-hosted-x64 | runner-self | yes | fmt/clippy/workspace tests + deterministic security/egress/capability/failure suite + accepted R007 regressions | run/job/log |
| J2 | C1-C6,C10 | github-actions | github-hosted-x64 | deterministic fixture/integration | yes | security integration fixture: redirect/egress, protected Secret, open-proxy/replay, redaction and request-surface checks | run/job/artifact summary |
| J3 | C8,C9 | github-actions | github-hosted-x64 | repository/workflow static trust model | yes | static validation of target-workflow triggers/permissions/runner labels/input handling + deferred Browser Worker boundary check | run/job/report |
| J4 | C1-C7 | github-actions | github-hosted-arm64 | generic Linux ARM64 | no | portable security suite when practical | run/job |

No ordinary R008 Job runs on the phone Target Runner. Phone-specific runtime security evidence is consumed from accepted INFRA-001; any later claim that truly needs new target execution must be separately justified rather than using the scarce runner by default.

### Execution Plane

```text
Required Evidence execution plane: github-actions
Worker/orchestrator: cloud-codex
Required runner: GitHub-hosted x64
Optional runner: GitHub-hosted ARM64
Target phone run: no, unless Coordinator revises Contract based on a concrete missing target-only claim
```

### Security fixture rules

Deterministic tests may use loopback fixture servers **only through an explicit test-only fixture scope that cannot be selected by browser/user input**. Test infrastructure must not weaken production/public-web policy to make the test reachable.

Use fake recognizable Secret sentinels and assert they do not appear in public/log/artifact outputs.

### Workflow trust validation

J3 must inspect every final-candidate workflow that can target `self-hosted` / `ubuntu-arm64` / `target-device` and verify, as applicable:

- no automatic untrusted PR/fork path to target execution;
- workflow/control implementation comes from trusted repository state;
- measured candidate is explicit and validated;
- untrusted text is not directly interpolated into shell;
- token permissions are least privilege;
- no default Vault/profile/long-term Secret access;
- bounded timeout/concurrency/cleanup for heavy target jobs.

If Issue #8's target workflow is merged before R008 final verification, it is automatically in this inspection scope.

## Success Criteria

### Task success

1. C1-C8 and C10 have executable PASS evidence or accepted upstream Evidence plus current-candidate regression as specified.
2. C9 is honestly classified: Browser Worker runtime is deferred/unverified when absent, with canonical safety requirements preserved; no false PASS.
3. Public-web EgressPolicy has deterministic forbidden-target + redirect coverage beyond the narrow public-media happy path.
4. A plugin/user/media-controlled input cannot obtain configured-local-service/private-network authority.
5. Site access fixtures prove cross-site/host/expiry isolation without exposing real Vault data.
6. ResolvedMedia/media capability Secret and replay protections remain intact on the final candidate.
7. In-scope external process invocation has no shell-string injection path from untrusted input.
8. Final candidate's target workflows satisfy the trusted self-hosted Runner gate.
9. Required J1-J3 GitHub Actions all PASS on the **exact final Candidate SHA**.
10. `docs/research/r008-security-boundary.md` maps all claims, accepted prior evidence, new evidence, limitations and architecture impact.
11. No real Secret/token/profile/signed sensitive URL is committed or uploaded as Evidence.
12. R008 does not claim R002/R003/R004/R005/R006 product/runtime results.

### Research result classification

Coordinator records one of:

```text
PASS
CONDITIONAL PASS
FAIL
BLOCKED
```

`PASS` means the currently implemented P0 Core security boundary is executable and evidence-backed, with unimplemented optional components explicitly deferred rather than silently trusted.

`CONDITIONAL PASS` is allowed only for a clearly bounded non-Core runtime/deployment limitation that does not create an open P0 Egress/Secret hole. Missing proof for a Core security invariant is not a conditional convenience; it is FAIL/BLOCKED/REVISE as appropriate.

## Evidence Contract

Each Attempt records at minimum:

```text
Role: combined
Task / Claim: R008 / C1-C10
Attempt:
Worker / Orchestrator:
Base commit:
Candidate commit:
PR:
Execution plane: github-actions
Runner class / image:
OS / architecture:
Rust/tool versions:
Workflow / run / job IDs:
J1/J2/J3 results:
Existing Evidence consumed from Issue #1/#2/#3:
Egress matrix summary:
Configured-local-service result:
SiteAccess capability result:
Secret sentinel/redaction result:
Media capability replay/open-proxy result:
Process invocation guard result:
Target workflow trust result:
Browser Worker runtime classification:
Artifacts / durable research doc:
Result:
```

Do not persist:

- Cookie/Authorization/API key;
- real account/session data;
- browser profiles;
- real site credentials;
- temporary target registration tokens;
- complete sensitive signed URLs/query strings;
- unnecessary raw environment dumps.

## Failure / Blocked Handling

### FAIL / REVISE-worthy findings

Examples:

- public-web can contact forbidden private/loopback/metadata targets;
- redirect/DNS transition bypasses EgressPolicy;
- caller/plugin can opt into configured-local-service scope;
- cross-site/expired SiteAccess capability succeeds;
- public ResolvedMedia/log/error leaks deterministic Secret;
- media endpoint becomes arbitrary open proxy or replay crosses binding;
- untrusted input reaches `sh -c` / equivalent shell-string execution;
- untrusted PR/fork can automatically execute on the phone Target Runner;
- R008 declares an unimplemented Browser Worker runtime secure.

### BLOCKED

Use BLOCKED only when required Evidence cannot be produced because of an unavailable repository/Actions capability or a concrete dependency that cannot be solved within the frozen Task scope.

Do not lower Success Criteria or temporarily disable SSRF/Secret checks to avoid a blocker.

### Architecture-impact rule

If executable evidence shows the canonical Egress/Secret model itself cannot work, stop and preserve evidence. Coordinator must review the design impact before another Attempt; R008 Worker must not silently introduce a private-network/plugin/Vault escape hatch.

## Deliverables

- R008 candidate branch/PR;
- reusable security implementation/test additions required by the frozen claims;
- required J1-J3 exact-SHA GitHub Actions Evidence;
- optional J4 portability evidence if run;
- durable `docs/research/r008-security-boundary.md`;
- explicit mapping to accepted Issue #1/#2/#3 Evidence;
- explicit deferred Browser Worker runtime boundary;
- no R002/R003/R006 false completion claim.

## Completion Protocol

Worker follows `docs/tasks/issue-lifecycle-protocol.md`:

```text
status:ready
→ claim / Attempt N
→ status:in-progress
→ implement + exact-SHA J1-J3
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Worker must not set `status:done`, merge/close the Issue, execute another Task, or reinterpret deferred Browser Worker runtime as PASS. Coordinator owns review and Final Acceptance.