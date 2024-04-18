import { PROVING_STATUS, mockTx } from '@aztec/circuit-types';
import { times } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';

import { type MemDown, default as memdown } from 'memdown';

import { makeEmptyProcessedTestTx } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

const logger = createDebugLogger('aztec:orchestrator-multi-public-functions');

describe('prover/orchestrator/public-functions', () => {
  let context: TestContext;

  beforeEach(async () => {
    context = await TestContext.new(logger);
  }, 20_000);

  afterEach(async () => {
    await context.cleanup();
  });

  describe('blocks with public functions', () => {
    let testCount = 1;
    it.each([[4, 2, 3]] as const)(
      'builds an L2 block with %i transactions each with %i revertible and %i non revertible',
      async (
        numTransactions: number,
        numberOfNonRevertiblePublicCallRequests: number,
        numberOfRevertiblePublicCallRequests: number,
      ) => {
        const txs = times(numTransactions, (i: number) =>
          mockTx(100000 * testCount++ + 1000 * i, {
            numberOfNonRevertiblePublicCallRequests,
            numberOfRevertiblePublicCallRequests,
          }),
        );
        for (const tx of txs) {
          tx.data.constants.historicalHeader = await context.actualDb.buildInitialHeader();
        }

        const blockTicket = await context.orchestrator.startNewBlock(
          numTransactions,
          context.globalVariables,
          [],
          await makeEmptyProcessedTestTx(context.actualDb),
        );

        const [processed, failed] = await context.processPublicFunctions(txs, numTransactions, context.orchestrator);
        expect(processed.length).toBe(numTransactions);
        expect(failed.length).toBe(0);

        await context.orchestrator.setBlockCompleted();

        const result = await blockTicket.provingPromise;
        expect(result.status).toBe(PROVING_STATUS.SUCCESS);
        const finalisedBlock = await context.orchestrator.finaliseBlock();

        expect(finalisedBlock.block.number).toEqual(context.blockNumber);
      },
      60_000,
    );
  });
});
