# Task — ENV-ARM64-GITHUB-SMARTHTTP-PREP

## Metadata

```text
GitHub Issue: #90
Task ID: ENV-ARM64-GITHUB-SMARTHTTP-PREP
Task kind: implementation + target transport verification
Planning Base: b64757d72c628ddd0f9ae2d1eb6d5e15c8337410
Parent environment Task: #89 ENV-ARM64-GITHUB-EGRESS-DIAG
Blocked product Task: #85 BROKER-FD-ISOLATION-LEGACY-KERNEL-PREP
Preserved #85 Candidate: 4af64b124af4d1599a87bd211395ee832e9d7e4b
Preferred worker: cloud-codex
Eligible environment: env:cloud
Target evidence: ubuntu-arm64-target-phone through trusted GitHub Actions
Required capabilities: github-read-write, workflow-authoring, actions-trigger-read, target-runner-routing, security-evidence-authoring
Session Bootstrap: docs/tasks/90-env-arm64-github-smarthttp-prep/prompt.md
Downstream Handoff: docs/tasks/handoffs/cloud.md
Freshness policy: dependency-aware
```

> Issue #90 owns live status, Attempt, owner and result. This file owns the stable recovery-path contract.

## Trigger Evidence

#89 Attempt 1 is BLOCKED after completing its Direct/Proxy characterization.

Accepted trigger summary from its durable Blocker Report:

```text
target: Linux 4.19.113-964403 / aarch64
runtime user: gateway-runner uid 999
Runner HTTP proxy inheritance: none; NO_PROXY/no_proxy only
Direct IPv4 curl repeat: 5/5
Direct git ls-remote: 5/5
Existing loopback mihomo curl: 5/5 pre-repair
Existing loopback mihomo git ls-remote: 5/5
IPv6: unavailable
Git: 2.43.0 / libcurl-GnuTLS
curl: 8.5.0 / OpenSSL
Exact Candidate fetch through proxy: 0/3
Exact Candidate fetch with proxy + HTTP/1.1: 0/1
Exact Candidate fetch Direct: 0/1
Temporary user Git proxy repair: rolled back
CHECKOUT_RESUME_ELIGIBLE_FOR_#85: no
```

This narrows the blocker from generic GitHub reachability to repository-object transfer / smart-HTTP behavior. Small HTTPS and Git advertisement/metadata operations are not sufficient proof for `actions/checkout` or an exact source tree.

The actual #85 J4 failure occurred inside `actions/checkout@v4` before fd-isolation execution. The target did successfully receive required Action packages, and prior accepted target work has successfully consumed GitHub Actions artifacts. Therefore a trusted workflow can diagnose the real Actions transport context without first checking out the repository and can also prove an artifact-based exact-source fallback without assuming it is needed.

## Goal

Produce one evidence-backed, security-preserving recovery route for delivering the exact #85 Candidate source to the accepted target verification environment.

Required decision flow:

```text
trusted workflow starts on target without repository checkout
        ↓
reproduce/classify Git smart-HTTP object transfer in real Actions context
        ↓
anonymous/public path vs Actions-token-authenticated path
        ↓
only bounded protocol/path variants justified by #89 evidence
        ↓
A. smart-HTTP exact Candidate fetch 3x succeeds
        → SMARTHTTP_RECOVERY_ELIGIBLE_FOR_#89=yes

or

B. smart-HTTP remains unreliable
        → hosted runner creates exact Candidate source bundle
        → immutable manifest + SHA256 + Candidate identity
        → Actions artifact transport to target
        → target verifies and extracts exact source 3x
        → TRUSTED_SOURCE_BUNDLE_ELIGIBLE_FOR_#85=yes
```

The Task does not itself resume #85 and does not execute #67. Coordinator decides parent routing after Review.

## Canonical / Durable Sources

Read before execution:

- `AGENTS.md`
- Issue #90 and all comments
- Issue #89 and its Attempt 1 Blocker Report
- Issue #85 current comments and PR #86 identity
- this `task.md`
- `docs/tasks/issue-lifecycle-protocol.md`
- `docs/tasks/freshness-integration-protocol.md`
- `docs/tasks/handoffs/cloud.md`
- `docs/runner-execution-architecture.md`
- target-related sections of `docs/security.md`
- `.github/workflows/broker-fd-isolation.yml` at `4af64b124af4d1599a87bd211395ee832e9d7e4b`

## Invariants

