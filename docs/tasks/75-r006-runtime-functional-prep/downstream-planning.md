# Downstream Planning — after #75 R006-RUNTIME-FUNCTIONAL-PREP

Status: planning-only. This file does not authorize a Worker and does not replace a future child Task Contract.

## Principle

#75 proves only the generic real Chromium BrowserWorker functional lifecycle. Its Final Acceptance should not automatically publish every Browser/Auth feature.

After #75 returns, Coordinator first classifies what was actually proven and then chooses the smallest valuable next child.

## Required #75 acceptance facts to read back

```text
accepted Candidate / merged main SHA
actual allowed browser executable(s)
headless/hosted runtime viability
BrowserCommand coverage
BrowserEvent ordering/coverage
NativePanelSession/token seam result
input support result
R008/navigation authority result
crash/timeout/cancel/close cleanup result
profile/temp-dir ownership/cleanup result
Secret/log/network leak result
known browser/runtime limitations
```

## Candidate downstream A — R006-REAL-SITE-NATIVE-PANEL

Choose this when #75 proves a stable generic runtime and the next product priority is original-site controls/UI rather than authentication.

Goal shape:

```text
accepted BrowserWorker runtime
→ one frozen public/non-auth site interaction scenario
→ Site Plugin interprets BrowserEvent
→ NativePanelSession/control token
→ bounded site-native UI interaction
→ Gateway Playback remains independent
```

Potential first Bilibili UX after public playback baseline:

- quality/site-native controls;
- danmaku toggle;
- collection/favorite UI where legally/publicly usable;
- navigation UI only if it maps back through accepted SourceLocator/R007 authority.

Publication prerequisites:

1. #75 Final Accepted.
2. Stable public playback baseline, preferably #68 accepted, when the panel must coexist with active playback.
3. One frozen legal real-site scenario.
4. Evidence-safe DOM/frame/input contract.
5. No login/Cookie/profile requirement unless routed through Auth authority.

Frozen boundaries:

- Browser Worker stays site-generic;
- concrete DOM/API semantics stay in Site Plugin interpretation;
- no raw page/frame/input/Secret normal logging;
- Native Panel failure cannot stop already-started playback;
- no media capture/DRM bypass;
- no target performance placement claim.

## Candidate downstream B — R005-AUTH-REAL

Choose this when #75 proves an approved interactive Auth Mode/runtime and product priority requires authenticated source access.

Goal shape:

```text
PendingIntent / auth required
→ approved Browser Auth Mode
→ user performs login
→ Site Plugin interprets bounded login/session outcome
→ Vault installs/replaces SiteSession
→ scoped SiteAccessCapability
→ retry original SourceLocator/play intent
```

Publication prerequisites:

1. #28 Auth/Vault foundation remains accepted.
2. #75 Final Accepted with an interaction seam adequate for Auth Mode.
3. Stable public playback baseline, normally #68 accepted.
4. Coordinator freezes one legal real-site login scenario.
5. Evidence contract proves passwords/codes/QR/Cookie/profile material are not normal logs/artifacts.

Frozen boundaries:

- Vault remains unique Secret/profile owner;
- Browser Worker receives only scoped/temporary profile attachment;
- Site Plugin cannot read Vault files directly;
- no manual Cookie/profile smuggling;
- no CAPTCHA/access-control bypass automation;
- R008 remains network authority;
- Playback remains R007 authority.

## Selection rule

Do not publish both children just because they are possible.

Prefer:

```text
if #68 is not yet stable:
    finish first-playback lane first unless Browser work independently blocks it

elif user value is site-native controls:
    materialize R006-REAL-SITE-NATIVE-PANEL

elif authenticated sources are the next priority:
    materialize R005-AUTH-REAL

else:
    keep both as planning-only
```

## Performance relationship

#75 functional acceptance does not prove phone placement or always-on suitability.

Those decisions remain downstream of #9 resource Evidence:

```text
#75 functional viability
+
#9 CPU/RSS/thermal/soak
→ always-on | on-demand | bounded pool | external host | defer
```

Do not let performance work retroactively block basic generic Browser contract acceptance, and do not let hosted functional Evidence become a production placement claim.
