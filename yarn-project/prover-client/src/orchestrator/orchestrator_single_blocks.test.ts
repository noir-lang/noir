import { PROVING_STATUS } from '@aztec/circuit-types';
import { NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { fr } from '@aztec/circuits.js/testing';
import { range } from '@aztec/foundation/array';
import { createDebugLogger } from '@aztec/foundation/log';
import { sleep } from '@aztec/foundation/sleep';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { makeBloatedProcessedTx, updateExpectedTreesFromTxs } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';

const logger = createDebugLogger('aztec:orchestrator-single-blocks');

describe('prover/orchestrator/blocks', () => {
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
    it('builds an empty L2 block', async () => {
      const blockTicket = await context.orchestrator.startNewBlock(2, context.globalVariables, []);

      await context.orchestrator.setBlockCompleted();

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(context.blockNumber);
    });

    it('builds a block with 1 transaction', async () => {
      const txs = await Promise.all([makeBloatedProcessedTx(context.actualDb, 1)]);

      await updateExpectedTreesFromTxs(expectsDb, txs);

      // This will need to be a 2 tx block
      const blockTicket = await context.orchestrator.startNewBlock(2, context.globalVariables, []);

      for (const tx of txs) {
        await context.orchestrator.addNewTx(tx);
      }

      //  we need to complete the block as we have not added a full set of txs
      await context.orchestrator.setBlockCompleted();

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(context.blockNumber);
    });

    it('builds a block concurrently with transaction simulation', async () => {
      const txs = await Promise.all([
        makeBloatedProcessedTx(context.actualDb, 1),
        makeBloatedProcessedTx(context.actualDb, 2),
        makeBloatedProcessedTx(context.actualDb, 3),
        makeBloatedProcessedTx(context.actualDb, 4),
      ]);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket = await context.orchestrator.startNewBlock(txs.length, context.globalVariables, l1ToL2Messages);

      for (const tx of txs) {
        await context.orchestrator.addNewTx(tx);
        await sleep(1000);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(context.blockNumber);
    });
  });
});