1. Preserve #85 runtime/security semantics. No fd-isolation, sandbox, R008 or generic-ytdlp product behavior changes.
2. Do not execute #85 J4 fd-isolation proof from this Task. Only transport/recovery-path evidence is allowed.
3. Do not execute #67 or any Bilibili/site request.
4. Target remains the accepted low-privilege `gateway-runner`; no root/sudo/ADB/Vault/production Secret authority.
5. Do not modify Android/kernel/VPN/TUN/firewall/routes/DNS daemon/system-wide proxy settings.
6. Do not rotate or invent public/residential proxies. The only proxy path eligible for comparison is a legitimate existing path already established by #89, unless Coordinator later revises this Contract.
7. Do not weaken TLS (`sslVerify=false`, custom insecure CA bypass, plaintext credential workaround, etc.).
8. Never print or persist `GITHUB_TOKEN`, `GH_TOKEN`, Authorization headers, proxy credentials or masked values reconstructed from them.
9. Token-authenticated Git tests must use ephemeral workflow-scoped credentials and leave no persistent credential helper or `.gitconfig` secret behind.
10. A source-bundle fallback must be produced on a trusted GitHub-hosted runner from the exact requested Candidate, contain no `.git` credentials/state, and be bound to an immutable manifest/hash checked on target before use.
11. Do not report artifact transport as equivalent to Git smart-HTTP recovery. These are separate outcomes with separate Coordinator consequences.
12. Worker reports and STOPs. Only Coordinator can unblock/accept #89 or revise/resume #85.

## Allowed Repository Surface

The Worker may add the smallest task-specific trusted workflow and helper needed for deterministic verification, preferably:

```text
.github/workflows/<task-specific target Git transport workflow>.yml
scripts/<task-specific manifest/bundle helper>   # only if justified
```

Avoid shared production code. Do not modify `gateway-core/**`, `gateway-egress/**`, `site-adapter-api/**`, `display-adapter-api/**`, `plugins/**` product semantics, or PR #86 fd-isolation implementation merely to make transport pass.

A later Coordinator decision may choose to integrate an accepted bundle mechanism into #85 or broader target workflows; that integration is not automatically authorized by this Task.

## Claims

```text
C1 — Pre-checkout target execution
A trusted GitHub Actions workflow can reach the accepted `ubuntu-arm64-target-phone` and perform bounded diagnostics before any repository checkout, under the accepted low-privilege boundary.

C2 — Real Actions credential/path classification
The Task distinguishes anonymous/public Git transport from the workflow's ephemeral repository-read credential path without exposing credentials, and records which path `actions/checkout`-equivalent Git requests actually need.

C3 — Smart-HTTP failure layer is isolated
Evidence separates advertisement/metadata success from object-transfer/upload-pack failure and records whether a bounded protocol/path variant changes the result.

C4 — Smart-HTTP recovery is proven when claimed
If a smart-HTTP route is selected, the exact preserved Candidate is fetched into three fresh target repositories consecutively and each checkout verifies the exact SHA.

C5 — Trusted source-bundle provenance is proven when claimed
If smart-HTTP remains unreliable, a GitHub-hosted trusted job creates a source bundle from the exact preserved Candidate, emits Candidate identity + manifest + SHA256, uploads it through GitHub Actions artifact transport, and the target verifies/extracts it without relying on repository Git object transfer.

C6 — Repetition distinguishes transport stability from one-off success
Any route recommended to Coordinator is exercised repeatedly. One lucky request is not sufficient.

C7 — Parent routing output is explicit
The final report distinguishes:
SMARTHTTP_RECOVERY_ELIGIBLE_FOR_#89: yes|no
TRUSTED_SOURCE_BUNDLE_ELIGIBLE_FOR_#85: yes|no
and states the minimal next Coordinator action without executing it.
```

Claim result vocabulary: `PASS | CONDITIONAL PASS | FAIL | BLOCKED | NOT RUN`.

## Verification / Implementation Plan

### J0 — Read-back and candidate freeze

Before editing:

- read #89 Blocker Report;
- read #85 / PR #86 current identity;
- verify preserved Candidate remains `4af64b124af4d1599a87bd211395ee832e9d7e4b` and is still reachable from the repository;
- read current `main` and compare only for semantic/integration effects defined below.

Do not silently substitute moving `main` for the preserved #85 Candidate.

### J1 — Implement trusted pre-checkout workflow

Create a narrowly scoped `workflow_dispatch` workflow that:

- validates a 40-hex `candidate_sha` input;
- uses least permissions (`contents: read`, `actions: read` only as required);
- can run target diagnostics before `actions/checkout`;
- never echoes token-bearing headers or full environment;
- has bounded timeouts and cleanup;
- targets only the accepted self-hosted labels for target jobs;
- cannot be automatically triggered by arbitrary fork/untrusted PR code.

