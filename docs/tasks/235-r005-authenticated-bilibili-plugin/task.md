# Task — Authenticated Bilibili Site Plugin feasibility and contract

## Metadata

```text
GitHub Issue: #235
Parent Goal / Research Item: Parent Goal #68; R005 authenticated source-site session
Task / Research ID: R005-AUTH-REAL-BILIBILI
Task kind: research
Base commit: 2f68273b530c21b99b9382cebc5be6a085370d67
Candidate commit: n/a until a package or research revision
Session bootstrap prompt: docs/tasks/235-r005-authenticated-bilibili-plugin/prompt.md
Preferred worker: cloud
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, cloud-interactive, authenticated-site-authorization, approved-interactive-auth-channel
Hard publication dependencies: accepted #28 R005-AUTH-PREP; accepted #33 R006-CONTRACT-PREP; accepted #232 R008 anonymous navigation evidence; Coordinator Publication Gate
```

> GitHub Actions / Runner is the execution backend, not the Worker that claims this Issue. Live status, owner, Attempt and result remain in the Issue history.

## Session Bootstrap

`prompt.md` is a navigation entry only. The live Issue, this contract, canonical architecture/security documents and lifecycle/recovery/freshness protocols are authoritative.

## Goal

Determine whether one legal, explicitly authorized Bilibili account can extend the generic browser-to-Site-Plugin-to-Session-Vault path without manual Cookie/profile smuggling, a second Secret owner, a Gateway identity model, or an access-control bypass. Produce a reviewable contract for the interactive login boundary, session validation and replacement, authenticated opaque `SourceLocator` resolution, and Secret-safe `ResolvedMedia` handoff; classify every required claim as `PASS`, `CONDITIONAL PASS`, `FAIL` or `BLOCKED`.

## Why / Context

#232 is accepted anonymous navigation evidence, not authenticated or playback evidence. Its one bounded tx-node session consistently reached an upstream lifecycle followed by a stable `4xx` navigation status and no media activity; the final cause intentionally remained `unknown`. The evidence does not justify changing access policy or adding a bypass.

#28 / R005-AUTH-PREP is Final Accepted and provides the deterministic server-side foundation: Session Vault ownership, `SiteAccount`/`SiteSessionRef`/`AccountState`, non-secret `PendingIntent`, independently scoped `SiteAccessCapability`/`SiteAccessContext`, server-side credential injection behind R008, and atomic candidate-session replacement. #33 / R006-CONTRACT-PREP is also accepted and provides target-neutral generic Browser Worker Auth Mode, bounded BrowserEvent and opaque profile attachment contracts; it does not prove real Chromium login.

#26 remains the draft R005-AUTH umbrella. Real login has separate runtime, legal, Secret and target Evidence authority, so this Task must decide feasibility and freeze boundaries before any implementation or live account action.

## Frozen Evidence Anchor

- #232 accepted Candidate `5cf05b8439c36844809f35db42f8b87106fb3a81`, PR #234, merged main `2f68273b530c21b99b9382cebc5be6a085370d67`.
- #232 hosted workflow `bilibili-browser-probe`, run `34317961713`; J1 `102358092117`, J2 `102358092088`, J3a `102358092172`, J3 `102358532802`, J4 `102358092106`, JI1 `102358091930` all passed.
- Exact #232 artifact `10090792228`, size `4,222,227` bytes, digest `sha256:e90f0142174e4f5b58014fa8d52e78a5a872f461bda390f76c669eeae06e8af0`.
- Target result: one fresh `gateway-verify` tx-node navigation-only session using selector `bilibili:BV14V411W7r5:part-2`; `navigation_status=4xx` was observed after upstream response/socket lifecycle, with 18 requests and 89,301 response bytes, no media, no consumer, and no access-policy conclusion.
- #28 accepted Candidate `2d46defdacc9cccf4090e1239290cb233842c9ec`, merged as `b72d3d236ebe37c5483b693ecad5e3b113612873`.
- #33 accepted Candidate `8520b73986b5b329b67e82962436ba2279db92fc`, merged as `81d08f02b6928543f06d43e9f6a7a2cfa54fbdd1`.

## Task Decomposition Decision

