# #146 Attempt 1 — non-phone execution evidence

Worker execution outcome: COMPLETED. Worker Claim results below are not a
Coordinator acceptance or permission to execute #67. No phone deployment or
live Bilibili request occurred.

## Identity and provenance

- Contract: #146 R3; Attempt 1; Orchestrator: codex-cloud.
- Planning Base: `b17a27ca5d8c2f76cddc4c7cf3fdaa239169593a`.
- Execution Base / observed main: `cdaad48bcdd6ac1a003f81fdf69d6a2103a857d1`.
- Verified Task/code Candidate: `89ea37ade718c88bb5be4c43b856c1518184ea8a`.
- Frozen runtime source: `80fb081b129f8f664124b84ddcc9698039e2cfd1`.
- PR: [#151](https://github.com/liqiangcc/jellyfin-web-media-gateway/pull/151).
- Workflow: `.github/workflows/non-phone-execution.yml`, push event, exact code
  Candidate; build and workflow source identities checked independently.
- Hosted execution plane: github-actions; Runner: GitHub-hosted Ubuntu 24.04
  x86_64. Target execution plane: external-codex/ssh; target alias: tx-node.
- Actual target: Ubuntu 26.04, x86_64, glibc 2.43, uid/gid 1001; only primary
  group; cleared inheritable/permitted/effective/bounding/ambient capabilities;
  no_new_privs=1. Account password locked, non-login shell, home mode 0700.

This evidence file is a documentation-only addition after verification. It does
not change the verified code, workflow, runbook, source layout or artifact.
Evidence-commit identity is recorded separately in the Issue execution report;
no runtime execution on that documentation-only commit is claimed.

Authenticated delivery binding:

```json
{
  "artifact_id": 10037555844,
  "candidate": "89ea37ade718c88bb5be4c43b856c1518184ea8a",
  "digest": "sha256:9706c027e7460ec26e6d158bca99c121593b57ec63f348d48f9c3b8acbd9514f",
  "job_id": 101907606484,
  "manifest_sha256": "93fe78f9f41c778a75faaa0214355eeb9cc35dcb5c2547544590070ec82303d0",
  "repository": "liqiangcc/jellyfin-web-media-gateway",
  "run_attempt": 1,
  "run_id": 34176767478,
  "runtime": "80fb081b129f8f664124b84ddcc9698039e2cfd1",
  "workflow_path": ".github/workflows/non-phone-execution.yml"
}
```

Target interpreter/browser identity:

```json
{
  "chrome": "Google Chrome 152.0.7977.82",
  "python": "3.14.4",
  "python_sha256": "fa9796cd3a30878e11a2f40372f773d3fcd913fff35e5bee8dd9a036e22e93ab"
}
```

The bootstrap `artifact.py` SHA256 is
`865537f22573abd98d5096f07024471ded0b6c9f2e7ce85a58fd4245c87dea0d`;
`readiness.py` SHA256 is
`e4aca44e4550f3900754e3f79f4c39d38dcbf50944be5f365d3ccf1f2b726c71`.
Both were transferred as the dedicated user and compared before execution.

## Hosted verification

[Run 34176767478](https://github.com/liqiangcc/jellyfin-web-media-gateway/actions/runs/34176767478)
completed SUCCESS at the verified code Candidate:

| Job | Result | Observation |
| --- | --- | --- |
| helpers / 101907606308 | PASS | 77 existing and new Python tests; includes traversal, duplicate/link/missing/mutated files, quota and rewritten receipt rejection; fixed diagnostic sentinel containment |
| build / 101907606484 | PASS | untouched frozen source at fixed path; 18 runtime tests passed; clean-build smoke/sibling regression passed; 1067 packaged files |
| consumer / 101907994978 | PASS | fresh Runner without runtime checkout; admit → verify → six direct tests + smoke → verify; missing/mutated worker and sandbox rejected 4/4 |

The frozen smoke's actual loopback request returns BROKER_EGRESS_REJECTED before
networking. This exercises the compiled worker path, fixed sibling startup,
offline import and broker boundary without a real-site request. Each selected
test reports one passed test; zero selected tests cannot yield PASS.

All 43 applicable PR check runs at the code Candidate succeeded after one
same-SHA retry of the Control Chromium test. The Linux 4.19 ARM64 target proof
was conditionally skipped, not passed. The existing navigation invalid workflow
still fails before jobs and belongs to #147; it is not counted as successful.

## Target verification and re-entry

Runtime root/layout is the one frozen in [runbook.md](runbook.md). The exact
GitHub archive digest and manifest digest were checked before admission. Target
admission accepted 1067 files, then `verify → probe → verify` passed. A separate
re-entry `probe → verify → readiness.py` also passed.

Each target probe directly executed these six existing runtime selectors:

- `pinned_worker_uses_actual_ytdlp_request_handler_and_existing_parser`
- `inherited_ipc_capability_supports_multiple_broker_requests`
- `seccomp_denies_worker_custom_handler_and_child_but_ipc_survives`
- `non_cloexec_ambient_fd_is_not_admitted_beyond_broker_fd`
- `diagnostics_consume_secret_sentinel_without_crossing_error_boundary`
- `r008_broker_rejects_secret_userinfo_and_private_targets_before_network`

Both target probe rounds reported:

```json
{"cache":"verified-warm","operation":"probe","result":"PASS","smoke":"BROKER_EGRESS_REJECTED","tests":6}
```

The target had no pip. The wheel/cache was built and installed on Actions,
restored read-only, then checked with the accepted offline helper's bundle and
cache/import verification. No target installation, compiler, cargo invocation
or implicit rebuild occurred.

Browser proof uses the existing Google Chrome executable with a fresh anonymous
profile and normal sandbox/autoplay settings. CPU/output/core limits are applied
by prlimit. Both browser and fixture are on tx-node; HTTP listens only on
loopback and no CDP listener exists. A browser JS POST checks matching Host and
Origin. The current helper passed twice, including post-runtime re-entry.
This proves fixture access only, not real media playback/autoplay/TV UX.

## Results by Claim

| Claim | Worker result | Evidence |
| --- | --- | --- |
| C1 exact provenance | PASS | checked source/run/job/artifact/digests; identical target archive |
| C2 bounded tooling | PASS | hosted tests, admission limits, fixed-source build and compile-free launchers |
| C3 low privilege | PASS | target uid/gid 1001, primary group only, no_new_privs, zero capabilities |
| C4 runtime readiness | PASS | six real prebuilt tests twice, compiled smoke loopback denial, cache verify/re-entry |
| C5 browser topology/access | PASS | normal isolated Chrome, loopback Host/Origin proof |
| C6 cleanup/privacy | PASS | fixed diagnostics, admission negatives, target cleanup observations below |
| C7 recoverability | PASS | delivered runbook plus actual second runtime/browser entry |

Final cleanup observation:

```json
{
  "account_password": "locked",
  "admission_staging": 0,
  "fixture_staging": 0,
  "remaining_runtime_processes": 0,
  "result": "PASS",
  "transfer_staging": 0
}
```

Retained intentionally: the locked dedicated account; reviewed readiness and
artifact helpers; authenticated expected.json and verified ZIP; admitted
binaries/worker/lock/helper/cache/manifest/receipt. No test process, temporary
fixture profile, transfer staging, admission staging, service or Runner remains.
No production state, personal profile, Vault, SSH key or GitHub token was copied.

## Failures and recovery preserved

- Initial package Candidate `0949bc142ec984fb6934721f834fad71985bb852`, run
  34176622125: build passed, consumer verification rejected extraction directory
  permissions. Fixed before the final Candidate; the old failure is not relabelled.
- Superseded queued/running validations were cancelled after a newer Candidate
  existed; cancelled jobs are not evidence for the final Candidate.
- Control UI run 34176767453 initially timed out waiting for pause feedback.
  No Control code changed. One failed-job retry on the same SHA passed; the
  initial timing failure remains a known intermittent observation.
- Workspace artifact download stalled; a bounded range recovery also failed.
  The successful delivery used a fresh GitHub-issued short-lived artifact
  download address passed over SSH stdin to the dedicated target user. No
  GitHub token was sent to the target and the address was neither logged nor
  retained. Target download was bounded and verified against the authenticated
  GitHub archive digest. Its manifest hash also matched the independently
  authenticated hosted consumer log. Incomplete target staging was removed.
  The normal `download.py` route itself passed on the hosted consumer.

## Handoff limits

Coordinator must review this Candidate, required Actions and target evidence
before Final Acceptance and publication of #67. #67 still owns live frozen
sample preflight and resolver compatibility; #68 owns real Web playback and
must build its own product Candidate. No phone/TV/resource claim is inferred.
