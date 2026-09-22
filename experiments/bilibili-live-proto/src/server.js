// Prototype Gateway server — acp-web-console style: plain Node http, no
// framework, no build step. Bind to the tailnet IP only.

import http from 'node:http';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { createRequire } from 'node:module';
import {
  recognize,
  resolve,
  danmaku,
  favorites,
  qrLoginStart,
  qrLoginPoll,
  setAuthCookie,
  getAuthCookie,
  loggedIn,
} from './sites/bilibili.js';
import { search } from './browser.js';
import { createSession, getSession, currentSession } from './session.js';
import { proxyStream, remuxToHls } from './media.js';

const require = createRequire(import.meta.url);
const qrcode = require('./qrcode.cjs');
const RUNTIME_DIR = new URL('../runtime/', import.meta.url).pathname;
const AUTH_FILE = join(RUNTIME_DIR, 'auth.json');

// Restore persisted cookie (prototype-local; real impl = Vault).
try {
  const saved = JSON.parse(readFileSync(AUTH_FILE, 'utf8'));
  if (saved.cookie) setAuthCookie(saved.cookie);
} catch {
  /* no saved auth */
}

let pendingQr = null; // {qrcode_key, expires}

const BIND = process.env.PROTO_BIND || '100.64.98.39';
const PORT = Number(process.env.PROTO_PORT || 8899);
const BASE = `http://${BIND}:${PORT}`;

const CONTROL_HTML = `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>proto control</title>
<style>body{font-family:system-ui;max-width:640px;margin:2rem auto;padding:0 1rem}input{width:100%;padding:.6rem;font-size:1rem}button{padding:.6rem 1.4rem;font-size:1rem;margin-top:.6rem}pre{background:#f4f4f4;padding:.8rem;overflow:auto;font-size:.8rem}</style>
<h2>Control</h2>
<form id="f"><input id="src" type="url" required placeholder="https://www.bilibili.com/video/BV…?p=2"><button>Play on display</button></form>
<hr>
<form id="sf"><input id="q" type="search" placeholder="search bilibili…"><button>Search</button></form>
<div id="results"></div>
<hr>
<button id="fav">我的收藏夹</button> <a href="/qr">登录</a>
<div id="favlist"></div>
<pre id="out">idle</pre>
<p><a href="/display">open display →</a></p>
<script>
const out=document.getElementById('out'),results=document.getElementById('results');
async function playSource(src){
  out.textContent='resolving…';
  try{
    const r=await fetch('/api/play',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({source:src})});
    const j=await r.json();
    out.textContent=r.ok?JSON.stringify({session:j.session_id,title:j.title,mode:j.media_mode},null,2):'ERR '+r.status+' '+JSON.stringify(j);
  }catch(err){out.textContent='ERR '+err}
}
document.getElementById('f').onsubmit=e=>{e.preventDefault();playSource(document.getElementById('src').value)};
document.getElementById('sf').onsubmit=async e=>{
  e.preventDefault();
  results.textContent='searching…';
  try{
    const r=await fetch('/api/search?q='+encodeURIComponent(document.getElementById('q').value));
    const list=await r.json();
    if(!r.ok)throw new Error(JSON.stringify(list));
    results.innerHTML=list.map(x=>'<div class="hit" data-bv="'+x.bvid+'"><img src="'+x.cover+'"><div><b>'+x.title+'</b><br><small>'+x.bvid+' '+x.duration+'</small></div></div>').join('')||'no results';
    results.querySelectorAll('.hit').forEach(el=>el.onclick=()=>playSource('https://www.bilibili.com/video/'+el.dataset.bv));
  }catch(err){results.textContent='ERR '+err}
};
const favlist=document.getElementById('favlist');
document.getElementById('fav').onclick=async()=>{
  favlist.textContent='loading…';
  try{
    const r=await fetch('/api/favorites');
    const folders=await r.json();
    if(!r.ok)throw new Error(folders.error||r.status);
    favlist.innerHTML=folders.map(f=>'<div class="hit" data-fid="'+f.id+'"><div><b>📁 '+f.title+'</b> <small>('+f.count+')</small></div></div>').join('')||'no folders';
    favlist.querySelectorAll('.hit').forEach(el=>el.onclick=async()=>{
      favlist.textContent='loading…';
      const items=await fetch('/api/favorites?folder='+el.dataset.fid).then(r=>r.json());
      favlist.innerHTML=items.map(x=>'<div class="hit" data-bv="'+x.bvid+'"><img src="'+x.cover+'"><div><b>'+x.title+'</b><br><small>'+x.bvid+' '+x.duration+'</small></div></div>').join('')||'empty folder';
      favlist.querySelectorAll('.hit').forEach(i=>i.onclick=()=>playSource('https://www.bilibili.com/video/'+i.dataset.bv));
    });
  }catch(err){favlist.textContent='ERR '+err.message}
};
</script>
<style>#results{display:flex;flex-direction:column;gap:.5rem;margin:.8rem 0}.hit{display:flex;gap:.6rem;cursor:pointer;align-items:center}.hit img{width:96px;height:60px;object-fit:cover;border-radius:4px}</style>`;

