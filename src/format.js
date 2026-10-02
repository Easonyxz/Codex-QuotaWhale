export function percent(value) { return `${Number(value.toFixed(1))}%`; }
export function quotaTone(value) {
  return value == null ? 'normal' : value <= 10 ? 'critical' : value <= 25 ? 'low' : 'normal';
}
export function resetText(seconds, now = Date.now()) {
  if (seconds == null) return '重置时间未提供';
  const date = new Date(seconds * 1000);
  const mins = Math.ceil((date.getTime() - now) / 60000);
  if (mins <= 0) return '等待服务更新重置时间';
  const duration = mins >= 1440 ? `${Math.floor(mins / 1440)}天 ${Math.floor(mins % 1440 / 60)}时`
    : mins >= 60 ? `${Math.floor(mins / 60)}时 ${mins % 60}分` : `${mins}分`;
  const at = `${date.getMonth() + 1}/${date.getDate()} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`;
  return `${duration}后重置 · ${at}`;
}
