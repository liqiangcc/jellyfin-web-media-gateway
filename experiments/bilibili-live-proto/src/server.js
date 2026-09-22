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
  discovery,
  subtitles,
  subtitleCues,
  navigation,
  sendDanmaku,
  resolveShortLink,
  qrLoginStart,
  qrLoginPoll,
  setAuthCookie,
  getAuthCookie,
  loggedIn,
} from './sites/bilibili.js';
import { search } from './browser.js';
import {
  createSession,
  getSession,
  currentSession,
  closeSession,
  pushHistory,
  listHistory,
  markHistoryPos,
  historyPos,
} from './session.js';
import { proxyStream, remuxToHls } from './media.js';

const require = createRequire(import.meta.url);
const qrcode = require('./qrcode.cjs');
const RUNTIME_DIR = new URL('../runtime/', import.meta.url).pathname;
const AUTH_FILE = join(RUNTIME_DIR, 'auth.json');

// Multi-account store (prototype-local; real impl = Vault).
// Shape: { active: mid|null, accounts: { mid: {cookie, uname} } }
let authStore = { active: null, accounts: {} };
try {
  const saved = JSON.parse(readFileSync(AUTH_FILE, 'utf8'));
  if (saved.cookie) {
    // migrate old single-cookie shape
    authStore.accounts['?'] = { cookie: saved.cookie, uname: '(imported)' };
    authStore.active = '?';
  } else {
    authStore = saved;
  }
} catch {
  /* no saved auth */
}
const saveAuth = () =>
  writeFileSync(AUTH_FILE, JSON.stringify(authStore));
const restoreActive = () => {
  const a = authStore.accounts[authStore.active];
  setAuthCookie(a?.cookie || null);
};
restoreActive();
try {
  mkdirSync(RUNTIME_DIR, { recursive: true });
} catch {
  /* exists */
}
saveAuth();

let pendingQr = null; // {qrcode_key, expires}

const BIND = process.env.PROTO_BIND || '100.64.98.39';
const PORT = Number(process.env.PROTO_PORT || 8899);
const BASE = `http://${BIND}:${PORT}`;

