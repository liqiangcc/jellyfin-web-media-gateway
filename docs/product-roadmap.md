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

Planning snapshot base: `cc03137175a14e61148f2c5b320e77313bba8346`.

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

Environment verification is not performance verification.

A functional path should not be blocked by 30/60-minute resource/thermal work unless the function genuinely cannot be implemented or safely verified without that evidence.

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

---

## 2. Milestone A — Ubuntu ARM64 functional environment

User outcome:

> The phone can safely build and run functional Gateway verification as the accepted low-privilege target user.

Route:

```text
#64 ENV-ARM64-RUST
→ #63 ENV-ARM64-READY Attempt 2
```

Acceptance gate:

- `gateway-runner` remains non-root/no-sudo/no-admin;
- user-owned Rust toolchain is available non-interactively;
- Gateway exact approved baseline builds;
- loopback start/routes/stop/cleanup pass;
- normal direct public network remains usable.

Explicitly not proved here:

- CPU/RSS/temperature/throughput/soak;
- production capacity.

---

## 3. Milestone B — First real Bilibili playback

User outcome:

> A public Bilibili URL can be entered from Control and the resulting media can play through Gateway on Web Display.

Current route:

```text
#66 GENERIC-YTDLP-EXTRACT-PREP          [accepted]
→ #73 GENERIC-YTDLP-REAL-HARNESS-PREP  [accepted]
→ #79 GENERIC-YTDLP-OFFLINE-RUNTIME-PREP
→ #67 GENERIC-YTDLP-BILIBILI-REAL
→ #68 BILIBILI-WEB-E2E
```

Why #79 is now a hard verification dependency:

- #67 Attempts 1 and 2 both proved direct public/Bilibili HTTP 200 but stopped before extractor traffic at `FROZEN_RUNTIME_SETUP` with `broker_request_count=0`;
- #73 R2 made target setup cacheable but cold preparation still depended on target-side `pip git+https` acquisition;
- #79 moves frozen dependency acquisition/build to GitHub Actions, produces an immutable manifest+SHA256 offline runtime bundle, and verifies offline consumption on hosted Linux x86_64 and ARM64;
- normal Target verification should consume a verified bundle and must not resolve/build the frozen runtime from source.

Architecture path:

```text
Bilibili public URL
→ SiteAdapterRegistry
→ generic-ytdlp Site Plugin
→ verified offline frozen runtime
→ R008-brokered extraction
→ current ResolvedMedia
→ Source/Session preparation
→ Gateway media capability
→ Web Display
→ Control play/pause/seek/stop
```

First-playback format policy is intentionally bounded:

- prefer one muxed audio+video HTTP/HLS format compatible with the current Web Display path;
- separate DASH audio/video composition is not required for this milestone;
- no login/Cookie/profile/access-control bypass;
- production `GenericYtdlpAdapter::default()` stays disabled until a later explicit enablement/production gate.

Acceptance gate:

1. #66 deterministic brokered real extraction implementation accepted. **Done.**
2. #73 safe real-site harness accepted. **Done.**
3. #79 provides a durable immutable offline runtime and x86_64/ARM64 offline-consume Evidence.
4. #64/#63 establish the selected low-privilege normal-network target environment. **Done.**
5. #67 Attempt 3 executes the accepted offline runtime + harness on the frozen public Bilibili sample and classifies real compatibility without bypass.
6. #68 proves the same accepted real-source shape reaches Gateway/Web Display and normal Control commands.

### Rolling buffer for this milestone

Current planning buffer:

```text
Active: #79
Blocked waiting on #79: #67
Draft materialized ahead: #68 Task Package
  - docs/tasks/68-bilibili-web-e2e/task.md
  - docs/tasks/68-bilibili-web-e2e/prompt.md
```

#68 may be designed while #79/#67 execute, but it must remain `status:draft` until #67 Final Acceptance PASS freezes the real media protocol/shape and exact Candidate.

---

## 4. Milestone C — Continuous content / navigation

User outcome:

> The user can naturally play previous/next content such as Bilibili multipart videos without Core understanding BVID/page/episode semantics.

This is a **generic capability first**, followed by site semantics.

Route:

```text
#71 SITE-NAVIGATION-PREP [accepted]
→ #72 BILIBILI-NAVIGATION [draft]
```

Generic capability #71 owns:

```text
SourceLocator
→ generic navigation result / equivalent contract
→ previous / next / bounded collection position
→ Playback NextItem / PreviousItem preparation
→ stale item/re-resolve protection under R007
```

Bilibili-specific implementation #72 owns:

- BVID/page/part interpretation;
- multipart ordering;
- mapping each part to opaque/versioned `SourceLocator`;
- real multipart Evidence.

Core must not learn Bilibili identifiers or navigation algorithms.

The old #23/PR #37 Navigation/ResolveContext implementation is historical Evidence only and is **not current API authority**.

