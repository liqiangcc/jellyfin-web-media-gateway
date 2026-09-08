# Task — BILIBILI-BROWSER-TARGET-RUNTIME

## Metadata

```text
GitHub Issue: #172
Parent Goal / Research Item: #68 / R005 browser source portability; downstream #166
Task ID: BILIBILI-BROWSER-TARGET-RUNTIME
Task kind: combined
Base commit: f9a48dbef6535c4bb38188b050384ac4156187af
Dependency Candidate: eb6598d473768f544744431c5af792d6de86d59f (accepted #169)
Session bootstrap prompt: docs/tasks/172-bilibili-browser-target-runtime/prompt.md
Preferred worker: cloud-codex
Eligible worker environments: env:cloud
Required capabilities: github-read-write, repository-static-analysis, code-authoring, automated-build, automated-test
Hard publication dependencies: #169 Final Acceptance at merged main f9a48dbef6535c4bb38188b050384ac4156187af
```

> GitHub Actions / Runner is the execution backend. It does not claim this Issue. Issue state, Attempt, owner, PR and result history remain the Coordinator/Worker lifecycle record.

## Goal / Context

Make the accepted #169 browser-probe artifact runnable from a clean target directory with the target's pre-installed system Chrome and Node, without an npm install, workspace symlink, personal profile, proxy or secret. The current bundle contains the probe scripts but omits `node_modules/playwright-core`; hosted J3 succeeds only because it installs a workspace dependency and links it into the downloaded bundle. A clean `tx-node` therefore cannot yet execute the documented live entry reproducibly.

This Task closes only the packaging/runtime-closure gap. It does not run a live Bilibili request, perform target/phone/TV verification, or claim Gateway playback. #166 remains the independent owner of later anonymous live-site evidence on tx-node.

## Task Decomposition / Routing

```text
Verification mode: inline for package closure, manifest and downloaded synthetic consumer claims
Separate target evidence: #166 owns tx-node live browser/source portability
Worker/client: cloud-codex
Eligible environment: env:cloud
Portable execution plane: GitHub-hosted Actions, x64
Target execution in this Task: none
```

Cloud is the coding/orchestration environment, not a Runner. Every build, package installation and test-binary run in this Task must happen in GitHub Actions. Local development may inspect or edit files, but no local compilation/test execution or tx-node installation/build is permitted. The worker must use `gpt-5.6-luna` with reasoning high; enable Fast only if the runtime exposes it and report the actual setting.

## Scope

### In scope

1. Extend the experimental artifact builder and its package metadata so the bundle contains the exact pinned `playwright-core` runtime needed by `probe.mjs` and `live.mjs`.
2. Pin `playwright-core` to `1.55.0` from the accepted #169 contract using a reproducible npm lock/integrity or an equivalently reviewable package archive. Install with scripts disabled in hosted Actions. Do not include a Chromium/Chrome binary; the target admission will use its system browser.
3. Copy the runtime into the artifact under a normal, non-symlinked `node_modules/playwright-core` tree, including the package metadata and license required for runtime/provenance. The build must fail closed if the package version or resolved path is not the frozen value.
4. Extend the manifest with dependency name/version, source/integrity when available, every packaged runtime file's size/digest, package layout, and the fact that the browser is external. Preserve the existing Candidate SHA, entry and budget fields.
5. Change the downloaded-artifact consumer job and checks so it verifies and executes the bundle's own runtime. It must not install npm packages after download, symlink workspace `node_modules`, or silently fall back to a workspace copy. It must prove module resolution from the bundle and run the synthetic consumer against a separately discovered browser executable.
6. Add deterministic hosted tests for missing/mismatched dependency, symlink/workspace leakage, manifest digest/version mismatch, and clean-directory module resolution. Keep #169 synthetic broker/selector behavior unchanged except for packaging seams.
7. Update the focused workflow, artifact README/runbook and #166 runbook admission wording only as needed to describe the self-contained Node runtime, external system-Chrome requirement, exact version and no-install command. Do not insert live Bilibili results or freeze target evidence here.

### Out of scope

- Live Bilibili requests, tx-node SSH execution, target/phone/TV deployment, VNC, autoplay/audible UX or manual observation.
- Production SiteAdapter/Registry, Core, Playback, Display, Control, Vault, ResolvedMedia or EgressPolicy semantic changes.
- Browser broker, selector, observation-schema or independent-consumer redesign except a narrowly necessary packaging/test hook.
- Downloading or vendoring Chrome/Chromium, installing packages on tx-node, using a personal browser/profile, cookies, Authorization, CDP endpoint or proxy.
- Remux/transcode/MSE/player work and any #67 generic-ytdlp rerun.

