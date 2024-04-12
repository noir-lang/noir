import { PROVING_STATUS, type ProvingFailure } from '@aztec/circuit-types';
import { type GlobalVariables, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { fr } from '@aztec/circuits.js/testing';
import { range } from '@aztec/foundation/array';
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

  beforeEach(async () => {
    const acvmConfig = await getConfig(logger);
    const simulationProvider = await getSimulationProvider({
      acvmWorkingDirectory: acvmConfig?.acvmWorkingDirectory,
      acvmBinaryPath: acvmConfig?.expectedAcvmPath,
    });
    prover = new TestCircuitProver(simulationProvider);

    builderDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
    builder = new ProvingOrchestrator(builderDb, prover, 1);
  }, 20_000);

  describe('lifecycle', () => {
    beforeEach(async () => {
      builder = await ProvingOrchestrator.new(builderDb, prover);
    });

    afterEach(async () => {
      await builder.stop();
    });

    it('cancels current block and switches to new ones', async () => {
      const txs1 = await Promise.all([makeBloatedProcessedTx(builderDb, 1), makeBloatedProcessedTx(builderDb, 2)]);

      const txs2 = await Promise.all([makeBloatedProcessedTx(builderDb, 3), makeBloatedProcessedTx(builderDb, 4)]);

      const globals1: GlobalVariables = makeGlobals(100);
      const globals2: GlobalVariables = makeGlobals(101);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket1 = await builder.startNewBlock(
        2,
        globals1,
        l1ToL2Messages,
        await makeEmptyProcessedTestTx(builderDb),
      );

      await builder.addNewTx(txs1[0]);
      await builder.addNewTx(txs1[1]);

      // Now we cancel the block. The first block will come to a stop as and when current proofs complete
      builder.cancelBlock();

      const result1 = await blockTicket1.provingPromise;

      // in all likelihood, the block will have a failure code as we cancelled it
      // however it may have actually completed proving before we cancelled in which case it could be a success code
      if (result1.status === PROVING_STATUS.FAILURE) {
        expect((result1 as ProvingFailure).reason).toBe('Proving cancelled');
      }

      await builderDb.rollback();

      const blockTicket2 = await builder.startNewBlock(
        2,
        globals2,
        l1ToL2Messages,
        await makeEmptyProcessedTestTx(builderDb),
      );

      await builder.addNewTx(txs2[0]);
      await builder.addNewTx(txs2[1]);

      const result2 = await blockTicket2.provingPromise;
      expect(result2.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(101);
    }, 20000);

    it('automatically cancels an incomplete block when starting a new one', async () => {
      const txs1 = await Promise.all([makeBloatedProcessedTx(builderDb, 1), makeBloatedProcessedTx(builderDb, 2)]);

      const txs2 = await Promise.all([makeBloatedProcessedTx(builderDb, 3), makeBloatedProcessedTx(builderDb, 4)]);

      const globals1: GlobalVariables = makeGlobals(100);
      const globals2: GlobalVariables = makeGlobals(101);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket1 = await builder.startNewBlock(
        2,
        globals1,
        l1ToL2Messages,
        await makeEmptyProcessedTestTx(builderDb),
      );

      await builder.addNewTx(txs1[0]);

      await builderDb.rollback();

      const blockTicket2 = await builder.startNewBlock(
        2,
        globals2,
        l1ToL2Messages,
        await makeEmptyProcessedTestTx(builderDb),
      );

      await builder.addNewTx(txs2[0]);
      await builder.addNewTx(txs2[1]);

      const result1 = await blockTicket1.provingPromise;
      expect(result1.status).toBe(PROVING_STATUS.FAILURE);
      expect((result1 as ProvingFailure).reason).toBe('Proving cancelled');

      const result2 = await blockTicket2.provingPromise;
      expect(result2.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await builder.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(101);
    }, 20000);
  });
});
