# Task Contract — #122 ENV-ARM64-CHROME-BETA-FIRST-RUN-READY

## 1. Identity

- Issue: `#122`
- Task ID: `ENV-ARM64-CHROME-BETA-FIRST-RUN-READY`
- Kind: environment provisioning / reusable diagnostic-state repair
- Preferred Worker: `ubuntu-arm64`
- Eligible environment: `env:ubuntu-arm64`
- Parent blocked Task: `#117 ENV-ARM64-BROWSER-WORKER-DIFFERENTIAL-DIAG`
- Accepted upstream authorities: `#116 Final Acceptance`, `#119 Final Acceptance`
- Planning Base: current accepted main containing #116 integration `6bc87803919479704e8ae5fa0209799cdd730cc8`
- Publication state: draft until Coordinator Publication Gate.

## 2. Trigger evidence

#117 Attempt 1 correctly stopped before any Bilibili or Worker measurement because the accepted diagnostic-browser prerequisite was not reusable after #116 cleanup:

- official `com.chrome.beta` remains installed;
- stable `com.android.chrome` remained absent/untouched;
- #116 post-proof cleanup had executed `pm clear com.chrome.beta`;
- the verified launcher then resumed `org.chromium.chrome.browser.firstrun.FirstRunActivity`;
- `@chrome_devtools_remote` did not appear in the bounded startup window;
- no relay, MCP target, Bilibili navigation or paired Worker probe could safely begin.

This is an environment-state regression, not a #117 browser-vs-worker result.

## 3. Goal

Establish a reusable dedicated Chrome Beta diagnostic state that is:

```text
official com.chrome.beta
→ first-run initialized
→ logged out / no account
→ sync disabled / no account import
→ no normal-Chrome state import
→ verified launcher reaches normal tabbed browser
→ exact Beta-owned DevTools endpoint available on demand
→ accepted local-only AF_UNIX transport + bounded MCP health PASS
→ force-stop/temporary-relay cleanup
→ initialized state survives next bounded restart
```

The Task does not navigate Bilibili and does not execute #117 measurements.

## 4. Core principle

The durable state needed downstream is **first-run acknowledgement without personal state**, not a byte-empty Chrome profile.

Therefore:

- successful teardown MUST NOT run `pm clear com.chrome.beta`;
- it may preserve only the dedicated app's normal first-run preference/state required to avoid `FirstRunActivity`;
- account, sync, imported profile/session/cache/history and site-specific state remain forbidden.

## 5. P0 — live gate and bounded preflight

Before UI/device mutation:

- live-read #122, this Task Contract, #117 Attempt 1 blocker review, #116 Final Acceptance and #119 Final Acceptance;
- confirm #122 is `OPEN + status:ready + env:ubuntu-arm64 + no owner` before claim;
- after claim write `[EXECUTION CHECKPOINT]`;
- confirm `com.chrome.beta` package exists;
- confirm stable `com.android.chrome` is not running;
- confirm no accepted relay socket or TCP 9222 listener is present;
- confirm Beta currently enters first-run and exact Beta DevTools endpoint is absent.

Do not enumerate normal-browser tabs/profile/state.

## 6. P1 — dedicated Beta first-run initialization

Interact only with `com.chrome.beta` first-run UI.

Allowed:

- inspect bounded UI hierarchy/text for the dedicated Beta first-run screen solely to identify required onboarding choices;
- accept mandatory product terms if required to reach the browser;
- select the least-state logged-out path;
- choose `No thanks`, `Use without an account`, `Not now`, or equivalent choices to reject sign-in/account/sync/import/personalization when available;
- decline default-browser changes when optional;
- use Android UI automation/input only within the dedicated Beta first-run activity when the action and target are unambiguous.

Forbidden:

- adding/signing into a Google/account identity;
- enabling account sync;
- importing normal Chrome/profile/session/history/cache/password data;
- making the user's normal Chrome the diagnostic target;
- inspecting app-private databases/files to bypass onboarding;
- accepting a choice whose effect is unclear and could import/share personal state;
- changing global device network, account or security configuration.

If first-run cannot be completed without account/sync/import or ambiguous unsafe state, BLOCK.

## 7. P2 — initialized-state isolation proof

After P1:

- force-stop only `com.chrome.beta`;
- start the verified launcher `com.chrome.beta/com.google.android.apps.chrome.Main` with a hard timeout;
- verify the resumed activity is no longer `FirstRunActivity`;
- verify stable `com.android.chrome` remains absent;
- do not navigate to a public/site URL for this proof;
- use `about:blank`/initial blank target only as needed by the browser itself.

