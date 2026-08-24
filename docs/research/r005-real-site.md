# R005 Real Site Resolution PoC — Public Phase

Date: 2026-08-24

Issue: #23 / `R005-PUBLIC`

Attempt: 1

Result: **BLOCKED** for the frozen real-site smoke; deterministic plugin evidence is complete.

## Scope and target

The frozen target is the public, no-login, non-DRM Bilibili page:

```text
https://www.bilibili.com/video/BV14V411W7r5/
```

No Cookie, Authorization, account state, signed URL, media payload, CAPTCHA, regional restriction or access-control workaround was used.

## Implemented contract path

```text
public page URL
→ SiteAdapterRegistry.recognize
→ plugins/bilibili BilibiliAdapter
→ SourceLocator(v1, plugin-owned hex-opaque payload)
→ central caller supplies a public document through ResolveContext
→ ResolvedMedia
→ existing R001 resource_from_resolved / Media Gateway boundary
→ generic NavigationContext and R007 media-refresh semantics
```

The Core-facing API only adds site-neutral context, navigation and error types. Bilibili URL, BVID, page and page-list interpretation remains in `plugins/bilibili`.

## Deterministic evidence

- C1 PASS: the frozen URL routes through `SiteAdapterRegistry` to `bilibili-public`; non-Bilibili URLs are not claimed.
- C2 PASS: locator version 1 round-trips inside the plugin, uses an opaque hex representation, excludes the BVID from the raw payload, rejects unsupported versions and rejects Cookie/Authorization/bearer/password/secret sentinels.
- C3 PASS for the deterministic public document: title, duration, HTTP media protocol, expiry metadata, clear protection and non-secret `Referer`/`User-Agent` public headers map to the generic `ResolvedMedia` shape.
- C4 PASS for the deterministic four-page document and frozen sample navigation: previous/next are new opaque locators and Core sees only `NavigationContext`.
- C5 PASS at the contract level: the stable locator is retained independently of the short-lived media URL/expiry, while the accepted R007 suite proves stale media-refresh generations cannot overwrite newer media.
- C6 PASS for the implemented boundary checks: the plugin emits no Secret headers or access reference; existing R001 `resource_from_resolved` rejects Secret public headers; no private-network exception or plugin-local egress bypass was added.
- C7 PASS: invalid/unsupported locator, denied upstream, not-found, schema/parse and unsupported-media outcomes are explicit stable `AdapterError` variants without echoing full URLs or query material.

## Real-site smoke

Bounded diagnostic, UTC `2026-08-24T17:34:26Z`, execution host `/root/jellyfin-web-media-gateway-issue-23`:

```text
curl -L --max-time 20 -sS -A 'Mozilla/5.0' \
  -o /dev/null -w '%{http_code}' \
  https://www.bilibili.com/video/BV14V411W7r5/
→ HTTP 412
```

The reproducible smoke entry point is `scripts/r005-real-site-smoke.sh`; it exits with code `2` for this blocked condition and only invokes the plugin parser after an HTTP 200 response.

The frozen public page could not be retrieved from this environment. This is treated as a real-site `BLOCKED` condition, not as a successful resolution, and no challenge/access-control bypass was attempted. Therefore C3/C4 real-site evidence, real expiry refresh, and real four-part navigation remain unverified even though their plugin fixtures and parser contract are deterministic.

## Verification authority

Exact final Candidate SHA, J1/J2 commands and R001/R007 regression job selectors are recorded in the Issue #23 Attempt 1 feedback. The implementation candidate consists of the new `plugins/bilibili` crate, the site-neutral API additions, the contract documentation and this evidence record.

## Architecture impact

**Continue** the existing SiteAdapter/SourceLocator/ResolvedMedia/navigation contracts for the public phase. **Defer** R005-PUBLIC classification to PASS/CONDITIONAL PASS until the frozen Bilibili page is accessible in a permitted verification environment. No Core site special case or security contract change is justified by this blocked smoke.
