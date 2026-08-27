# Product Delivery Roadmap

This document is the **current product-delivery route** for the media gateway project.

It maps user-visible capabilities to architecture capabilities, executable Tasks and acceptance gates. Live Task status/ownership still belongs to GitHub Issues; this document does not replace Issue lifecycle state.

Canonical architecture/security authority remains:

```text
requirements.md
→ architecture.md
→ implementation-contracts.md
→ security.md
```

`mvp-plan.md` remains the broader phase framework. When an older Task route in `mvp-plan.md` conflicts with this current delivery map, use this roadmap for **current Task sequencing/decomposition**, without weakening canonical architecture.

Steering snapshot base before this document update: `82aa6ef53451667c6130fb80849ed581c7f8c82f`.

---

## 1. Delivery principle

Current execution priority is:

```text
Environment readiness
→ Functional closure
→ Real-world compatibility
→ Performance / capacity
→ Production hardening
```

Environment verification is not performance verification. Hosted/browser functional Evidence is not physical-TV Evidence. A successful real-site extraction is not yet a user-visible playback closure.

### Rolling planning buffer

Coordinator should keep only a small amount of work ahead of active Workers:

```text
Active: roughly 1-2 Tasks
Ready: 0-2 executable Tasks
Draft: 1-3 next-layer Task Packages
```

Plan one layer ahead, at most two when dependencies are stable. Drafts may be materialized early, but unresolved real Evidence, artifact identity, Candidate SHA or semantic authority must **not** be guessed or frozen before the dependency returns.

The intended handoff rhythm is:

```text
Worker executes current Task
→ Coordinator prepares next-layer draft
→ dependency Evidence returns
→ Coordinator fills exact Evidence/Candidate/freshness fields
→ Publication Gate
→ immediate downstream dispatch
```

Do not create broad speculative Ready queues merely because execution is slow.

### Evidence-driven anti-drift rule

The first-playback lane has accumulated several runtime/security closure Tasks because real Target Evidence exposed concrete blockers. Those repairs remain valid, but they must not create an infrastructure-expansion habit.

From the current #67 Attempt 4 onward:

```text
no new generic-ytdlp/runtime/sandbox/distribution Task
unless current real #67/#68 Evidence exposes a concrete blocker
```

Result routing must be minimal:

```text
#67 PASS
→ immediately materialize/publish #68

#67 FAIL: unsupported current media shape
→ create the smallest generic media-format capability Task required by Evidence

#67 BLOCKED: concrete R008/runtime/site-access blocker
→ create one bounded repair/research Task for that blocker

otherwise
→ do not invent additional infrastructure work
```

---

## 2. Milestone A — Ubuntu ARM64 functional environment

User outcome:

> The phone can safely build and run functional Gateway verification as the accepted low-privilege target user.

Route:

```text
#64 ENV-ARM64-RUST            [accepted]
→ #63 ENV-ARM64-READY         [accepted]
```

Accepted outcome:

- `gateway-runner` is non-root/no-sudo/no-admin;
- user-owned Rust toolchain is usable non-interactively;
- Gateway can build/start/serve/stop on the accepted target;
- normal direct public network is available;
- target execution does not inherit production Secret/Vault authority.

Explicitly not proved here:

- CPU/RSS/temperature/throughput/soak;
- production capacity.

---

## 3. Milestone B — First real Bilibili Web playback

User outcome:

> A public Bilibili URL can be entered from Control and the resulting media can play through Gateway on Web Display.

Current accepted route:

```text
#66 GENERIC-YTDLP-EXTRACT-PREP          [accepted]
→ #73 GENERIC-YTDLP-REAL-HARNESS-PREP  [accepted]
→ #79 GENERIC-YTDLP-OFFLINE-RUNTIME    [accepted]
→ #83 GENERIC-YTDLP-SANDBOX-ARM64      [accepted]
→ #67 GENERIC-YTDLP-BILIBILI-REAL      [Attempt 4 active]
→ #68 BILIBILI-WEB-E2E                 [draft]
```

Why the extra runtime Tasks exist:

