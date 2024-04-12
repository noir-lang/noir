import { PROVING_STATUS, type ProcessedTx } from '@aztec/circuit-types';
import { Fr, type GlobalVariables } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { openTmpStore } from '@aztec/kv-store/utils';
import { WASMSimulator } from '@aztec/simulator';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { jest } from '@jest/globals';
import { type MemDown, default as memdown } from 'memdown';

import { getConfig, getSimulationProvider, makeEmptyProcessedTx, makeGlobals } from '../mocks/fixtures.js';
import { type CircuitProver } from '../prover/index.js';
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

  const makeEmptyProcessedTestTx = (): Promise<ProcessedTx> => {
    return makeEmptyProcessedTx(builderDb, Fr.ZERO, Fr.ZERO);
  };

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

  describe('error handling', () => {
    let mockProver: CircuitProver;

    beforeEach(async () => {
      mockProver = new TestCircuitProver(new WASMSimulator());
      builder = await ProvingOrchestrator.new(builderDb, mockProver);
    });

    it.each([
      [
        'Base Rollup Failed',
        () => {
          jest.spyOn(mockProver, 'getBaseRollupProof').mockRejectedValue('Base Rollup Failed');
        },
      ],
      [
        'Merge Rollup Failed',
        () => {
          jest.spyOn(mockProver, 'getMergeRollupProof').mockRejectedValue('Merge Rollup Failed');
        },
      ],
      [
        'Root Rollup Failed',
        () => {
          jest.spyOn(mockProver, 'getRootRollupProof').mockRejectedValue('Root Rollup Failed');
        },
      ],
      [
        'Base Parity Failed',
        () => {
          jest.spyOn(mockProver, 'getBaseParityProof').mockRejectedValue('Base Parity Failed');
        },
      ],
      [
        'Root Parity Failed',
        () => {
          jest.spyOn(mockProver, 'getRootParityProof').mockRejectedValue('Root Parity Failed');
        },
      ],
    ] as const)(
      'handles a %s error',
      async (message: string, fn: () => void) => {
        fn();
        const txs = await Promise.all([
          makeEmptyProcessedTestTx(),
          makeEmptyProcessedTestTx(),
          makeEmptyProcessedTestTx(),
          makeEmptyProcessedTestTx(),
        ]);

        const blockTicket = await builder.startNewBlock(
          txs.length,
          globalVariables,
          [],
          await makeEmptyProcessedTestTx(),
        );

        for (const tx of txs) {
          await builder.addNewTx(tx);
        }
        await expect(blockTicket.provingPromise).resolves.toEqual({ status: PROVING_STATUS.FAILURE, reason: message });
      },
      60000,
    );

    afterEach(async () => {
      await builder.stop();
    });
  });
});
