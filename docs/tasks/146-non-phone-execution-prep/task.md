# Task — NON-PHONE-EXECUTION-PREP

## Metadata

- GitHub Issue: #146
- Parent goal: non-phone real Web playback; downstream #67 and #68
- Task kind: combined (bounded execution tooling/runbook + actual host readiness)
- Planning Base: `b17a27ca5d8c2f76cddc4c7cf3fdaa239169593a`
- Compatibility/runtime source to prepare: `80fb081b129f8f664124b84ddcc9698039e2cfd1`
- Worker: Codex Cloud, `env:cloud`
- Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test, interactive-linux-debug, authenticated existing SSH alias `tx-node`
- Hard publication dependencies: none; accepted #79/#83/#85/#95/#97/#99/#114 and R008 are semantic authorities, not pending tasks.
- Browser capability: Chrome DevTools MCP if available; an isolated repository-owned Playwright browser is an allowed equivalent for the local fixture access Claim.
- Candidate: Worker creates a durable exact Task Candidate for any tooling/runbook changes before required verification; report separately from the frozen runtime source.

## Required reading

AGENTS.md canonical startup set; docs/product-roadmap.md; docs/non-phone-web-playback-plan.md; docs/security.md; docs/runner-execution-architecture.md; docs/research/generic-ytdlp-egress-research.md; docs/adr/0007-r008-anonymous-response-secret-containment.md; #67 history; scripts/generic-ytdlp-real-smoke.sh; scripts/generic-ytdlp-offline-runtime.py and lock; plugins/generic-ytdlp runtime/smoke/sandbox code; docs/tasks/issue-lifecycle-protocol.md and freshness/recovery/terminal-write protocols.

## Goal and decomposition

Deliver and prove a reproducible, isolated ordinary-Linux execution path on tx-node that a later Worker can use without relying on this chat or root runtime. It must prepare the frozen #114 source/runtime for #67 and demonstrate safe browser access to an isolated local fixture for #68.

This is an independent execution-environment/runbook deliverable. It does not repeat #67 real extraction or #68 product Claims. No phone or site reachability dependency is needed to author and prove it. Keep all preparation Jobs in this one Task; do not spawn further environment Tasks by default.

## Scope and authorized host preparation

1. Inspect only non-secret identity/capability facts: OS/arch, available tools, bounded capacity sufficiency, dedicated paths, SSH reachability. No network scan, process/env/profile/credential dump.
2. Reuse an appropriate existing dedicated non-root account if positively verified; otherwise creation of one local locked-password account `gateway-verify`, its primary group, and `/home/gateway-verify` is explicitly permitted using the existing tx-node management connection. If the name belongs to unrelated work, BLOCK rather than modify it. No login key/password/sudo entry is added.
3. Record actual numeric uid/gid and freeze the handoff command. The default root SSH account may perform only this narrow ownership/setup operation. All build, fixtures, Gateway, broker/extractor and test-browser runtime must be non-root with a minimal explicit environment and no inherited privilege/capabilities. Test account must not read root/private profiles, Gateway Vault or other account Secrets; no production file reads to test this.
4. Use only owned subdirectories for exact source, verified cache and temporary runtime. Do not change production services, firewall, SSH config, system browser settings or system packages. No phone access, ADB, Tailscale recovery, Runner install or re-registration.
5. Reuse accepted offline bundle tooling and frozen yt-dlp wheel/source identity. Artifact expiration is not permission to install latest yt-dlp; rebuild with accepted workflow and verify lock plus actual build Candidate/run/artifact. Artifact ID is resolved from GitHub at execution, never guessed.
6. Select the smallest viable build method after read-only ABI/toolchain inspection: verified hosted-x64 build artifact plus exact source/manifest, or a pinned verified Rust toolchain in the dedicated user's home with locked dependencies. Preparation-only public package/source acquisition is permitted with fixed versions/provenance; it cannot enter the live extractor environment. No `curl | sh`, arbitrary shell interpolation or root package install. Record exact versions and hashes. If neither route is feasible within these boundaries, BLOCK with one actionable incompatibility.
7. Reproduce the existing smoke launcher semantics: both smoke binary and fixed sibling ytdlp-sandbox derive from the exact runtime source. A prebuilt route needs source/Candidate/artifact hash/ABI binding and safe extraction; do not weaken #99 clean-build or sibling discovery. Prefer existing script on a user toolchain when this avoids a new packaging system.
8. Verify existing no_new_privs/fd/socket denial/broker IPC contracts on x86_64 using deterministic tests; no live Bilibili request. Repository guards must have a real exercise; finding a binary is insufficient.
9. Determine where the chosen browser runs. Use an isolated anonymous test browser/context with normal sandbox/autoplay settings. Do not inspect, copy or depend on the user's existing Chrome profile. Demonstrate bounded local fixture access to a non-root loopback test server through same-host access or a dedicated loopback SSH tunnel. Confirm matching Host/Origin and no public Gateway/CDP listener. A Playwright browser may substitute for MCP if MCP topology cannot be proven; record the actual choice.
10. Deliver restart/re-entry/cleanup commands and prove cleanup. Temporary server/browser/tunnel/processes exit; keep only the explicitly documented dedicated user and verified toolchain/cache/source needed by downstream Tasks. No always-on production instance or daemon is created.

## Implementation boundaries / files

