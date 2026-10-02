import { test } from 'node:test';
import assert from 'node:assert/strict';

test('hover delay cancels flyovers, bridges the gap and removes the upper hit area', async t => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  class Element extends EventTarget {
    constructor(rect) { super(); this.rect = rect; this.classes = new Set(); }
    classList = { contains: value => this.classes.has(value), add: value => this.classes.add(value), remove: value => this.classes.delete(value) };
    getBoundingClientRect() { return this.rect; }
  }
  const main = new Element({});
  const pet = new Element({ x: 80, y: 184, width: 157, bottom: 341 });
  const mini = new Element({ x: 80, y: 342, width: 157, bottom: 364 });
  const bubble = new Element({ x: 12, y: 42, width: 218, height: 130 });
  const calls = [];
  globalThis.window = new EventTarget();
  window.__TAURI__ = { core: { invoke: async (name, payload) => calls.push(payload.rects) } };
  globalThis.document = { getElementById: id => id === 'pet' ? pet : mini,
    querySelector: selector => selector === 'main' ? main : bubble };
  globalThis.ResizeObserver = class { observe() {} };
  const { setupDesktop } = await import('../src/desktop.js');
  setupDesktop();
  await Promise.resolve();
  assert.equal(calls.at(-1).length, 1);
  assert.equal(calls.at(-1)[0].y, 184);
  pet.dispatchEvent(new Event('pointerenter'));
  t.mock.timers.tick(150);
  pet.dispatchEvent(new Event('pointerleave'));
  t.mock.timers.tick(400);
  assert.equal(main.classList.contains('expanded'), false);
  pet.dispatchEvent(new Event('pointerenter'));
  t.mock.timers.tick(220);
  await Promise.resolve();
  assert.equal(main.classList.contains('expanded'), true);
  assert.equal(calls.at(-1).length, 2);
  pet.dispatchEvent(new Event('pointerleave'));
  t.mock.timers.tick(150);
  bubble.dispatchEvent(new Event('pointerenter'));
  t.mock.timers.tick(400);
  assert.equal(main.classList.contains('expanded'), true);
  bubble.dispatchEvent(new Event('pointerleave'));
  t.mock.timers.tick(350);
  await Promise.resolve();
  assert.equal(main.classList.contains('expanded'), false);
  assert.equal(calls.at(-1).length, 1);
});
