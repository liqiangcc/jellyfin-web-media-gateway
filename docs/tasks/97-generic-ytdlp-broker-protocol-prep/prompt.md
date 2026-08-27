# Session Bootstrap — GENERIC-YTDLP-BROKER-PROTOCOL-PREP

Execute Issue #97 using the repository Worker protocol.

## Read first

1. `AGENTS.md`
2. Issue #97 and all comments
3. `docs/tasks/97-generic-ytdlp-broker-protocol-prep/task.md`
4. `docs/tasks/issue-lifecycle-protocol.md`
5. `docs/tasks/freshness-integration-protocol.md`
6. #67 Attempt 6 authoritative `[BLOCKER REPORT]`
7. #60 Final Acceptance and generic-ytdlp broker runtime authority
8. #95 Final Acceptance / ADR 0007 response Secret containment
9. #85 and #83 accepted fd-isolation/sandbox authority
10. #79 frozen offline-runtime authority

Claim only if live #97 is:

```text
status:ready
env:cloud
no active owner
```

## Frozen trigger

```text
Parent: #67 Attempt 6
Triggering Candidate: 804fd60343b081e5e055ba87f68e7939b106bb19
Observed real Target result, repeated 2/2:
broker_status_class: 2xx
broker_error_code: n/a
broker_request_count: 1
process_error: BROKER_PROTOCOL
```

The real Target already proved the request reached R008 and #95 response containment. Do not reopen network, sandbox, fd-isolation or response-Secret policy work unless deterministic protocol evidence proves the Task Contract itself is invalid; in that case report a blocker and stop.

## Goal

Locate and fix only the bounded broker wire/framing failure between an already R008-accepted `BrokerResponse` and the Python worker.

Required decomposition:

```text
BrokerResponse
→ serialization
→ frame admission
→ fd-3 write
→ Python length/payload read
→ decode
→ response reconstruction
→ extractor continuation
```

You MUST prove or disprove the Task Contract hypothesis that binary-body serialization amplification can make a valid R008-bounded response exceed the current IPC frame envelope. Do not assume that hypothesis is true and do not simply increase limits without root-cause Evidence.

## Hard boundaries

- do not increase R008 HTTP body/header/count/value limits;
- any IPC bound change must be fixed and derived from existing admitted response bounds plus protocol overhead, never caller-configurable;
- malformed/zero/truncated/oversize frames remain fail closed;
- no alternate socket, raw tunnel, shared-file body handoff or direct worker network authority;
- response Secrets remain contained before worker-visible serialization under #95 / ADR 0007;
- request Cookie/Auth/token remains rejected before prohibited egress;
- do not change #83 seccomp/no_new_privs or #85 fd isolation;
- do not change frozen yt-dlp identity/#79 provenance;
- production `GenericYtdlpAdapter::default()` remains `DisabledRunner`;
- no real Bilibili/site request in this Task;
- no DASH/remux/FFmpeg/navigation/Browser/Web E2E/performance work;
- no raw body/Secret/signed URL/raw stderr in Evidence.

## Verification

Produce one exact final Candidate and exact-Candidate J1-J3 Evidence from `task.md`:

```text
J1 protocol root cause + actual Rust/Python round-trip near accepted response bound
J2 zero/truncated/malformed/oversize + Secret/no-direct-egress + timeout/cancel negatives
J3 workspace + generic-ytdlp runtime/security regressions
```

The near-limit proof must exercise the actual `BrokerProcessRunner` and Python worker protocol, not only an isolated codec helper.

## Stop boundary

Normal completion:

```text
[EXECUTION REPORT]
→ status:review
→ release active owner
→ STOP
```

Blocker:

```text
[BLOCKER REPORT]
→ status:blocked
→ release active owner
→ STOP
```

Never merge your own PR, set `status:done`, close #97, execute/re-freeze #67, or start #68.