#72 publication hard-depends on #71 Final Acceptance and a stable accepted public Bilibili playback baseline (#68 or later equivalent).

---

## 5. Milestone D — Source-site accounts / login

User outcome:

> Source-site accounts can be managed, login can be performed through an approved interaction path, and an authenticated play intent can safely resume.

Accepted foundation:

```text
#28 R005-AUTH-PREP
→ SiteAccount / SiteSessionRef
→ Session Vault
→ scoped SiteAccessCapability
→ AccountState / PendingIntent
→ controlled Secret injection
```

Current umbrella:

```text
#26 R005-AUTH
```

Future real-auth child publication gate should require:

- #28 accepted foundation;
- a stable public playback baseline (normally #68 or a later accepted equivalent);
- an approved interactive Auth Mode / Browser runtime path;
- selected site/login sample and evidence-safe handling rules.

It must **not** depend on the superseded #23 Task reaching Final Acceptance.

Bilibili may be the first real login site, but Core/Vault/Auth contracts remain site-generic.

---

## 6. Milestone E — Native Site Panel / original-site controls

User outcome:

> Control can expose site-native interactions such as quality selection, danmaku, collection/favorite or other site UI while playback authority stays in Gateway.

Accepted foundation:

```text
#33 R006-CONTRACT-PREP
→ BrowserWorker
→ BrowserCommand / BrowserEvent
→ ProfileAttachmentRef
→ NativePanelSession / short-lived control token
```

Current umbrella:

```text
#27 R006-DESIGN
```

Current functional child:

```text
#75 R006-RUNTIME-FUNCTIONAL-PREP
→ prove real Chromium lifecycle + BrowserEvent + Native Panel function
→ no performance/capacity claim
```

After #75 acceptance, Coordinator should choose the next child from actual Evidence rather than auto-publish both:

```text
R006-REAL-SITE / Native Panel functional child
and/or
future R005-AUTH-REAL child using approved Auth Mode
```

Later target performance work:

```text
#9 resource Evidence
→ choose phone/external host placement
→ always-on / on-demand / pool
→ concurrency / idle timeout / resource envelope
```

Site semantics remain in Site Plugin interpretation, never Browser Worker/Core.

Native Site Panel failure must not stop already-started playback.

---

## 7. Milestone F — Performance / capacity / production hardening

Only after the primary functional paths are stable:

```text
#9 R003-TARGET
→ CPU / RSS / temperature
→ Direct / Remux / Chromium boundaries
→ continuous 5/30/60-minute evidence
→ capacity decisions

then
→ production hardening
```

Performance evidence may constrain later runtime policy, but it should not retroactively be treated as a prerequisite for basic functional contract implementation.

---

## 8. Superseded legacy Bilibili route

The following early route is no longer the current delivery path:

```text
#23 R005-PUBLIC
→ #36 R005-PUBLIC-REAL
→ PR #37 bilibili-public candidate
```

Current durable disposition:

- #23: closed `not_planned`, explicitly **not** PASS;
- #36: closed `not_planned`;
- #65 integration attempt: closed `not_planned` after contract-invalidating conflict;
- PR #37: closed unmerged and retained as historical reference.

Historical Evidence must not be merged or reported as if it were current-main integration Evidence.

---

## 9. Current dependency graph

```text
Environment lane
#64 → #63 ────────────────────────────────┐
                                         │
First-playback lane                      │
#66(done) → #73(done) → #79 ─────────────┼→ #67 → #68
                                         │
                                         └ target readiness joins at #67

Navigation
#71(done) ─────────────────────────────────────→ #72
                                                   ↑
                                            wait stable #68

Browser / Native Panel
#33(done) → #75
              ↓
       future evidence-driven child
       ├→ R006 real-site/native-panel
       └→ may unlock future AUTH-REAL

Auth
#28(done) + #75 accepted + stable #68
→ future R005-AUTH-REAL child

Performance later
#9 R003-TARGET
```

Independent ready Tasks should still execute in parallel when no hard dependency exists.

---

## 10. Planning rules for future site work

Before creating a concrete-site implementation Task, classify the requested behavior:

```text
media extraction
→ generic or site SiteAdapter resolution

runtime dependency distribution
→ immutable offline artifact + cross-architecture verification

real-site verification
→ repository-owned safe harness + exact target Evidence

previous/next/playlist semantics
→ generic navigation capability + concrete plugin mapping

account/session/login
→ #26 / Vault / SiteAccess / Auth Mode

site-native UI operations
→ #27 / Browser Worker / Native Panel + concrete Site Plugin interpretation

playback mutation
→ existing Playback/R007 command authority

performance/resource decision
→ #9 / target-specific Evidence
```

Do not let a concrete plugin or a target-only smoke script silently redefine shared SiteAdapter, ResolvedMedia, Playback or security contracts. If a real site proves a generic contract is insufficient, open a generic capability/contract Task first, then implement the site mapping.
