// Minimal PlaybackSession registry — single-user prototype.
// Mirrors session/item identity (session_id, item_id, item_revision,
// media_generation) but has no CAS/revision machinery: the Rust layer owns
// that, and the prototype must not pretend to validate it.

import { randomUUID } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const sessions = new Map();
let currentId = null;
const history = [];
const HISTORY_MAX = 10;
const HIST_FILE = join(
  dirname(fileURLToPath(import.meta.url)),
  '../runtime/history.json',
);
try {
  history.push(
    ...JSON.parse(readFileSync(HIST_FILE, 'utf8')).slice(0, HISTORY_MAX),
  );
} catch {
  /* first run */
}
const saveHistory = () => {
  try {
    mkdirSync(dirname(HIST_FILE), { recursive: true });
    writeFileSync(HIST_FILE, JSON.stringify(history));
  } catch {
    /* runtime dir may be readonly */
  }
};

export function pushHistory(locator, title) {
  const key = JSON.stringify(locator?.opaque_payload || {});
  const i = history.findIndex(
    (h) => JSON.stringify(h.locator?.opaque_payload) === key,
  );
  if (i >= 0) history.splice(i, 1);
  history.unshift({ locator, title, at: Date.now() });
  if (history.length > HISTORY_MAX) history.pop();
  saveHistory();
}

export function listHistory() {
  return history;
}

export function createSession(locator, media) {
  const session = {
    session_id: `s-${randomUUID().replaceAll('-', '')}`,
    current_item: {
      item_id: `i-${randomUUID().replaceAll('-', '')}`,
      item_revision: 1,
      source_locator: locator,
      resolved_media: media,
      media_generation: 0,
    },
    active_display: null,
    created_at: Date.now(),
  };
  sessions.set(session.session_id, session);
  currentId = session.session_id;
  return session;
}

export function getSession(id) {
  return sessions.get(id) ?? null;
}

export function currentSession() {
  return currentId ? sessions.get(currentId) : null;
}

export function closeSession(id) {
  sessions.delete(id);
  if (currentId === id) currentId = null;
}