Static verification must prove no secret dumping, no TLS weakening, no product/site work and no unsafe shell interpolation of untrusted input.

### J2 — Hosted exact-source bundle producer

In the trusted hosted job:

1. validate Candidate input;
2. checkout exactly the requested Candidate with `persist-credentials: false`;
3. verify `git rev-parse HEAD == candidate_sha`;
4. create a source-only archive from that exact tree using a repository-owned/deterministic mechanism such as `git archive`;
5. create a manifest containing at minimum:
   - repository identity;
   - exact Candidate SHA;
   - archive filename;
   - SHA256;
   - creation workflow/run identity where safely available;
6. assert no `.git`, credential helper, token, Vault or production Secret is included;
7. upload as a same-run Actions artifact with bounded retention.

Bundle production does not imply the target will use it; it prepares a controlled fallback proof.

### J3 — Target pre-checkout identity / boundary

The target job must begin without repository checkout and record only safe bounded identity:

```text
uname -a
uname -m
id
whoami
git --version
curl --version
runner labels/name as available
```

Require:

- aarch64;
- non-root;
- no sudo requirement;
- workspace outside production state;
- no Vault/production Secret access.

### J4 — Smart-HTTP stage classification in real Actions context

Without persisting credentials, compare the minimum bounded matrix needed to reproduce #89's gap:

```text
anonymous/public git ls-remote
workflow-token git ls-remote
anonymous exact Candidate fetch
workflow-token exact Candidate fetch
```

Use fresh temporary repositories. The authenticated path may use an ephemeral command-scoped header equivalent to repository read access, but the credential value must be masked and never printed.

If exact fetch fails, classify whether failure occurs before connection, during TLS, during service advertisement, during `git-upload-pack`/object response, or during checkout. Local verbose traces may be inspected only after sanitization; durable Evidence contains summaries, not raw Authorization-bearing traces.

### J5 — Bounded protocol/path variants only when needed

Only after J4 evidence, test a small bounded set of non-security-weakening variants relevant to the observed layer, for example:

- Git protocol v2 vs v1 if service negotiation appears relevant;
- default HTTP vs command-scoped HTTP/1.1 only as a comparison, acknowledging #89 already found HTTP/1.1 insufficient interactively;
- Direct vs the already-established loopback private proxy path if the real Actions context permits a clean comparison;
- branch-ref fetch vs exact SHA only to distinguish ref-advertisement semantics from transfer transport, while final proof must still verify the exact Candidate SHA.

Do not expand into generic tuning, throughput benchmarking, package replacement or privileged network changes.

### J6A — Smart-HTTP exact Candidate proof

If one safe smart-HTTP path becomes viable, perform at least:

```text
3 consecutive fresh target repo initializations
→ fetch source through the selected path
→ checkout
→ verify git rev-parse HEAD == 4af64b124af4d1599a87bd211395ee832e9d7e4b
→ cleanup
```

Also verify any required configuration is actually available to the real target Actions job without storing credentials or weakening TLS.

Only then set:

```text
SMARTHTTP_RECOVERY_ELIGIBLE_FOR_#89=yes
```

### J6B — Trusted source-bundle fallback proof

If smart-HTTP remains unreliable, do not force a PASS. Instead prove the hosted bundle path.

The target must, without repository checkout:

1. download the hosted exact-source artifact through GitHub Actions;
2. verify manifest identity and SHA256 before extraction/use;
3. extract into a fresh non-production temporary directory;
4. verify the extracted tree is the exact Candidate source by a deterministic bundle manifest/tree proof defined by the implementation;
5. prove there is no `.git` credential state or production Secret material;
6. clean the directory;
7. repeat the transport/verify/extract proof sufficiently to demonstrate stability, preferably three fresh downloads/paths in one bounded target workflow or equivalent repeated target jobs.

Only then set:

```text
TRUSTED_SOURCE_BUNDLE_ELIGIBLE_FOR_#85=yes
```

This is a transport alternative, not a claim that `actions/checkout` works.

### J7 — Negative / cleanup proof

Verify:

- no user/system proxy configuration was left changed;
- no credential helper or token-bearing Git config persists;
- no target root/sudo/Vault access occurred;
- temporary repositories/bundles were removed;
- no product/site workflow ran;
- target Runner returns to an idle/clean state sufficient for Coordinator review.

### J8 — Final routing report

The Worker report must include:

