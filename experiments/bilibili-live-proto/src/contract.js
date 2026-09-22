// Contract-mirror types for the live prototype. Field names deliberately
// match docs/implementation-contracts.md so a working prototype proves the
// real contracts can express reality; machinery the Rust layer owns
// (CAS revisions, Vault, EgressPolicy, conformance) is intentionally absent.

/**
 * @typedef {Object} SourceLocator
 * @property {string} site_id
 * @property {string} plugin_id
 * @property {number} locator_version
 * @property {object} opaque_payload   // plugin-owned; core/proxy never reads
 */

/**
 * @typedef {Object} ResolvedStream
 * @property {string} id
 * @property {'muxed'|'video'|'audio'} kind
 * @property {'http_file'|'hls'|'dash'} protocol
 * @property {string} url              // upstream URL; may expire
 * @property {Record<string,string>} upstream_headers // server-side only
 * @property {number} [width]
 * @property {number} [height]
 * @property {number} [bitrate]
 */

/**
 * @typedef {Object} ResolvedMedia
 * @property {string} title
 * @property {number} [duration]
 * @property {string} source_site
 * @property {ResolvedStream[]} streams
 * @property {number} [expires_at]     // epoch ms; refresh via locator
 * @property {'clear'|'drm_unsupported'|'unsupported'} protection
 */

/**
 * @typedef {Object} PlaybackItem
 * @property {string} item_id
 * @property {number} item_revision
 * @property {SourceLocator} source_locator
 * @property {ResolvedMedia} resolved_media
 * @property {number} media_generation
 */

/**
 * @typedef {Object} PlaybackSession
 * @property {string} session_id
 * @property {PlaybackItem} current_item
 * @property {string|null} active_display
 */
