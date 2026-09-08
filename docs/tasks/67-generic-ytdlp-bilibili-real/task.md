# Task — GENERIC-YTDLP-BILIBILI-REAL

## Metadata

- GitHub Issue: #67; Task kind: verification / real public network
- Contract Revision: R19 (GitHub-hosted build artifacts only; preserves R18 ordinary-Linux Claims and all historical Attempts)
- Planning Base: `b17a27ca5d8c2f76cddc4c7cf3fdaa239169593a`
- Exact runtime Candidate: `80fb081b129f8f664124b84ddcc9698039e2cfd1`
- Preferred Worker: Codex Cloud; eligible environment after publication: env:cloud
- Required capabilities: github-read-write, repository-static-analysis, automated-test, authenticated existing SSH tx-node, accepted #146 non-root execution path
- Target: ordinary Linux x86_64 tx-node; no phone evidence
- Hard publication dependency: #146 Final Acceptance with executable non-phone runbook/host identity/provenance
- Downstream: #68 only after Final Acceptance PASS
- Frozen public sample: `BV14V411W7r5`
- Attempt number: next unused number from live Issue history, never reuse historical Attempt 17

## Required reading

AGENTS.md startup canonical set; current Issue and all relevant comments; docs/product-roadmap.md; docs/non-phone-web-playback-plan.md; #146 task/runbook/accepted Evidence when available; docs/adr/0007-r008-anonymous-response-secret-containment.md; docs/research/generic-ytdlp-egress-research.md; runtime smoke/offline helper/lock; issue-lifecycle, terminal-write, recovery and freshness protocols.

## Authority / preserved history

The user's temporary pause on phone deployment changes execution routing and the current milestone, not R008/plugin/playback semantics or earlier result classification. R17 J2 returned frozen-page 4xx and J3 was NOT RUN. R16 remains a real pre-#114 compatibility FAIL at FALLBACK_WEBPAGE / RESPONSE_BODY_TOO_LARGE. #114 repaired that bounded normalization seam but has not yet proved live source compatibility.

Accepted chain: #79 frozen offline runtime; #83 x86_64/AArch64 sandbox authority; #85 fd isolation; #95 anonymous response Secret containment; #97 broker framing; #99 clean-build/sibling binding; #101 outcome taxonomy; #103 normalization; #105 narrow normal-extract-first continuation; #107 stage; #109 stage/reason; #111 response encoding; #114 bounded webpage marker scan. #90 remains historical accepted source transport; #146 owns the ordinary-Linux preparation/handoff.

R18 is recoverable at `88d302c8673817eb15093bf053886dbbf28f105a`; R17/earlier contract is recoverable at `b17a27ca5d8c2f76cddc4c7cf3fdaa239169593a:docs/tasks/67-generic-ytdlp-bilibili-real/task.md`. No old mobile PASS/FAIL is relabelled as x86_64 Evidence. #113/#142/#131 are not publication dependencies for R19.

## Publication completion required

Keep draft until #146 Final Accepted. Coordinator must then add to this contract the actual accepted #146 runbook Candidate, Final Acceptance URL, verified uid/gid and source/runtime preparation reference. These values are intentionally not fabricated now. If #146 changes the runtime semantic implementation rather than preparation only, review freshness and formally freeze the new exact runtime before publication.

No required phone readiness gate is inherited. #147 is independent; if its repair is needed for a required workflow, classify that concrete verification dependency rather than inventing a phone blocker.

## Goal / scope

On the accepted ordinary-Linux low-privilege path, execute one bounded Attempt using the exact runtime Candidate and the unchanged anonymous public non-DRM sample:

```text
verified exact source + frozen offline runtime
→ own current direct/no-proxy page preflight
→ accepted sandbox + fd isolation
→ BrokerProcessRunner / R008Broker
→ normal yt_dlp.extract_info(download=False)
→ existing bounded fallback only when admitted
→ current ResolvedMedia or closed failure classification
→ safe cleanup
```

