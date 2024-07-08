import { PROVING_STATUS, type ProvingFailure, type ServerCircuitProver } from '@aztec/circuit-types';
import {
  type GlobalVariables,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
} from '@aztec/circuits.js';
import { fr, makeGlobalVariables } from '@aztec/circuits.js/testing';
import { range } from '@aztec/foundation/array';
import { createDebugLogger } from '@aztec/foundation/log';
import { type PromiseWithResolvers, promiseWithResolvers } from '@aztec/foundation/promise';
import { sleep } from '@aztec/foundation/sleep';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { jest } from '@jest/globals';

import { TestCircuitProver } from '../../../bb-prover/src/test/test_circuit_prover.js';
import { makeBloatedProcessedTx, makeGlobals } from '../mocks/fixtures.js';
import { TestContext } from '../mocks/test_context.js';
import { ProvingOrchestrator } from './orchestrator.js';

const logger = createDebugLogger('aztec:orchestrator-lifecycle');

describe('prover/orchestrator/lifecycle', () => {
  let context: TestContext;

  beforeEach(async () => {
    context = await TestContext.new(logger);
  });

  afterEach(async () => {
    await context.cleanup();
  });

  describe('lifecycle', () => {
    it('cancels current block and switches to new ones', async () => {
      const txs1 = await Promise.all([
        makeBloatedProcessedTx(context.actualDb, 1),
        makeBloatedProcessedTx(context.actualDb, 2),
      ]);

      const txs2 = await Promise.all([
        makeBloatedProcessedTx(context.actualDb, 3),
        makeBloatedProcessedTx(context.actualDb, 4),
      ]);

      const globals1: GlobalVariables = makeGlobals(100);
      const globals2: GlobalVariables = makeGlobals(101);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket1 = await context.orchestrator.startNewBlock(2, globals1, l1ToL2Messages);

      await context.orchestrator.addNewTx(txs1[0]);
      await context.orchestrator.addNewTx(txs1[1]);

      // Now we cancel the block. The first block will come to a stop as and when current proofs complete
      context.orchestrator.cancelBlock();

      const result1 = await blockTicket1.provingPromise;

      // in all likelihood, the block will have a failure code as we cancelled it
      // however it may have actually completed proving before we cancelled in which case it could be a success code
      if (result1.status === PROVING_STATUS.FAILURE) {
        expect((result1 as ProvingFailure).reason).toBe('Proving cancelled');
      }

      await context.actualDb.rollback();

      const blockTicket2 = await context.orchestrator.startNewBlock(2, globals2, l1ToL2Messages);

      await context.orchestrator.addNewTx(txs2[0]);
      await context.orchestrator.addNewTx(txs2[1]);

      const result2 = await blockTicket2.provingPromise;
      expect(result2.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(101);
    });

    it('automatically cancels an incomplete block when starting a new one', async () => {
      const txs1 = await Promise.all([
        makeBloatedProcessedTx(context.actualDb, 1),
        makeBloatedProcessedTx(context.actualDb, 2),
      ]);

      const txs2 = await Promise.all([
        makeBloatedProcessedTx(context.actualDb, 3),
        makeBloatedProcessedTx(context.actualDb, 4),
      ]);

      const globals1: GlobalVariables = makeGlobals(100);
      const globals2: GlobalVariables = makeGlobals(101);

      const l1ToL2Messages = range(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, 1 + 0x400).map(fr);

      const blockTicket1 = await context.orchestrator.startNewBlock(2, globals1, l1ToL2Messages);

      await context.orchestrator.addNewTx(txs1[0]);

      await context.actualDb.rollback();

      const blockTicket2 = await context.orchestrator.startNewBlock(2, globals2, l1ToL2Messages);

      await context.orchestrator.addNewTx(txs2[0]);
      await context.orchestrator.addNewTx(txs2[1]);

      const result1 = await blockTicket1.provingPromise;
      expect(result1.status).toBe(PROVING_STATUS.FAILURE);
      expect((result1 as ProvingFailure).reason).toBe('Proving cancelled');

      const result2 = await blockTicket2.provingPromise;
      expect(result2.status).toBe(PROVING_STATUS.SUCCESS);
      const finalisedBlock = await context.orchestrator.finaliseBlock();

      expect(finalisedBlock.block.number).toEqual(101);
    }, 60000);

    it('cancels proving requests', async () => {
      const prover: ServerCircuitProver = new TestCircuitProver(new NoopTelemetryClient());
      const orchestrator = new ProvingOrchestrator(context.actualDb, prover, new NoopTelemetryClient());

      const spy = jest.spyOn(prover, 'getBaseParityProof');
      const deferredPromises: PromiseWithResolvers<any>[] = [];
      spy.mockImplementation(() => {
        const deferred = promiseWithResolvers<any>();
        deferredPromises.push(deferred);
        return deferred.promise;
      });
      await orchestrator.startNewBlock(2, makeGlobalVariables(1), []);

      await sleep(1);

      expect(spy).toHaveBeenCalledTimes(NUM_BASE_PARITY_PER_ROOT_PARITY);
      expect(spy.mock.calls.every(([_, signal]) => !signal?.aborted)).toBeTruthy();

      orchestrator.cancelBlock();
      expect(spy.mock.calls.every(([_, signal]) => signal?.aborted)).toBeTruthy();
    });
  });
});
