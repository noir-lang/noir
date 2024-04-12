import { PROVING_STATUS } from '@aztec/circuit-types';
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

  describe('multiple blocks', () => {
    beforeEach(async () => {
      builder = await ProvingOrchestrator.new(builderDb, prover);
    });

    afterEach(async () => {
      await builder.stop();
    });

    it('builds multiple blocks in sequence', async () => {
      const numBlocks = 5;
      let header = await builderDb.buildInitialHeader();

      for (let i = 0; i < numBlocks; i++) {
        const tx = await makeBloatedProcessedTx(builderDb, i + 1);
        const emptyTx = await makeEmptyProcessedTestTx(builderDb);
        tx.data.constants.historicalHeader = header;
        emptyTx.data.constants.historicalHeader = header;

        const blockNum = i + 1000;

        const globals = makeGlobals(blockNum);

        // This will need to be a 2 tx block
        const blockTicket = await builder.startNewBlock(2, globals, [], emptyTx);

        await builder.addNewTx(tx);

        //  we need to complete the block as we have not added a full set of txs
        await builder.setBlockCompleted();

        const result = await blockTicket.provingPromise;
        expect(result.status).toBe(PROVING_STATUS.SUCCESS);
        const finalisedBlock = await builder.finaliseBlock();

        expect(finalisedBlock.block.number).toEqual(blockNum);
        header = finalisedBlock.block.header;

        await builderDb.commit();
      }
    }, 60_000);
  });
});