No product/core/plugin implementation changes in this verification Task. Host preparation is reused from #146, never improvised as root. No phone operations, real login/Cookie/profile, header/fingerprint/IP-family variation, proxy/endpoint steering, CAPTCHA bypass, arbitrary browser-CDP extraction, browser cache substitution, media downloading, DASH/remux/transcoding, navigation, #68 execution or production enablement.

## Frozen source/runtime/security

- Source task input: `https://www.bilibili.com/video/BV14V411W7r5/`.
- Anonymous direct public networking; no existing browser identity/state.
- yt-dlp `2026.08.19`; source `3a08beaf031ab68f966401ead017ac81fe8486cf`; wheel sha256 `86a521c89017200d7cc20173b9f1d04c6588dda4eabad324b5c76d5269ee1bf9`; repository offline lock is authority.
- Raw R008/broker and JSON fallback 96 KiB remain unchanged. Only accepted FALLBACK_WEBPAGE normalized marker scanning may reach 512 KiB; only existing html/initial_state/bangumi decisions, strict accepted decoding/error rejection.
- Fixed sibling ytdlp-sandbox from same runtime Candidate, no_new_privs, denied direct socket/socketpair, inherited broker IPC and clean env remain required.
- Default production DisabledRunner remains.

## Verification Job Matrix

| Job | Plane / executor | Required evidence |
| --- | --- | --- |
| J0 | external-codex/ssh / tx-node | Revalidate #146 low-privilege identity/workspace and exact runtime SHA/source integrity; no root/capability/Secret inheritance |
| J1 | github-actions artifact + external-codex/ssh consume | Frozen lock/wheel/source verified; offline-hit/offline-prepared; verified GitHub-hosted build provenance/artifact identity from #146; no target compilation |
| J2 | external-codex/ssh / tx-node | One bounded unchanged anonymous direct preflight below, no response body/header persistence |
| J3 | external-codex/ssh / tx-node | Once only after J2 PASS: existing exact-runtime smoke; broker/protocol/stream_count/closed failure fields |
| J4 | external-codex/ssh / tx-node | Cleanup, zero owned worker/sandbox/temporary residue, only #146 allowed cache/source retained; safe-output scan |

J2: use ordinary curl, clear upper/lower HTTP_PROXY/HTTPS_PROXY/ALL_PROXY and use `--noproxy '*'`; no custom identity headers, forced address family or destination. At most 3 requests, 5 seconds between completed requests, connect timeout 5 seconds, total timeout 15 seconds per request, normal TLS validation. Do not follow unreviewed redirects: any non-2xx is not a successful sample. Stop early when two consecutive 2xx occur; otherwise after request 3 classify BLOCKED and stop before J3. Only status class/transport class is retained, stderr/body/headers discarded. Optional accepted #128 passive sanitizer must not change requests. No automatic rerun/second set without a new Coordinator-authorized Attempt and changed external condition or approved bounded diagnosis.

J3 uses only the compile-free launcher and verified binary pair accepted in #146 R2. Before publication, Coordinator must freeze the actual launcher path/arguments, wrapper Candidate, binary runtime source SHA, hosted build Candidate/run/job/artifact/manifest hashes and #146 target identity here. These are unresolved dependency outputs, so this Task remains draft until they exist.

The original scripts/generic-ytdlp-real-smoke.sh and cargo build/run/test/check/clippy are forbidden on tx-node and Codex workspace because they compile. All clean-build/provenance/test-binary generation occurs on remote GitHub-hosted Actions; target-side work only verifies and executes those artifacts with the accepted offline runtime. ABI/artifact failure returns BLOCKED, never local toolchain installation. Runtime maximum 35 minutes; no retry loop, long soak, extra extract calls or raw diagnostic expansion. J4 runs even if J2/J3 fails.

## Claims and result semantics

- C1: exact Candidate and accepted non-phone authority/provenance.
- C2: low-privilege sandbox/fd/broker/Secret boundary preserved.
- C3: current frozen source anonymously reachable under bounded J2 rule.
- C4: real accepted broker path exercised and normalized media or valid closed result obtained.
- C5: safety, clean output and cleanup.

