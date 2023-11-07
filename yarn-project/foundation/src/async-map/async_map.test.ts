import { asyncMap } from './index.js';

describe('asyncMap', () => {
  it('execute list item sequentially', async () => {
    const sleepAndLog = (ms: number, idx: number) => new Promise(resolve => setTimeout(() => resolve(idx), ms));
    const result = await asyncMap([100, 0, 30, 1], (ms, i) => sleepAndLog(ms, i));
    expect(result).toEqual([0, 1, 2, 3]);
  });
});
