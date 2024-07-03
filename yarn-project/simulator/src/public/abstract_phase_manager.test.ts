import { Fr } from '@aztec/circuits.js';
import { makePublicKernelCircuitPublicInputs } from '@aztec/circuits.js/testing';

import { AbstractPhaseManager } from './abstract_phase_manager.js';

describe('AbstractPhaseManager utils', () => {
  describe('getMaxSideEffectCounter', () => {
    it('correctly identifies the highest side effect counter in a transaction', () => {
      const inputs = makePublicKernelCircuitPublicInputs();

      const startingCounter = AbstractPhaseManager.getMaxSideEffectCounter(inputs);

      inputs.endNonRevertibleData.noteHashes.at(-1)!.counter = startingCounter + 1;
      expect(AbstractPhaseManager.getMaxSideEffectCounter(inputs)).toBe(startingCounter + 1);

      inputs.endNonRevertibleData.publicCallStack.at(-1)!.startSideEffectCounter = new Fr(startingCounter + 2);
      expect(AbstractPhaseManager.getMaxSideEffectCounter(inputs)).toBe(startingCounter + 2);

      inputs.end.noteHashes.at(-1)!.counter = startingCounter + 3;
      expect(AbstractPhaseManager.getMaxSideEffectCounter(inputs)).toBe(startingCounter + 3);

      inputs.end.nullifiers.at(-1)!.counter = startingCounter + 4;
      expect(AbstractPhaseManager.getMaxSideEffectCounter(inputs)).toBe(startingCounter + 4);
    });
  });
});
