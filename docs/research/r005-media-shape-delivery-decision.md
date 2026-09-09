# R005 Media Shape and Delivery Decision

Date: 2026-09-09
Issue: #265
Parent Goal / Research Item: #68 Bilibili Web E2E
Candidate: recorded in the Issue execution report

This is a synthetic and source-level research result. It does not perform a Bilibili request, login, target navigation, phone/TV/VNC/CDP action, or local build/test. It does not claim that Bilibili playback works.

## Decision

**CONDITIONAL PASS — select server-side stream-copy remux to browser-compatible fMP4/HTTP-file for separated A/V, while retaining direct muxed HTTP-file and the existing Gateway HLS path.**

The selected route is a delivery decision and an implementation handoff. It is not an implementation result and it is not live-site evidence. The condition is a hosted synthetic browser/media proof after the generic contract is implemented, plus a later authorized Site Plugin observation proof under #246. Until those exist, #68 remains open.

The first implementation must follow this path:

    Site Plugin observation
    -> generic versioned media shape
    -> server-owned EgressPolicy-bound inputs
    -> bounded stream-copy remux process
    -> one Gateway capability for the remuxed fMP4 output
    -> Web Display video using the same-origin capability path

Muxed HTTP-file remains the lowest-cost direct path. Muxed HLS remains available through the existing Gateway manifest/child-capability path. DASH + MSE and source-browser playback remain comparison/deferred paths.

## Source-level gap

The current main source gives direct evidence for the gap:

| Evidence | Current behavior | Consequence |
| --- | --- | --- |
| site-adapter-api/src/lib.rs:401-414 | StreamProtocol has only HttpFile and Hls; ResolvedStream has id, protocol, URL, public headers and an opaque access reference. | There is no generic source protocol for separated A/V or a typed role, pairing group, codec, container, MIME or bounded expiry. |
| plugins/bilibili/src/lib.rs:269-318 | Candidate selection requires BrowserMediaKind::Muxed and HttpFile/Hls; video/audio candidates fall into UnsupportedMedia. | Existing Bilibili code cannot turn separated A/V observations into a ResolvedMedia. |
| gateway-core/src/source_session.rs:41-57,345-361,519-552 | Public SessionMediaStream exposes only id/protocol/Gateway path; media views start at media_generation: 0. | The Display cannot choose a role/pair or know which server-produced resource represents an A/V group. |
| gateway-core/src/lib.rs:552-573 | Gateway creates a capability from one ResolvedStream and keeps upstream URL/headers server-side. | The security boundary is reusable, but remux input orchestration and group output are absent. |
| gateway-core/src/lib.rs TV display page | The Web Display picks the first http_file stream and assigns one video source. | It supports the current muxed path; it cannot independently coordinate separated tracks or DASH/MSE. |
| gateway-core/src/playback.rs:535-562 | begin_media_refresh and commit_media_refresh provide a generation sequence and item checks. | A future refresh path can use this authority, but it must add a preparation ticket/CAS around resolve, capability issuance, publication and cleanup. |

These observations are repository facts. They do not identify the real Bilibili response shape and do not infer playback from an earlier browser page observation.

## Generic MediaShapeV1

The follow-up implementation should add a versioned, site-neutral shape. The names below are the contract; the exact Rust representation can be chosen in the implementation Task.

    MediaShapeV1
    |- schema_version: 1
    |- media_generation: u64
    |- tracks: Track[]
    +- public_projection: server-created Gateway resources only

    Track
    |- id: opaque bounded identifier
    |- role: muxed | video | audio
    |- group_id: opaque bounded pairing identity
    |- source_protocol: http_file | hls | dash
    |- delivery: direct | server_remux
    |- container: mp4 | fmp4 | ts | webm | unknown
    |- codec: bounded codec identifier
    |- mime: bounded MIME type
    |- range: supported | unsupported | unknown
    |- expires_at_ms: server-only bounded expiry
    +- upstream_access_ref: server-only opaque capability reference

Rules:

1. A group contains either one muxed track or exactly one video and one audio track. A video/audio track without a matching group partner is rejected before publication.
2. source_protocol describes the server-owned input. delivery describes what the Gateway prepares for Display. A separated DASH input therefore becomes one server_remux output represented as browser-compatible fMP4/HTTP-file.
3. codec, container and mime are generic bounded metadata. They must not contain a site URL, DOM term, cookie name, account identifier or signed query.
4. expires_at_ms and upstream_access_ref never cross the Control/Display DTO boundary. Public projection contains only Gateway resource IDs/paths, role/group metadata needed for selection, and media_generation.
5. Every Gateway resource is bound to session_id, item_id, item_revision, media_generation, group/resource identity, method and a short expiry. The browser receives a same-origin path; it never receives the upstream URL, Cookie, Authorization or signed query.
6. The shape is versioned independently of SourceLocator. A SourceLocator remains plugin-owned opaque content identity; a short-lived input URL is never used as content identity.

The bounded harness uses only synthetic opaque references and verifies the two valid group forms, mismatch rejection, expiry bounds and redacted public projection.

## Delivery comparison