- #67 Attempts 1/2 proved direct public/Bilibili reachability but stopped at frozen-runtime preparation before broker traffic;
- #79 moved frozen runtime preparation to a repository-locked offline artifact rather than weakening Target or allowing ad-hoc source/package resolution;
- #67 Attempt 3 then proved offline verification/install on the real ARM64 target but exposed an x86_64-only seccomp architecture gate;
- #83 added target-bound AArch64 seccomp support while preserving `no_new_privs`, socket/socketpair denial and inherited broker IPC.

These Tasks are therefore evidence-driven security/runtime closures, not a new product layer.

### Current #67 Attempt 4

Frozen runtime Candidate:

```text
c23b49adbe1cad8a93ff4377dfeba3f12aac7ffe
```

Target path:

```text
exact #79 offline bundle
→ repository trust-anchor verification
→ low-privilege ARM64 cache hit/prepare
→ direct/no-proxy Bilibili reachability
→ accepted ARM64 ytdlp-sandbox
→ BrokerProcessRunner
→ R008Broker
→ yt_dlp.extract_info(download=False)
→ safe compatibility result
```

Decisive signal:

```text
runtime_cache: offline-hit | offline-prepared
process_error != SANDBOX_UNAVAILABLE
broker_request_count > 0
```

Only after broker traffic occurs may #67 classify real Bilibili compatibility.

### #68 publication gate

#68 remains `status:draft` until #67 is Final Accepted with a compatible current first-playback media shape.

If #67 PASSes with muxed `http-file` or `hls`, #68 should become the immediate product-mainline Task:

```text
Control enters frozen public Bilibili URL
→ SiteAdapterRegistry
→ generic-ytdlp
→ accepted real ResolvedMedia shape
→ SourceSession / PlaybackSession
→ Gateway same-origin media capability
→ Web Display <video>
→ play / pause / seek / stop
→ refresh / reconnect
```

#68 must not become a second extraction experiment and must not add navigation, login, Native Site Panel, performance or production enablement.

---

## 4. Cross-cutting product gate — Physical TV remote playback

The product target is not merely a hosted browser. The original Web-only Core experience includes a real TV/browser that can remain on `/display` and later receive playback from a phone/control browser with acceptably low interaction.

Route:

```text
#6 R002-PREP             [accepted]
→ #7 R002-TV             [draft; physical-TV Evidence]
```

#7 is independent of Bilibili extraction and may run in parallel whenever a concrete reachable deployment, physical target TV/browser and phone/remote trigger path are available.

Required interpretation:

```text
#68 PASS
= real Bilibili Web product path proven
!= physical-TV audible autoplay/remote UX proven

#7 PASS | acceptable CONDITIONAL PASS
= physical-TV remote playback behavior proven
```

A first genuinely usable TV-oriented product milestone should consider both real-source Web playback Evidence and physical-TV behavior. Hosted Chromium, synthetic activation or desktop browser Evidence cannot substitute for #7.

Do not publish #7 merely to fill the queue if the physical TV/deployment is unavailable; keep it draft until its real-device prerequisites are actually satisfied.

---

## 5. Milestone C — Continuous content / navigation

User outcome:

> The user can naturally play previous/next content such as Bilibili multipart videos without Core understanding BVID/page/episode semantics.

Route:

```text
#71 SITE-NAVIGATION-PREP [accepted]
→ #72 BILIBILI-NAVIGATION [draft]
```

#72 hard-depends on a stable accepted public Bilibili playback baseline (#68 or later equivalent). It must not displace #67/#68 or physical-TV validation.

Bilibili-specific BVID/page/part interpretation remains in plugin code; Core consumes only generic opaque/versioned `SourceLocator` navigation results.

---

## 6. Milestone D — Source-site accounts / login

Accepted foundation:

```text
#28 R005-AUTH-PREP [accepted]
```

Umbrella:

```text
#26 R005-AUTH [draft umbrella]
```

A future real-auth child requires:

- #28 accepted foundation;
- stable public playback baseline, normally #68;
- approved interactive Browser/Auth Mode;
- a legal frozen real-site login scenario and evidence-safe handling rules;
- no Cookie/profile smuggling or access-control bypass.

Do not auto-publish Auth work merely because Browser runtime now exists.

