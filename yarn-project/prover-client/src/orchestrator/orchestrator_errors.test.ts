import { PROVING_STATUS } from '@aztec/circuit-types';
import { Fr, type GlobalVariables } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { type MemDown, default as memdown } from 'memdown';

import {
  getConfig,
  getSimulationProvider,
  makeBloatedProcessedTx,
  makeEmptyProcessedTestTx,
  makeGlobals,
} from '../mocks/fixtures.js';
import { TestCircuitProver } from '../prover/test_circuit_prover.js';
import { ProvingOrchestrator } from './orchestrator.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

const logger = createDebugLogger('aztec:orchestrator-test');

describe('prover/orchestrator', () => {
  let builder: ProvingOrchestrator;
  let builderDb: MerkleTreeOperations;

  let prover: TestCircuitProver;

  let blockNumber: number;

  let globalVariables: GlobalVariables;

  beforeEach(async () => {
    blockNumber = 3;
    globalVariables = makeGlobals(blockNumber);

    const acvmConfig = await getConfig(logger);
    const simulationProvider = await getSimulationProvider({
      acvmWorkingDirectory: acvmConfig?.acvmWorkingDirectory,
      acvmBinaryPath: acvmConfig?.expectedAcvmPath,
    });
    prover = new TestCircuitProver(simulationProvider);

    builderDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
    builder = new ProvingOrchestrator(builderDb, prover, 1);
  }, 20_000);

  describe('errors', () => {
    beforeEach(async () => {
      builder = await ProvingOrchestrator.new(builderDb, prover);
    });

    afterEach(async () => {
      await builder.stop();
    });

    it('throws if adding too many transactions', async () => {
      const txs = await Promise.all([
        makeBloatedProcessedTx(builderDb, 1),
        makeBloatedProcessedTx(builderDb, 2),
        makeBloatedProcessedTx(builderDb, 3),
        makeBloatedProcessedTx(builderDb, 4),
      ]);

      const blockTicket = await builder.startNewBlock(
        txs.length,
        globalVariables,
        [],
        await makeEmptyProcessedTestTx(builderDb),
      );

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      await expect(async () => await builder.addNewTx(await makeEmptyProcessedTestTx(builderDb))).rejects.toThrow(
        'Rollup not accepting further transactions',
      );

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(blockNumber);
    }, 30_000);

    it('throws if adding a transaction before start', async () => {
      await expect(async () => await builder.addNewTx(await makeEmptyProcessedTestTx(builderDb))).rejects.toThrow(
        `Invalid proving state, call startNewBlock before adding transactions`,
      );
    }, 1000);

    it('throws if completing a block before start', async () => {
      await expect(async () => await builder.setBlockCompleted()).rejects.toThrow(
        'Invalid proving state, call startNewBlock before adding transactions or completing the block',
      );
    }, 1000);

    it('throws if finalising an incomplete block', async () => {
      await expect(async () => await builder.finaliseBlock()).rejects.toThrow(
        'Invalid proving state, a block must be proven before it can be finalised',
      );
    }, 1000);

    it('throws if finalising an already finalised block', async () => {
      const txs = await Promise.all([makeEmptyProcessedTestTx(builderDb), makeEmptyProcessedTestTx(builderDb)]);

      const blockTicket = await builder.startNewBlock(
        txs.length,
        globalVariables,
        [],
        await makeEmptyProcessedTestTx(builderDb),
      );

      for (const tx of txs) {
        await builder.addNewTx(tx);
      }

      const result = await blockTicket.provingPromise;
      expect(result.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();
      expect(finalisedBlock.block.number).toEqual(blockNumber);
      await expect(async () => await builder.finaliseBlock()).rejects.toThrow('Block already finalised');
    }, 60000);

    it('throws if adding to a cancelled block', async () => {
      await builder.startNewBlock(2, globalVariables, [], await makeEmptyProcessedTestTx(builderDb));

      builder.cancelBlock();

      await expect(async () => await builder.addNewTx(await makeEmptyProcessedTestTx(builderDb))).rejects.toThrow(
        'Rollup not accepting further transactions',
      );
    }, 10000);

    it.each([[-4], [0], [1], [3], [8.1], [7]] as const)(
      'fails to start a block with %i transactions',
      async (blockSize: number) => {
        await expect(
          async () =>
            await builder.startNewBlock(blockSize, globalVariables, [], await makeEmptyProcessedTestTx(builderDb)),
        ).rejects.toThrow(`Length of txs for the block should be a power of two and at least two (got ${blockSize})`);
      },
    );

    it('rejects if too many l1 to l2 messages are provided', async () => {
      // Assemble a fake transaction
      const l1ToL2Messages = new Array(100).fill(new Fr(0n));
      await expect(
        async () =>
          await builder.startNewBlock(2, globalVariables, l1ToL2Messages, await makeEmptyProcessedTestTx(builderDb)),
      ).rejects.toThrow('Too many L1 to L2 messages');
    });
  });
});
