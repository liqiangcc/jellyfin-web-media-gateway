// Minimal BrowserWorker mirror: attaches to an existing Chrome via CDP,
// navigates to a page, extracts an observation, closes the tab.
// Mirrors BrowserObservationHandoff: bounded navigation + bounded payload.
// Requires node --experimental-websocket (Node 20) for global WebSocket.

const CDP_HTTP = process.env.PROTO_CDP || 'http://127.0.0.1:9333';
const NAV_TIMEOUT_MS = 20000;

let msgId = 0;
function rpc(ws, method, params = {}) {
  const id = ++msgId;
  return new Promise((resolve, reject) => {
    const onMsg = (ev) => {
      const m = JSON.parse(ev.data);
      if (m.id === id) {
        ws.removeEventListener('message', onMsg);
        m.error ? reject(new Error(m.error.message)) : resolve(m.result);
      }
    };
    ws.addEventListener('message', onMsg);
    ws.send(JSON.stringify({ id, method, params }));
  });
}

async function evaluate(ws, expr) {
  const r = await rpc(ws, 'Runtime.evaluate', {
    expression: expr,
    returnByValue: true,
    awaitPromise: true,
  });
  if (r.exceptionDetails) {
    throw new Error(
      r.exceptionDetails.exception?.description || 'evaluate failed',
    );
  }
  return r.result.value;
}

/**
 * Navigate a fresh tab to `url`, wait until `waitFor` evaluates truthy,
 * then run `extract` and return its value. Tab is always closed.
 * `cookies` (optional "k=v; k=v" string) is injected for .bilibili.com
 * before navigation so the browser reuses the QR-login session.
 */
export async function observe(url, waitFor, extract, cookies) {
  const tab = await fetch(`${CDP_HTTP}/json/new`, { method: 'PUT' }).then(
    (r) => r.json(),
  );
  const wsUrl = tab.webSocketDebuggerUrl;
  try {
    const ws = await new Promise((res, rej) => {
      const w = new WebSocket(wsUrl);
      w.onopen = () => res(w);
      w.onerror = (e) => rej(new Error('cdp ws failed'));
    });
    try {
      await rpc(ws, 'Runtime.enable');
      await rpc(ws, 'Network.enable');
      if (cookies) {
        for (const pair of cookies.split(';')) {
          const eq = pair.indexOf('=');
          if (eq < 0) continue;
          await rpc(ws, 'Network.setCookie', {
            name: pair.slice(0, eq).trim(),
            value: pair.slice(eq + 1).trim(),
            domain: '.bilibili.com',
          });
        }
      }
      await rpc(ws, 'Page.enable');
      await rpc(ws, 'Page.navigate', { url });
      // Poll for readiness rather than racing the load event.
      const deadline = Date.now() + NAV_TIMEOUT_MS;
      let ready = false;
      while (Date.now() < deadline) {
        try {
          ready = await evaluate(ws, waitFor);
        } catch {
          /* navigation in flight */
        }
        if (ready) break;
        await new Promise((r) => setTimeout(r, 400));
      }
      if (!ready) throw new Error('navigation timeout');
      return await evaluate(ws, extract);
    } finally {
      ws.close();
    }
  } finally {
    await fetch(`${CDP_HTTP}/json/close/${tab.id}`).catch(() => {});
  }
}

/**
 * Navigate a fresh tab to `url` and capture the response body of the first
 * request matching `urlRe` (checked on loadingFinished). Returns the body
 * string, or null on timeout/punish. Used by browser-first sites (youku)
 * where the API request must be signed by the page's own JS.
 */