PASS requires C1–C5 and J0–J4 PASS, broker_request_count > 0, valid muxed http-file/HLS ResolvedMedia, stream_count >= 1, and no safety failure. Environment-only or 2xx-only is not PASS.

CONDITIONAL PASS needs valid media plus a contract-compatible non-security limitation; unsupported is not conditional. Current #68 hard gate requires PASS; a conditional result returns to Coordinator without auto-publication.

FAIL: complete safe runtime executes but returns UNSUPPORTED_FORMAT with a valid stage/reason pairing and no media. Stage must be one of PRE_FALLBACK, FALLBACK_WEBPAGE, FALLBACK_NAV, FALLBACK_VIEW, FALLBACK_DETAIL, FALLBACK_PLAYURL, MEDIA_SHAPE, UNCLASSIFIED, with the exact accepted #109 mapping in the frozen worker/smoke code. Arbitrary exception text or guessed reason is not evidence.

BLOCKED: source/network/runtime/provenance/sandbox/spawn/broker/Secret/cleanup/evidence failure, EXTRACTOR_FAILURE, or malformed taxonomy. Missing required tests are NOT RUN and block acceptance, not silently passed.

## Failure and source revision decision

If J2 fails, no resolver/Browser bypass. Coordinator may retain BLOCKED, or formally revise sample only after evidence shows the old sample is unsuitable and an independently normally accessible public non-DRM sample is selected. Any replacement requires fixed identity and new #67/#68 contracts before execution; it does not prove the original sample fixed. No sample lottery or unlimited retries.

If J3 identifies media-shape deficiency, report closed fields and STOP. Coordinator may create one smallest generic compatibility Task supported by Evidence; Worker cannot add DASH/remux or site business code to Core.

## Evidence / completion

Durable bounded report: Task/Attempt/revision; exact runtime SHA; #146 accepted preparation SHA; Orchestrator=codex-cloud; Execution plane=external-codex/ssh for live steps; Executor/Target=tx-node ordinary Linux x86_64; actual version/identity classes; Actions run/job/artifact for preparation when used; J0–J4/C1–C5; bounded status/error/protocol/stream count; cleanup and limitations.

Do not retain Cookie/Auth/token/profile/Vault, raw bodies/headers/stderr, signed media URL, media metadata/content, raw endpoint data, lease or capability tokens. Report only existing safe result fields.

Worker publishes EXECUTION REPORT or BLOCKER REPORT, transitions to review/blocked, releases owner after fresh authority read and STOPs. Coordinator reviews Evidence, records ACCEPT/REVISE/BLOCK, and alone owns Final Acceptance/publication of #68. No auto-close or downstream execution.

## Freshness / Integration Contract

Freshness policy: dependency-aware; strict-main reason: n/a.
Semantic authorities: R008/ADR0007, accepted runtime chain and #146 non-phone boundary.
Semantic domains: plugins/generic-ytdlp/**, gateway-egress/**, site-adapter-api security/media schema, offline runtime helper/lock/smoke, #146 execution boundary.
Integration surfaces: Cargo.toml/Cargo.lock, runtime workflow/build inputs.
Task-owned surfaces: this contract and bounded evidence report only; no implementation.
Authority/domain → Claim mapping: source/provenance C1; runtime/security C2/C4/C5; live network C3; output/cleanup C5.
JI1: exact integration runtime/clean-build deterministic tests on remote GitHub-hosted Actions and artifact/offline verification if build surfaces overlap; no compilation on the target. JI2: rerun mapped live J0–J4 when runtime/host/source semantics change; a prior site's temporal success cannot replace current Attempt J2.
Unrelated main/docs preserve Candidate Evidence. Integration-only composition requires Coordinator frozen base and JI; semantic changes reverify mapped Claims. Exact frozen runtime does not become moving main merely because planning docs change.