const DISPLAY_HTML = `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>proto display</title>
<style>body{margin:0;background:#000}video{width:100vw;height:100vh;object-fit:contain}#s{position:fixed;top:0;color:#fff;font:14px system-ui;background:#0008;padding:.4rem}</style>
<video id="v" controls playsinline></video><div id="s">waiting for control…</div>
<div id="dm" style="position:fixed;inset:0;pointer-events:none;overflow:hidden"></div>
<script src="/hls.min.js"></script>
<script>
const v=document.getElementById('v'),s=document.getElementById('s'),dm=document.getElementById('dm');
let cur=null,hls=null,pool=[],pidx=0,laneFree=[];
const LANES=10,LINE_H=Math.floor(innerHeight*0.05),FLY_MS=7000;
function laneFor(){for(let i=0;i<LANES;i++)if((laneFree[i]||0)<performance.now())return i;return -1}
function spawn(d){
  const lane=laneFor();if(lane<0)return;
  const el=document.createElement('div');
  el.textContent=d.text;
  el.style.cssText='position:absolute;white-space:nowrap;font-size:'+LINE_H*0.8+'px;color:#'+d.color.toString(16).padStart(6,'0')+';text-shadow:1px 1px 2px #000;top:'+(lane*LINE_H)+'px;left:100%;will-change:transform';
  dm.appendChild(el);
  const w=el.offsetWidth+innerWidth;
  const a=el.animate([{transform:'translateX(0)'},{transform:'translateX(-'+w+'px)'}],{duration:FLY_MS,easing:'linear'});
  a.onfinish=()=>el.remove();
  if(v.paused)a.pause();
  laneFree[lane]=performance.now()+FLY_MS*(el.offsetWidth/w)+300;
}
setInterval(()=>{
  if(!pool.length)return;
  const t=v.currentTime;
  if(pidx>0&&(pool[pidx-1]&&pool[pidx-1].t>t+2)){dm.innerHTML='';pidx=pool.findIndex(d=>d.t>t);if(pidx<0)pidx=pool.length}
  while(pidx<pool.length&&pool[pidx].t<=t){if(pool[pidx].mode===1)spawn(pool[pidx]);pidx++}
},200);
v.addEventListener('pause',()=>{v.getAnimations?0:0;dm.querySelectorAll('div').forEach(e=>e.getAnimations().forEach(a=>a.pause()))});
v.addEventListener('play',()=>{dm.querySelectorAll('div').forEach(e=>e.getAnimations().forEach(a=>a.play()))});
setInterval(async()=>{
  try{
    const j=await (await fetch('/api/now')).json();
    if(!j.session){s.textContent='waiting for control…';return}
    if(cur!==j.session.session_id+'|'+j.session.media_url){
      cur=j.session.session_id+'|'+j.session.media_url;
      if(hls){hls.destroy();hls=null}
      pool=[];pidx=0;dm.innerHTML='';laneFree=[];
      fetch('/dm/'+j.session.session_id).then(r=>r.json()).then(l=>{pool=l;pidx=0}).catch(()=>{});
      const url=j.session.media_url;
      if(url.endsWith('.m3u8')&&window.Hls&&Hls.isSupported()){
        hls=new Hls();hls.loadSource(url);hls.attachMedia(v);
      }else{
        v.src=url;v.load();
      }
      s.textContent=j.session.title+' ('+j.session.media_mode+')';
      v.play().catch(()=>{s.textContent+=' — tap to play'});
    }
  }catch(e){s.textContent='display reconnecting…'}
},1500);
</script>`;

