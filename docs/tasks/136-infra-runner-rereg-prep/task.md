# INFRA-RUNNER-REREG-PREP

## Identity

- Issue: #136
- Kind: infrastructure recovery PREP / tooling + protocol
- Preferred Worker: cloud
- Eligible environment: `env:cloud`
- Parent incident: #131 `INFRA-TARGET-RUNNER-LISTENER-RECOVERY`
- Planning Base: `8e93fd2c6b580f2adef28aef8cebad411fcec97a`
- Trigger authority: #131 Attempt 5 diagnosis + Coordinator Attempt 6 recovery direction
- Scope: repository-owned recovery guard/wrapper, deterministic offline tests, runbook/docs only
- Live phone/GitHub runner mutation: forbidden in this Task

## Problem

#131 exhausted bounded restart/wait diagnosis for the existing phone Runner. The old stuck workflow assignment was cleared, but the existing registration continued to return `TaskAgentSessionConflictException` / `A session for this runner already exists` and no local Listener remained alive.

The next supported recovery class is remove/re-register of the self-hosted Runner. That operation is destructive enough that it must not be improvised after phone connectivity returns.

## Goal

Prepare a deterministic, token-safe, fail-closed recovery procedure that a later Coordinator-authorized #131 Attempt may execute only after the accepted phone is durably reachable through a persistent management context.

This Task is PREP only. It MUST NOT create live registration/remove tokens, remove/register a real Runner, connect to the phone, restart the Runner, rerun/cancel/dispatch workflows, or change product/runtime behavior.

## Official-interface constraint

The supported runner interface uses the registration/remove token as the secret `--token` argument of `config.sh`. Therefore token material is permitted to exist only in the short-lived argv of that official `config.sh` child process. It MUST NOT be present in the caller's argv, shell history, xtrace, durable logs, Issue comments, artifacts, repository files, process-inventory evidence, temp files, or debug dumps.

A wrapper may receive the token over stdin/ephemeral FD/in-memory input, disable tracing, invoke only the expected official `config.sh` operation, then immediately clear its token variable. Durable output contains success/failure classes only.

## Frozen recovery sequence for later #131 use

A later real recovery must follow this order and may not skip gates:

1. **Connectivity/persistence gate**
   - accepted phone is Tailnet reachable;
   - SSH succeeds without interactive uncertainty;
   - a persistent SSH → Ubuntu/chroot management context is already alive and proven to remain alive independently of one-shot command lifetime.
2. **Idle/safety gate**
   - no active Runner Listener/Worker on the old registration;
   - GitHub runner is not busy and no active assigned workflow job exists;
   - no concurrent recovery attempt/owner exists.
3. **Non-secret identity snapshot**
   - repository scope;
   - runner name `ubuntu-arm64-target-phone`;
   - exact accepted labels as observed live at execution time;
   - work directory contract `_work` (or exact accepted current value if live authority differs);
   - final runtime identity uid 999 `gateway-runner` and accepted no-privilege-expansion control boundary.
   - Numeric GitHub runner id is not an invariant: remove/re-register may produce a new id. Identity preservation means repository/name/labels/workspace/final uid semantics.
4. **Remove phase**
   - obtain one short-lived repository remove token through an authorized Coordinator/GitHub control path;
   - deliver token to the phone wrapper through stdin/ephemeral input, never local/SSH command argv;
   - run exactly one official `config.sh remove --token <secret>` under the accepted runner installation/user boundary;
   - verify old local registration files are removed only through official tooling and old GitHub runner entry is absent/non-authoritative.
5. **Register phase**
   - obtain one fresh short-lived registration token only after remove phase success;
   - deliver it through the same token-safe input boundary;
   - run exactly one official unattended `config.sh` registration with the frozen repository/name/labels/workspace identity;
   - do not use `--replace` as a substitute for a failed remove phase.
6. **Low-privilege control restore**
   - restore/use the accepted `gateway-runnerctl` / `setpriv` final runtime boundary;
   - prove `Runner.Listener` remains alive as uid 999 `gateway-runner`, accepted workspace unchanged, no capability/sudo privilege expansion.
