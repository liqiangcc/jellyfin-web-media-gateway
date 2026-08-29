# Task Contract — #118 ENV-ARM64-ANDROID-DIAG-BROWSER-PROVISION

## Identity

- Issue: #118
- Task kind: environment provisioning / diagnostic prerequisite
- Parent blocked Task: #116 ENV-ARM64-ANDROID-BROWSER-CDP-BRIDGE-PREP
- Downstream: #116 may resume C1/C2 only after #118 Final Acceptance; #117 remains draft.
- Preferred Worker: ubuntu-arm64
- Eligible environment: env:ubuntu-arm64
- Planning base: current main before this task package

## Trigger evidence

#116 Attempt 1 C0 established:

- accepted phone exposes Android 13 / API 33 / arm64-v8a command layer;
- `com.android.chrome` is present but is the normal browser and MUST NOT be used as the diagnostic browser;
- no approved separate debug-capable browser was available to the Worker;
- no deterministic local-only CDP transport was available from the Worker environment;
- no normal browser/profile/tab was started, attached, enumerated or inspected.

This is an environment/tooling prerequisite blocker only.

## Goal

Provision exactly two prerequisites for #116:

1. a distinct, empty/logged-out Android diagnostic browser package/profile; and
2. a deterministic on-demand local-only CDP transport from the accepted phone into the Ubuntu ARM64 diagnostic environment.

This Task does NOT implement the browser-diag MCP and does NOT navigate Bilibili.

## P0 — bounded discovery

Before any installation or mutation, inspect only bounded package/tool identities needed to answer:

- Is an approved separate browser already installed?
- Is a safe local CDP transport already possible?

Exact browser candidates may include official packages with separate identities such as:

- `com.chrome.beta`
- `com.chrome.dev`
- `com.chrome.canary`
- `org.chromium.chrome`
- `org.chromium.webview_shell`

Other candidates require explicit evidence that they are a distinct debug-capable browser and do not share the normal Chrome profile.

Do not enumerate arbitrary user applications. Do not inspect browser data directories/profile contents.

## P1 — browser provisioning

Preferred order:

1. reuse an already-installed approved distinct browser;
2. use a trusted device package source / official store path;
3. use an operator-provided official package artifact.

Prohibited:

- arbitrary web APK download;
- unofficial mirrors;
- copying or cloning `com.android.chrome` profile/data;
- logging into the diagnostic browser;
- importing Cookies, storage, history, cache, passwords or account state.

If installation requires a device UI confirmation or operator action, report the exact bounded prerequisite and STOP rather than bypassing it.

## P2 — local-only CDP transport

Establish the narrowest reversible transport supported by the phone. Acceptable classes include:

- standard on-demand ADB forwarding to the diagnostic browser's DevTools abstract socket;
- a root/Termux local abstract-socket bridge that binds loopback/process-local scope only;
- an equivalent local-only mechanism with the same exposure properties.

Requirements:

- endpoint must be loopback/process-local only;
- no `0.0.0.0`, LAN, public or Tailscale DevTools listener;
- no permanent TCP adbd enabled solely for this Task;
- no forwarding to the normal `com.android.chrome` DevTools socket;
- deterministic start/stop and cleanup;
- transport must be usable from the Ubuntu ARM64 diagnostic Worker after provisioning.

## P3 — neutral proof

Use only a neutral HTTPS page or browser version/target endpoint sufficient to prove:

- diagnostic browser identity is distinct;
- browser is empty/logged-out for this Task;
- CDP transport answers a bounded version/target health query;
- target enumeration cannot escape the dedicated diagnostic browser instance.

Do not navigate Bilibili.

Allowed evidence fields only:

- package name / browser product + version;
- Android/API/ABI class;
- transport type;
- local endpoint class (`loopback`, `abstract-socket`, etc.);
- bounded target count and neutral target title class if needed;
- start/stop result;
- cleanup result.

Never publish response/page bodies, Cookies, Auth, storage, arbitrary headers, raw CDP events, signed/query URLs, tokens or media URLs.

## P4 — teardown

After proof:

- stop forwarding/bridge created by this Task;
- stop diagnostic debug session if required;
- verify no public/LAN/Tailscale/0.0.0.0 DevTools listener remains;
- do not uninstall an operator-approved diagnostic browser unless the contract explicitly requires it;
- leave normal Chrome untouched.

## Success criteria

PASS requires all of:

- approved package identity distinct from `com.android.chrome`;
- no login/profile-state import;
- deterministic local-only CDP start/stop;
- bounded neutral CDP health/target proof;
- Ubuntu ARM64 side can reach the local transport;
- teardown proves no widened DevTools exposure;
- safe-output boundary PASS.

BLOCK if any prerequisite would require normal-browser use, untrusted installation, public debug exposure, permanent TCP adbd solely for this Task, or bypassing an operator/device confirmation.

## Hard exclusions

- no Bilibili URL/navigation;
- no #67 J3, #113 or #117 execution;
- no #68;
- no yt-dlp/generic-ytdlp/R008/broker/sandbox;
- no browser-diag MCP implementation;
- no proxy/rotation/header spoofing/fingerprint emulation/CAPTCHA/access-control bypass.

## Worker lifecycle

`claim -> P0 discovery -> provision if allowed -> P2 local transport -> P3 neutral proof -> P4 teardown -> [EXECUTION REPORT]/[BLOCKER REPORT] -> status:review|status:blocked -> release owner -> STOP`

If repository files must change solely to make provisioning reproducible, create a narrow Candidate/PR and report it for Coordinator review; otherwise this Task may remain verification/environment-only.