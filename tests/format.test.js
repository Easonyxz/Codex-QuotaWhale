import { test } from 'node:test';
import assert from 'node:assert/strict';
import { percent, resetText } from '../src/format.js';
test('preserves fractional quota without showing unknown as zero', () => {
  assert.equal(percent(99.5), '99.5%');
  assert.equal(percent(0), '0%');
});
test('reset countdown handles future, past and absent timestamps', () => {
  assert.match(resetText(7200, 0), /^2时 0分后重置/);
  assert.match(resetText(90000, 0), /^1天 1时后重置/);
  assert.equal(resetText(0, 1), '等待服务更新重置时间');
  assert.equal(resetText(null), '重置时间未提供');
});
