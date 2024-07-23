import { MerkleTreeId, PROVING_STATUS } from '@aztec/circuit-types';
import { NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { fr } from '@aztec/circuits.js/testing';
import { range } from '@aztec/foundation/array';
import { times } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { makeBloatedProcessedTx, updateExpectedTreesFromTxs } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';

const logger = createDebugLogger('aztec:orchestrator-mixed-blocks-2');

describe('prover/orchestrator/mixed-blocks', () => {
  let context: TestContext;
  let expectsDb: MerkleTreeOperations;

  beforeEach(async () => {
    context = await TestContext.new(logger);
    expectsDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
  });

  afterEach(async () => {
    await context.cleanup();
  });

  describe('blocks', () => {
    it.each([2, 4, 5, 8] as const)('builds an L2 block with %i bloated txs', async (totalCount: number) => {
      const txs = times(totalCount, (i: number) => makeBloatedProcessedTx(context.actualDb, i));

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket = await context.orchestrator.startNewBlock(txs.length, context.globalVariables, l1ToL2Messages);

      for (const tx of txs) {
        await context.orchestrator.addNewTx(tx);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);

      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(context.blockNumber);

      await updateExpectedTreesFromTxs(expectsDb, txs);
      const noteHashTreeAfter = await context.actualDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE);

      const expectedNoteHashTreeAfter = await expectsDb.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE).then(t => t.root);
      expect(noteHashTreeAfter.root).toEqual(expectedNoteHashTreeAfter);
    });
  });
});