Expected changes: a small `scripts/non-phone-execution/` helper if required, focused deterministic tests, one focused hosted workflow if existing Actions cannot express required preparation tests, and `docs/tasks/146-non-phone-execution-prep/runbook.md` plus sanitized Evidence doc. Avoid creating helpers when a concrete runbook plus existing scripts suffice.

Do not change Core/plugin/media/security semantics, Cargo dependencies for convenience, real sample, frozen yt-dlp, default DisabledRunner, raw/normalized body limits, or existing phone tooling. Do not repair navigation workflow here; #147 owns it. No Bilibili homepage/sample/API/media request, resolver smoke against real sites, #67 claim, #68 product execution, performance/thermal/soak or production deployment.

## Claims / Verification Job Matrix

| Job | Claim | Plane / host | Required selector / observation |
| --- | --- | --- | --- |
| J1 | C1 exact provenance, C2 bounded tooling | github-actions / hosted x64 | Frozen runtime build/verify workflow with exact source SHA; new helper tests when added; bash -n/Python compile as applicable; assert Candidate/run/artifact identities |
| J2 | C3 low privilege, C4 runtime readiness | external-codex/ssh / tx-node | Exact-source clean build or validated artifact; non-root uid/gid/env/capabilities; deterministic sandbox/broker tests (no site network); offline install + warm-cache reuse |
| J3 | C5 browser topology/access | external-codex / tx-node plus identified browser host | Isolated fixture health/readiness, normal browser sandbox, loopback/tunnel and Host/Origin, no public listener |
| J4 | C6 cleanup/privacy, C7 recoverability | github-actions tests + external-codex/ssh checks | Negative secret sentinel in reporter tests if reporter added; bounded safe-output scan; stop only owned processes; exercise runbook re-entry once with no user data |

If helpers change, required hosted commands include their focused tests plus existing `python3 -m unittest discover -s scripts/tests -p 'test_*.py'`; if Rust/runtime code would need changing, STOP for scope review. Deterministic runtime checks use `cargo test -p generic-ytdlp --features runtime-prep --test runtime` with verified offline runtime, and `bash scripts/test-generic-ytdlp-clean-build.sh` after inspecting its prerequisites; record selectors and bounded output. Inspect tests before remote execution and suppress any real-site selectors. Build/test timeout per job <=35 minutes, fixture/browser window <=5 minutes. Do not run benchmark loops.

## Success Criteria

All C1–C7 PASS; exact frozen runtime can be prepared safely on actual tx-node; final runtime non-root/no-sudo/no extra capabilities; existing sandbox and broker tests truly run; local fixture browser access is proved with actual topology; downstream runbook specifies identities, source/cache/bootstrap command/cleanup and artifact regeneration; required Actions pass; no live site/phone operation; changes and sanitized Evidence are durable Candidate/PR.

A missing browser/access/runtime requirement is BLOCKED, not CONDITIONAL PASS. Host setup success alone is not #67/#68 success. No known unsupported ABI may be hidden by recording build-only PASS.

## Evidence Contract

Record Task/Attempt, Planning Base, Task Candidate, frozen runtime SHA, Orchestrator=codex-cloud, actual Execution plane (github-actions or external-codex/ssh), runner/host/target/OS/arch, uid/gid capability classes, exact versions, workflow/run/job/artifact identities, test selectors, C1–C7, browser host/access classification, cleanup and intentionally retained dedicated resources. No secret, SSH key, token, browser profile, raw source/media payload or sensitive network endpoint data. Use non-secret alias tx-node; do not publish machine credentials or public endpoints.

## Failure / handoff

Runtime incompatibility or missing capabilities: bounded BLOCKER REPORT with the failed layer and reusable Candidate/PR; no package/host/restart loops or security relaxation. SSH unavailable: no alternate host/phone substitution. Ordinary in-scope helper/test bugs can be fixed in the same Attempt; independent semantic blocker requires Coordinator.

After Final Acceptance, Coordinator publishes #67 R18 using the actual accepted runbook/host Evidence. #67 will run its own current site preflight; #146 does not certify reachability. Worker reports → review/blocked → release owner → STOP; never publishes #67 itself.

## Freshness / Integration Contract

Freshness policy: dependency-aware. Strict-main reason: n/a.
Semantic authorities: R008/#79/#83/#85/#95/#97/#99/#114 and non-phone security boundary.
Semantic freshness domains: scripts/generic-ytdlp-real-smoke.sh; offline runtime helper/lock; plugins/generic-ytdlp runtime/smoke/sandbox; gateway-egress; relevant security docs.
Integration surfaces: Cargo.toml/Cargo.lock and existing runtime workflow/toolchain.
Task-owned surfaces: scripts/non-phone-execution/ and focused tests/workflow if needed; this runbook/Evidence.
Authority/domain → Claim mapping: provenance/build C1/C2/C4; privilege/sandbox C3/C4/C6; browser topology C5/C6; runbook C7.
JI1: focused new helper tests + existing offline runtime/cache tests on exact Integration Candidate.
JI2: deterministic runtime/clean-build tests when shared runtime build surfaces overlap; rerun live J2/J3 only when host/runtime/browser semantics changed.
Unrelated main changes preserve exact-Candidate Evidence. Integration overlap composes with Coordinator-frozen base and runs JI; semantic change reruns mapped Claims; contract-invalidating change returns to draft/publication. No blanket latest-main/full rerun rule.
