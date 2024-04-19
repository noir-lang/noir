import { PROVING_STATUS } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

import { makeBloatedProcessedTx, makeEmptyProcessedTestTx } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';

const logger = createDebugLogger('aztec:orchestrator-errors');

describe('prover/orchestrator/errors', () => {
  let context: TestContext;

  beforeEach(async () => {
    context = await TestContext.new(logger);
  }, 20_000);

  afterEach(async () => {
    await context.cleanup();
  });

  afterAll(async () => {});

  describe('errors', () => {
    it('throws if adding too many transactions', async () => {
      const txs = await Promise.all([
        makeBloatedProcessedTx(context.actualDb, 1),
        makeBloatedProcessedTx(context.actualDb, 2),
        makeBloatedProcessedTx(context.actualDb, 3),
        makeBloatedProcessedTx(context.actualDb, 4),
      ]);

      const blockTicket = await context.orchestrator.startNewBlock(
        txs.length,
        context.globalVariables,
        [],
        await makeEmptyProcessedTestTx(context.actualDb),
      );

      for (const tx of txs) {
        await context.orchestrator.addNewTx(tx);
      }

      await expect(
        async () => await context.orchestrator.addNewTx(await makeEmptyProcessedTestTx(context.actualDb)),
      ).rejects.toThrow('Rollup not accepting further transactions');

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(context.blockNumber);
    }, 40_000);

    it('throws if adding a transaction before start', async () => {
      await expect(
        async () => await context.orchestrator.addNewTx(await makeEmptyProcessedTestTx(context.actualDb)),
      ).rejects.toThrow(`Invalid proving state, call startNewBlock before adding transactions`);
    }, 1000);

    it('throws if completing a block before start', async () => {
      await expect(async () => await context.orchestrator.setBlockCompleted()).rejects.toThrow(
        'Invalid proving state, call startNewBlock before adding transactions or completing the block',
      );
    }, 1000);

    it('throws if finalising an incomplete block', async () => {
      await expect(async () => await context.orchestrator.finaliseBlock()).rejects.toThrow(
        'Invalid proving state, a block must be proven before it can be finalised',
      );
    }, 1000);

    it('throws if finalising an already finalised block', async () => {
      const txs = await Promise.all([
        makeEmptyProcessedTestTx(context.actualDb),
        makeEmptyProcessedTestTx(context.actualDb),
      ]);

      const blockTicket = await context.orchestrator.startNewBlock(
        txs.length,
        context.globalVariables,
        [],
        await makeEmptyProcessedTestTx(context.actualDb),
      );

      for (const tx of txs) {
        await context.orchestrator.addNewTx(tx);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();
      expect(finalisedBlock.block.number).toEqual(context.blockNumber);
      await expect(async () => await context.orchestrator.finaliseBlock()).rejects.toThrow('Block already finalised');
    }, 60000);

    it('throws if adding to a cancelled block', async () => {
      await context.orchestrator.startNewBlock(
        2,
        context.globalVariables,
        [],
        await makeEmptyProcessedTestTx(context.actualDb),
      );

      context.orchestrator.cancelBlock();

      await expect(
        async () => await context.orchestrator.addNewTx(await makeEmptyProcessedTestTx(context.actualDb)),
      ).rejects.toThrow('Rollup not accepting further transactions');
    }, 10000);

    it.each([[-4], [0], [1], [3], [8.1], [7]] as const)(
      'fails to start a block with %i transactions',
      async (blockSize: number) => {
        await expect(
          async () =>
            await context.orchestrator.startNewBlock(
              blockSize,
              context.globalVariables,
              [],
              await makeEmptyProcessedTestTx(context.actualDb),
            ),
        ).rejects.toThrow(`Length of txs for the block should be a power of two and at least two (got ${blockSize})`);
      },
    );

    it('rejects if too many l1 to l2 messages are provided', async () => {
      // Assemble a fake transaction
      const l1ToL2Messages = new Array(100).fill(new Fr(0n));
      await expect(
        async () =>
          await context.orchestrator.startNewBlock(
            2,
            context.globalVariables,
            l1ToL2Messages,
            await makeEmptyProcessedTestTx(context.actualDb),
          ),
      ).rejects.toThrow('Too many L1 to L2 messages');
    });
  });
});