If a required change crosses these boundaries, stop and report a Contract Revision/Blocker with evidence rather than weakening the task.

## Architecture / Security Invariants

- Gateway remains the `PlaybackSession` authority; this artifact is an experimental acquisition probe, not a playback path.
- Browser runtime is generic; Bilibili URL/part interpretation remains plugin-owned.
- `SourceLocator` is opaque and versioned; short-lived media URLs never enter the manifest, Issue, logs or exported evidence.
- No Cookie, Authorization, signed URL, raw DOM/playinfo, HAR, media body, Vault data or long-lived secret is packaged or exported.
- The package archive is treated as executable dependency content: scripts are disabled during installation, only the pinned package is copied, and no post-download network install is allowed.
- Target Runner security boundaries remain unchanged. The target later supplies an ordinary low-privilege user and system Chrome; it does not inherit Gateway/Vault/root/ADB authority.

## Implementation Requirements

1. Define one authoritative dependency pin for `playwright-core@1.55.0` and make the workflow, builder and tests consume it. If an npm lock or package archive is used, record the integrity/digest and reject drift. Avoid adding unrelated dependencies.
2. Build the bundle in a clean staging directory. The resulting `node_modules/playwright-core` must contain regular files (no symlink into `$GITHUB_WORKSPACE`, runner cache or developer home), resolve with `require.resolve('playwright-core', { paths: [bundle] })`, and report the frozen package version. The artifact must remain usable when copied to a directory with no ancestor `node_modules`.
3. Include all files needed by the package's Node runtime and license/provenance, while excluding browser binaries, npm cache, tests and unrelated workspace modules. Use a deterministic allowlisted archive/layout or verify the full staged package file set before manifesting it.
4. Preserve `artifact.mjs verify` as a strict digest/size check. It must validate schema, Candidate SHA, dependency metadata, package path and no-symlink/no-outside-root conditions before a consumer starts.
5. Make `consumer.mjs` and the downloaded-artifact workflow invoke only the bundle entry and bundled runtime. A separate browser executable may be installed/discovered by the hosted runner for the synthetic check; the workflow must pass its path explicitly and must not make the bundle depend on the full `playwright` package.
6. Keep the #169 live entry opt-in and unchanged in authority: the caller supplies only `--mode live --selector bilibili:BV14V411W7r5:part-2` in the later #166 run; no URL/profile/header/proxy/CDP argument may appear in package metadata or command construction.
7. Ensure cleanup and failure paths remove temporary staging and do not leave npm credentials, cache, cookies, signed URLs or raw media bodies in uploaded artifacts.
8. Do not run live mode in hosted CI. Synthetic fixtures remain the only browser/network exercise for this Task.

## Claims

```text
C1: the artifact has a self-contained, exact playwright-core@1.55.0 runtime closure and no workspace/symlink dependency.
C2: manifest/provenance verification detects package version, integrity, path, digest, size and browser-runtime drift before execution.
C3: a downloaded bundle in a clean directory runs the synthetic probe/consumer with an explicitly supplied external Chrome path and no npm install or workspace module access.
C4: GitHub-hosted Actions reproduce the package, downloaded-consumer and integration checks from the exact Candidate SHA; no live/target result is claimed.
```

## Verification Job Matrix

| Job ID | Claim(s) | Execution plane | Runner / target | Required commands / checks | Evidence |
| --- | --- | --- | --- | --- | --- |
| J1 | C1,C2 | github-actions | hosted x64 / clean Node workspace | install only pinned `playwright-core` with scripts disabled; build artifact; assert package version/integrity, regular files, manifest entries and no outside-root paths | exact run/job, manifest, sanitized package inventory |
| J2 | C1,C3 | github-actions | hosted x64 / downloaded bundle + external Chromium | download J1 artifact; do not run npm install or create a workspace symlink; verify manifest; move/copy bundle to a clean directory; resolve bundled module and run synthetic `consumer.mjs` with discovered browser path | exact run/job, resolution path, consumer PASS and byte/request counts |
| J3 | C2,C4 | github-actions | hosted x64 / static boundary | reject modified/missing package, symlink, digest/version mismatch and workspace fallback; scan for browser binaries, secrets, URLs and live execution in workflow | exact Candidate static output |
| J4 | C4 | github-actions | hosted x64 / workspace | exact Candidate remote `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace --all-targets` | run/job IDs and logs |