Do not publish full UI hierarchy, titles, URLs or profile paths.

## 8. P3 — exact endpoint and transport proof

Use the accepted #119 method; do not redesign transport.

Two bounded lifecycle cycles are required:

1. stable Chrome absent + Beta force-stopped + exact candidate absent;
2. start only Beta verified launcher;
3. identify `@chrome_devtools_remote` as the exact Beta endpoint using bounded lifecycle/package evidence;
4. create the Termux-host Python AF_UNIX relay to a temporary filesystem Unix socket visible to Ubuntu;
5. from Ubuntu run only bounded `/json/version` + `/json/list` or #116 MCP `health`/`list_targets` field checks;
6. publish no raw JSON, target URL/title, body, header or browser state;
7. stop relay and force-stop Beta;
8. verify endpoint/relay/TCP 9222 absent and stable Chrome unchanged.

Codex/GPT control-plane may use `127.0.0.1:7890`; Browser/CDP Task traffic must explicitly bypass/clear HTTP proxy variables.

## 9. P4 — reusable restart proof

After the first successful teardown, WITHOUT `pm clear`:

- start Beta a second time;
- prove `FirstRunActivity` does not return;
- prove exact Beta endpoint becomes available again;
- run one bounded MCP health/list-target proof through the local AF_UNIX path;
- final force-stop/relay cleanup;
- leave Beta package installed and first-run initialized, logged out/non-sync.

This P4 restart proof is mandatory because it is the capability needed by #117.

## 10. Safe-output boundary

Durable Issue Evidence may include only:

- package/version/product family;
- first-run before/after boolean/classification;
- logged-out/no-sync/no-import confirmation based on the choices performed;
- endpoint/relay/MCP health PASS/FAIL;
- target count only;
- teardown booleans;
- bounded sanitized capability errors.

Never publish:

- UI hierarchy dump in full;
- account identifiers;
- cookies/auth/storage/password/profile contents;
- response/page bodies or arbitrary headers;
- target URLs/titles or raw DevTools JSON/events;
- signed/query URLs, tokens or media payloads.

## 11. Hard boundaries

- Dedicated `com.chrome.beta` only.
- Never start/attach/enumerate/read/copy normal `com.android.chrome` profile/tabs/state for diagnosis.
- No Bilibili or other public-site navigation.
- No proxy/rotation, UA/Referer/header spoofing, fingerprint emulation, CAPTCHA/challenge/access-control bypass.
- No public/LAN/Tailscale/`0.0.0.0` DevTools listener.
- No permanent TCP adbd solely for this Task.
- No #117 paired measurements, no #113 Worker probes, no yt-dlp/generic-ytdlp/R008/broker/sandbox/#67/#68 execution.
- No product/runtime/security code changes are expected.

## 12. Success criteria

PASS requires all of:

1. first-run no longer intercepts the verified Beta launcher after initialization;
2. performed choices establish logged-out/no-account/no-sync/no-import state;
3. stable `com.android.chrome` remains absent/untouched;
4. exact Beta-owned endpoint attribution PASS in two bounded lifecycle cycles;
5. accepted Termux-host AF_UNIX transport PASS from Ubuntu;
6. bounded `/json/version` + `/json/list` or #116 MCP health/list-target PASS;
7. teardown leaves Beta process/endpoint/temp relay/TCP 9222 absent while preserving initialized state;
8. P4 post-teardown restart proves initialized state survives without returning to `FirstRunActivity`;
9. final cleanup PASS and no prohibited data retained/published;
10. evidence is sufficient for Coordinator to resume #117 as fresh Attempt 2 under its unchanged contract.

BLOCK if the only way forward requires normal-browser state, account login/sync/import, security bypass, ambiguous onboarding choices, untrusted package installation or public debug exposure.

## 13. Worker lifecycle

Normal:

```text
live gate
→ claim
→ [EXECUTION CHECKPOINT]
→ P0 preflight
→ P1 first-run initialization
→ P2 isolation proof
→ P3 two-cycle endpoint/transport proof
→ P4 reusable restart proof
→ [EXECUTION REPORT]
→ status:review
→ release owner
→ STOP
```

Blocked:

```text
claim
→ bounded safe blocker evidence
→ cleanup
→ [BLOCKER REPORT]
→ status:blocked
→ release owner
→ STOP
```

Worker must not merge/close/mark done, execute #117/#67/#113/#68, or create another Task.