---

## 7. Milestone E — Native Site Panel / original-site controls

Accepted foundations:

```text
#33 R006-CONTRACT-PREP             [accepted]
#75 R006-RUNTIME-FUNCTIONAL-PREP  [accepted]
```

#75 proves a real bounded Chromium BrowserWorker runtime with R008 navigation boundaries, normal Chromium sandboxing, caller-env/proxy isolation, fixed trusted executable discovery, NativePanel token/input seam and lifecycle cleanup.

This capability is now **parked until product Evidence requires it**. Acceptance of #75 is not a mandate to immediately create a real-site Native Panel child.

Future real-site Browser/Auth work should be selected only after first public playback is stable and from concrete user/product need.

---

## 8. Performance / capacity / production hardening

#9 R003-TARGET remains the authoritative phone resource/performance verification Task:

```text
CPU / RSS / temperature
Direct / Remux / Chromium
continuous 5 / 30 / 60-minute Evidence
capacity / placement decisions
```

It is intentionally later than the current functional critical path. Functional work may proceed without #9 when it makes no phone-capacity claim.

Important separation:

```text
Gateway/Chromium can function
!=
phone is suitable for always-on production placement
```

#22 CORE-FEASIBILITY-REVIEW remains a later synthesis/final feasibility gate requiring the relevant P0 Evidence, including physical-TV and target-performance results. It must not be interpreted as a publication blocker for current #67/#68 functional delivery.

---

## 9. Superseded legacy Bilibili route

The following route is historical only:

```text
#23 R005-PUBLIC
→ #36 R005-PUBLIC-REAL
→ PR #37
```

Durable disposition:

- #23 closed `not_planned`, explicitly not PASS;
- #36 closed `not_planned`;
- #65 closed `not_planned` after semantic conflict;
- PR #37 closed unmerged.

Do not revive their old API authority.

---

## 10. Current dependency graph

```text
Environment
#64(done) → #63(done) ───────────────────────────────┐
                                                    │
First real Bilibili Web playback                    │
#66(done) → #73(done) → #79(done) → #83(done) ─────┼→ #67(active) → #68(draft)
                                                    │
                                                    └ accepted ARM64 target joins at #67

Physical TV product Evidence
#6(done) → #7(draft; run when real TV/deployment available)

Navigation
#71(done) → #72(draft; wait stable #68)

Browser / Native Panel
#33(done) → #75(done) → parked until product Evidence selects next child

Auth
#28(done) + #75(done) + stable #68
→ future evidence-driven AUTH-REAL child

Performance / final feasibility
#9 later
→ #22 final Core/deployment feasibility synthesis
```

---

## 11. Product-completion naming

Use precise milestone language:

```text
#67 PASS
= real Bilibili extraction compatibility

#68 PASS
= first real Bilibili Web playback/control closure

#7 PASS or accepted CONDITIONAL PASS
= physical-TV remote audible playback behavior established

#68 + #7 accepted Evidence
= first TV-oriented user journey can be evaluated as a product milestone

#9 + #22 later
= target capacity / broader Core feasibility and deployment decision
```

Do not call #68 alone “full TV MVP validated”, and do not call #67 alone “playback completed”.

---

## 12. Planning rules for future site work

Before creating a concrete-site implementation Task, classify the requested behavior:

```text
media extraction
→ SiteAdapter resolution

runtime/distribution/sandbox repair
→ only when current real Evidence proves a blocker

real-site verification
→ repository-owned safe harness + exact target Evidence

previous/next/playlist semantics
→ generic navigation capability + concrete plugin mapping

account/session/login
→ Vault / SiteAccess / approved Auth Mode

site-native UI operations
→ Browser Worker / Native Panel + concrete Site Plugin interpretation

playback mutation
→ existing Playback/R007 command authority

physical TV behavior
→ manual/real-device Evidence, never hosted substitution

performance/resource decision
→ #9 target-specific Evidence
```

Do not let a concrete plugin or target-only smoke script silently redefine shared SiteAdapter, ResolvedMedia, Playback or security contracts. If real Evidence proves a generic contract is insufficient, open only the smallest generic capability/contract Task required by that Evidence.
