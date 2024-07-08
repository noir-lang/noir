import { PROVING_STATUS, type ServerCircuitProver } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { WASMSimulator } from '@aztec/simulator';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { jest } from '@jest/globals';

import { TestCircuitProver } from '../../../bb-prover/src/test/test_circuit_prover.js';
import { makeBloatedProcessedTx } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';
import { ProvingOrchestrator } from './orchestrator.js';

const logger = createDebugLogger('aztec:orchestrator-failures');

describe('prover/orchestrator/failures', () => {
  let context: TestContext;
  let orchestrator: ProvingOrchestrator;

  beforeEach(async () => {
    context = await TestContext.new(logger);
  });

  afterEach(async () => {
    await context.cleanup();
  });

  describe('error handling', () => {
    let mockProver: ServerCircuitProver;

    beforeEach(() => {
      mockProver = new TestCircuitProver(new NoopTelemetryClient(), new WASMSimulator());
      orchestrator = new ProvingOrchestrator(context.actualDb, mockProver, new NoopTelemetryClient());
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
    ] as const)('handles a %s error', async (message: string, fn: () => void) => {
      fn();
      const txs = await Promise.all([
        makeBloatedProcessedTx(context.actualDb, 1),
        makeBloatedProcessedTx(context.actualDb, 2),
        makeBloatedProcessedTx(context.actualDb, 3),
      ]);

      const blockTicket = await orchestrator.startNewBlock(txs.length, context.globalVariables, []);

      for (const tx of txs) {
        await orchestrator.addNewTx(tx);
      }
      await expect(blockTicket.provingPromise).resolves.toEqual({ status: PROVING_STATUS.FAILURE, reason: message });
    });
  });
});
