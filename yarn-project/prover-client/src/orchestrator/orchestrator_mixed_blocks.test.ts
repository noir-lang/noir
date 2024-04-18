import { PROVING_STATUS } from '@aztec/circuit-types';
import { NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { fr } from '@aztec/circuits.js/testing';
import { range } from '@aztec/foundation/array';
import { createDebugLogger } from '@aztec/foundation/log';

import { type MemDown, default as memdown } from 'memdown';

import { makeBloatedProcessedTx, makeEmptyProcessedTestTx } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

const logger = createDebugLogger('aztec:orchestrator-mixed-blocks');

describe('prover/orchestrator/mixed-blocks', () => {
  let context: TestContext;

  beforeEach(async () => {
    context = await TestContext.new(logger);
  }, 20_000);

  afterEach(async () => {
    await context.cleanup();
  });

  describe('blocks', () => {
    it('builds an unbalanced L2 block', async () => {
      const txs = await Promise.all([
        makeBloatedProcessedTx(context.actualDb, 1),
        makeBloatedProcessedTx(context.actualDb, 2),
        makeBloatedProcessedTx(context.actualDb, 3),
      ]);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      // this needs to be a 4 tx block that will need to be completed
      const blockTicket = await context.orchestrator.startNewBlock(
        4,
        context.globalVariables,
        l1ToL2Messages,
        await makeEmptyProcessedTestTx(context.actualDb),
      );

      for (const tx of txs) {
        await context.orchestrator.addNewTx(tx);
      }

      await context.orchestrator.setBlockCompleted();

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(context.blockNumber);
    }, 60_000);
  });
});