```text
Verification mode: separate-task if live account/target Evidence is required; static feasibility research remains this Task
Linked implementation task: n/a; create a child implementation Task only after this contract is accepted
Linked verification task: n/a; Coordinator may split an independently owned authenticated target proof after prerequisites are frozen
Decision reason: legal authorization, interactive Secret handling and real-site Evidence authority differ from #28 deterministic auth infrastructure and #232 anonymous R008 diagnostics
```

No Runner or environment variant creates a duplicate business Task. A later target proof is a separate Evidence slice only if its lifecycle, owner and authority are independently gated.

## Worker Routing Decision

```text
Preferred Worker: cloud-codex
Eligible environment: env:cloud
Model: gpt-5.6-luna
Reasoning: high
Fast: enabled per current user routing
Execution plane for repository/static work: GitHub-hosted Actions when automation is required
```

The cloud Worker may inspect repository contracts and author the research record. It is not a browser, target runner or account operator. A live login requires the separately approved interactive auth channel and explicit external authorization described below.

## Work Role

### Research / Contract Design

1. Compare the accepted #28 Vault/capability API and #33 generic Auth Mode/BrowserEvent/ProfileAttachmentRef contracts with the Bilibili plugin boundary.
2. Define the smallest legal authenticated scenario: one operator-owned or explicitly delegated test account, one bounded login attempt, one fresh disposable profile, one account/session scope, one fixed source locator and one post-login resolve.
3. Trace the lifecycle from `SITE_AUTH_REQUIRED` and non-secret `PendingIntent`, through Auth Mode events and plugin interpretation, to candidate-session validation, atomic Vault swap, capability issuance, retry and `ResolvedMedia` handoff.
4. Identify missing implementation/runtime contracts and state whether each is a follow-up Task, a canonical design change requiring Coordinator review, or an external prerequisite.
5. Keep all evidence finite and sanitized; raw account credentials, cookies, tokens, profile archives, QR images, page bodies, full URLs, authorization headers and personal account data are never retained in the Task record.

### Implementation Requirements

N/A for this research package. If an enforceable API/runtime gap requires code changes, stop and propose a focused implementation child rather than silently changing Core, Vault, Browser Worker or plugin authority in this Task.

## External Prerequisites / Authorization Gate

No authenticated target action is admissible until all items below are written into the Issue by the Coordinator or authorized operator:

- a legal use case and written owner/admin authorization for the specific Bilibili account and test scenario;
- a dedicated test account whose owner permits this automation and whose content/access is authorized; no personal account or copied existing browser profile;
- an approved interactive input channel owned by the Browser/Auth Mode design, with bounded cancellation and timeout; raw VNC/CDP/personal-browser control is not an implicit substitute;
- explicit consent for any target host, account scope, session duration, login method and evidence retention;
- a fresh disposable profile/runtime and low-privilege target identity, with no source-runtime/profile reuse;
- exact implementation Candidate and hosted Actions evidence if a child implementation exists;
- Coordinator publication/read-back gate for the authenticated target Task.

If any prerequisite is absent, the result is `BLOCKED` for target evidence. Do not infer feasibility from #232, #28 or #33 alone.

## Preconditions

- Read `AGENTS.md`, `docs/README.md`, `docs/requirements.md`, `docs/architecture.md`, `docs/implementation-contracts.md`, `docs/technical-feasibility-validation.md`, `docs/mvp-plan.md`, `docs/security.md`, `docs/development-environments.md`, `docs/runner-execution-architecture.md`, `docs/site-plugin-architecture.md`, `docs/control-experience-architecture.md`, relevant ADRs, `docs/tasks/issue-lifecycle-protocol.md`, `docs/tasks/execution-anchor-recovery-protocol.md` and `docs/tasks/freshness-integration-protocol.md`.
- Read live Issue #235 and its complete comments, accepted #232 history/evidence, #28 Final Acceptance, #33 Final Acceptance and #26 scheduling history.
- Confirm this Issue is `status:ready`, `env:cloud`, owner-free before a Worker claims an Attempt. Package preparation remains `status:draft` and makes no claim of readiness.
- Confirm #28, #33 and #232 identities from GitHub read-back; do not rely on local copies or old chat.
- Confirm no canonical authority has changed. A proposed change to Vault ownership, Browser Worker Auth Mode, Site Plugin boundaries, R008 EgressPolicy or R007 Playback authority stops for Coordinator design review.

