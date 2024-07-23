import { PROVING_STATUS } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';

import { makeBloatedProcessedTx, makeGlobals } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';

const logger = createDebugLogger('aztec:orchestrator-multi-blocks');

describe('prover/orchestrator/multi-block', () => {
  let context: TestContext;

  beforeEach(async () => {
    context = await TestContext.new(logger);
  });

  afterEach(async () => {
    await context.cleanup();
  });

  describe('multiple blocks', () => {
    it('builds multiple blocks in sequence', async () => {
      const numBlocks = 5;
      let header = context.actualDb.getInitialHeader();

      for (let i = 0; i < numBlocks; i++) {
        const tx = makeBloatedProcessedTx(context.actualDb, i + 1);
        tx.data.constants.historicalHeader = header;
        tx.data.constants.vkTreeRoot = getVKTreeRoot();

        const blockNum = i + 1000;

        const globals = makeGlobals(blockNum);

        // This will need to be a 2 tx block
        const blockTicket = await context.orchestrator.startNewBlock(2, globals, []);

        await context.orchestrator.addNewTx(tx);

        //  we need to complete the block as we have not added a full set of txs
        await context.orchestrator.setBlockCompleted();

        const result = await blockTicket.provingPromise;
        expect(result.status).toBe(PROVING_STATUS.SUCCESS);
        const finalisedBlock = await context.orchestrator.finaliseBlock();

        expect(finalisedBlock.block.number).toEqual(blockNum);
        header = finalisedBlock.block.header;

        await context.actualDb.commit();
      }
    });
  });
});