7. **GitHub consistency gate**
   - fresh-read the runner object and prove matching name/labels plus `online + busy=false`;
   - only after local + GitHub consistency PASS may #131 reconsider the already-authorized same-run rerun.

If connectivity disappears after the destructive gate, a token operation fails, the old registration cannot be cleanly removed, identity cannot be reproduced, or Listener readiness cannot be proven, STOP. Do not loop remove/register operations.

## Implementation requirements

Implement the smallest repository-owned recovery PREP surfaces:

1. a pure precondition/identity guard that consumes a normalized non-secret snapshot and returns bounded `AUTHORIZED | BLOCKED` with a reason;
2. a narrow token wrapper or equivalent tested adapter that:
   - reads the sentinel/real token from stdin or an ephemeral FD, not wrapper argv;
   - disables xtrace before reading the token;
   - only permits the official `config.sh` `remove` or `register` shape;
   - never echoes the token or reflects it on failures;
   - clears the token variable after the child returns;
   - supports deterministic tests using a fake `config.sh`, never a real runner;
3. a runbook showing the later #131 sequencing and exact stop boundaries.

Do not create general remote-execution/orchestration infrastructure.

## Claims

- C1: PREP is fully offline and cannot mutate a real Runner/GitHub/phone state in its tests.
- C2: guard rejects missing persistent phone context, busy/active-job state, concurrent authority, incomplete identity snapshot, or privilege-boundary mismatch.
- C3: guard authorizes only the fully satisfied non-secret precondition snapshot.
- C4: wrapper token ingress is stdin/ephemeral input; sentinel token never appears in wrapper argv/stdout/stderr/durable test output.
- C5: token can appear only in the fake/official `config.sh --token` child argv; wrapper never logs/reflects it and tracing is disabled.
- C6: only one `remove` or one `register` child invocation is permitted per wrapper execution; unknown modes/paths/arguments fail closed.
- C7: register plan preserves repository/name/live-frozen-labels/workspace and final uid-999 boundary; numeric runner id is explicitly not preserved.
- C8: no `--replace` fallback/retry loop is authorized.
- C9: runbook requires local Listener + GitHub online/idle consistency before any workflow rerun.
- C10: #131 remains the only authority for live execution; #136 does not itself authorize registration mutation.

## Deterministic verification

Offline tests MUST cover at least:

- authorized complete precondition snapshot;
- phone/Tailnet/SSH/persistent-context negative cases;
- runner busy / active job negative cases;
- missing or changed runner name/labels/workspace/uid boundary negative cases;
- concurrent owner/attempt negative case;
- remove wrapper with sentinel token: fake child receives token but wrapper stdout/stderr do not;
- register wrapper with sentinel token: fake child receives the intended non-secret identity args and token, without wrapper leakage;
- token absent from wrapper process argv and generated durable files;
- child failure produces only bounded failure class without token reflection;
- unknown mode / unexpected config path / `--replace` request rejected;
- one-invocation bound;
- static scan for xtrace/debug/token-echo/history/temp-file anti-patterns in changed files.

## Boundaries

- no live phone network connection;
- no live token creation;
- no live GitHub runner remove/register/delete;
- no runner restart/start/stop;
- no workflow cancel/rerun/dispatch;
- no Secret/config/credential dump;
- no product/media/browser/site/security changes;
- no Bilibili/#67/#68/#113 execution;
- no credential material in repository/tests/artifacts.

## Evidence required

Worker report must include:

- exact Candidate + PR;
- offline guard/wrapper test count and PASS result;
- compile/lint/syntax result;
- diff-scope proof;
- targeted secret/leak scan result and any tooling limitation;
- explicit `Live Runner mutation: NOT RUN`;
- explicit `Live phone connection: NOT RUN`.

## Success criteria

PASS requires C1-C10, deterministic offline tests, bounded/non-reflective token behavior, exact destructive sequencing and stop gates, no live mutation, and a narrow reviewable Candidate.
