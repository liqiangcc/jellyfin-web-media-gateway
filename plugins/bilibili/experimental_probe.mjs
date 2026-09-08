/**
 * Experimental, non-production interpretation of browser observations for #165.
 *
 * This module deliberately accepts only a small, already-sanitised DTO. It does
 * not parse DOM, HAR, cookies, signed URLs, or arbitrary CDP input. The browser
 * runtime owns lifecycle and egress; this module only interprets site fields.
 */

export const SCHEMA_VERSION = 1;
export const MAX_EVENTS = 200;
export const MAX_RESOURCES = 64;
export const MAX_METADATA_BYTES = 1024 * 1024;

const ROLES = new Set(["muxed", "video", "audio"]);
const SECRET_NAMES = /^(authorization|cookie|set-cookie|proxy-authorization|x-api-key|access-token|refresh-token|id-token|signature|token)$/i;
const SAFE_HEADER_NAMES = new Set(["accept-ranges", "content-range", "content-type", "content-length", "etag", "last-modified"]);
const CODEC = /^[A-Za-z0-9._-]{1,64}$/;
const CONTAINER = /^[A-Za-z0-9._+/-]{1,64}$/;

function object(value, label) {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error(`${label} must be an object`);
  return value;
}

function string(value, label, max = 128) {
  if (typeof value !== "string" || value.length === 0 || value.length > max) throw new Error(`${label} invalid`);
  return value;
}

function safeHeaderNames(value) {
  if (!Array.isArray(value) || value.length > 16) throw new Error("header_names invalid");
  return [...new Set(value.map((name) => string(name, "header_name", 64).toLowerCase()))]
    .filter((name) => !SECRET_NAMES.test(name))
    .filter((name) => SAFE_HEADER_NAMES.has(name));
}

function expiryClass(value) {
  if (value === undefined || value === null) return "unknown";
  if (!["none-observed", "short-lived", "expired", "unknown"].includes(value)) throw new Error("expiry_hint invalid");
  return value;
}

function candidate(value, index) {
  const item = object(value, `candidate[${index}]`);
  const role = string(item.role, "candidate.role", 16);
  if (!ROLES.has(role)) throw new Error("candidate.role invalid");
  const codec = item.codec === undefined || item.codec === null ? "unknown" : string(item.codec, "candidate.codec", 64);
  const container = item.container === undefined || item.container === null ? "unknown" : string(item.container, "candidate.container", 64);
  if (codec !== "unknown" && !CODEC.test(codec)) throw new Error("candidate.codec invalid");
  if (container !== "unknown" && !CONTAINER.test(container)) throw new Error("candidate.container invalid");
  const statusClass = item.http_status_class === undefined ? "unknown" : string(item.http_status_class, "candidate.http_status_class", 8);
  if (!/^(2xx|3xx|4xx|5xx|unknown)$/.test(statusClass)) throw new Error("candidate status invalid");
  if (typeof item.range_supported !== "boolean" && item.range_supported !== "unknown") throw new Error("range_supported invalid");
  if (typeof item.egress_allowed !== "boolean") throw new Error("egress_allowed invalid");
  return {
    role,
    codec,
    container,
    http_status_class: statusClass,
    range_supported: item.range_supported,
    header_names: safeHeaderNames(item.header_names || []),
    egress_allowed: item.egress_allowed,
    expiry_hint: expiryClass(item.expiry_hint),
  };
}

export function summarizeObservation(input) {
  const raw = object(input, "observation");
  if (raw.schema_version !== SCHEMA_VERSION) throw new Error("schema_version unsupported");
  const selector = object(raw.selector, "selector");
  const siteId = string(selector.site_id, "selector.site_id", 32);
  const contentId = string(selector.content_id, "selector.content_id", 96);
  const part = Number(selector.part);
  if (!Number.isInteger(part) || part < 1 || part > 9999) throw new Error("selector.part invalid");
  if (raw.part_match !== true && raw.part_match !== false) throw new Error("part_match invalid");
  if (!Number.isInteger(raw.events) || raw.events < 0 || raw.events > MAX_EVENTS) throw new Error("events over budget");
  if (!Number.isInteger(raw.resources) || raw.resources < 0 || raw.resources > MAX_RESOURCES) throw new Error("resources over budget");
  const candidates = Array.isArray(raw.candidates) ? raw.candidates.slice(0, MAX_RESOURCES).map(candidate) : [];
  if (!Array.isArray(raw.candidates) || raw.candidates.length !== candidates.length) throw new Error("candidates over budget");
  const kind = ["muxed", "av_separated", "blob_only", "unknown"].includes(raw.source_kind) ? raw.source_kind : "unknown";
  const independent = raw.independent_after_browser_exit === true;
  const metadataBytes = Number(raw.metadata_bytes);
  if (!Number.isInteger(metadataBytes) || metadataBytes < 0 || metadataBytes > MAX_METADATA_BYTES) throw new Error("metadata over budget");
  return {
    schema_version: SCHEMA_VERSION,
    selector: { site_id: siteId, content_id: contentId, part },
    part_match: raw.part_match,
    source_kind: kind,
    candidate_counts: {
      total: candidates.length,
      muxed: candidates.filter((item) => item.role === "muxed").length,
      video: candidates.filter((item) => item.role === "video").length,
      audio: candidates.filter((item) => item.role === "audio").length,
    },
    candidates,
    independent_after_browser_exit: independent,
    budget: { events: raw.events, resources: raw.resources, metadata_bytes: metadataBytes },
  };
}

export function rejectSensitiveInput(value) {
  const text = JSON.stringify(value);
  if (/https?:\/\/|blob:|file:|cookie|authorization|set-cookie|signed-url|sessdata|bearer/i.test(text)) {
    throw new Error("sensitive or non-opaque input rejected");
  }
}