## In Scope

- A legal and evidence-safe authenticated Bilibili scenario and its account/interactive prerequisites.
- Generic Browser Worker Auth Mode lifecycle: fresh profile materialization, bounded input/event surface, cancellation, timeout, expiry, browser disconnect/crash and cleanup.
- Bilibili Site Plugin interpretation of generic browser events into `AccountState`, session validation and an opaque authenticated `SourceLocator`; no concrete login-success rule in Core or Worker.
- Session Vault candidate validation and atomic replacement while preserving the previous valid session on failed/cancelled/expired login.
- Independently scoped site/account capability issuance and controlled HTTP/browser access with host, redirect, expiry and session binding under R008.
- Non-secret `PendingIntent` recovery: retry the original locator, display/action context and expected Playback revision only after session validation.
- Authenticated `ResolvedMedia`/candidate handoff: no Cookie/Authorization in Display, Control, locator, media URL, logs or target evidence; explicit unsupported/expired/private/DRM outcomes.
- Bounded target evidence requirements and PASS/CONDITIONAL PASS/FAIL/BLOCKED classification for a later Coordinator-controlled run.
- Deterministic contract/static checks needed to establish the above boundaries.

## Out of Scope

- Building or deploying a real login runtime, changing production Gateway, changing #26 umbrella state, or implementing a Bilibili login bypass.
- CAPTCHA circumvention, DRM/region/paywall/access-control bypass, anonymous-policy bypass, proxy rotation, open proxy or arbitrary egress.
- Manual Cookie, Authorization, token, localStorage, profile archive, QR image or browser-session injection/smuggling.
- Password or verification-code persistence, raw login form/body capture, full page/HAR/DOM/playinfo capture, or account-identifying evidence.
- Core knowledge of Bilibili URL/DOM/private API/login-success rules; those remain plugin-owned interpretation.
- Playback implementation, click/play/full preload, media extraction, independent consumer proof, or claiming public/authenticated playback success.
- Phone/TV deployment, VNC/CDP observation, source-runtime/profile reuse, production credentials or production service mutation.
- Work on #191, #195, #166, #182, #188, #223, #226, #229 or #232; accepted #232 is evidence only.
- Local build/test/package/install/compile. Any implementation verification belongs on GitHub-hosted Actions.

## Architecture Invariants

- Session Vault is the unique owner of source-site Cookie/token/localStorage/profile Secret material.
- Site Plugin, Core-facing Control, Display and Target Runner receive refs/capabilities, not raw Vault material; only the controlled server-side path may inject Secret.
- `SiteAccessCapability` is independently bound to site/account/session/allowed hosts/expiry and cannot self-attest scope.
- Browser Worker is generic Chromium/Auth Mode infrastructure. Site-specific login interpretation remains in the Bilibili Site Plugin.
- `SourceLocator` and `PendingIntent` remain opaque/non-secret; Core does not parse Bilibili payloads or implement login rules.
- R008 `EgressPolicy` validates every destination and redirect; authentication never widens public/local authority.
- Gateway remains `PlaybackSession` authority; authenticated retry must use normal R007 revision/transition semantics.
- Display receives no source-site credentials, profile, or Vault access. Auth failure must not corrupt an already active playback session.
- Fresh target runtime is low privilege and isolated from source-runtime, production Vault and existing browser/profile state.

## Files Expected to Change

- `docs/tasks/235-r005-authenticated-bilibili-plugin/task.md`
- `docs/tasks/235-r005-authenticated-bilibili-plugin/prompt.md`
- A later implementation child may change plugin/auth contracts only after Coordinator acceptance; this research Task does not authorize those changes.

## Verification Plan

### Claims