export async function captureResponse(url, urlRe, waitMs = 25000, opts = {}) {
  const tab = await fetch(`${CDP_HTTP}/json/new`, { method: 'PUT' }).then(
    (r) => r.json(),
  );
  const wsUrl = tab.webSocketDebuggerUrl;
  try {
    const ws = await new Promise((res, rej) => {
      const w = new WebSocket(wsUrl);
      w.onopen = () => res(w);
      w.onerror = () => rej(new Error('cdp ws failed'));
    });
    try {
      await rpc(ws, 'Page.enable');
      await rpc(ws, 'Runtime.enable');
      await rpc(ws, 'Network.enable');
      if (opts.ua)
        await rpc(ws, 'Emulation.setUserAgentOverride', {
          userAgent: opts.ua,
        });
      const rid = {};
      let hit = null;
      ws.addEventListener('message', (ev) => {
        const m = JSON.parse(ev.data);
        if (m.method === 'Network.requestWillBeSent')
          rid[m.params.requestId] = m.params.request.url;
        else if (
          m.method === 'Network.loadingFinished' &&
          urlRe.test(rid[m.params.requestId] || '')
        )
          hit = m.params.requestId;
      });
      await rpc(ws, 'Page.navigate', { url });
      const deadline = Date.now() + waitMs;
      while (!hit && Date.now() < deadline)
        await new Promise((r) => setTimeout(r, 300));
      if (!hit) return null;
      const body = await rpc(ws, 'Network.getResponseBody', {
        requestId: hit,
      });
      return body.body || null;
    } finally {
      ws.close();
    }
  } finally {
    await fetch(`${CDP_HTTP}/json/close/${tab.id}`).catch(() => {});
  }
}

/**
 * Navigate a fresh tab to `url`, poll until `waitFor` is truthy (or
 * waitMs elapses), then evaluate `expr` in page context and return the
 * value. Used by browser-first sites that need the page's own JS
 * (e.g. youku's lib.mtop.request which signs API calls).
 */
export async function evalInPage(url, waitFor, expr, waitMs = 20000) {
  const tab = await fetch(`${CDP_HTTP}/json/new`, { method: 'PUT' }).then(
    (r) => r.json(),
  );
  try {
    const ws = await new Promise((res, rej) => {
      const w = new WebSocket(tab.webSocketDebuggerUrl);
      w.onopen = () => res(w);
      w.onerror = () => rej(new Error('cdp ws failed'));
    });
    try {
      await rpc(ws, 'Runtime.enable');
      await rpc(ws, 'Page.enable');
      await rpc(ws, 'Page.navigate', { url });
      const deadline = Date.now() + waitMs;
      while (Date.now() < deadline) {
        try {
          if (await evaluate(ws, waitFor)) break;
        } catch {
          /* navigation in flight */
        }
        await new Promise((r) => setTimeout(r, 400));
      }
      return await evaluate(ws, expr);
    } finally {
      ws.close();
    }
  } finally {
    await fetch(`${CDP_HTTP}/json/close/${tab.id}`).catch(() => {});
  }
}

/** Search bilibili via the real search page; extract result cards. */
export async function search(keyword, { limit = 12, cookies } = {}) {
  const url = `https://search.bilibili.com/all?keyword=${encodeURIComponent(keyword)}`;
  const waitFor = `!!document.querySelector('.bili-video-card a[href*=BV]')`;
  const extract = `
    [...document.querySelectorAll('.bili-video-card')]
      .filter(c=>c.querySelector('a[href*=BV]'))
      .slice(0, ${limit})
      .map(c=>{
        const a=c.querySelector('a[href*=BV]');
        const t=c.querySelector('[class*=info--tit], [title]');
        const dur=c.querySelector('[class*=duration]');
        const img=c.querySelector('img');
        const raw=img?(img.dataset.src||img.currentSrc||img.src||''):'';
        const cover=raw.startsWith('//')?'https:'+raw:raw;
        return {
          bvid:(a.href.match(/BV\\w+/)||[])[0],
          title:(t&&(t.getAttribute('title')||t.textContent)||'').trim(),
          duration:dur?dur.textContent.trim():'',
          cover,
        };
      })`;
  return observe(url, waitFor, extract, cookies);
}