async function play(body) {
  const rec = recognize(body.source || '');
  if (!rec.matched) return [400, { error: 'SOURCE_NOT_RECOGNIZED' }];
  const media = await resolve(rec.locator, {
    prefer: body.force_dash ? 'dash' : undefined,
  });
  const streams = media.streams;

  // Live: proxy the upstream playlist, rewriting segment URIs through us.
  if (media.live) {
    const session = createSession(rec.locator, media);
    return [
      200,
      {
        session_id: session.session_id,
        title: media.title,
        media_mode: 'hls-live',
        media_url: `${BASE}/livepl/${session.session_id}/index.m3u8`,
        streams: media.streams.map((s) => ({
          id: s.id,
          kind: s.kind,
          protocol: s.protocol,
        })),
      },
    ];
  }

  // Muxed file → direct gateway proxy path. Separate A/V → remux to HLS.
  // The session is only created after the media path is ready, so the
  // display never sees a half-prepared session (media_url is stable).
  const muxed = streams.find((s) => s.kind === 'muxed');
  let hlsDir = null;
  if (!muxed) {
    const video = streams.find((s) => s.kind === 'video');
    const audio = streams.find((s) => s.kind === 'audio');
    if (!video || !audio) return [422, { error: 'SOURCE_UNSUPPORTED' }];
    const tag = rec.locator.opaque_payload;
    hlsDir = (await remuxToHls(video, audio, `${tag.bvid}-p${tag.page}`)).dir;
  }
  const session = createSession(rec.locator, media);
  session.hls_dir = hlsDir;
  const media_mode = hlsDir ? 'hls-remux' : 'file';
  const media_url = hlsDir
    ? `${BASE}/hls/${session.session_id}/index.m3u8`
    : `${BASE}/stream/${session.session_id}/0`;
  return [
    200,
    {
      session_id: session.session_id,
      title: media.title,
      media_mode,
      media_url,
      streams: streams.map((s) => ({ id: s.id, kind: s.kind, protocol: s.protocol })),
    },
  ];
}

