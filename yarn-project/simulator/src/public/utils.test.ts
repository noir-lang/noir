import { mockTx } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';

import { lastSideEffectCounter } from './utils.js';

describe('sequencer utils', () => {
  describe('lastSideEffectCounter', () => {
    it('correctly identifies the highest side effect counter in a transaction', () => {
      const tx = mockTx();
      // mockTx creates a Tx with side effect counts of all 0
      expect(lastSideEffectCounter(tx)).toBe(0);

      tx.data.forPublic!.endNonRevertibleData.newNoteHashes.at(-1)!.counter = new Fr(8);
      expect(lastSideEffectCounter(tx)).toBe(8);

      tx.data.forPublic!.endNonRevertibleData.publicCallStack.at(-1)!.startSideEffectCounter = new Fr(9);
      expect(lastSideEffectCounter(tx)).toBe(9);

      tx.data.forPublic!.end.newNoteHashes.at(-1)!.counter = new Fr(10);
      expect(lastSideEffectCounter(tx)).toBe(10);

      tx.data.forPublic!.end.newNullifiers.at(-1)!.counter = new Fr(11);
      expect(lastSideEffectCounter(tx)).toBe(11);
    });
  });
});
