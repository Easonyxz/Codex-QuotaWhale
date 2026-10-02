const api = window.__TAURI__;
let ready = false;
let syncing = false;
let pending = false;

export async function syncRegions() {
  if (!ready) return;
  pending = true;
  if (syncing) return;
  syncing = true;
  try {
    while (pending) {
      pending = false;
      const pet = document.getElementById('pet').getBoundingClientRect();
      const mini = document.getElementById('mini').getBoundingClientRect();
      const rects = [{ x: pet.x, y: pet.y, width: pet.width, height: mini.bottom - pet.y }];
      if (document.querySelector('main').classList.contains('expanded')) {
        const bubble = document.querySelector('.bubble').getBoundingClientRect();
        const scale = pet.width / 157;
        rects.push({ x: bubble.x - 5 * scale, y: bubble.y - 5 * scale,
          width: bubble.width + 10 * scale, height: bubble.height + 29 * scale });
      }
      await api.core.invoke('set_hit_regions', { rects });
    }
  } catch (error) {
    console.error('Window region update failed', error);
  } finally { syncing = false; }
}

export function setupDesktop() {
  const main = document.querySelector('main');
  let openTimer;
  let closeTimer;
  const open = () => {
    clearTimeout(closeTimer);
    clearTimeout(openTimer);
    if (!main.classList.contains('expanded')) {
      openTimer = setTimeout(() => { main.classList.add('expanded'); syncRegions(); }, 220);
    }
  };
  const close = () => {
    clearTimeout(openTimer);
    clearTimeout(closeTimer);
    closeTimer = setTimeout(() => { main.classList.remove('expanded'); syncRegions(); }, 350);
  };
  for (const element of [document.getElementById('pet'), document.getElementById('mini'), document.querySelector('.bubble')]) {
    element.addEventListener('pointerenter', open);
    element.addEventListener('pointerleave', close);
  }
  main.addEventListener('focusin', open);
  main.addEventListener('focusout', close);
  window.addEventListener('blur', close);
  window.addEventListener('resize', syncRegions);
  window.addEventListener('pet-scale', syncRegions);
  new ResizeObserver(syncRegions).observe(document.querySelector('.bubble'));
  ready = true;
  syncRegions();
}