http
  .createServer(async (req, res) => {
    const url = new URL(req.url, BASE);
    try {
      if (req.method === 'GET' && url.pathname === '/hls.min.js') {
        res.writeHead(200, { 'content-type': 'application/javascript' });
        return res.end(readFileSync(join(import.meta.dirname, 'hls.min.js')));
      }
      if (req.method === 'GET' && url.pathname === '/control') {
        res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
        return res.end(CONTROL_HTML);
      }
      if (req.method === 'GET' && url.pathname === '/display') {
        res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
        return res.end(DISPLAY_HTML);
      }
      if (req.method === 'POST' && url.pathname === '/api/play') {
        let raw = '';
        for await (const c of req) raw += c;
        const [code, body] = await play(JSON.parse(raw || '{}'));
        res.writeHead(code, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(body));
      }
      if (req.method === 'GET' && url.pathname === '/api/search') {
        const q = url.searchParams.get('q') || '';
        if (!q.trim()) {
          res.writeHead(400, { 'content-type': 'application/json' });
          return res.end(JSON.stringify({ error: 'EMPTY_QUERY' }));
        }
        const list = await search(q, { cookies: getAuthCookie() });
        // Route covers through the gateway so no third-party host is
        // contacted by the client (same-origin boundary).
        for (const x of list) {
          x.cover = `${BASE}/img?u=${encodeURIComponent(x.cover)}`;
        }
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(list));
      }
      if (req.method === 'GET' && url.pathname === '/qr') {
        // Fresh QR each visit; page polls /api/qr-status.
        const qr = await qrLoginStart();
        pendingQr = {
          qrcode_key: qr.qrcode_key,
          expires: Date.now() + 170_000,
        };
        const qrSvg = qrcode(0, 'M');
        qrSvg.addData(qr.url);
        qrSvg.make();
        res.writeHead(200, { 'content-type': 'text/html; charset=utf-8' });
        return res.end(`<!doctype html><meta name="viewport" content="width=device-width"><title>bilibili login</title>
<body style="font-family:system-ui;text-align:center;padding-top:2rem">
<h3>用哔哩哔哩 App 扫码登录</h3>
<div style="display:inline-block;border:1px solid #ddd;padding:8px">${qrSvg.createSvgTag(6)}</div>
<p id="st">等待扫码…（二维码约 3 分钟过期，过期请刷新本页）</p>
<script>
setInterval(async()=>{
  const j=await fetch('/api/qr-status').then(r=>r.json());
  document.getElementById('st').textContent=j.status;
  if(j.logged_in)location.href='/control';
  if(j.status==='二维码已过期，请刷新')location.reload();
},1500);
</script>`);
      }
      if (req.method === 'GET' && url.pathname === '/api/qr-status') {
        if (loggedIn()) {
          res.writeHead(200, { 'content-type': 'application/json' });
          return res.end(JSON.stringify({ logged_in: true, status: '已登录' }));
        }
        if (!pendingQr || Date.now() > pendingQr.expires) {
          res.writeHead(200, { 'content-type': 'application/json' });
          return res.end(
            JSON.stringify({ logged_in: false, status: '二维码已过期，请刷新' }),
          );
        }
        const p = await qrLoginPoll(pendingQr.qrcode_key);
        res.writeHead(200, { 'content-type': 'application/json' });
        if (p.code === 0 && p.cookies) {
          setAuthCookie(p.cookies);
          mkdirSync(RUNTIME_DIR, { recursive: true });
          writeFileSync(AUTH_FILE, JSON.stringify({ cookie: p.cookies }));
          pendingQr = null;
          return res.end(JSON.stringify({ logged_in: true, status: '已登录' }));
        }
        const label =
          { 86101: '等待扫码…', 86090: '已扫码，请在手机上确认' }[p.code] ||
          `code ${p.code}`;
        return res.end(JSON.stringify({ logged_in: false, status: label }));
      }
      if (req.method === 'GET' && url.pathname === '/api/auth') {
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ logged_in: loggedIn() }));
      }
      if (req.method === 'GET' && url.pathname === '/api/favorites') {
        if (!loggedIn()) {
          res.writeHead(401, { 'content-type': 'application/json' });
          return res.end(JSON.stringify({ error: 'NOT_LOGGED_IN' }));
        }
        const fid = url.searchParams.get('folder');
        const list = await favorites(fid ? Number(fid) : null);
        if (fid) {
          for (const x of list) {
            x.cover = `${BASE}/img?u=${encodeURIComponent(x.cover)}`;
          }
        }
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(list));
      }
      if (req.method === 'GET' && url.pathname === '/img') {
        // Bounded cover proxy: only bilibili image CDN hosts allowed.
        const u = url.searchParams.get('u') || '';
        let up;
        try {
          up = new URL(u.startsWith('//') ? `https:${u}` : u);
        } catch {
          up = null;
        }
        if (!up || !/^[a-z0-9-]+\.hdslb\.com$/.test(up.hostname)) {
          res.writeHead(403);
          return res.end('host not allowed');
        }
        const img = await fetch(up, {
          headers: { Referer: 'https://www.bilibili.com' },
        });
        res.writeHead(img.status, {
          'content-type': img.headers.get('content-type') || 'image/jpeg',
        });
        return res.end(Buffer.from(await img.arrayBuffer()));
      }
      const livepl = /^\/livepl\/([\w-]+)\/index\.m3u8$/.exec(url.pathname);
      if (livepl && req.method === 'GET') {
        const s = getSession(livepl[1]);
        if (!s || !s.current_item.resolved_media.live) {
          res.writeHead(404);
          return res.end('no live session');
        }
        let st = s.current_item.resolved_media.streams[0];
        let up = await fetch(st.url, { headers: st.upstream_headers });
        // Live playlist URLs expire fast — refresh via the locator on failure.
        if (!up.ok) {
          const fresh = await resolve(s.current_item.source_locator, {});
          s.current_item.resolved_media = fresh;
          st = fresh.streams[0];
          up = await fetch(st.url, { headers: st.upstream_headers });
        }
        if (!up.ok) {
          res.writeHead(502);
          return res.end('upstream playlist failed');
        }
        const text = await up.text();
        const base = st.url.slice(0, st.url.lastIndexOf('/') + 1);
        // Rewrite segment URIs (standalone lines AND URI= attrs like
        // #EXT-X-MAP) to route through /seg with upstream headers.
        const rewrite = (u) => {
          const abs = u.startsWith('http') ? u : base + u;
          return `${BASE}/seg?u=${encodeURIComponent(abs)}`;
        };
        const rewritten = text
          .split('\n')
          .map((line) => {
            const t = line.trim();
            if (!t) return line;
            if (t.startsWith('#')) {
              return line.replace(/URI="([^"]+)"/g, (_, u) => `URI="${rewrite(u)}"`);
            }
            return rewrite(t);
          })
          .join('\n');
        res.writeHead(200, {
          'content-type': 'application/vnd.apple.mpegurl',
          'cache-control': 'no-store',
        });
        return res.end(rewritten);
      }
      if (req.method === 'GET' && url.pathname === '/seg') {
        const u = url.searchParams.get('u') || '';
        let up;
        try {
          up = new URL(u);
        } catch {
          up = null;
        }
        if (!up || !/^[a-z0-9-]+\.bilivideo\.com$/.test(up.hostname)) {
          res.writeHead(403);
          return res.end('host not allowed');
        }
        return proxyStream(req, res, {
          url: u,
          upstream_headers: {
            'User-Agent': 'Mozilla/5.0',
            Referer: 'https://live.bilibili.com',
          },
        });
      }
      const dmMatch = /^\/dm\/([\w-]+)$/.exec(url.pathname);
      if (dmMatch && req.method === 'GET') {
        const s = getSession(dmMatch[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        const list = await danmaku(s.current_item.source_locator);
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(list));
      }
      if (req.method === 'GET' && url.pathname === '/api/now') {
        const s = currentSession();
        const body = s
          ? {
              session: {
                session_id: s.session_id,
                title: s.current_item.resolved_media.title,
                media_mode: s.current_item.resolved_media.live
                  ? 'hls-live'
                  : s.hls_dir
                    ? 'hls-remux'
                    : 'file',
                media_url: s.current_item.resolved_media.live
                  ? `/livepl/${s.session_id}/index.m3u8`
                  : s.hls_dir
                    ? `/hls/${s.session_id}/index.m3u8`
                    : `/stream/${s.session_id}/0`,
              },
            }
          : { session: null };
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(body));
      }
      const streamMatch = /^\/stream\/([\w-]+)\/(\d+)$/.exec(url.pathname);
      if (streamMatch && (req.method === 'GET' || req.method === 'HEAD')) {
        const s = getSession(streamMatch[1]);
        const stream = s?.current_item.resolved_media.streams[Number(streamMatch[2])];
        if (!stream) {
          res.writeHead(404);
          return res.end('no such stream');
        }
        return await proxyStream(req, res, stream);
      }
      const hlsMatch = /^\/hls\/([\w-]+)\/([\w.]+)$/.exec(url.pathname);
      if (hlsMatch && req.method === 'GET') {
        const s = getSession(hlsMatch[1]);
        if (!s?.hls_dir) {
          res.writeHead(404);
          return res.end('no hls');
        }
        const file = join(s.hls_dir, hlsMatch[2]);
        const type = hlsMatch[2].endsWith('.m3u8')
          ? 'application/vnd.apple.mpegurl'
          : 'video/mp2t';
        res.writeHead(200, { 'content-type': type });
        return res.end(readFileSync(file));
      }
      res.writeHead(404);
      res.end('not found');
    } catch (err) {
      console.error(`[${req.method} ${url.pathname}]`, err.message);
      if (!res.headersSent) {
        res.writeHead(500, { 'content-type': 'application/json' });
        res.end(JSON.stringify({ error: String(err.message || err) }));
      } else {
        res.destroy();
      }
    }
  })
  .listen(PORT, BIND, () => console.log(`proto gateway listening on ${BASE}`));