const CONTROL_HTML = `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,viewport-fit=cover"><meta name="apple-mobile-web-app-capable" content="yes">
<title>B站遥控器</title>
<link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><rect width='100' height='100' rx='20' fill='%230b0e14'/><text x='50' y='68' font-size='52' text-anchor='middle' fill='%234a7dff'>▶</text></svg>">
<meta name="theme-color" content="#0b0e14">
<style>
*{box-sizing:border-box;margin:0;padding:0;-webkit-tap-highlight-color:transparent}
body{font-family:-apple-system,'PingFang SC',system-ui,sans-serif;background:#0b0e14;color:#e8eaf0;max-width:640px;margin:0 auto;padding:0 14px calc(40px + env(safe-area-inset-bottom));overscroll-behavior-y:none}
header{position:sticky;top:0;background:#0b0e14f0;backdrop-filter:blur(12px);padding:14px 0 10px;z-index:9;border-bottom:1px solid #1d2330}
h1{font-size:1.15rem;font-weight:600;display:flex;align-items:center;gap:8px}
.dot{width:9px;height:9px;border-radius:50%;background:#666}
.dot.on{background:#4ade80;box-shadow:0 0 8px #4ade8080}
.acct{font-size:.78rem;color:#8b93a7;margin-left:auto;display:flex;gap:10px;align-items:center}
.acct a{color:#6ea8fe;text-decoration:none}
form{display:flex;gap:8px;margin:10px 0}
input{flex:1;min-width:0;padding:11px 14px;font-size:16px;border-radius:12px;border:1px solid #2a3242;background:#141926;color:#e8eaf0;outline:none}
input:focus{border-color:#4a7dff}
button{padding:11px 16px;font-size:.9rem;border-radius:12px;border:1px solid #2a3242;background:#1c2333;color:#e8eaf0;cursor:pointer;white-space:nowrap}
button:active{background:#2a3542;transform:scale(.97)}
button.primary{background:#4a7dff;border-color:#4a7dff;color:#fff;font-weight:600}
button.on{background:#4a7dff;border-color:#4a7dff;color:#fff}
.chips{display:flex;gap:8px;flex-wrap:wrap;margin:12px 0}
.chip{font-size:.82rem;padding:7px 13px;border-radius:999px}
.list{display:flex;flex-direction:column;gap:8px;margin:12px 0}
.hit{display:flex;gap:12px;padding:9px;border-radius:14px;background:#141926;border:1px solid #1d2330;cursor:pointer;align-items:center}
.hit:active{background:#1c2333}
.hit .cov{position:relative;flex-shrink:0}
.hit .cov::after{content:'▶';position:absolute;inset:0;display:grid;place-items:center;font-size:1.1rem;color:#fff;text-shadow:0 1px 6px #000;opacity:.85}
.hit img{width:112px;height:66px;object-fit:cover;border-radius:9px;background:#000;display:block}
.chip.on{background:#4a7dff;border-color:#4a7dff;color:#fff}
.badge{display:inline-block;font-size:.68rem;padding:2px 7px;border-radius:6px;background:#253050;color:#9db4ff;vertical-align:middle;margin-left:6px}
.badge.live{background:#3d1f2a;color:#ff8fa3}
.badge.prev{background:#3a2d12;color:#ffd479}
.section{font-size:.75rem;color:#8b93a7;letter-spacing:.06em;margin:14px 2px 2px}
@keyframes shimmer{from{opacity:.4}to{opacity:.9}}
.sk{height:76px;border-radius:14px;background:#141926;border:1px solid #1d2330;animation:shimmer 1s infinite alternate}
.hit .t{font-size:.88rem;font-weight:500;line-height:1.35;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}
.hit .m{font-size:.72rem;color:#8b93a7;margin-top:4px}
.hit.folder img{display:none}
#sess{background:#141926;border:1px solid #253050;border-radius:16px;padding:14px;margin:14px 0;position:sticky;top:58px;z-index:8;box-shadow:0 8px 24px #0008}
#sess h3{font-size:.95rem;font-weight:600;margin-bottom:10px;line-height:1.4}
#sess .row{display:flex;align-items:flex-start;gap:8px;margin:9px 0;flex-wrap:wrap}
#sess .lbl{font-size:.78rem;color:#8b93a7;min-width:38px;padding-top:7px}
#sess .opts{display:flex;gap:6px;flex-wrap:wrap}
#sess .opts button{padding:7px 11px;font-size:.78rem;border-radius:9px}
#out{font-size:.78rem;color:#8b93a7;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;background:#0d1119;border-radius:9px;padding:8px 12px;margin:4px 0;transition:color .3s}
#out.err{color:#ff8fa3}
#out.ok{color:#4ade80}
.hit.playing{border-color:#4a7dff;background:#182136}
.hit.playing .t::before{content:'▶ ';color:#4a7dff}
.prog{height:16px;border-radius:8px;background:#253050;overflow:hidden;margin:10px 0 4px;cursor:pointer;position:relative}
#pfill{height:100%;width:0;background:#4a7dff;border-radius:8px;transition:width .8s linear;pointer-events:none}
.ptimes{display:flex;justify-content:space-between;font-size:.72rem;color:#8b93a7;margin-bottom:8px}
.transport{display:flex;gap:8px;justify-content:center;margin:6px 0 10px}
.transport button{min-width:52px;font-size:.95rem}
.muted{color:#566;font-size:.8rem;padding:8px 2px}
</style>
<header><h1><span id="dot" class="dot"></span> B站遥控器 <span class="acct"><span id="who"></span><a href="/qr">登录/换号</a><a href="#" id="logout" style="display:none">退出</a></span></h1></header>
<div id="sess" style="display:none">
  <h3 id="stitle"></h3>
  <div class="prog"><div id="pfill"></div></div>
  <div class="ptimes"><span id="pcur">0:00</span><span id="pdur">0:00</span></div>
  <div class="transport">
    <button id="tprev">⏮</button>
    <button id="tback">−15s</button>
    <button id="tpp" class="primary">⏸</button>
    <button id="tfwd">+15s</button>
    <button id="tnext">⏭</button>
  </div>
  <div class="row"><span class="lbl">清晰度</span><div class="opts" id="quals"></div></div>
  <div class="row"><span class="lbl">选集</span><div class="opts" id="pages"></div></div>
  <div class="row"><span class="lbl">字幕</span><div class="opts" id="subs"></div></div>
  <div class="row"><span class="lbl">倍速</span><div class="opts" id="rates"></div></div>
  <div class="row"><span class="lbl">音量</span><div class="opts"><button data-vd="-0.1">−</button><button id="vmute">🔇</button><button data-vd="0.1">＋</button><span id="vvol" class="lbl" style="padding-top:7px">100%</span></div></div>
  <div class="row"><span class="lbl">弹幕</span><div class="opts"><button id="dmt" class="on">开</button><button data-ds="0.8">小</button><button data-ds="1">中</button><button data-ds="1.3">大</button></div>
    <span class="opts" style="margin-left:auto"><button id="tstop" style="color:#ff8fa3">■ 停止</button></span></div>
  <div class="row" id="dmrow"><input id="dminput" type="text" maxlength="100" placeholder="发条弹幕…" style="flex:1"><button class="primary" id="dmsend">发送</button></div>
  <div class="section" id="relsec" style="display:none">相关推荐</div>
  <div id="rel" class="list"></div>
</div>
<pre id="out">就绪</pre>
<form id="f"><input id="src" type="text" placeholder="粘贴 BV 链接 / 番剧 ep / 直播房号…"><button class="primary">播放</button></form>
<form id="sf"><input id="q" type="search" placeholder="搜索 bilibili…"><button>搜索</button></form>
<div class="chips">
  <button class="chip" id="fav">⭐ 收藏夹</button>
  <button class="chip" id="hot">🔥 热门</button>
  <button class="chip" id="feed">✨ 推荐</button>
  <button class="chip" id="hist">🕘 最近</button>
</div>
<div class="section" id="sect" style="display:none"></div>
<div id="results" class="list"></div>
<div id="favlist" class="list"></div>
<p class="muted"><a href="/display" style="color:#6ea8fe">打开播放页 →</a></p>
<script type="module">
const out=document.getElementById('out'),results=document.getElementById('results');
let curSrc=null,curCover=null;
let toastT=null;
const toast=(t,cls)=>{out.textContent=t;out.className=cls||'';clearTimeout(toastT);if(cls==='ok')toastT=setTimeout(()=>{out.textContent='就绪';out.className=''},4000)};
async function playSource(src,qn,cover){
  curSrc=src;if(cover)curCover=cover;
  toast('解析中…');
  try{
    const r=await fetch('/api/play',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({source:src,cover:curCover,...(qn?{qn}:{})})});
    const j=await r.json();
    if(!r.ok){toast((j.error||'ERR '+r.status),'err');return}
    toast('已投放到播放页','ok');
    markPlaying();
    renderSession(j);
  }catch(err){toast(String(err),'err')}
}
function markPlaying(){
  const bv=/video\\/(BV\\w+)/.exec(curSrc||'')?.[1];
  document.querySelectorAll('.hit').forEach(h=>h.classList.toggle('playing',h.dataset.bv===bv));
}
const skeleton=n=>'<div class="sk"></div>'.repeat(n);
const fmtDur=d=>typeof d==='number'?fmt(d):(d||'');
const statFmt=n=>n>=10000?(n/10000).toFixed(1)+'万':(n||'');
const metaOf=x=>[x.bvid,fmtDur(x.duration),x.owner,statFmt(x.stat)].filter(Boolean).join(' · ');
async function renderSession(j){
  const sess=document.getElementById('sess');
  sess.style.display='';
  const mode=j.media_mode==='hls-live'?'<span class="badge live">直播</span>':j.is_preview?'<span class="badge prev">预览</span>':'<span class="badge">'+j.media_mode+'</span>';
  document.getElementById('stitle').innerHTML=j.title+mode+(curSrc?' <a href="'+curSrc+'" target="_blank" style="color:#6ea8fe;font-size:.75rem;text-decoration:none;white-space:nowrap">🔗 原视频</a>':'');
  // 会话卡片封面（从列表点进来时带上）
  let cov=document.getElementById('sesscov');
  if(curCover){if(!cov){cov=document.createElement('img');cov.id='sesscov';cov.style.cssText='width:100%;border-radius:10px;margin-bottom:8px;display:block';sess.insertBefore(cov,document.getElementById('stitle'))}cov.src=curCover}
  else if(cov)cov.remove();
  // Restore curSrc from locator when panel is rebuilt on page load.
  if(!curSrc&&j.locator?.opaque_payload?.bvid)
    curSrc='https://www.bilibili.com/video/'+j.locator.opaque_payload.bvid+(j.locator.opaque_payload.page?'?p='+j.locator.opaque_payload.page:'');
  // 清晰度：re-play same source at chosen qn
  document.getElementById('quals').innerHTML=(j.quality_options||[]).map(q=>'<button data-qn="'+q.qn+'"'+(q.qn===j.quality?' class="on"':'')+'>'+q.label+'</button>').join(' ')||'（仅一档）';
  document.querySelectorAll('#quals button').forEach(b=>b.onclick=()=>playSource(curSrc,b.dataset.qn));
  // 选集：BV 分 P 列表
  const nav=await fetch('/api/session/'+j.session_id+'/nav').then(r=>r.json()).catch(()=>null);
  if(nav&&nav.queue&&nav.queue.length>1){
    const bv=nav.queue[0].opaque_payload.bvid;
    document.getElementById('pages').innerHTML=nav.queue.map((l,i)=>'<button data-p="'+l.opaque_payload.page+'"'+(i===nav.current_index?' class="on"':'')+'>P'+l.opaque_payload.page+'</button>').join(' ');
    document.querySelectorAll('#pages button').forEach(b=>b.onclick=()=>playSource('https://www.bilibili.com/video/'+bv+'?p='+b.dataset.p));
  }else document.getElementById('pages').textContent='（单集）';
  // 字幕轨选择（当前选中高亮）
  const subs=await fetch('/api/session/'+j.session_id+'/subs').then(r=>r.json()).catch(()=>[]);
  document.getElementById('subs').innerHTML=['<button data-u=""'+(!j.subtitle_url?' class="on"':'')+'>关闭</button>'].concat((subs||[]).map(x=>'<button data-u="'+x.url+'"'+(x.url===j.subtitle_url?' class="on"':'')+'>'+x.label+'</button>')).join(' ');
  document.querySelectorAll('#subs button').forEach(b=>b.onclick=()=>fetch('/api/session/'+j.session_id+'/subtitle',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({url:b.dataset.u||null})}));
  // 倍速（当前倍速高亮）
  const curRate=j.playback_rate||1;
  document.getElementById('rates').innerHTML=[0.5,1,1.25,1.5,2].map(r=>'<button data-r="'+r+'"'+(r===curRate?' class="on"':'')+'>'+r+'x</button>').join(' ');
  document.querySelectorAll('#rates button').forEach(b=>b.onclick=()=>fetch('/api/session/'+j.session_id+'/rate',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({rate:Number(b.dataset.r)})}));
  // 播放控制：命令经 session 下发给 display
  const cmd=(op,pos)=>fetch('/api/session/'+j.session_id+'/cmd',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({op,pos})});
  const st=()=>fetch('/api/now').then(r=>r.json()).then(x=>x.session);
  document.getElementById('tpp').onclick=async()=>{const x=await st();cmd(x&&x.paused?'play':'pause');setTimeout(pollPos,600)};
  document.getElementById('tback').onclick=async()=>{const x=await st();if(x)cmd('seek',(x.pos||0)-15)};
  document.getElementById('tfwd').onclick=async()=>{const x=await st();if(x)cmd('seek',(x.pos||0)+15)};
  document.getElementById('tprev').onclick=async()=>{const n=await fetch('/api/session/'+j.session_id+'/nav').then(r=>r.json()).catch(()=>null);if(n&&n.previous)playSource('https://www.bilibili.com/video/'+n.previous.opaque_payload.bvid+'?p='+n.previous.opaque_payload.page)};
  document.getElementById('tnext').onclick=async()=>{const n=await fetch('/api/session/'+j.session_id+'/nav').then(r=>r.json()).catch(()=>null);if(n&&n.next)playSource('https://www.bilibili.com/video/'+n.next.opaque_payload.bvid+'?p='+n.next.opaque_payload.page)};
  // 进度条点击 seek
  document.querySelector('.prog').onclick=async e=>{
    const x=await st();if(!x||!x.dur)return;
    const r=e.currentTarget.getBoundingClientRect();
    cmd('seek',Math.floor((e.clientX-r.left)/r.width*x.dur));
    setTimeout(pollPos,800);
  };
  // 发弹幕
  const sendDm=async()=>{
    const t=document.getElementById('dminput').value.trim();if(!t)return;
    const r=await fetch('/api/session/'+j.session_id+'/senddm',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({text:t})});
    const x=await r.json().catch(()=>({}));
    toast(r.ok?'弹幕已发送':'发送失败: '+(x.error||r.status),r.ok?'ok':'err');
    if(r.ok)document.getElementById('dminput').value='';
  };
  document.getElementById('dmsend').onclick=sendDm;
  document.getElementById('dminput').addEventListener('keydown',e=>{if(e.key==='Enter')sendDm()});
  // 弹幕开关
  document.getElementById('dmt').onclick=async()=>{
    const x=await st();const on=x?x.dm_on===false:true;
    fetch('/api/session/'+j.session_id+'/dm',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({on})});
    document.getElementById('dmt').classList.toggle('on',on);
    document.getElementById('dmt').textContent=on?'开':'关';
  };
  // 弹幕字号
  document.querySelectorAll('[data-ds]').forEach(b=>{
    b.classList.toggle('on',Number(b.dataset.ds)===(j.dm_scale||1));
    b.onclick=()=>{fetch('/api/session/'+j.session_id+'/dmscale',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({scale:Number(b.dataset.ds)})});document.querySelectorAll('[data-ds]').forEach(x=>x.classList.toggle('on',x===b))};
  });
  // 音量
  document.querySelectorAll('[data-vd]').forEach(b=>b.onclick=()=>fetch('/api/session/'+j.session_id+'/vol',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({delta:Number(b.dataset.vd)})}).then(r=>r.json()).then(y=>{if(typeof y.volume==='number')document.getElementById('vvol').textContent=Math.round(y.volume*100)+'%'}));
  document.getElementById('vvol').textContent=Math.round((j.volume??1)*100)+'%';
  document.getElementById('vmute').onclick=async()=>{const x=await st();fetch('/api/session/'+j.session_id+'/vol',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({vol:x&&x.volume>0?0:1})}).then(r=>r.json()).then(y=>{if(typeof y.volume==='number'){document.getElementById('vvol').textContent=Math.round(y.volume*100)+'%';document.getElementById('vmute').textContent=y.volume>0?'🔊':'🔇'}})};
  document.getElementById('vmute').textContent=(j.volume??1)>0?'🔊':'🔇';
  // 停止播放：关 session，display 回 idle
  document.getElementById('tstop').onclick=async()=>{
    await fetch('/api/session/'+j.session_id+'/stop',{method:'POST'});
    document.getElementById('sess').style.display='none';
    document.querySelectorAll('.hit.playing').forEach(h=>h.classList.remove('playing'));
    curSrc=null;
    toast('已停止');
  };
  // 弹幕开关状态同步（恢复会话时也要对）
  const dmt=document.getElementById('dmt');
  const dmOn=j.dm_on!==false;
  dmt.classList.toggle('on',dmOn);dmt.textContent=dmOn?'开':'关';
  // 直播：隐藏进度条和 seek 按钮 + 弹幕输入
  const live=j.media_mode==='hls-live';
  document.querySelector('.prog').style.display=live?'none':'';
  document.querySelector('.ptimes').style.display=live?'none':'';
  document.getElementById('dmrow').style.display=live?'none':'';
  document.getElementById('tback').style.display=live?'none':'';
  document.getElementById('tfwd').style.display=live?'none':'';
  document.getElementById('tprev').style.display=live?'none':'';
  document.getElementById('tnext').style.display=live?'none':'';
  // 相关推荐（播完接着看的入口）
  const bv=j.locator?.opaque_payload?.bvid;
  const rel=document.getElementById('rel'),relsec=document.getElementById('relsec');
  if(bv&&!live){
    rel.innerHTML=skeleton(3);relsec.style.display='';
    const items=await fetch('/api/discover?kind=related&bvid='+bv).then(r=>r.json()).catch(()=>[]);
    rel.innerHTML=(items||[]).slice(0,8).map(x=>'<div class="hit" data-bv="'+x.bvid+'"><div class="cov"><img src="'+x.cover+'" loading="lazy"></div><div><div class="t">'+x.title+'</div><div class="m">'+metaOf(x)+'</div></div></div>').join('')||'';
    if(!rel.innerHTML)relsec.style.display='none';
    rel.querySelectorAll('.hit').forEach(i=>i.onclick=()=>playSource('https://www.bilibili.com/video/'+i.dataset.bv,0,i.querySelector('img')?.src));markPlaying();
  }else{rel.innerHTML='';relsec.style.display='none'}
}
// 页面加载时恢复正在播放的会话面板
(async()=>{
  const x=await fetch('/api/now').then(r=>r.json()).then(y=>y.session).catch(()=>null);
  if(x){renderSession(x);pollPos();
    const bv=x.locator?.opaque_payload?.bvid;
    if(bv)setTimeout(()=>document.querySelectorAll('.hit').forEach(h=>h.classList.toggle('playing',h.dataset.bv===bv)),0);
  }
})();
const fmt=t=>{if(!isFinite(t))return'0:00';const m=Math.floor(t/60),s=Math.floor(t%60);return m+':'+String(s).padStart(2,'0')};
async function pollPos(){
  const x=await fetch('/api/now').then(r=>r.json()).then(y=>y.session).catch(()=>null);
  if(!x){
    // Session closed elsewhere — hide the panel and clear highlights.
    document.getElementById('sess').style.display='none';
    document.querySelectorAll('.hit.playing').forEach(h=>h.classList.remove('playing'));
    lastPos=null;curSrc=null;
    return;
  }
  document.getElementById('pcur').textContent=fmt(x.pos);
  document.getElementById('pdur').textContent=fmt(x.dur);
  document.getElementById('pfill').style.width=(x.dur?Math.min(100,x.pos/x.dur*100):0)+'%';
  document.getElementById('tpp').textContent=x.paused?'▶':'⏸';
  document.getElementById('vvol').textContent=Math.round((x.volume??1)*100)+'%';
  document.getElementById('vmute').textContent=(x.volume??1)>0?'🔊':'🔇';
  lastPos={pos:x.pos,dur:x.dur,paused:x.paused,at:performance.now()};
}
let lastPos=null;
// Smooth the progress bar between 2s polls when playing.
setInterval(()=>{
  if(!lastPos||lastPos.paused||!lastPos.dur)return;
  const est=lastPos.pos+(performance.now()-lastPos.at)/1000;
  if(est>lastPos.dur)return;
  document.getElementById('pfill').style.width=Math.min(100,est/lastPos.dur*100)+'%';
  document.getElementById('pcur').textContent=fmt(est);
},500);
setInterval(()=>{if(document.getElementById('sess').style.display!=='none')pollPos()},2000);
document.getElementById('f').onsubmit=e=>{e.preventDefault();playSource(document.getElementById('src').value)};
document.getElementById('sf').onsubmit=async e=>{
  e.preventDefault();
  results.innerHTML=skeleton(4);
  favlistClear();
  try{
    const r=await fetch('/api/search?q='+encodeURIComponent(document.getElementById('q').value));
    const list=await r.json();
    if(!r.ok)throw new Error(JSON.stringify(list));
    document.getElementById('sect').textContent='搜索结果';
    document.getElementById('sect').style.display='';
    results.innerHTML=list.map(x=>'<div class="hit" data-bv="'+x.bvid+'"><div class="cov"><img src="'+x.cover+'" loading="lazy"></div><div><div class="t">'+x.title+'</div><div class="m">'+metaOf(x)+'</div></div></div>').join('')||'<div class="muted">无结果</div>';
    results.querySelectorAll('.hit').forEach(el=>el.onclick=()=>playSource('https://www.bilibili.com/video/'+el.dataset.bv,0,el.querySelector('img')?.src));markPlaying();
  }catch(err){results.textContent='ERR '+err}
};
const favlist=document.getElementById('favlist'),sect=document.getElementById('sect');
const favlistClear=()=>{favlist.innerHTML=''};
const clearSearch=()=>{results.innerHTML='';document.getElementById('sect').style.display='none'};
const markChip=id=>{document.querySelectorAll('.chip').forEach(c=>c.classList.remove('on'));document.getElementById(id).classList.add('on');clearSearch()};
document.getElementById('fav').onclick=async()=>{
  markChip('fav');sect.style.display='';sect.textContent='收藏夹';favlist.innerHTML=skeleton(3);
  try{
    const r=await fetch('/api/favorites');
    const folders=await r.json();
    if(!r.ok)throw new Error(folders.error||r.status);
    favlist.innerHTML=folders.map(f=>'<div class="hit folder" data-fid="'+f.id+'"><div><div class="t">📁 '+f.title+'</div><div class="m">'+f.count+' 个内容</div></div></div>').join('')||'<div class="muted">无收藏夹</div>';
    favlist.querySelectorAll('.hit').forEach(el=>el.onclick=async()=>{
      sect.textContent='📁 '+el.querySelector('.t').textContent.replace(/^📁 /,'');
      favlist.innerHTML=skeleton(3);
      const items=await fetch('/api/favorites?folder='+el.dataset.fid).then(r=>r.json());
      const back='<div class="hit folder" id="favback"><div><div class="t">← 返回收藏夹</div></div></div>';
      favlist.innerHTML=back+(items.map(x=>'<div class="hit" data-bv="'+x.bvid+'"><div class="cov"><img src="'+x.cover+'" loading="lazy"></div><div><div class="t">'+x.title+'</div><div class="m">'+metaOf(x)+'</div></div></div>').join('')||'<div class="muted">空收藏夹</div>');
      document.getElementById('favback').onclick=()=>document.getElementById('fav').click();
      favlist.querySelectorAll('.hit[data-bv]').forEach(i=>i.onclick=()=>playSource('https://www.bilibili.com/video/'+i.dataset.bv,0,i.querySelector('img')?.src));markPlaying();
    });
  }catch(err){favlist.textContent='ERR '+err.message}
};
const showVideos=(items)=>{
  favlist.innerHTML=items.map(x=>'<div class="hit" data-bv="'+x.bvid+'"><div class="cov"><img src="'+x.cover+'" loading="lazy"></div><div><div class="t">'+x.title+'</div><div class="m">'+metaOf(x)+'</div></div></div>').join('')||'<div class="muted">无内容</div>';
  favlist.querySelectorAll('.hit').forEach(i=>i.onclick=()=>playSource('https://www.bilibili.com/video/'+i.dataset.bv,0,i.querySelector('img')?.src));markPlaying();
};
document.getElementById('hot').onclick=async()=>{
  markChip('hot');sect.style.display='';sect.textContent='热门视频';favlist.innerHTML=skeleton(5);
  showVideos(await fetch('/api/discover?kind=popular').then(r=>r.json()));
};
document.getElementById('feed').onclick=async()=>{
  markChip('feed');sect.style.display='';sect.textContent='为你推荐';favlist.innerHTML=skeleton(5);
  showVideos(await fetch('/api/discover?kind=rcmd').then(r=>r.json()));
};
document.getElementById('hist').onclick=async()=>{
  markChip('hist');sect.style.display='';sect.textContent='最近播放';favlist.innerHTML=skeleton(3);
  const items=await fetch('/api/history').then(r=>r.json()).catch(()=>[]);
  favlist.innerHTML=items.map(x=>'<div class="hit" data-src="'+encodeURIComponent(x.source)+'">'+(x.cover?'<div class="cov"><img src="'+x.cover+'" loading="lazy"></div>':'')+'<div><div class="t">'+x.title+'</div><div class="m">'+new Date(x.at).toLocaleTimeString()+(x.pos?' · 续播 '+fmt(x.pos):'')+'</div></div></div>').join('')||'<div class="muted">暂无记录</div>';
  favlist.querySelectorAll('.hit').forEach(i=>i.onclick=()=>playSource(decodeURIComponent(i.dataset.src),0,i.querySelector('img')?.src));
};
const auth=await fetch('/api/auth').then(r=>r.json()).catch(()=>({}));
if(auth.logged_in){
  document.getElementById('who').textContent=auth.uname||auth.mid;
  document.getElementById('dot').classList.add('on');
  document.getElementById('logout').style.display='';
}
document.getElementById('logout').onclick=async e=>{
  e.preventDefault();
  await fetch('/api/logout',{method:'POST'});
  location.reload();
};
// mark active chip selection for quality/page/sub buttons
document.addEventListener('click',e=>{
  const b=e.target.closest('#sess .opts button');if(!b)return;
  b.parentElement.querySelectorAll('button').forEach(x=>x.classList.remove('on'));
  b.classList.add('on');
});
</script>`;

