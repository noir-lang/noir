import { Fr } from '@aztec/circuits.js';
import { makePublicKernelCircuitPublicInputs } from '@aztec/circuits.js/testing';

import { lastSideEffectCounter } from './utils.js';

describe('sequencer utils', () => {
  describe('lastSideEffectCounter', () => {
    it('correctly identifies the highest side effect counter in a transaction', () => {
      const inputs = makePublicKernelCircuitPublicInputs();

      const startingCounter = lastSideEffectCounter(inputs);

      inputs.endNonRevertibleData.newNoteHashes.at(-1)!.counter = startingCounter + 1;
      expect(lastSideEffectCounter(inputs)).toBe(startingCounter + 1);

      inputs.endNonRevertibleData.publicCallStack.at(-1)!.startSideEffectCounter = new Fr(startingCounter + 2);
      expect(lastSideEffectCounter(inputs)).toBe(startingCounter + 2);

      inputs.end.newNoteHashes.at(-1)!.counter = startingCounter + 3;
      expect(lastSideEffectCounter(inputs)).toBe(startingCounter + 3);

      inputs.end.newNullifiers.at(-1)!.counter = startingCounter + 4;
      expect(lastSideEffectCounter(inputs)).toBe(startingCounter + 4);
    });
  });
});