J1–J3 timeout ≤15 minutes; J4 timeout ≤35 minutes. All jobs must checkout and assert the same exact Candidate SHA. Jobs do not claim Issue state and do not access tx-node, a phone, TV, VNC or live Bilibili.

## Success Criteria

1. C1–C4 are PASS on one exact Candidate SHA with a reviewable PR and sanitized Actions evidence.
2. `artifact.mjs verify` and the downloaded consumer succeed when the bundle is isolated from the repository and all ancestor `node_modules`.
3. The manifest identifies the pinned `playwright-core` dependency and external-browser requirement; no browser binary or secret is packaged.
4. The #166 runbook has a truthful no-install target admission note and still clearly reserves live Bilibili execution and target evidence for #166.
5. No production Core/adapter/playback semantics change and no local/tx-node build or test is used.

Claim interpretation:

```text
C1 PASS = exact package files/version are inside the bundle and module resolution never leaves it.
C2 PASS = tampering and layout/provenance negatives fail closed before browser start.
C3 PASS = downloaded isolated bundle runs synthetic browser/consumer with external Chrome and no install/symlink.
C4 PASS = required hosted jobs pass for the exact Candidate and artifacts are addressable; no live or target claim is inferred.
```

## Evidence Contract

Every Attempt report and Coordinator Review must record:

```text
Role: implementation | combined
Task / Claims: #172 / C1..C4
Attempt: N
Worker / Orchestrator: cloud-codex / gpt-5.6-luna high (+ actual Fast availability)
Execution plane: github-actions
Runner: github-hosted x64 / ubuntu-latest
Target: clean synthetic artifact workspace; no live site
OS / architecture and versions: actual Actions Node, npm, playwright-core and external browser versions
Network path: npm package retrieval only in producer; loopback synthetic fixtures in consumer
Base / Candidate commit: exact SHAs
Workflow / run / job: exact IDs and URLs
Commands / package pin: exact no-script install, artifact and consumer commands
Artifact / digest: exact IDs, manifest dependency entries and archive digest
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Keep implementation result, each Claim result, Coordinator decision and parent #68/R005 gate separate. Never include signed URLs, cookies, Authorization, raw bodies or full browser profile data. Worker must follow the lifecycle protocol: claim only from `status:ready`, report `[EXECUTION REPORT]` or `[BLOCKER REPORT]`, move to review/blocked, release ownership and stop. Worker may not close the Issue.

## Freshness / Integration Contract

Freshness policy: dependency-aware.

Semantic authorities:

- accepted #169 artifact/experimental boundary at merged main `f9a48dbef6535c4bb38188b050384ac4156187af`;
- `AGENTS.md`, `docs/security.md`, `docs/implementation-contracts.md`, browser/plugin contracts and R008 egress/secret rules;
- `docs/research/bilibili-browser-probe-runbook.md` and #166 target contract.

Task-owned surfaces:

- `experiments/bilibili-browser-probe/artifact.mjs`, `consumer.mjs`, package metadata and focused tests;
- `.github/workflows/bilibili-browser-probe.yml` package/consumer jobs;
- focused artifact README/runbook wording and this task's manifest schema extension.

Integration surfaces:

- npm dependency pin and artifact layout;
- workflow artifact transfer and browser discovery;
- #166 runbook admission reference.

Unrelated main changes do not automatically stale exact-Candidate package evidence. If a semantic authority or #169 artifact layout changes, reconcile the dependency, update the Task Contract if needed, and rerun mapped Claims. If integration conflict changes runtime/security semantics, return to Contract Revision instead of using an integration-only shortcut.

## Preconditions / Stop Conditions

- #169 Final Acceptance is complete; use the accepted Candidate/provenance above as the starting point.
- GitHub-hosted x64 Actions and npm registry access are available to the workflow.
- No live Bilibili, tx-node, phone, TV or production service is needed.

Stop and report `BLOCKED` if the pinned package cannot be retrieved reproducibly, cannot run without a browser binary in the bundle, or requires a production API/security change. Do not relax no-install, no-symlink, secret, browser-external or hosted-only constraints to make a job green.

## Completion Protocol

Coordinator publishes only after Issue/task/prompt read-back, `status:ready + env:cloud` queue verification and a real downstream handoff. After Worker evidence, Coordinator must read history, task, candidate/PR and all required jobs, post `[COORDINATOR REVIEW]`, and decide `ACCEPT`, `REVISE`, `BLOCK`, `SPLIT` or `NOT_PLANNED`. Only after `[FINAL ACCEPTANCE]` may the Issue become `status:done` and be closed. Acceptance of #172 unlocks revision/publication of #166; it does not close #166 or #68.
