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
#66 GENERIC-YTDLP-EXTRACT-PREP
→ #67 GENERIC-YTDLP-BILIBILI-REAL
→ #68 BILIBILI-WEB-E2E
```

Architecture path:

```text
Bilibili public URL
→ SiteAdapterRegistry
→ generic-ytdlp Site Plugin
→ brokered frozen yt-dlp extraction
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

1. #66 deterministic brokered real extraction implementation accepted.
2. #67 exact Candidate resolves the frozen public Bilibili sample on a permitted normal network.
3. #68 proves the same real-source path reaches Gateway/Web Display and normal Control commands.

---

## 4. Milestone C — Continuous content / navigation

User outcome:

> The user can naturally play previous/next content such as Bilibili multipart videos without Core understanding BVID/page/episode semantics.

This is a **generic capability first**, followed by site semantics.

Planned route:

```text
#71 SITE-NAVIGATION-PREP
→ #72 BILIBILI-NAVIGATION
```

Both are currently planning-only draft Tasks and must not displace the #66 → #67 → #68 first-playback critical path unless Coordinator explicitly reprioritizes.

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

The old #23/PR #37 Navigation/ResolveContext implementation is historical Evidence only and is **not current API authority**. New navigation work starts from accepted current SiteAdapter conformance (#39) plus canonical `implementation-contracts.md`.

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

Replanned runtime split:

```text
R006-RUNTIME-FUNCTIONAL
→ prove real Chromium lifecycle + BrowserEvent + Native Panel function
→ no performance/capacity claim

later R006-TARGET-PERF
→ consume #9 resource Evidence
→ choose always-on / on-demand / pool / external host / defer
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

Reason:

- #65 attempted integration and found semantic conflicts between the preserved #23 branch and the accepted current SiteAdapter API/conformance authority;
- #39 explicitly treated #23-only `NavigationContext`, `ResolveContext`, DASH/expiry and site-specific error additions as non-authoritative;
- the project now has an accepted secure generic-ytdlp runtime (#60), which is the shortest route to first real playback;
- navigation/auth/native-panel capabilities are being separated into generic capability layers rather than bundled into one legacy plugin branch.

Historical value retained:

- #23 Attempt 1 deterministic results;
- frozen sample selection;
- parser/navigation experiments;
- PR #37 / `eb03c199...` as reference material.

Historical Evidence must not be merged or reported as if it were current-main integration Evidence.

---

## 9. Current dependency graph

```text
Environment lane
#64 → #63

First-playback lane
#66 → #67 → #68

After first playback
        ├→ #71 SITE-NAVIGATION-PREP → #72 BILIBILI-NAVIGATION
#68 ────┼→ #26 future AUTH-REAL child
        └→ #27 R006-RUNTIME-FUNCTIONAL → site Native Panel work

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

Do not let a concrete plugin silently redefine shared SiteAdapter, ResolvedMedia, Playback or security contracts. If a real site proves a generic contract is insufficient, open a generic capability/contract Task first, then implement the site mapping.