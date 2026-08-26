# Delivery Priority

This document records the current execution priority for the media gateway project.

The ordering is intentionally:

```text
Environment readiness
→ Functional closure
→ Real-world compatibility
→ Performance / capacity
→ Production hardening
```

## Environment readiness

Environment work answers only whether a target can be used for functional development and verification:

- Runner/host is reachable and schedulable;
- low-privilege and workspace/Vault boundaries remain intact;
- required build/runtime tools are present or the concrete missing capability is known;
- the Gateway can be built/started/stopped in an isolated test location;
- required public network paths can be exercised without credential/access-control bypass;
- Evidence can be returned through GitHub Actions.

Environment readiness must not claim CPU/RSS/temperature/throughput/soak feasibility.

Current dedicated environment task:

```text
#63 ENV-ARM64-READY
→ phone Runner / low-privilege boundary
→ functional toolchain/runtime readiness
→ isolated Gateway functional smoke
→ public-network route classification
```

A fresh accepted `target-runner-smoke` rerun on 2026-08-26 already proves the first scheduling/security slice on `ubuntu-arm64-target-phone`; #63 owns only the remaining functional-environment readiness, not performance.

## Functional closure

After environment readiness, prioritize user-visible end-to-end behavior. Current real-site priority is:

```text
R005-PUBLIC-REAL (#36)
→ R005-PUBLIC (#23)
→ real Bilibili URL → Site Plugin → ResolvedMedia → Gateway → Web Display E2E
```

If #63 finds a concrete environment blocker required by this functional chain, fix that blocker first and then resume the same functional Task. Do not convert environment work into premature performance benchmarking.

## Performance / capacity

R003-TARGET (#9) remains the authoritative Ubuntu ARM64 performance/resource verification task, including sustained Direct/Remux, CPU/RSS/temperature and 5/30/60-minute Evidence.

It is intentionally deferred until the primary functional paths are stable enough that measurements are representative. Deferring execution does not weaken or rewrite the #9 Task Contract.

## Physical-device gates

Physical-TV verification remains independent and should execute when the TV environment is available. Missing physical-TV Evidence must not be substituted by hosted browser evidence.