```text
C1: A legal authorized-account scenario and bounded interactive login/Secret handling gate are explicit and externally actionable.
C2: Generic Browser Worker Auth Mode, Bilibili Plugin interpretation and Session Vault candidate-session swap form one enforceable lifecycle without a competing Secret owner.
C3: Authenticated SourceLocator retry, scoped SiteAccessCapability and ResolvedMedia handoff preserve R007/R008 and all Secret/redaction boundaries.
C4: Cancellation, timeout, expiry, disconnect/crash, failed replacement and cleanup behavior are finite, deterministic and do not damage the previous valid session or active playback.
C5: A later target proof can be admitted with exact provenance, fresh low-privilege runtime, sanitized output and bounded evidence; absent prerequisites are classified BLOCKED rather than guessed.
C6: The research does not claim authenticated access or playback from anonymous #232 or deterministic #28/#33 contracts.
```

### Verification Job Matrix

| Job ID | Claim(s) | Execution Plane | Runner / Host | Target | Required | Commands / Selector | Evidence |
|---|---|---|---|---|---|---|---|
| J1 | C1,C2,C3,C6 | repository-static-analysis | GitHub-hosted x64 | runner-self | yes | exact package/file/lifecycle and canonical boundary checks | hosted logs/read-back |
| J2 | C2,C3,C4 | github-actions | GitHub-hosted x64 | runner-self | conditional | existing accepted auth/browser/R008 deterministic contracts if an implementation child is linked | exact Candidate logs |
| J3 | C5 | external-codex | authenticated approved channel | later target | no at package/research stage | one fresh bounded authenticated session only after authorization and publication gate | sanitized target report |
| JI1 | C2,C3,C6 | github-actions | GitHub-hosted x64 | runner-self | conditional | affected integration/security regression for a later implementation Candidate | exact Candidate job |

Jobs do not claim this Issue and do not authorize an account or target request. A future implementation child must define its exact Candidate and hosted jobs before target execution.

### Execution Plane / Runner Selection

```text
Research and repository/static verification: cloud-codex with GitHub-hosted Actions when automated checks are required
Authenticated target proof, if separately published: external-codex through the approved interactive auth/control channel
Target runtime, if separately admitted: low-privilege fresh disposable Browser Worker slot; no source-runtime/profile reuse
```

No phone, TV, VNC or CDP target is required by this package. An interactive login channel must be approved explicitly; it is not created by this Task.

### Target Verification Contract (Coordinator-controlled later)

A later target Task may run at most one clean authenticated session first, with a second only after a Coordinator-approved ambiguity. It must:

1. verify the legal account/authorization record and exact Candidate/artifact gate;
2. use a fresh mode-700 profile and low-privilege runtime, never the source-runtime or a personal browser;
3. route all site traffic through plugin-owned authority and R008 EgressPolicy;
4. expose only bounded login status/events and sanitized account/session state;
5. validate/atomically swap the session before retrying the original opaque locator;
6. observe only bounded post-login source resolution and media metadata handoff; no playback click/play/full preload or media consumer;
7. clean browser/profile/capability/pending intent and any attempt-owned staging, then read back cleanup.

If written authorization, account, interactive channel, target runtime, exact artifact or cleanup proof is unavailable, classify the target claim `BLOCKED`.

## PASS / CONDITIONAL PASS / FAIL / BLOCKED Rules

- `PASS`: all applicable contract claims are directly supported; the legal account/interactive prerequisites exist; no Secret or authority boundary is crossed; any separately required target evidence is complete and sanitized.
- `CONDITIONAL PASS`: the architecture and deterministic contracts are sound, but a declared external prerequisite or target-specific runtime fact remains unproven without any contradictory evidence.
- `FAIL`: the proposed flow requires raw Cookie/profile/password smuggling, Site Plugin Vault reads, Core site knowledge, unscoped egress, bypass authority, Secret output, unsafe session replacement, or damage to Playback/Display authority.
- `BLOCKED`: written authorization, dedicated account, approved interactive channel, target capability or exact hosted artifact is absent; Actions is unavailable; or a canonical design change is required and has not received Coordinator review. Do not convert missing evidence into PASS or FAIL.

## Freshness / Integration Contract

Freshness policy: dependency-aware

