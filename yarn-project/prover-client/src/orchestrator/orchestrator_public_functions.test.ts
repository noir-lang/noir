import { PROVING_STATUS, mockTx } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';

import { TestContext } from '../mocks/test_context.js';

const logger = createDebugLogger('aztec:orchestrator-public-functions');

describe('prover/orchestrator/public-functions', () => {
  let context: TestContext;

  beforeEach(async () => {
    context = await TestContext.new(logger);
  });

  afterEach(async () => {
    await context.cleanup();
  });

  describe('blocks with public functions', () => {
    let testCount = 1;

    it.each([
      [0, 4],
      [1, 0],
      [2, 0],
      [1, 5],
      [2, 4],
      [8, 1],
    ] as const)(
      'builds an L2 block with %i non-revertible and %i revertible calls',
      async (numberOfNonRevertiblePublicCallRequests: number, numberOfRevertiblePublicCallRequests: number) => {
        const tx = mockTx(1000 * testCount++, {
          numberOfNonRevertiblePublicCallRequests,
          numberOfRevertiblePublicCallRequests,
        });
        tx.data.constants.historicalHeader = context.actualDb.getInitialHeader();
        tx.data.constants.vkTreeRoot = getVKTreeRoot();

        const [processed, _] = await context.processPublicFunctions([tx], 1, undefined);

        // This will need to be a 2 tx block
        const blockTicket = await context.orchestrator.startNewBlock(2, context.globalVariables, []);

        for (const processedTx of processed) {
          await context.orchestrator.addNewTx(processedTx);
        }

        //  we need to complete the block as we have not added a full set of txs
        await context.orchestrator.setBlockCompleted();

        const result = await blockTicket.provingPromise;
        expect(result.status).toBe(PROVING_STATUS.SUCCESS);
        const finalisedBlock = await context.orchestrator.finaliseBlock();

        expect(finalisedBlock.block.number).toEqual(context.blockNumber);
      },
    );
  });
});
