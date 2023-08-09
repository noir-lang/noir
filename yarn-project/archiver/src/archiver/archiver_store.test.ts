import { INITIAL_L2_BLOCK_NUM, L2Block, L2BlockL2Logs, LogType } from '@aztec/types';

import { ArchiverDataStore, MemoryArchiverStore } from './archiver_store.js';

describe('Archiver Memory Store', () => {
  let archiverStore: ArchiverDataStore;

  beforeEach(() => {
    archiverStore = new MemoryArchiverStore();
  });

  it('can store and retrieve blocks', async () => {
    const blocks = Array(10)
      .fill(0)
      .map((_, index) => L2Block.random(index));
    await archiverStore.addL2Blocks(blocks);
    // Offset indices by INTIAL_L2_BLOCK_NUM to ensure we are correctly aligned
    for (const [from, limit] of [
      [0 + INITIAL_L2_BLOCK_NUM, 10],
      [3 + INITIAL_L2_BLOCK_NUM, 3],
      [1 + INITIAL_L2_BLOCK_NUM, 7],
      [5 + INITIAL_L2_BLOCK_NUM, 8],
      [10 + INITIAL_L2_BLOCK_NUM, 1],
      [11 + INITIAL_L2_BLOCK_NUM, 1],
    ]) {
      const expected = blocks.slice(from - INITIAL_L2_BLOCK_NUM, from - INITIAL_L2_BLOCK_NUM + limit);
      const actual = await archiverStore.getL2Blocks(from, limit);
      expect(expected).toEqual(actual);
    }
  });

  test.each([LogType.ENCRYPTED, LogType.UNENCRYPTED])('can store and retrieve logs', async (logType: LogType) => {
    const logs = Array(10)
      .fill(0)
      .map(_ => L2BlockL2Logs.random(6, 3, 2));
    await archiverStore.addLogs(logs, logType);
    // Offset indices by INTIAL_L2_BLOCK_NUM to ensure we are correctly aligned
    for (const [from, limit] of [
      [0 + INITIAL_L2_BLOCK_NUM, 10],
      [3 + INITIAL_L2_BLOCK_NUM, 3],
      [1 + INITIAL_L2_BLOCK_NUM, 7],
      [5 + INITIAL_L2_BLOCK_NUM, 8],
      [10 + INITIAL_L2_BLOCK_NUM, 1],
      [11 + INITIAL_L2_BLOCK_NUM, 1],
    ]) {
      const expected = logs.slice(from - INITIAL_L2_BLOCK_NUM, from - INITIAL_L2_BLOCK_NUM + limit);
      const actual = await archiverStore.getLogs(from, limit, logType);
      expect(expected).toEqual(actual);
    }
  });

  it('throws if we try and request less than 1 block', async () => {
    const blocks = Array(10)
      .fill(0)
      .map((_, index) => L2Block.random(index));
    await archiverStore.addL2Blocks(blocks);
    await expect(async () => await archiverStore.getL2Blocks(1, 0)).rejects.toThrow(
      `Invalid block range from: 1, limit: 0`,
    );
  });

  test.each([LogType.ENCRYPTED, LogType.UNENCRYPTED])(
    'throws if we try and request less than 1 log',
    async (logType: LogType) => {
      const logs = Array(10)
        .fill(0)
        .map(_ => L2BlockL2Logs.random(6, 3, 2));
      await archiverStore.addLogs(logs, logType);
      await expect(async () => await archiverStore.getLogs(1, 0, logType)).rejects.toThrow(
        `Invalid block range from: 1, limit: 0`,
      );
    },
  );
});
