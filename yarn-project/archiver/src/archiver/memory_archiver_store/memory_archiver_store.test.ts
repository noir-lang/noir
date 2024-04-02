import { L2Block } from '@aztec/circuit-types';

import { type ArchiverDataStore } from '../archiver_store.js';
import { describeArchiverDataStore } from '../archiver_store_test_suite.js';
import { MemoryArchiverStore } from './memory_archiver_store.js';

describe('MemoryArchiverStore', () => {
  let archiverStore: ArchiverDataStore;

  beforeEach(() => {
    archiverStore = new MemoryArchiverStore(1000);
  });

  describeArchiverDataStore('implements ArchiverStore', () => archiverStore);

  describe('getUnencryptedLogs config', () => {
    it('does not return more than "maxLogs" logs', async () => {
      const maxLogs = 5;
      archiverStore = new MemoryArchiverStore(maxLogs);
      const blocks = {
        lastProcessedL1BlockNumber: 3n,
        retrievedData: Array(10)
          .fill(0)
          .map((_, index: number) => L2Block.random(index + 1, 4, 2, 3, 2, 2)),
      };

      await archiverStore.addBlocks(blocks);
      await Promise.all(
        blocks.retrievedData.map(block =>
          archiverStore.addLogs(block.body.encryptedLogs, block.body.unencryptedLogs, block.number),
        ),
      );

      const response = await archiverStore.getUnencryptedLogs({});

      expect(response.maxLogsHit).toBeTruthy();
      expect(response.logs.length).toEqual(maxLogs);
    });
  });
});
