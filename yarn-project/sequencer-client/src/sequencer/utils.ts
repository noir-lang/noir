import { Tx } from '@aztec/circuit-types';
import { CallRequest } from '@aztec/circuits.js';

/**
 * Looks at the side effects of a transaction and returns the highest counter
 * @param tx - A transaction
 * @returns The highest side effect counter in the transaction so far
 */
export function lastSideEffectCounter(tx: Tx): number {
  const sideEffectCounters = [
    ...tx.data.endNonRevertibleData.newNoteHashes,
    ...tx.data.endNonRevertibleData.newNullifiers,
    ...tx.data.endNonRevertibleData.publicCallStack,
    ...tx.data.end.newNoteHashes,
    ...tx.data.end.newNullifiers,
    ...tx.data.end.publicCallStack,
  ];

  let max = 0;
  for (const sideEffect of sideEffectCounters) {
    if (sideEffect instanceof CallRequest) {
      // look at both start and end counters because for enqueued public calls start > 0 while end === 0
      max = Math.max(max, sideEffect.startSideEffectCounter.toNumber(), sideEffect.endSideEffectCounter.toNumber());
    } else {
      max = Math.max(max, sideEffect.counter.toNumber());
    }
  }

  return max;
}
