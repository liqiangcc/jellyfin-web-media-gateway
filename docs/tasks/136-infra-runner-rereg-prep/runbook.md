# Phone Runner remove/re-register recovery runbook

Status: **PREP only**. This runbook is not authority to mutate the live phone or GitHub Runner. Live execution requires a fresh #131 Coordinator gate after the phone is reachable.

## Why this exists

#131 established that repeated start/restart/wait loops do not repair the existing registration: the Runner repeatedly receives a GitHub session conflict and exits. The old stuck workflow assignment was already cancelled/released. A later repair therefore needs one controlled official remove/re-register cycle, not another blind restart loop.

## Non-negotiable entry gate

Before any destructive action, fresh evidence must prove all of:

- phone Tailnet reachability;
- non-interactive SSH reachability;
- a persistent SSH → Ubuntu/chroot management context that will remain alive independently of a one-shot command;
- #131 live authority for the current recovery Attempt;
- GitHub runner not busy and no active assigned job;
- no old local `Runner.Listener` / `Runner.Worker` process;
- frozen non-secret identity: repository scope, runner name `ubuntu-arm64-target-phone`, exact full live label set, accepted work directory, final uid 999 `gateway-runner`;
- the full live label set contains the normal Linux ARM64 default labels (`self-hosted`, `Linux`, `ARM64`); capture the remaining custom-label subset separately. If default-label semantics cannot be established, BLOCK rather than guessing.
- rollback/recovery plan is available.

Normalize that evidence and pass it through `scripts/runner_rereg_guard.py`. Any `BLOCKED` result stops the recovery before token acquisition/removal.

A GitHub numeric runner id is **not** an identity invariant after remove/re-register. Preserve repository, runner name, labels, workspace and final low-privilege runtime semantics.

## Token handling boundary

The supported actions/runner interface takes the short-lived secret as `config.sh --token TOKEN`. Therefore:

- never paste a token through an Agent Runtime / terminal text transcript;
- never put a token in the wrapper/SSH command argv;
- disable xtrace before acquiring/handling it;
- do not write API token responses to files;
- do not print token API output to a visible terminal;
- do not inspect process argv while the official `config.sh` child is running;
- do not include the token in logs, Issues, artifacts, shell history or diagnostic captures;
- clear/unset token state immediately after the official child returns.

`bash scripts/runner_rereg_token_wrapper.sh ...` receives the token only on stdin and suppresses the official child stdout/stderr. The token may exist briefly in that `config.sh` child argv because this is the official interface; durable Evidence records only bounded success/failure classes.

If the available control plane can only deliver secrets by visible `write_text`/terminal paste, **BLOCK** rather than exposing the token.

## Recovery transaction

### Phase A — remove old registration

1. Re-run the entry guard immediately before token creation.
2. Obtain exactly one short-lived repository remove token through the authorized GitHub control plane.
3. Pipe the token directly into `bash scripts/runner_rereg_token_wrapper.sh ...` stdin/ephemeral input boundary.
4. Invoke wrapper mode `remove` against the accepted installation `config.sh`.
5. Require wrapper `SUCCESS`.
6. Fresh-read GitHub/non-secret local state to establish that the old registration is no longer authoritative.

If removal fails or connectivity is lost, STOP. Do not invoke `--replace`, do not request another remove token loop, and do not proceed to registration.

### Phase B — register the same semantic identity

1. Fresh-read #131 authority and phone connectivity again.
2. Freeze the exact full live labels captured before removal. Derive the custom-label subset by removing the proven default labels (`self-hosted`, `Linux`, `ARM64`); do not invent or broaden labels.
3. Obtain exactly one fresh repository registration token.
4. Pipe it directly to wrapper mode `register` with:
   - repository URL/scope;
   - runner name `ubuntu-arm64-target-phone`;
   - exact frozen custom-label subset (`-` only when the subset is proven empty, causing the wrapper to omit `--labels`);
   - accepted work directory `_work` unless live accepted authority explicitly froze another value.
5. Require wrapper `SUCCESS`.

Do not use `--replace`. A failed registration requires Coordinator review before any second destructive attempt.

### Phase C — restore the accepted final runtime

After successful registration:

1. Use the accepted `gateway-runnerctl` / `setpriv` control path.
2. Prove a durable `Runner.Listener` exists as uid 999 `gateway-runner`.
3. Prove accepted workspace and no privilege/capability expansion.
4. Fresh-read GitHub and prove the runner with the frozen name and **full** label set is `online + busy=false`.
5. Only after local + GitHub consistency PASS may #131 reconsider the already-authorized same-run workflow rerun.

## Failure classes

Durable Evidence may record only bounded classes such as:

- `PRECONDITION_BLOCKED:<reason>`
- `REMOVE_SUCCESS`
- `REMOVE_FAILED`
- `REGISTER_SUCCESS`
- `REGISTER_FAILED`
- `LISTENER_NOT_READY`
- `GITHUB_NOT_IDLE`
- `RECOVERY_READY_FOR_RERUN_GATE`

Never include token values, credential/config contents, raw API responses, shell transcripts containing secrets, or process-argv captures.

## #136 execution boundary

For Issue #136 itself:

- Live Runner mutation: **NOT RUN**
- Live phone connection: **NOT RUN**
- real token creation: **NOT RUN**
- tests use only a temporary fake `config.sh`.
