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
  });

  afterEach(async () => {
    await context.cleanup();
  });

  afterAll(async () => {});

  describe('errors', () => {
    it('throws if adding too many transactions', async () => {
      const txs = [
        makeBloatedProcessedTx(context.actualDb, 1),
        makeBloatedProcessedTx(context.actualDb, 2),
        makeBloatedProcessedTx(context.actualDb, 3),
        makeBloatedProcessedTx(context.actualDb, 4),
      ];

      const blockTicket = await context.orchestrator.startNewBlock(txs.length, context.globalVariables, []);

      for (const tx of txs) {
        await context.orchestrator.addNewTx(tx);
      }

      await expect(
        async () => await context.orchestrator.addNewTx(makeEmptyProcessedTestTx(context.actualDb)),
      ).rejects.toThrow('Rollup not accepting further transactions');

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(context.blockNumber);
    });

    it('throws if adding a transaction before start', async () => {
      await expect(
        async () => await context.orchestrator.addNewTx(makeEmptyProcessedTestTx(context.actualDb)),
      ).rejects.toThrow(`Invalid proving state, call startNewBlock before adding transactions`);
    });

    it('throws if completing a block before start', async () => {
      await expect(async () => await context.orchestrator.setBlockCompleted()).rejects.toThrow(
        'Invalid proving state, call startNewBlock before adding transactions or completing the block',
      );
    });

    it('throws if finalising an incomplete block', async () => {
      await expect(async () => await context.orchestrator.finaliseBlock()).rejects.toThrow(
        'Invalid proving state, a block must be proven before it can be finalised',
      );
    });

    it('throws if setting an incomplete block completed', async () => {
      await context.orchestrator.startNewBlock(3, context.globalVariables, []);
      await expect(async () => await context.orchestrator.setBlockCompleted()).rejects.toThrow(
        `Block not ready for completion: expecting ${3} more transactions.`,
      );
    });

    it('throws if finalising an already finalised block', async () => {
      const txs = await Promise.all([
        makeEmptyProcessedTestTx(context.actualDb),
        makeEmptyProcessedTestTx(context.actualDb),
      ]);

      const blockTicket = await context.orchestrator.startNewBlock(txs.length, context.globalVariables, []);

      await context.orchestrator.setBlockCompleted();

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();
      expect(finalisedBlock.block.number).toEqual(context.blockNumber);
      await expect(async () => await context.orchestrator.finaliseBlock()).rejects.toThrow('Block already finalised');
    });

    it('throws if adding to a cancelled block', async () => {
      await context.orchestrator.startNewBlock(2, context.globalVariables, []);

      context.orchestrator.cancelBlock();

      await expect(
        async () => await context.orchestrator.addNewTx(makeEmptyProcessedTestTx(context.actualDb)),
      ).rejects.toThrow('Rollup not accepting further transactions');
    });

    it.each([[-4], [0], [1], [8.1]] as const)(
      'fails to start a block with %i transactions',
      async (blockSize: number) => {
        await expect(
          async () => await context.orchestrator.startNewBlock(blockSize, context.globalVariables, []),
        ).rejects.toThrow(`Length of txs for the block should be at least two (got ${blockSize})`);
      },
    );

    it('rejects if too many l1 to l2 messages are provided', async () => {
      // Assemble a fake transaction
      const l1ToL2Messages = new Array(100).fill(new Fr(0n));
      await expect(
        async () => await context.orchestrator.startNewBlock(2, context.globalVariables, l1ToL2Messages),
      ).rejects.toThrow('Too many L1 to L2 messages');
    });
  });
});
