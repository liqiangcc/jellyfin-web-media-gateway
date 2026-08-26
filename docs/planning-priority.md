# Delivery Priority

This document records the current execution priority for the media gateway project.

The detailed current product capability/task map is `product-roadmap.md`.

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
- Evidence can be returned through GitHub Actions / the approved execution plane.

Environment readiness must not claim CPU/RSS/temperature/throughput/soak feasibility.

Current environment route:

```text
#64 ENV-ARM64-RUST
→ #63 ENV-ARM64-READY
```

A fresh accepted Target Runner smoke and #63 Attempt 1 already prove the scheduling/security/direct-network slices, including direct/no-proxy HTTP 200 reachability for the frozen Bilibili sample. The remaining environment blocker is low-privilege functional toolchain/Gateway execution, not Bilibili network reachability.

## Functional closure

After environment readiness, prioritize user-visible end-to-end behavior.

The current first real-site playback route is:

```text
#66 GENERIC-YTDLP-EXTRACT-PREP          [accepted]
→ #73 GENERIC-YTDLP-REAL-HARNESS-PREP
→ #67 GENERIC-YTDLP-BILIBILI-REAL
→ #68 BILIBILI-WEB-E2E
```

#73 is a hard Evidence/reproducibility dependency for #67. It adds the repository-owned safe command that connects the accepted #66 extraction library to the real R008Broker/BrokerProcessRunner path and emits only bounded, non-secret diagnostics. The phone must not invent ad-hoc real-site code.

#73 must not increase the current R008 96 KiB broker body limit, weaken egress/Secret policy, use ambient proxy authority, or enable generic-ytdlp production registration. Real compatibility failures remain Evidence for #67/Coordinator classification.

Target user-visible path:

```text
Bilibili URL
→ SiteAdapterRegistry
→ generic-ytdlp
→ current ResolvedMedia
→ Gateway
→ Web Display
→ Control play/pause/seek/stop
```

The legacy `#36 → #23 → PR #37` route is superseded for first playback because #65 proved that the preserved #23 branch conflicts semantically with the accepted current SiteAdapter authority. #23/#36 are closed `not_planned`, and PR #37 is closed unmerged as historical Evidence.

## Current parallelism

The two immediate lanes are independent until #67:

```text
phone environment: #64 → #63 ──┐
                                ├→ #67
Cloud harness:     #73 ─────────┘
```

Therefore #73 and #64 should execute in parallel. #67 waits for both accepted harness Evidence and a ready selected target environment.

## Functional expansion after first playback

Once the first Bilibili playback path is accepted, expand capabilities in generic layers rather than rebuilding a monolithic site plugin:

```text
Continuous content
→ #71 SITE-NAVIGATION-PREP
→ #72 BILIBILI-NAVIGATION

Source-site accounts
→ #28 accepted auth foundation
→ #26 future real-auth child

Native site controls
→ #33 accepted Browser/Native Panel contracts
→ #27 future R006-RUNTIME-FUNCTIONAL child
→ concrete site panel interpretation
```

#71/#72 are planning-only drafts. They must not displace the current first-playback critical path unless Coordinator explicitly reprioritizes.

Navigation, authentication and Native Panel work may be planned in advance, but should not displace the current first-playback critical path unless they expose a hard dependency.

## Performance / capacity

R003-TARGET (#9) remains the authoritative Ubuntu ARM64 performance/resource verification task, including sustained Direct/Remux/Chromium scenarios, CPU/RSS/temperature and 5/30/60-minute Evidence.

It is intentionally deferred until primary functional paths are stable enough that measurements are representative.

Important separation:

```text
Chromium/Gateway can function
!=
Chromium/Gateway is performant enough for always-on production use
```

Functional runtime Tasks may therefore proceed before #9 when they make no resource/capacity claim.

## Physical-device gates

Physical-TV verification remains independent and should execute when the TV environment is available. Missing physical-TV Evidence must not be substituted by hosted browser evidence.

## Current top-level execution graph

```text
#64 → #63 ───────────────┐
                        ├→ #67 → #68
#66(done) → #73 ────────┘
                              ├→ #71 → #72
                              ├→ #26 future real-auth child
                              └→ #27 future functional Browser/Native Panel child

#9 performance/capacity later
```

No-hard-dependency ready Tasks should execute in parallel.