| Route | Result | Reason and required boundary |
| --- | --- | --- |
| Existing muxed HTTP-file | PASS for current generic baseline | Already proven by R001; keep direct capability binding, Range handling, EgressPolicy and Web Display behavior. |
| Existing muxed HLS through Gateway | CONDITIONAL PASS | R001 proves manifest/variant/segment Gateway handling, while native browser HLS compatibility is still a Display concern. Keep child capabilities bound to the same item/generation. |
| Server remux to fMP4/HTTP-file | CONDITIONAL PASS; selected first route | One browser video source preserves the current Display shape and avoids exposing A/V inputs. It still needs implementation and hosted synthetic browser proof for codec/container, Range/seek, reconnect, cancellation, expiry and bounded CPU/RSS. |
| Server remux to HLS | CONDITIONAL PASS; fallback | It can cover more browser profiles, but adds manifest/segment state, child capability churn and more cleanup points. Use only when the selected fMP4 output is not compatible and prove the same binding/expiry behavior. |
| DASH + MSE in Web Display | CONDITIONAL PASS; deferred | It may avoid remux but adds a player/MSE compatibility surface and independent segment scheduling. It is not the first route while the Display is a single native video source. |
| Source-browser playback / forwarding the site page | FAIL as Gateway delivery | A browser blob or page player is local to the source browser, cannot be bound as a Gateway media capability for another Display, and would couple playback to the Browser Worker/profile. Browser observation remains acquisition input only. |

## Same-item refresh contract

Refresh is required when a Gateway capability expires or the server receives an upstream 401, 403 or 410 that is eligible for re-resolution.

    1. Read the current PlaybackSession snapshot.
    2. Create RefreshTicket(session_id, item_id, item_revision,
       expected_media_generation, expected_display_generation, SourceLocator).
    3. Re-resolve the opaque SourceLocator through SiteAdapterRegistry.
    4. Validate the new MediaShapeV1 and EgressPolicy for every input.
    5. Prepare new direct/remux capabilities with bounded expiry.
    6. CAS commit only if session, item revision, media generation and display
       generation still match the ticket.
    7. Publish media_generation + 1 and the new public projection atomically.
    8. Revoke/cleanup old capabilities and remux processes after the new
       projection is committed.
    9. If CAS fails, discard the resolved result and revoke its capabilities;
       never replace a newer item, display or media generation.

A successful same-item refresh keeps item_id, item_revision and the active display. It increments media_generation, refreshes the Display projection, and preserves position only after the new resource reports compatible metadata. A failed refresh keeps the last authoritative snapshot and returns an explicit bounded error. It must not silently replace the source with a stale result.

The current PlaybackSession begin_media_refresh/commit_media_refresh is useful evidence for the existing generation authority, but the implementation Task must ensure the generation is reserved/committed with the full resolve and capability publication CAS. In particular, a delayed remux completion must not publish after a NextItem or handoff.

## Follow-up implementation contract

Publish a separate combined implementation Task after this research is accepted. Its scope should include:

1. Add MediaShapeV1/track metadata and conformance validation to site-adapter-api; keep old muxed ResolvedStream construction source-compatible during migration.
2. Add a generic resolver-to-Gateway mapping for direct muxed streams and a separated A/V group. The first output is one server-owned stream-copy fMP4/HTTP-file resource; no site-specific Core branch.
3. Add a bounded remux supervisor using structured argv, explicit public EgressPolicy inputs, cancellation, timeout, output-size/CPU/process limits and cleanup. It must never accept a caller-provided arbitrary host or shell string.
4. Bind each input and output to scoped server capabilities. Keep upstream_access_ref, upstream URL and headers out of Display/Control DTOs, logs and artifacts.
5. Add refresh orchestration around SourceLocator re-resolution and media-generation CAS. Revoke stale capabilities and stop stale remux processes.
6. Extend Web Display selection to prefer the public muxed/remuxed fMP4 resource, retain current HTTP-file/HLS behavior, and reject unsupported codec/container combinations with an explicit UI error.
7. Add hosted x64 synthetic verification for valid/mismatched shapes, public secret boundary, separated A/V pairing, remux cancellation/cleanup, expired/401/403/410 refresh, stale item/media/display CAS, Range/seek/reconnect and bounded output. Add browser compatibility evidence for the synthetic fMP4 fixture.
8. Keep real Bilibili observation and target playback out of this implementation Task until #246 authorization is independently satisfied and a new Verification Task is published.

Required implementation acceptance must include the existing Playback concurrency minimum set, plus:

    duplicate request_id
    stale expected revision
    stale item callback
    stale re-resolve result
    stale display generation
    overlapping handoff
    two-Control concurrent mutation

## Claim results for this Attempt

- C1: PASS — source-level gap is directly anchored above; no live-site inference.
- C2: PASS — the bounded harness validates the generic versioned shape, pairing and redacted public projection.
- C3: CONDITIONAL PASS — route comparison and refresh/CAS model pass synthetically; production remux, browser compatibility and resource measurements remain unimplemented.
- C4: PASS for contract expression — the refresh/CAS semantics are expressible and the harness rejects stale item/media/display results. Production orchestration remains follow-up scope.
- C5: CONDITIONAL PASS — the fMP4/HTTP-file server-remux route is selected with explicit conditions and a concrete implementation contract.
- C6: PASS — hosted workflow is exact-Candidate bound, secret-free and emits bounded JSON/provenance only.
- C7: PASS — this Attempt makes no real Bilibili/auth/target/playback claim; #246 remains independently blocked.

## Evidence and limits

The required hosted workflow runs four jobs:

- J0: Coordinator read-back of current main/canonical source.
- J1: hosted x64 static source-gap and public-shape checks.
- J2: hosted x64 synthetic shape, pairing and refresh/CAS checks.
- J3: hosted x64 bounded route/browser-compatibility model and artifact guard.

The workflow does not compile the repository or contact a site. It is not a substitute for production implementation, browser codec testing with a real fMP4 fixture, target ARM64 resource evidence, or authorized Bilibili playback.

Only the workflow run, job IDs, artifact ID and exact Candidate SHA recorded in the Issue report are authoritative for this Attempt.
