# Task — CI-NAVIGATION-WORKFLOW-REPAIR

## Metadata

- GitHub Issue: #147
- Task kind: combined
- Planning Base: `b17a27ca5d8c2f76cddc4c7cf3fdaa239169593a`
- Worker: Codex Cloud; eligible environment: env:cloud
- Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-test
- Hard publication dependencies: none
- Candidate: exact durable Worker commit containing the minimal repair

## Goal / observed defect

Main run https://github.com/liqiangcc/jellyfin-web-media-gateway/actions/runs/33374985619 ended failure with zero jobs, and gh run view suggests a workflow-file issue. Diagnose from actual validation evidence and restore executable exact-Candidate navigation verification. This is an independent CI defect, not a Bilibili/phone Task or a new CI platform.

Read AGENTS.md startup docs, docs/tasks/issue-lifecycle-protocol.md, freshness/recovery/terminal-write protocols, .github/workflows/site-navigation-prep.yml, #71 accepted navigation authority and existing tests referenced by this workflow.

## Scope / requirements

1. Read actual run/job/check/annotation/PR validation, reproduce structural/expression/YAML failure with appropriate parser/actionlint where available, and record the proven cause. Do not assume zero jobs means a test assertion failed.
2. Minimally repair `.github/workflows/site-navigation-prep.yml`; focused validation script/docs are allowed only if useful. Preserve least permissions, Candidate SHA validation/checkout assertion, timeouts, all J1–J4 verification intent, navigation/CAS/idempotency/stale/security coverage and normal triggers.
3. Use safe block scalars/structured inputs for shell and no untrusted string interpolation. Do not change product code, assertions to hide failures, workflow security scope or unrelated workflows.
4. Keep Candidate-specific Evidence and declare any actual unrelated baseline failure. No phone, SSH tx-node, live Bilibili, Runner recovery or infrastructure changes.

## Claims / Verification Job Matrix

| Job | Claim | Plane / runner | Required evidence |
| --- | --- | --- | --- |
| J1 | C1 proven root cause, C2 valid workflow | local diagnosis + github-actions hosted x64 | Before/after structural validation, exact changed workflow; actionlint/YAML validation when available |
| J2 | C3 executable complete workflow | github-actions / hosted x64 | Candidate PR-triggered site-navigation-prep creates all four jobs and they PASS; each asserts exact PR head SHA |
| J3 | C4 coverage/security unchanged | static review + J2 runtime | J1–J4 selectors retained and really run; no skips/cancel replacing required Evidence; contents:read/checkout persist-credentials:false/timeouts intact |

Use pull_request path trigger on the repaired Candidate, not workflow_dispatch of an invalid default-branch workflow. If PR validation cannot execute until definition is accepted, report that exact blocker; do not merge your own PR or bypass checks. Inspect Actions logs/artifacts for failures and fix only in-scope defects. Required existing workflow runs navigation/conformance, preparation/stale cases, complete Playback concurrency/HTTP command tests, workspace fmt/clippy/test and architecture/security regression.

## Success Criteria / Evidence Contract

C1–C4 PASS on one Candidate; known root cause documented; all repaired workflow jobs run and pass; no weakened assertions/permissions. Report Task/Attempt/base/Candidate/PR; actual Actions run/job URLs; validation commands; root cause; selector/permission diff; freshness; no site/device work. Never call a no-job/invalid workflow successful.

## Freshness / Integration Contract

Freshness policy: dependency-aware; strict-main reason: n/a.
Semantic authorities: #71 navigation + R007/R008 and current exact-Candidate workflow rules.
Semantic domains: navigation APIs/tests, command/revision/stale contracts, workflow Candidate security.
Integration surfaces: Cargo.toml/Cargo.lock, shared build/toolchain; task-owned: .github/workflows/site-navigation-prep.yml and optional focused validation doc/script.
Authority/domain → Claim mapping: workflow syntax C1/C2; test/domain/build surface C3/C4; permissions/Candidate identity C4.
JI1: actionlint/structural validation + the repaired four-job workflow on exact Integration Candidate. Rerun mapped tests when semantic authority changes; unrelated main docs preserve Evidence. Integration overlap uses Coordinator-frozen base. Contract-invalidating change returns to draft.

## Completion / failure

Produce focused Candidate/PR and report C1–C4 plus run/job evidence. Actual product regression discovered by restored jobs is reported rather than weakening tests or repairing product outside scope. Report → review/blocked → release owner → STOP. Coordinator owns review/merge/Final Acceptance; Worker cannot close Issue or start #146/#67.
