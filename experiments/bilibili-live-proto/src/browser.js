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