const DISPLAY_HTML = `<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width,viewport-fit=cover">
<link rel="icon" href="data:image/svg+xml,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'><rect width='100' height='100' rx='20' fill='%23000'/><text x='50' y='68' font-size='52' text-anchor='middle' fill='%234a7dff'>▶</text></svg>">
<title>display</title>
<style>
html,body{margin:0;height:100%;background:#000;overflow:hidden}
video{width:100vw;height:100vh;object-fit:contain;display:block}
#s{position:fixed;top:0;left:0;right:0;color:#fff;font:600 15px/1.5 -apple-system,'PingFang SC',system-ui;background:linear-gradient(#000b,#0000);padding:14px 18px 26px;pointer-events:none;transition:opacity .5s}
#s.hide{opacity:0}
#dm{position:fixed;inset:0;pointer-events:none;overflow:hidden}
.dm-item{position:absolute;white-space:nowrap;font-weight:600;color:#fff;will-change:transform;pointer-events:none;
  text-shadow:-1px -1px 0 #000,1px -1px 0 #000,-1px 1px 0 #000,1px 1px 0 #000,0 0 6px #0006}
.dm-fixed{position:absolute;white-space:nowrap;font-weight:600;left:50%;transform:translateX(-50%);pointer-events:none;
  text-shadow:-1px -1px 0 #000,1px -1px 0 #000,-1px 1px 0 #000,1px 1px 0 #000,0 0 6px #0006;
  animation:dmfade .4s ease-in,dmout .4s ease-out forwards 4s}
@keyframes dmfade{from{opacity:0}to{opacity:1}}
@keyframes dmout{to{opacity:0}}
#sub{position:fixed;bottom:7%;left:0;right:0;text-align:center;pointer-events:none;color:#fff;font:500 4vmin/1.5 -apple-system,'PingFang SC',system-ui;
  text-shadow:-1px -1px 2px #000,1px -1px 2px #000,-1px 1px 2px #000,1px 1px 2px #000,0 2px 8px #000}
#buf{position:fixed;inset:0;display:none;place-items:center;pointer-events:none}
#buf.on{display:grid}
#buf .ring{width:52px;height:52px;border-radius:50%;border:3px solid #fff3;border-top-color:#fff;animation:spin .8s linear infinite}
@keyframes spin{to{transform:rotate(360deg)}}
body.idle{cursor:none}
body.idle video::-webkit-media-controls-panel{opacity:0}
#idle{position:fixed;inset:0;display:grid;place-items:center;background:#000;z-index:5;transition:opacity .4s}
#idle.hide{opacity:0;pointer-events:none}
.idle-card{text-align:center;color:#8b93a7}
.idle-title{font-size:5vmin;font-weight:600;color:#e8eaf0;margin-bottom:12px}
.idle-url{font-size:3vmin;color:#4a7dff}
</style>
<video id="v" controls playsinline></video>
<div id="idle"><div class="idle-card"><div class="idle-title">等待控制端点播</div><div class="idle-url" id="iurl"></div></div></div>
<div id="s">等待控制端点播…</div>
<div id="dm"></div>
<div id="sub"></div>
<div id="buf"><div class="ring"></div></div>
<script src="/hls.min.js"></script>
<script>
const v=document.getElementById('v'),s=document.getElementById('s'),dm=document.getElementById('dm'),subEl=document.getElementById('sub'),idleEl=document.getElementById('idle');
document.getElementById('iurl').textContent=location.origin+'/control';
let cur=null,hls=null,pool=[],pidx=0,cues=[],cuesUrl=null,lastCmd=0,sid=null;
let statusTimer=null;
function flash(t){s.textContent=t;s.classList.remove('hide');clearTimeout(statusTimer);statusTimer=setTimeout(()=>s.classList.add('hide'),3500)}
// Hide cursor/native controls after 3s idle (display mode)
let idleT=null;
document.addEventListener('mousemove',()=>{document.body.classList.remove('idle');clearTimeout(idleT);idleT=setTimeout(()=>document.body.classList.add('idle'),3000)});
idleT=setTimeout(()=>document.body.classList.add('idle'),3000);
// --- danmaku engine: scroll lanes + fixed top/bottom ---
let dmScale=1;
let FONT=Math.max(18,Math.floor(innerHeight*0.052));
let LANE_H=Math.floor(FONT*1.45),TOP_N=Math.floor(innerHeight*0.45/LANE_H),BOT_N=Math.floor(innerHeight*0.25/LANE_H);
function setDmScale(k){
  dmScale=k;
  FONT=Math.max(14,Math.floor(innerHeight*0.052*k));
  LANE_H=Math.floor(FONT*1.45);
  TOP_N=Math.floor(innerHeight*0.45/LANE_H);
  BOT_N=Math.floor(innerHeight*0.25/LANE_H);
}
const laneFree=[],topFree=[],botFree=[];
const FLY_MS=7500,FIX_MS=4500;
function colorOf(d){return '#'+d.color.toString(16).padStart(6,'0')}
function freeLane(arr,n){const now=performance.now();for(let i=0;i<n;i++)if(!arr[i]||arr[i]<now)return i;return -1}
function spawn(d){
  // mode 4 = bottom-fixed, 5 = top-fixed, else scroll
  if(d.mode===4||d.mode===5){
    const arr=d.mode===5?topFree:botFree,n=d.mode===5?TOP_N:BOT_N;
    const lane=freeLane(arr,n);if(lane<0)return;
    const el=document.createElement('div');
    el.className='dm-fixed';el.textContent=d.text;
    el.style.cssText+='font-size:'+FONT+'px;color:'+colorOf(d)+';'+(d.mode===5?'top:'+lane*LANE_H+'px':'bottom:'+(lane*LANE_H+Math.floor(innerHeight*0.12))+'px');
    dm.appendChild(el);
    arr[lane]=performance.now()+FIX_MS;
    setTimeout(()=>el.remove(),FIX_MS+400);
    return;
  }
  const lane=freeLane(laneFree,Math.floor(innerHeight*0.85/LANE_H));if(lane<0)return;
  const el=document.createElement('div');
  el.className='dm-item';el.textContent=d.text;
  el.style.cssText+='font-size:'+FONT+'px;color:'+colorOf(d)+';top:'+(lane*LANE_H)+'px;left:100%';
  dm.appendChild(el);
  const w=el.offsetWidth+innerWidth;
  const a=el.animate([{transform:'translateX(0)'},{transform:'translateX(-'+w+'px)'}],{duration:FLY_MS,easing:'linear'});
  a.onfinish=()=>el.remove();
  if(v.paused)a.pause();
  laneFree[lane]=performance.now()+FLY_MS*(el.offsetWidth/w)+200;
}
setInterval(()=>{
  if(!pool.length)return;
  const t=v.currentTime;
  if(pidx>0&&(pool[pidx-1]&&pool[pidx-1].t>t+2)){dm.innerHTML='';laneFree.length=0;pidx=pool.findIndex(d=>d.t>t);if(pidx<0)pidx=pool.length}
  while(pidx<pool.length&&pool[pidx].t<=t){spawn(pool[pidx]);pidx++}
},200);
// Subtitle overlay: cues synced to currentTime.
setInterval(()=>{
  if(!cues.length){subEl.textContent='';return}
  const t=v.currentTime;
  const c=cues.find(x=>t>=x.from&&t<x.to);
  subEl.textContent=c?c.text:'';
},200);
v.addEventListener('pause',()=>{v.getAnimations?0:0;dm.querySelectorAll('div').forEach(e=>e.getAnimations().forEach(a=>a.pause()))});
v.addEventListener('play',()=>{dm.querySelectorAll('div').forEach(e=>e.getAnimations().forEach(a=>a.play()))});
const bufEl=document.getElementById('buf');
v.addEventListener('waiting',()=>bufEl.classList.add('on'));
v.addEventListener('playing',()=>bufEl.classList.remove('on'));
v.addEventListener('canplay',()=>bufEl.classList.remove('on'));
// 播放错误自动重试一次
let errRetried=false;
v.addEventListener('error',()=>{
  if(!v.src||errRetried)return;
  errRetried=true;flash('播放出错，重试中…');
  setTimeout(()=>{v.load();v.play().catch(()=>{})},1500);
});
v.addEventListener('playing',()=>{errRetried=false});
// 播完自动连播下一集（BV 分 P 场景）
v.addEventListener('ended',async()=>{
  if(!sid)return;
  const nav=await fetch('/api/session/'+sid+'/nav').then(r=>r.json()).catch(()=>null);
  if(nav&&nav.next){
    const nx=nav.next.opaque_payload;
    fetch('/api/play',{method:'POST',headers:{'content-type':'application/json'},
      body:JSON.stringify({source:'https://www.bilibili.com/video/'+nx.bvid+'?p='+nx.page})});
  }
});
setInterval(async()=>{
  try{
    const j=await (await fetch('/api/now')).json();
    if(!j.session){
      idleEl.classList.remove('hide');
      if(cur){cur=null;sid=null;if(hls){hls.destroy();hls=null}v.pause();v.removeAttribute('src');v.load();dm.innerHTML='';pool=[];cues=[];subEl.textContent=''}
      return;
    }
    idleEl.classList.add('hide');
    if(cur!==j.session.session_id+'|'+j.session.media_url){
      cur=j.session.session_id+'|'+j.session.media_url;
      if(hls){hls.destroy();hls=null}
      pool=[];pidx=0;dm.innerHTML='';laneFree.length=0;topFree.length=0;botFree.length=0;
      fetch('/dm/'+j.session.session_id).then(r=>r.json()).then(l=>{pool=l;pidx=0}).catch(()=>{});
      // Load subtitle cues if control picked a track.
      cuesUrl=j.session.subtitle_url;
      if(cuesUrl){
        fetch('/api/session/'+j.session.session_id+'/subcues?u='+encodeURIComponent(cuesUrl))
          .then(r=>r.json()).then(l=>{cues=l}).catch(()=>{});
      } else cues=[];
      const url=j.session.media_url;
      if(url.endsWith('.m3u8')&&window.Hls&&Hls.isSupported()){
        hls=new Hls();hls.loadSource(url);hls.attachMedia(v);
      }else{
        v.src=url;v.load();
      }
      flash(j.session.title+' · '+j.session.media_mode);
      v.play().catch(()=>{s.classList.remove('hide');s.textContent=j.session.title+' — 点按播放'});
    }
    if(j.session.playback_rate&&v.playbackRate!==j.session.playback_rate)v.playbackRate=j.session.playback_rate;
    if(typeof j.session.volume==='number'&&Math.abs(v.volume-j.session.volume)>0.02)v.volume=j.session.volume;
    dm.style.display=j.session.dm_on===false?'none':'';
    if((j.session.dm_scale||1)!==dmScale)setDmScale(j.session.dm_scale||1);
    sid=j.session.session_id;
    // Apply transport commands (guarded by seq).
    if(j.session.cmd&&j.session.cmd.seq!==lastCmd){
      lastCmd=j.session.cmd.seq;
      const c=j.session.cmd;
      if(c.op==='pause')v.pause();
      else if(c.op==='play')v.play().catch(()=>{});
      else if(c.op==='seek'&&typeof c.pos==='number')v.currentTime=Math.max(0,c.pos);
    }
    // Subtitle track change without session change.
    if(j.session.subtitle_url!==cuesUrl){
      cuesUrl=j.session.subtitle_url;
      if(cuesUrl){
        fetch('/api/session/'+j.session.session_id+'/subcues?u='+encodeURIComponent(cuesUrl))
          .then(r=>r.json()).then(l=>{cues=l}).catch(()=>{});
      } else cues=[];
    }
  }catch(e){s.textContent='display reconnecting…'}
},1500);
// Report position back to the gateway every 2s for the control UI.
setInterval(()=>{
  if(!sid)return;
  fetch('/api/session/'+sid+'/pos',{method:'POST',headers:{'content-type':'application/json'},
    body:JSON.stringify({pos:v.currentTime||0,dur:v.duration||0,paused:v.paused})}).catch(()=>{});
},2000);
// Re-fetch the danmaku pool every 60s so newly-posted ones appear.
setInterval(()=>{
  if(!sid||!pool.length)return;
  fetch('/dm/'+sid).then(r=>r.json()).then(l=>{
    const t=v.currentTime;
    pool=l;pidx=pool.findIndex(d=>d.t>t);if(pidx<0)pidx=pool.length;
  }).catch(()=>{});
},60000);
</script>`;

