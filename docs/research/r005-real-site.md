# R005 Real Site Resolution PoC — Public Phase

Updated: 2026-08-25

Issue: #23 / `R005-PUBLIC`

Attempt: existing Attempt 1, resumed in the original worktree

Result: **BLOCKED** for the frozen real-site smoke; deterministic plugin and
integrated security/concurrency evidence is complete.

## Scope and target

The frozen target is the public, no-login, non-DRM Bilibili page:

```text
https://www.bilibili.com/video/BV14V411W7r5/
```

No Cookie, Authorization, account state, signed URL, media payload, CAPTCHA,
regional restriction or access-control workaround was used.

## Implemented contract path

```text
public page URL
→ SiteAdapterRegistry.recognize
→ plugins/bilibili BilibiliAdapter
→ SourceLocator(v1, plugin-owned opaque payload)
→ central public-web fetch supplies a public document
→ ResolvedMedia
→ existing R001 media boundary
→ generic NavigationContext and accepted R007 refresh semantics
```

Concrete Bilibili URL, BVID, page and page-list interpretation remains in
`plugins/bilibili`; Core only routes through the generic adapter API.

## Deterministic evidence

- C1 **PASS**: the frozen URL routes through `SiteAdapterRegistry` to
  `bilibili-public`; non-Bilibili URLs are not claimed.
- C2 **PASS**: locator version 1 round-trips inside the plugin, remains opaque
  to Core, rejects unsupported versions and rejects Cookie/Authorization/
  bearer/password/secret sentinels.
- C3 **PASS for fixture evidence**: title, duration, HTTP-file/DASH protocol,
  expiry metadata, clear/DRM protection and non-secret public headers map to
  `ResolvedMedia`.
- C4 **PASS for fixture evidence**: the four-page document produces previous
  and next opaque locators; Core does not interpret BVID/page values.
- C5 **PASS**: the stable locator is retained independently of short-lived
  media expiry, and the integrated accepted R007 suite proves stale media
  refresh results cannot overwrite newer generations.
- C6 **PASS**: the real-site smoke uses central `public_web` EgressPolicy and
  the R008 validated-address-pinned client. The plugin emits no Secret header
  or access reference, and no private-network exception was added.
- C7 **PASS**: invalid/unsupported locator, denied upstream, not-found,
  schema/parse and unsupported-media outcomes are explicit and do not echo
  full URLs or query material.

## Verification commands

The final exact Candidate SHA and execution metadata are recorded in the
Issue #23 `[EXECUTION REPORT]`. On that candidate, the worker ran:

```text
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p gateway-core --test security_baseline -- --nocapture
cargo test -p gateway-core --test http_security -- --nocapture
cargo test -p gateway-core --test media_gateway -- --nocapture
cargo test -p gateway-core --test playback_concurrency -- --nocapture
git diff --check
```

The workspace suite included 8 Bilibili tests, 18 accepted R007 playback
concurrency tests, the R001 media regression tests, and the accepted R008
security baseline. R008 merge `e8981ada4e9a51b2856cd9206d2e7546bada5eca` and
current `main` were integrated before final verification.

## Real-site smoke

Bounded diagnostic, UTC `2026-08-24T23:48:52Z`, from the original cloud
worktree:

```text
scripts/r005-real-site-smoke.sh
→ HTTP 412
→ exit 2
→ result=BLOCKED (public page returned HTTP 412; no challenge bypass attempted)
```

The smoke entry point validates every request through central `public_web`,
revalidates redirects, uses the validated address set for the connection, and
limits the document to 8 MiB. It only invokes the plugin parser after HTTP
200. The frozen public page was not legally/reliably retrievable from this
execution environment, so C3/C4 real-site resolution, real expiry refresh and
real four-part navigation remain unverified. Fixture evidence is not used to
rewrite this result as a real-site PASS.

## Architecture impact

**Continue** the existing SiteAdapter/SourceLocator/ResolvedMedia/navigation
contracts; no Core site special case or security contract change is justified.
**Defer** R005-PUBLIC PASS/CONDITIONAL PASS classification until the frozen
sample can be exercised in a permitted environment. R005-AUTH remains out of
scope.