Semantic authorities:
- #28 R005-AUTH-PREP accepted Session Vault/SiteAccess/AccountState/PendingIntent boundary
- #33 R006-CONTRACT-PREP accepted generic Browser Worker Auth Mode/BrowserEvent/ProfileAttachmentRef boundary
- R008 `docs/security.md`, `docs/implementation-contracts.md` and central `EgressPolicy`
- R007 PlaybackSession/PlaybackItem revision authority
- `docs/site-plugin-architecture.md` and the accepted #232 R008 diagnostic evidence

Semantic freshness domains:
- `gateway-core` auth/session/capability contracts
- generic Browser Worker Auth Mode and BrowserEvent/ProfileAttachmentRef contracts
- `plugins/bilibili/**`
- R008 egress/secret/redaction and R007 playback authority docs/contracts
- this Task package and its external authorization criteria

Integration surfaces:
- workspace/package manifests and shared auth/browser contract types
- SiteAdapterRegistry/Resolution Service integration
- hosted verification workflow and any artifact manifest

Task-owned surfaces:
- research contract and feasibility evidence for the authenticated Bilibili scenario
- no production source or Secret storage surface

Authority/domain → Claim mapping:
- legal/interactive authorization and account gate → C1,C5
- #28 Vault/capability/session swap → C2,C3,C4
- #33 generic Auth Mode/BrowserEvent → C2,C4,C5
- R008 egress/secret/redaction → C3,C5,C6
- R007 Playback authority → C3,C4,C6
- Bilibili plugin boundary and #232 evidence → C2,C3,C5,C6

Integration verification:
- JI1: later implementation Candidate's affected auth/browser/R008/R007 integration checks; n/a for this docs-only package

Unrelated-main policy:
- existing accepted #28/#33/#232 Evidence remains valid; no rebase/full rerun solely because main advanced

Integration-overlap policy:
- preserve accepted semantic Evidence; compose any later implementation Candidate with the Coordinator-frozen Integration Base and run only declared JI jobs unless conflict changes auth/egress/playback semantics

Semantic-authority-change policy:
- reconcile accepted auth/browser/R008/R007 authorities and rerun mapped claims; if impact cannot be bounded, return to Contract Revision before execution

Strict-main reason:
- n/a; this research uses dependency-aware freshness

## Evidence Contract

Every Attempt report must distinguish research result, hosted verification result and any separately authorized target result and include:

```text
Task / Research: R005-AUTH-REAL-BILIBILI / C1..C6
Attempt: actual Attempt number
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high
Fast: enabled (actual availability must be recorded)
Execution plane: repository-static-analysis, github-actions or separately approved external channel
Account/authorization: presence and scope only; never credentials or account identifiers
Candidate/base/run/job/artifact: exact values when applicable
Scenario: bounded login mode and source-locator handoff class
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
Limitations: explicit unverified runtime/legal/target facts
Cleanup: profile/session/capability/pending-intent/staging cleanup state
```

Never place passwords, verification codes, Cookies, Authorization, tokens, profile archives, QR images, raw URLs, headers, bodies, DOM/HAR data or account-identifying details in Issue comments or artifacts.

## Failure / Recovery Handling

- Missing external authorization/account/channel is a genuine `BLOCKED` target prerequisite, not permission to substitute a personal account or profile.
- A failed/cancelled login must preserve the previous valid Vault session and pending playback intent without leaking Secret or corrupting Playback state.
- A browser disconnect, timeout or expiry must converge to a finite sanitized outcome and clean only Attempt-owned resources.
- Any proposed canonical/security change pauses for Coordinator Review and Contract Revision; do not implement a bypass to make login work.
- If a Worker session stops after producing a Candidate or Evidence, retain the durable Issue/PR and continue through the next Attempt; do not duplicate the Task.

## Deliverables

- Feasibility/contract evidence covering C1–C6.
- Explicit external prerequisites and an approved later target-evidence plan.
- No real account Secret, profile or login artifact.
- If an implementation gap is found, a narrow follow-up Issue proposal with no automatic dispatch.

## Completion Protocol

Package preparation must create the draft Issue and exactly `task.md` plus `prompt.md`, perform independent GitHub read-back, and leave #235 `status:draft` and owner-free. A later Worker Attempt follows:

```text
status:ready → status:in-progress → Attempt N
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner → STOP
```

Worker must not set `status:done`, close #235, publish an authenticated target, or start an implementation child automatically.
