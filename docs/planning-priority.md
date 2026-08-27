# Delivery Priority

This document records the current execution priority for the media gateway project.

The detailed capability/task map is `product-roadmap.md`. Live Task status and ownership remain GitHub Issue authority.

The ordering is intentionally:

```text
Environment readiness
→ Functional closure
→ Real-world compatibility
→ Performance / capacity
→ Production hardening
```

## 1. Immediate critical path

The current highest-priority product-mainline Task is:

```text
#67 GENERIC-YTDLP-BILIBILI-REAL — Attempt 4
```

Its job is to obtain the first real brokered Bilibili extraction compatibility result on the accepted Ubuntu ARM64 target using the accepted offline runtime and ARM64 sandbox.

Expected decisive path:

```text
verified offline runtime
→ ARM64 sandbox
→ BrokerProcessRunner
→ R008Broker
→ broker_request_count > 0
→ real compatibility result
```

Do not interrupt #67 with speculative infrastructure work.

If #67 PASSes with a current first-playback-compatible muxed `http-file` or `hls` shape, the immediate next product Task is:

```text
#68 BILIBILI-WEB-E2E
```

#68 closes:

```text
Control Bilibili URL
→ SiteAdapterRegistry
→ generic-ytdlp
→ SourceSession / PlaybackSession
→ Gateway media capability
→ Web Display <video>
→ play / pause / seek / stop
→ refresh / reconnect
```

## 2. Stop infrastructure expansion unless Evidence requires it

#79 and #83 were valid because prior real #67 Attempts exposed concrete runtime blockers. That pattern must not become an automatic expansion strategy.

Current rule:

```text
No new generic-ytdlp runtime/distribution/sandbox/security Task
unless current #67/#68 Evidence identifies a concrete blocker.
```

Minimal routing:

```text
#67 PASS
→ #68

#67 FAIL because current media shape is unsupported
→ smallest generic media-format capability Task required by Evidence

#67 BLOCKED by a concrete R008/runtime/site condition
→ one bounded repair/research Task

no concrete blocker
→ no infrastructure Task
```

Do not proactively optimize Actions artifact distribution, sandbox architecture, packaging, production registration or runtime abstractions while first playback is still unproven.

## 3. Physical TV is a parallel product Evidence lane

Physical-TV behavior remains part of the intended TV-oriented product experience.

Current route:

```text
#6 R002-PREP [done]
→ #7 R002-TV [draft]
```

#7 may execute independently when the real prerequisites exist:

- reachable accepted deployment;
- physical target TV/browser;
- real phone/control trigger path;
- observable audible playback behavior.

Do not publish #7 only to fill capacity when those prerequisites are unavailable.

Important distinction:

```text
#68 PASS
= Bilibili Web playback/control closure

#7 PASS | acceptable CONDITIONAL PASS
= physical-TV remote audible playback behavior established
```

Hosted/headless/browser Evidence cannot replace #7.

## 4. Functional expansion is parked behind first playback

The following capabilities are prepared but must not displace the current first-playback path:

```text
Navigation
#71 [done] → #72 [draft; wait stable #68]

Auth
#28 [done] + #75 [done] + stable #68
→ future evidence-driven AUTH-REAL child

Native Site Panel
#33 [done] + #75 [done]
→ future real-site child only when product need selects it
```

#75 being accepted means the Browser runtime capability exists. It does **not** mean a Native Panel/Auth task should automatically be created next.

## 5. Environment readiness is already sufficient for current functional work

Accepted route:

```text
#64 ENV-ARM64-RUST [done]
→ #63 ENV-ARM64-READY [done]
```

The current target is sufficiently ready for #67 functional verification under the accepted low-privilege/security boundaries.

Do not reopen environment work unless a new concrete target failure invalidates that accepted Evidence.

Environment readiness does not claim CPU/RSS/temperature/throughput/soak capacity.

## 6. Performance / capacity remains later

#9 R003-TARGET remains the authoritative phone resource/performance verification Task:

```text
CPU / RSS / temperature
Direct / Remux / Chromium
5 / 30 / 60-minute sustained Evidence
```

It is intentionally deferred until primary functional paths are stable enough to make measurements representative.

Important separation:

```text
Gateway/Chromium functions
!=
phone is suitable for always-on production placement
```

#9 must not retroactively block #67/#68 functional closure.

## 7. Core feasibility synthesis is a later gate, not the current queue gate

#22 CORE-FEASIBILITY-REVIEW remains a later synthesis task requiring the relevant P0 Evidence, including R002 physical-TV and R003 target-performance conclusions.

Interpret it as:

```text
final/broader Core + deployment feasibility synthesis
```

not as:

```text
permission required before current functional delivery can continue
```

## 8. Current execution graph

```text
ACTIVE PRODUCT MAINLINE
#67 Attempt 4
  ↓
PASS → #68
FAIL/BLOCKED → one evidence-driven minimal repair only

PARALLEL WHEN REAL DEVICE AVAILABLE
#6(done) → #7 physical TV

PARKED UNTIL FIRST PLAYBACK STABLE
#71(done) → #72
#28(done) + #75(done) → future Auth-real
#33(done) + #75(done) → future Native Panel real-site

LATER
#9 performance/capacity
→ #22 broader Core/deployment feasibility synthesis
```

## 9. Coordinator scheduling rule

Use this decision order whenever a Worker finishes:

```text
1. Is there a review-ready current product-mainline Task?
   → review it first.

2. Did real Evidence expose a blocker to the next user-visible milestone?
   → create the smallest repair Task.

3. Is the next product Task already drafted and its dependencies satisfied?
   → Publication Gate and dispatch it.

4. Is a real physical-device Evidence lane available now?
   → run it in parallel if independent.

5. Otherwise
   → do not manufacture new work merely to keep Workers busy.
```

The project should optimize for **user-visible evidence throughput**, not number of active Issues or amount of infrastructure completed.
