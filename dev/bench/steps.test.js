// Run with: node --test dev/bench/steps.test.js
'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { stepsOf } = require('./steps.js');

const THRESHOLD = 2;
const series = (unit, values, tool = 'customSmallerIsBetter') => values.map(value => ({ value, unit, tool }));
const repeat = (v, n) => Array(n).fill(v);
const indices = steps => steps.map(s => s.index);

// Timing series with ~2% commit-to-commit noise around 10s.
const FLAT_TIMING = [10, 10.2, 9.9, 10.1, 10, 9.8, 10.1];

test('a timing spike on the newest commit flags nothing', () => {
  assert.deepEqual(stepsOf(series('s', [...FLAT_TIMING, 16]), THRESHOLD), []);
});

test('a timing spike one or two commits back flags nothing', () => {
  assert.deepEqual(stepsOf(series('s', [...FLAT_TIMING, 16, 10]), THRESHOLD), []);
  assert.deepEqual(stepsOf(series('s', [...FLAT_TIMING, 16, 10, 10.1]), THRESHOLD), []);
});

test('a timing spike on the oldest commit flags nothing', () => {
  assert.deepEqual(stepsOf(series('s', [16, ...FLAT_TIMING]), THRESHOLD), []);
});

test('a lasting timing step is attributed to the commit that introduced it', () => {
  const steps = stepsOf(series('s', [...FLAT_TIMING, 15, 15.2, 14.9, 15.1, 15]), THRESHOLD);
  assert.deepEqual(indices(steps), [7]);
  assert.ok(steps[0].pct > 40 && steps[0].pct < 60, `pct was ${steps[0].pct}`);
});

test('a timing step is flagged once three points confirm it', () => {
  assert.deepEqual(stepsOf(series('s', [...FLAT_TIMING, 15, 15.2]), THRESHOLD), []);
  assert.deepEqual(indices(stepsOf(series('s', [...FLAT_TIMING, 15, 15.2, 14.9]), THRESHOLD)), [7]);
});

test('millisecond rounding on timings is not a step', () => {
  const values = [0.002, 0.003, 0.002, 0.002, 0.003, 0.003, 0.002, 0.003, 0.002, 0.002];
  assert.deepEqual(stepsOf(series('s', values), THRESHOLD), []);
});

test('two nearby opcode changes are two steps on their own commits', () => {
  const steps = stepsOf(series('opcodes', [...repeat(100, 6), 105, 105, ...repeat(110.25, 6)]), THRESHOLD);
  assert.deepEqual(indices(steps), [6, 8]);
  assert.deepEqual(steps.map(s => Math.round(s.pct * 100) / 100), [5, 5]);
});

test('an opcode change reverted two commits later is flagged both ways', () => {
  const steps = stepsOf(series('opcodes', [...repeat(100, 6), 105, 105, ...repeat(100, 6)]), THRESHOLD);
  assert.deepEqual(indices(steps), [6, 8]);
  assert.ok(steps[0].pct > 0 && steps[1].pct < 0);
});

test('an opcode change on the newest commit is flagged immediately', () => {
  assert.deepEqual(indices(stepsOf(series('opcodes', [...repeat(100, 6), 103]), THRESHOLD)), [6]);
});

test('size changes below the threshold and memory jitter are not steps', () => {
  assert.deepEqual(stepsOf(series('opcodes', [...repeat(100, 6), 101]), THRESHOLD), []);
  assert.deepEqual(stepsOf(series('MB', [297.25, 297.26, 297.25, 297.27, 297.25]), THRESHOLD), []);
});

test('for bigger-is-better tools a drop is a regression', () => {
  const steps = stepsOf(series('ops/sec', [...FLAT_TIMING, 5, 5.1, 4.9, 5, 5], 'customBiggerIsBetter'), THRESHOLD);
  assert.deepEqual(indices(steps), [7]);
  assert.ok(steps[0].raw < 0 && steps[0].pct > 0);
});

test('an opcode change of a couple of units is flagged when it crosses the threshold', () => {
  const steps = stepsOf(series('opcodes', [...repeat(10, 6), 12, 12]), THRESHOLD);
  assert.deepEqual(indices(steps), [6]);
  assert.equal(steps[0].pct, 20);
});

test('a step after commits with no value reports how many were missing', () => {
  const points = series('opcodes', [100, 100, 100, 110, 110]).map((p, i) => ({ ...p, entry: [0, 1, 2, 5, 6][i] }));
  const steps = stepsOf(points, THRESHOLD);
  assert.deepEqual(indices(steps), [3]);
  assert.equal(steps[0].missing, 2);
});

test('a step between consecutive commits reports none missing', () => {
  const points = series('opcodes', [100, 100, 110]).map((p, i) => ({ ...p, entry: i }));
  assert.equal(stepsOf(points, THRESHOLD)[0].missing, 0);
});
