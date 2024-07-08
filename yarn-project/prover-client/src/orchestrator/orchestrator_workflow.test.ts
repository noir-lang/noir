import { type ServerCircuitProver } from '@aztec/circuit-types';
import {
  Fr,
  NESTED_RECURSIVE_PROOF_LENGTH,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
  RECURSIVE_PROOF_LENGTH,
  type RootParityInput,
} from '@aztec/circuits.js';
import { makeGlobalVariables, makeRootParityInput } from '@aztec/circuits.js/testing';
import { promiseWithResolvers } from '@aztec/foundation/promise';
import { sleep } from '@aztec/foundation/sleep';
import { openTmpStore } from '@aztec/kv-store/utils';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';
import { type MerkleTreeOperations, MerkleTrees } from '@aztec/world-state';

import { type MockProxy, mock } from 'jest-mock-extended';

import { ProvingOrchestrator } from './orchestrator.js';

describe('prover/orchestrator', () => {
  describe('workflow', () => {
    let orchestrator: ProvingOrchestrator;
    let mockProver: MockProxy<ServerCircuitProver>;
    let actualDb: MerkleTreeOperations;
    beforeEach(async () => {
      actualDb = await MerkleTrees.new(openTmpStore()).then(t => t.asLatest());
      mockProver = mock<ServerCircuitProver>();
      orchestrator = new ProvingOrchestrator(actualDb, mockProver, new NoopTelemetryClient());
    });

    it('calls root parity circuit only when ready', async () => {
      // create a custom L2 to L1 message
      const message = Fr.random();

      // and delay its proof
      const pendingBaseParityResult = promiseWithResolvers<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>>();
      const expectedBaseParityResult = makeRootParityInput(RECURSIVE_PROOF_LENGTH, 0xff);

      mockProver.getRootParityProof.mockResolvedValue(makeRootParityInput(NESTED_RECURSIVE_PROOF_LENGTH));

      mockProver.getBaseParityProof.mockImplementation(inputs => {
        if (inputs.msgs[0].equals(message)) {
          return pendingBaseParityResult.promise;
        } else {
          return Promise.resolve(makeRootParityInput(RECURSIVE_PROOF_LENGTH));
        }
      });

      await orchestrator.startNewBlock(2, makeGlobalVariables(1), [message]);

      await sleep(10);
      expect(mockProver.getBaseParityProof).toHaveBeenCalledTimes(NUM_BASE_PARITY_PER_ROOT_PARITY);
      expect(mockProver.getRootParityProof).not.toHaveBeenCalled();

      await sleep(10);
      // even now the root parity should not have been called
      expect(mockProver.getRootParityProof).not.toHaveBeenCalled();

      // only after the base parity proof is resolved, the root parity should be called
      pendingBaseParityResult.resolve(expectedBaseParityResult);

      // give the orchestrator a chance to calls its callbacks
      await sleep(10);
      expect(mockProver.getRootParityProof).toHaveBeenCalledTimes(1);

      orchestrator.cancelBlock();
    });
  });
});