```text
Preserved #85 Candidate:
Workflow / run / target job:
Target identity / privilege:
Anonymous advertisement result:
Token-authenticated advertisement result:
Anonymous exact fetch result:
Token-authenticated exact fetch result:
Protocol/path variant result:
Exact smart-HTTP fetch 3x:
Hosted bundle SHA256 / manifest identity (if used):
Target artifact transport repetitions (if used):
SMARTHTTP_RECOVERY_ELIGIBLE_FOR_#89: yes | no
TRUSTED_SOURCE_BUNDLE_ELIGIBLE_FOR_#85: yes | no
Result: PASS | CONDITIONAL PASS | FAIL | BLOCKED
```

Worker must not change #89/#85 lifecycle after posting the report.

## Result Criteria

### PASS

At least one recovery route is fully proven without weakening boundaries:

1. smart-HTTP exact Candidate fetch succeeds 3 consecutive times in the real target Actions context, **or**
2. smart-HTTP remains unreliable but the exact hosted source-bundle artifact route is proven repeatedly with immutable provenance/hash verification on the target.

All applicable claims for the selected route are supported by exact run/job Evidence and C1/C2/C3/C6/C7 are accepted.

### CONDITIONAL PASS

A recovery route is strong enough for a bounded parent follow-up, but one non-critical diagnostic dimension remains unavailable and does not obscure source identity, transport integrity or security. State the exact limitation.

### FAIL

Any required route needs TLS weakening, credential leakage/persistence, privileged target expansion, untrusted workflow execution, proxy rotation/bypass, or cannot bind source bytes to the exact Candidate.

### BLOCKED

Examples:

- target Runner unavailable;
- workflow cannot run before checkout;
- no safe ephemeral repository credential path can be tested and artifact transport is also unavailable;
- both smart-HTTP and Actions artifact transport fail repeatedly;
- source identity/provenance cannot be verified without prohibited authority.

## Success Criteria

1. #89 Attempt 1 blocker and preserved #85 Candidate are read back correctly.
2. A trusted pre-checkout target workflow exists and is bounded by accepted Runner security rules.
3. Real Actions-context smart-HTTP advertisement vs object-transfer behavior is classified without secret leakage.
4. Anonymous vs workflow-token repository-read paths are distinguished safely.
5. No route is accepted from a single lucky request.
6. Smart-HTTP recovery, if claimed, proves exact Candidate checkout 3x on target.
7. Source-bundle fallback, if claimed, is built on hosted trusted infrastructure from the exact Candidate and verified by immutable manifest/hash on target with repeated delivery proof.
8. No product/runtime/site/security semantics are weakened or changed.
9. Cleanup and low-privilege target boundaries are preserved.
10. Final Worker report explicitly returns both parent-routing booleans and STOPs.

## Freshness / Integration Contract

### Policy

```text
Freshness policy: dependency-aware
```

### Semantic authorities

- #89 Attempt 1 Blocker Evidence for the observed target failure class;
- accepted Target Runner low-privilege/security boundary;
- #85 exact Candidate identity and verification requirement;
- `docs/runner-execution-architecture.md` target workflow trust model;
- `docs/security.md` Secret/TLS/target isolation rules.

### Semantic freshness domains

```text
#89 blocker classification
#85 Candidate identity / PR #86 target verification requirement
Target Runner trust / privilege boundary
GitHub Actions token / artifact permission model used by this workflow
```

### Integration surfaces

```text
.github/workflows/**
workflow permissions / target labels
repository scripts used only for source manifest/bundle proof
```

### Task-owned surfaces

Prefer only a new task-specific workflow and, if necessary, a task-specific helper script. Product/runtime source is not owned by this Task.

### Authority/domain → Claim mapping

```text
#89 failure classification          → C2, C3, C4
#85 exact Candidate identity        → C4, C5, C7
Target Runner security authority    → C1, C2, C5, C7
Actions artifact/token semantics    → C2, C5, C6
```

### Integration verification

If current `main` changes only in unrelated product/docs surfaces, Task-specific Evidence remains valid. If target workflow permissions, Runner labels/security boundary, #89 blocker evidence, or #85 Candidate identity changes materially before execution/review, Coordinator must reclassify freshness and may require partial rerun or Contract Revision.

### Unrelated-main policy

Unrelated `main` advancement does not require broad product regression or rebase solely for freshness. The Task must never silently replace the preserved #85 Candidate with moving `main`.

## Lifecycle

```text
status:draft
→ Publication Gate
→ status:ready + env:cloud + no active owner
→ Worker claim / Attempt N
→ implementation + hosted/target transport proof
→ [EXECUTION REPORT] or [BLOCKER REPORT]
→ status:review or status:blocked
→ release owner
→ STOP
→ Coordinator Review
```

Worker must not set `status:done`, close #90, unblock/close #89, resume #85, merge PR #86, or start #67.