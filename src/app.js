import { percent, resetText, quotaTone } from './format.js';
import { setupDesktop, syncRegions } from './desktop.js';
const $ = id => document.getElementById(id);
const api = window.__TAURI__;
// Optional private/local artwork; the public repository ships the original SVG above.
const localArtwork = new Image();
localArtwork.onload = () => { $('pet').querySelector('img').src = localArtwork.src; };
localArtwork.src = 'assets/whale.png';
function applyScale(scale) {
  document.querySelector('main').style.transform = `scale(${scale})`;
  window.dispatchEvent(new Event('pet-scale'));
}
await api.event.listen('scale-changed', event => applyScale(event.payload));
applyScale(await api.core.invoke('get_scale'));
let snapshot = null;
let busy = false;
let error = '';

function render() {
  $('short-row').hidden = !snapshot?.short;
  $('mini-value').textContent = snapshot?.weekly ? percent(snapshot.weekly.remaining) : '—';
  $('mini-bar').style.width = `${snapshot?.weekly?.remaining ?? 0}%`;
  $('mini').dataset.tone = quotaTone(snapshot?.weekly?.remaining);
  $('mini').title = error || (snapshot?.weekly ? `周额度剩余 ${percent(snapshot.weekly.remaining)} · ${resetText(snapshot.weekly.resetsAt)}` : '周额度尚未提供');
  $('mini').classList.toggle('stale', Boolean(error));
  for (const name of ['short', 'weekly']) {
    const window = snapshot?.[name];
    $(`${name}-value`).textContent = window ? percent(window.remaining) : '—';
    $(`${name}-bar`).style.width = `${window?.remaining ?? 0}%`;
    $(`${name}-bar`).style.backgroundColor = window?.remaining < 10 ? '#c18b70' : '#6a83c0';
    $(`${name}-reset`).textContent = window ? resetText(window.resetsAt) : snapshot ? '此窗口未提供' : '等待额度';
  }
  $('status').className = error ? 'error' : '';
  const updated = snapshot ? new Date(snapshot.updatedAt * 1000).toLocaleTimeString('zh-CN', {hour: '2-digit', minute: '2-digit'}) : '';
  $('status').textContent = error ? `${error}${updated ? ` · 保留 ${updated} 数据` : ''}`
    : busy ? '正在刷新…' : `${updated} 更新 · 每分钟刷新`;
  syncRegions();
}
async function refresh() {
  if (busy) return;
  busy = true;
  $('refresh').disabled = true;
  render();
  try {
    snapshot = await api.core.invoke('get_quota');
    error = '';
  } catch (reason) {
    error = typeof reason === 'string' ? reason : '读取失败，稍后自动重试';
  } finally {
    busy = false;
    $('refresh').disabled = false;
    render();
  }
}
$('refresh').addEventListener('click', refresh);
$('close').addEventListener('click', () => api.core.invoke('hide_pet'));
$('pet').addEventListener('mousedown', async event => {
  if (event.button !== 0) return;
  try { await api.window.getCurrentWindow().startDragging(); }
  catch { error = '窗口移动失败，请重试'; render(); }
});
$('pet').addEventListener('dblclick', refresh);
document.addEventListener('contextmenu', async event => {
  event.preventDefault();
  try { await api.core.invoke('show_menu'); }
  catch { error = '菜单打开失败，请重试'; render(); }
});
api.event.listen('refresh-requested', refresh);
api.event.listen('settings-error', event => { error = event.payload; render(); });
window.addEventListener('online', refresh);
setupDesktop();
refresh();
setInterval(refresh, 60_000);
setInterval(render, 30_000);
