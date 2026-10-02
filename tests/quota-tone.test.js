import { test } from 'node:test';
import assert from 'node:assert/strict';
import { quotaTone } from '../src/format.js';
test('quota colors distinguish missing data and exact thresholds', () => {
  assert.equal(quotaTone(null), 'normal');
  assert.equal(quotaTone(undefined), 'normal');
  assert.equal(quotaTone(25.1), 'normal');
  assert.equal(quotaTone(25), 'low');
  assert.equal(quotaTone(10.1), 'low');
  assert.equal(quotaTone(10), 'critical');
  assert.equal(quotaTone(0), 'critical');
});
