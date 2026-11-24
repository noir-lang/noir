// Step detection for the benchmark dashboard: decides which commits moved a benchmark.
//
// Loaded by index.html as a classic script (exposes the global `BenchSteps`) and by
// steps.test.js under Node (`node --test dev/bench/steps.test.js`), which pins the behaviour below.
//
// A point is { value, unit, tool, ... } as recorded by github-action-benchmark; points are in
// commit order. A step is { index, before, after, raw, pct }:
//   - index: position of the commit that introduced the change;
//   - before, after: the levels on either side of it;
//   - raw: signed relative change in percent;
//   - pct: the same, signed so that a positive value is always a regression.
'use strict';

const BenchSteps = (() => {
  const BIGGER_IS_BETTER = new Set(['customBiggerIsBetter', 'benchmarkjs', 'benchmarkluau']);

  // Counts and sizes are reproducible: the same commit always yields the same value, give or
  // take ~0.01 MB of allocator jitter in the memory reports (far below any useful threshold).
  // Every change at or above the threshold between two consecutive commits is real, including
  // one reverted a commit later, and however few units it is.
  const DETERMINISTIC_UNITS = new Set(['opcodes', 'gates', 'B', 'bytes', 'KB', 'KiB', 'MB', 'MiB', 'GB', 'GiB']);

  // Timings run on shared CI runners and routinely jump 20-80% between consecutive commits
  // with no code change: some flip between two levels depending on the runner, others are
  // rounded to the millisecond so 2ms -> 3ms reads as +50%. For those, a commit is flagged
  // when the median of the WINDOW points from it onwards differs from the median of the
  // WINDOW points before it by at least:
  //   - the threshold,
  //   - NOISE_MULTIPLE times the benchmark's noise (the 75th percentile of its
  //     commit-to-commit relative changes, excluding the largest, which in a short series
  //     is the step being looked for), and
  //   - more than RESOLUTION_MULTIPLE times the measurement's rounding step, so that
  //     millisecond rounding (2ms -> 3ms) is not a step.
  // Both windows need at least MIN_SIDE points: the median of three values ignores a single
  // spike, while the median of two is their mean and moves by half the spike. So a timing
  // change on one of the two newest (or two oldest) commits is not flagged yet. A commit is
  // only credited with a step when its own value is on the new level and its predecessor's
  // on the old one.
  const WINDOW = 5;
  const MIN_SIDE = 3;
  const NOISE_MULTIPLE = 3;
  const RESOLUTION_MULTIPLE = 2;

  function median(xs) {
    const s = xs.slice().sort((a, b) => a - b);
    const m = s.length >> 1;
    return s.length % 2 ? s[m] : (s[m - 1] + s[m]) / 2;
  }

  function quantile(xs, q) {
    const s = xs.slice().sort((a, b) => a - b);
    return s.length ? s[Math.min(s.length - 1, Math.floor(q * s.length))] : 0;
  }

  // One unit in the last decimal place the values are recorded with.
  function resolutionOf(vals) {
    let decimals = 0;
    for (const v of vals) {
      const m = /\.(\d+)$/.exec(String(v));
      if (m) decimals = Math.max(decimals, m[1].length);
    }
    return Math.pow(10, -decimals);
  }

  function relative(before, after) {
    return (after - before) / Math.abs(before) * 100;
  }

  function regressionSign(point) {
    return BIGGER_IS_BETTER.has(point.tool) ? -1 : 1;
  }

  function isDeterministic(points) {
    return points.length > 0 && DETERMINISTIC_UNITS.has(points[0].unit);
  }

  function step(points, index, before, after) {
    const raw = relative(before, after);
    return { index, before, after, raw, pct: regressionSign(points[index]) * raw };
  }

  // Change of each point against the previous one, in the same shape as a step.
  function changesOf(points) {
    const out = [];
    for (let i = 1; i < points.length; i++) {
      if (points[i - 1].value) out.push(step(points, i, points[i - 1].value, points[i].value));
    }
    return out;
  }

  function stepsOf(points, threshold) {
    if (isDeterministic(points)) return changesOf(points).filter(c => Math.abs(c.raw) >= threshold);
    return windowedSteps(points, threshold);
  }

  function windowedSteps(points, threshold) {
    const minAbs = RESOLUTION_MULTIPLE * resolutionOf(points.map(p => p.value));
    const vals = points.map(p => p.value);
    const rel = changesOf(points).map(c => Math.abs(c.raw)).sort((a, b) => a - b).slice(0, -1);
    const cutoff = Math.max(threshold, NOISE_MULTIPLE * quantile(rel, 0.75));

    const cand = [];
    for (let i = MIN_SIDE; i + MIN_SIDE <= vals.length; i++) {
      const before = median(vals.slice(Math.max(0, i - WINDOW), i));
      const after = median(vals.slice(i, i + WINDOW));
      if (!before || Math.abs(after - before) <= minAbs) continue;
      // The commit itself must sit on the new level and its predecessor on the old one;
      // otherwise the windows straddle a change that happened at a neighbouring commit.
      const nearer = (v, a, b) => Math.abs(v - a) < Math.abs(v - b);
      if (!nearer(vals[i], after, before) || !nearer(vals[i - 1], before, after)) continue;
      const c = step(points, i, before, after);
      if (Math.abs(c.raw) >= cutoff) cand.push(c);
    }

    // The window medians shift over several neighbouring commits around one real step.
    // Each run of consecutive same-direction candidates is one step, attributed to the
    // commit inside it with the largest single-commit move, with that commit's medians.
    const steps = [];
    let run = [];
    const flush = () => {
      if (!run.length) return;
      const sign = Math.sign(run[0].raw);
      const move = c => (vals[c.index] - vals[c.index - 1]) * sign;
      steps.push(run.reduce((best, c) => (move(c) > move(best) ? c : best), run[0]));
      run = [];
    };
    for (const c of cand) {
      const last = run[run.length - 1];
      if (last && (c.index !== last.index + 1 || Math.sign(c.raw) !== Math.sign(last.raw))) flush();
      run.push(c);
    }
    flush();
    return steps;
  }

  return { WINDOW, MIN_SIDE, NOISE_MULTIPLE, changesOf, stepsOf };
})();

if (typeof module !== 'undefined') module.exports = BenchSteps;
