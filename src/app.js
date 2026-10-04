import { percent, resetText, quotaTone } from './format.js';
import { setupDesktop, syncRegions } from './desktop.js';
const $ = id => document.getElementById(id);
const api = window.__TAURI__;
function applyScale(scale) {
  document.querySelector('main').style.transform = `scale(${scale})`;
  window.dispatchEvent(new Event('pet-scale'));
}
await api.event.listen('scale-changed', event => applyScale(event.payload));
applyScale(await api.core.invoke('get_scale'));
let snapshot = null;
let credits = null;
let creditsError = false;
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
    : busy ? '正在刷新…' : `${updated} 更新 · 每5分钟刷新`;
  syncRegions();
}
function renderCredits() {
  const row = $('credits');
  row.hidden = !credits && !creditsError;
  if (!credits) { row.textContent = '重置卡暂未读取，手动刷新可重试'; return; }
  const dates = credits.expirations.map(value => new Date(typeof value === 'number' ? value * 1000 : value))
    .filter(date => !Number.isNaN(date.getTime())).sort((a, b) => a - b);
  const expiry = dates.length ? dates[0].toLocaleString('zh-CN', {month:'numeric', day:'numeric', hour:'2-digit', minute:'2-digit', hour12:false}) : '';
  row.textContent = `重置卡 ${credits.count} 次${credits.count ? (expiry ? ` · ${expiry} 最早到期` : ' · 到期时间未提供') : ''}${creditsError ? '（上次数据）' : ''}`;
  row.title = dates.map(date => date.toLocaleString('zh-CN')).join('\n');
  syncRegions();
}
async function refreshCredits() {
  try { credits = await api.core.invoke('get_reset_credits'); creditsError = false; }
  catch { creditsError = true; }
  renderCredits();
}
async function refresh(includeCredits = true) {
  if (busy) return;
  busy = true;
  $('refresh').disabled = true;
  render();
  try {
    const [quotaResult] = await Promise.allSettled([api.core.invoke('get_quota'), includeCredits ? refreshCredits() : Promise.resolve()]);
    if (quotaResult.status === 'rejected') throw quotaResult.reason;
    snapshot = quotaResult.value;
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
window.addEventListener('online', () => refresh(false));
setupDesktop();
refresh();
setInterval(() => refresh(false), 300_000);
setInterval(render, 30_000);