async function play(body) {
  let source = body.source || '';
  // b23.tv share links: expand to canonical before recognize.
  if (/b23\.tv\//i.test(source)) {
    try {
      source = await resolveShortLink(
        source.startsWith('http') ? source : 'https://' + source,
      );
    } catch {
      return [400, { error: 'SHORT_LINK_FAILED' }];
    }
  }
  const rec = recognize(source);
  if (!rec.matched || rec.short) return [400, { error: 'SOURCE_NOT_RECOGNIZED' }];
  const media = await resolve(rec.locator, {
    prefer: {
      ...(body.force_dash ? { mode: 'dash' } : {}),
      ...(body.qn ? { qn: Number(body.qn) } : {}),
    },
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

  const expose = (session, media, mode, media_url) => ({
    session_id: session.session_id,
    title: media.title,
    media_mode: mode,
    media_url,
    quality: media.quality,
    quality_options: media.quality_options || [],
    is_preview: !!media.is_preview,
    streams: media.streams.map((s) => ({
      id: s.id,
      kind: s.kind,
      protocol: s.protocol,
    })),
  });

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
  const resume = historyPos(rec.locator);
  pushHistory(rec.locator, media.title, body.cover);
  session.hls_dir = hlsDir;
  // Resume from last position — display seeks once the stream is up.
  if (resume > 0) session.cmd = { seq: 1, op: 'seek', pos: resume };
  const media_mode = hlsDir ? 'hls-remux' : 'file';
  const media_url = hlsDir
    ? `${BASE}/hls/${session.session_id}/index.m3u8`
    : `${BASE}/stream/${session.session_id}/0`;
  return [200, expose(session, media, media_mode, media_url)];
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
        return res.end(`<!doctype html><meta name="viewport" content="width=device-width,viewport-fit=cover"><title>bilibili 登录</title>
<body style="font-family:-apple-system,'PingFang SC',system-ui;background:#0b0e14;color:#e8eaf0;text-align:center;padding-top:3rem;min-height:100vh">
<h3 style="font-weight:600">用哔哩哔哩 App 扫码登录</h3>
<div style="display:inline-block;background:#fff;padding:14px;border-radius:18px;margin:1.4rem 0;box-shadow:0 8px 40px #4a7dff22">${qrSvg.createSvgTag(6)}</div>
<p id="st" style="color:#8b93a7;font-size:.9rem">等待扫码…（二维码约 3 分钟过期，过期自动刷新）</p>
<p><a href="/control" style="color:#6ea8fe;font-size:.85rem;text-decoration:none">← 返回控制端</a></p>
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
        if (pendingQr && Date.now() <= pendingQr.expires) {
          const p = await qrLoginPoll(pendingQr.qrcode_key);
          if (p.code === 0 && p.cookies) {
            // New login confirmed — store keyed by mid, make active.
            const mid = /DedeUserID=(\d+)/.exec(p.cookies)?.[1] || '?';
            const uname = await fetch(
              'https://api.bilibili.com/x/web-interface/nav',
              {
                headers: {
                  'User-Agent': 'Mozilla/5.0',
                  Cookie: p.cookies,
                },
              },
            )
              .then((r) => r.json())
              .then((d) => d.data?.uname)
              .catch(() => null);
            authStore.accounts[mid] = { cookie: p.cookies, uname };
            authStore.active = mid;
            setAuthCookie(p.cookies);
            saveAuth();
            pendingQr = null;
            res.writeHead(200, { 'content-type': 'application/json' });
            return res.end(
              JSON.stringify({
                logged_in: true,
                status: `已登录${uname ? '：' + uname : ''}`,
              }),
            );
          }
          const label =
            { 86101: '等待扫码…', 86090: '已扫码，请在手机上确认' }[p.code] ||
            `code ${p.code}`;
          res.writeHead(200, { 'content-type': 'application/json' });
          return res.end(JSON.stringify({ logged_in: false, status: label }));
        }
        if (loggedIn()) {
          res.writeHead(200, { 'content-type': 'application/json' });
          return res.end(JSON.stringify({ logged_in: true, status: '已登录' }));
        }
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(
          JSON.stringify({
            logged_in: false,
            status: pendingQr ? '二维码已过期，请刷新' : '打开 /qr 获取二维码',
          }),
        );
      }
      if (req.method === 'POST' && url.pathname === '/api/logout') {
        authStore.active = null;
        setAuthCookie(null);
        pendingQr = null;
        saveAuth();
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ logged_in: false }));
      }
      if (req.method === 'GET' && url.pathname === '/api/accounts') {
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(
          JSON.stringify({
            active: authStore.active,
            accounts: Object.entries(authStore.accounts).map(
              ([mid, a]) => ({ mid, uname: a.uname }),
            ),
          }),
        );
      }
      if (req.method === 'POST' && url.pathname === '/api/accounts/switch') {
        let raw = '';
        for await (const c of req) raw += c;
        const body = JSON.parse(raw || '{}');
        if (!authStore.accounts[body.mid]) {
          res.writeHead(404, { 'content-type': 'application/json' });
          return res.end(JSON.stringify({ error: 'NO_ACCOUNT' }));
        }
        authStore.active = String(body.mid);
        restoreActive();
        saveAuth();
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ active: authStore.active }));
      }
      if (req.method === 'GET' && url.pathname === '/api/auth') {
        const a = authStore.accounts[authStore.active];
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(
          JSON.stringify({
            logged_in: loggedIn(),
            uname: a?.uname || null,
            mid: authStore.active,
          }),
        );
      }
      if (req.method === 'GET' && url.pathname === '/api/history') {
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(
          JSON.stringify(
            listHistory().map((h) => {
              const p = h.locator?.opaque_payload || {};
              const source = p.bvid
                ? `https://www.bilibili.com/video/${p.bvid}${p.page ? '?p=' + p.page : ''}`
                : p.ep_id
                  ? `https://www.bilibili.com/bangumi/play/ep${p.ep_id}`
                  : p.room_id
                    ? `https://live.bilibili.com/${p.room_id}`
                    : '';
              return {
                title: h.title,
                source,
                at: h.at,
                pos: h.pos || 0,
                cover: h.cover || null,
              };
            }),
          ),
        );
      }
      if (req.method === 'GET' && url.pathname === '/api/discover') {
        const kind = url.searchParams.get('kind') || 'popular';
        const bvid = url.searchParams.get('bvid');
        const list = await discovery(kind, bvid);
        for (const x of list) {
          if (x.cover) x.cover = `${BASE}/img?u=${encodeURIComponent(x.cover)}`;
        }
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(list));
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
      // Per-session navigation (BV pages), subtitles, quality for control UI.
      const navMatch = /^\/api\/session\/([\w-]+)\/nav$/.exec(url.pathname);
      if (navMatch && req.method === 'GET') {
        const s = getSession(navMatch[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        const nav = await navigation(s.current_item.source_locator);
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(nav));
      }
      const subsMatch = /^\/api\/session\/([\w-]+)\/subs$/.exec(url.pathname);
      if (subsMatch && req.method === 'GET') {
        const s = getSession(subsMatch[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        const list = await subtitles(s.current_item.source_locator);
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(list));
      }
      // Cue list for a chosen subtitle track (bounded to hdslb/json).
      const cuesMatch = /^\/api\/session\/([\w-]+)\/subcues$/.exec(
        url.pathname,
      );
      if (cuesMatch && req.method === 'GET') {
        const s = getSession(cuesMatch[1]);
        const u = url.searchParams.get('u') || '';
        let up;
        try {
          up = new URL(u);
        } catch {
          up = null;
        }
        if (!s || !up || !/\.hdslb\.com$/.test(up.hostname)) {
          res.writeHead(up ? 404 : 403);
          return res.end('not allowed');
        }
        const cues = await subtitleCues(u);
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify(cues));
      }
      // Transport commands: {op:'pause'|'play'|'seek', pos?} — display
      // applies them on next poll; seq guards against replays.
      const cmdSel = /^\/api\/session\/([\w-]+)\/cmd$/.exec(url.pathname);
      if (cmdSel && req.method === 'POST') {
        const s = getSession(cmdSel[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        let raw = '';
        for await (const c of req) raw += c;
        const { op, pos } = JSON.parse(raw || '{}');
        s.cmd = { seq: (s.cmd?.seq || 0) + 1, op, pos };
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ ok: true }));
      }
      // Display reports its playback position back; control renders it.
      const posSel = /^\/api\/session\/([\w-]+)\/pos$/.exec(url.pathname);
      if (posSel && req.method === 'POST') {
        const s = getSession(posSel[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        let raw = '';
        for await (const c of req) raw += c;
        const { pos, dur, paused } = JSON.parse(raw || '{}');
        s.pos = pos;
        s.dur = dur;
        s.paused = paused;
        markHistoryPos(s.current_item.source_locator, pos, dur);
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ ok: true }));
      }
      // Send a danmaku from control at the display's current position.
      const sendDm = /^\/api\/session\/([\w-]+)\/senddm$/.exec(url.pathname);
      if (sendDm && req.method === 'POST') {
        const s = getSession(sendDm[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        let raw = '';
        for await (const c of req) raw += c;
        const { text } = JSON.parse(raw || '{}');
        try {
          await sendDanmaku(
            s.current_item.source_locator,
            String(text || '').slice(0, 100),
            s.pos || 0,
          );
          res.writeHead(200, { 'content-type': 'application/json' });
          res.end(JSON.stringify({ ok: true }));
        } catch (e) {
          res.writeHead(400, { 'content-type': 'application/json' });
          res.end(JSON.stringify({ error: String(e.message || e) }));
        }
        return;
      }
      // Volume: {vol 0..1} or {delta}; display applies to v.volume.
      const volSel = /^\/api\/session\/([\w-]+)\/vol$/.exec(url.pathname);
      if (volSel && req.method === 'POST') {
        const s = getSession(volSel[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        let raw = '';
        for await (const c of req) raw += c;
        const { vol, delta } = JSON.parse(raw || '{}');
        if (typeof vol === 'number') s.volume = Math.min(1, Math.max(0, vol));
        else if (typeof delta === 'number')
          s.volume = Math.min(1, Math.max(0, (s.volume ?? 1) + delta));
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ ok: true, volume: s.volume }));
      }
      // Stop: close the session; display returns to idle.
      const stopSel = /^\/api\/session\/([\w-]+)\/stop$/.exec(url.pathname);
      if (stopSel && req.method === 'POST') {
        closeSession(stopSel[1]);
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ ok: true }));
      }
      // Danmaku font scale for the session.
      const dmsSel = /^\/api\/session\/([\w-]+)\/dmscale$/.exec(url.pathname);
      if (dmsSel && req.method === 'POST') {
        const s = getSession(dmsSel[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        let raw = '';
        for await (const c of req) raw += c;
        const { scale } = JSON.parse(raw || '{}');
        if (typeof scale === 'number' && scale > 0.3 && scale <= 3)
          s.dm_scale = scale;
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ ok: true, dm_scale: s.dm_scale }));
      }
      // Danmaku visibility toggle for the session.
      const dmSel = /^\/api\/session\/([\w-]+)\/dm$/.exec(url.pathname);
      if (dmSel && req.method === 'POST') {
        const s = getSession(dmSel[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        let raw = '';
        for await (const c of req) raw += c;
        s.dm_on = !!JSON.parse(raw || '{}').on;
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ ok: true }));
      }
      // Control sets playback rate on the session; display applies it.
      const rateSel = /^\/api\/session\/([\w-]+)\/rate$/.exec(url.pathname);
      if (rateSel && req.method === 'POST') {
        const s = getSession(rateSel[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        let raw = '';
        for await (const c of req) raw += c;
        const { rate } = JSON.parse(raw || '{}');
        s.playback_rate = Number(rate) || 1;
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ ok: true }));
      }
      // Control sets the active subtitle track for the session.
      const subSel = /^\/api\/session\/([\w-]+)\/subtitle$/.exec(url.pathname);
      if (subSel && req.method === 'POST') {
        const s = getSession(subSel[1]);
        if (!s) {
          res.writeHead(404);
          return res.end('no session');
        }
        let raw = '';
        for await (const c of req) raw += c;
        const { url: subUrl } = JSON.parse(raw || '{}');
        s.subtitle_url = subUrl || null;
        res.writeHead(200, { 'content-type': 'application/json' });
        return res.end(JSON.stringify({ ok: true }));
      }
      if (req.method === 'GET' && url.pathname === '/api/now') {
        const s = currentSession();
        const m = s?.current_item.resolved_media;
        const body = s
          ? {
              session: {
                session_id: s.session_id,
                title: m.title,
                locator: s.current_item.source_locator,
                quality: m.quality,
                quality_options: m.quality_options || [],
                subtitle_url: s.subtitle_url || null,
                playback_rate: s.playback_rate || 1,
                cmd: s.cmd || null,
                dm_on: s.dm_on !== false,
                dm_scale: s.dm_scale || 1,
                volume: s.volume ?? 1,
                pos: s.pos || 0,
                dur: s.dur || 0,
                paused: !!s.paused,
                media_mode: m.live
                  ? 'hls-live'
                  : s.hls_dir
                    ? 'hls-remux'
                    : 'file',
                is_preview: !!m.is_preview,
                media_url: m.live
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
