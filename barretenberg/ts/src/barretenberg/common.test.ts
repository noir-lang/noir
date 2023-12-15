import { Barretenberg } from './index.js';

describe('env', () => {
  let api: Barretenberg;

  beforeAll(async () => {
    api = await Barretenberg.new({ threads: 3 });
  }, 15000);

  afterAll(async () => {
    await api.destroy();
  });

  it('thread test', async () => {
    // Main thread doesn't do anything in this test, so -1.
    const threads = (await api.getNumThreads()) - 1;
    const iterations = 100000;
    const result = await api.testThreads(threads, iterations);
    expect(result).toBe(iterations);
  });
});
