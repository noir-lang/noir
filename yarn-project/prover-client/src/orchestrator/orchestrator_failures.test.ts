import { PROVING_STATUS } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { WASMSimulator } from '@aztec/simulator';

import { jest } from '@jest/globals';

import { makeEmptyProcessedTestTx } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';
import { CircuitProverAgent } from '../prover-pool/circuit-prover-agent.js';
import { ProverPool } from '../prover-pool/prover-pool.js';
import { type CircuitProver } from '../prover/index.js';
import { TestCircuitProver } from '../prover/test_circuit_prover.js';
import { ProvingOrchestrator } from './orchestrator.js';

const logger = createDebugLogger('aztec:orchestrator-failures');

describe('prover/orchestrator/failures', () => {
  let context: TestContext;
  let orchestrator: ProvingOrchestrator;
  let proverPool: ProverPool;

  beforeEach(async () => {
    context = await TestContext.new(logger);
  }, 20_000);

  afterEach(async () => {
    await context.cleanup();
  });

  describe('error handling', () => {
    let mockProver: CircuitProver;

    beforeEach(async () => {
      mockProver = new TestCircuitProver(new WASMSimulator());
      proverPool = new ProverPool(1, i => new CircuitProverAgent(mockProver, 10, `${i}`));
      orchestrator = new ProvingOrchestrator(context.actualDb, proverPool.queue);
      await proverPool.start();
    });

    afterEach(async () => {
      await proverPool.stop();
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
          makeEmptyProcessedTestTx(context.actualDb),
          makeEmptyProcessedTestTx(context.actualDb),
          makeEmptyProcessedTestTx(context.actualDb),
          makeEmptyProcessedTestTx(context.actualDb),
        ]);

        const blockTicket = await orchestrator.startNewBlock(
          txs.length,
          context.globalVariables,
          [],
          await makeEmptyProcessedTestTx(context.actualDb),
        );

        for (const tx of txs) {
          await orchestrator.addNewTx(tx);
        }
        await expect(blockTicket.provingPromise).resolves.toEqual({ status: PROVING_STATUS.FAILURE, reason: message });
      },
      60000,
    );
  });
});
