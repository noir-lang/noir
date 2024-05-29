import { type PublicKernelCircuitPublicInputs } from '@aztec/circuits.js';

/**
 * Looks at the side effects of a transaction and returns the highest counter
 * @param tx - A transaction
 * @returns The highest side effect counter in the transaction so far
 */
export function lastSideEffectCounter(inputs: PublicKernelCircuitPublicInputs): number {
  const sideEffectCounters = [
    ...inputs.endNonRevertibleData.newNoteHashes,
    ...inputs.endNonRevertibleData.newNullifiers,
    ...inputs.endNonRevertibleData.noteEncryptedLogsHashes,
    ...inputs.endNonRevertibleData.encryptedLogsHashes,
    ...inputs.endNonRevertibleData.unencryptedLogsHashes,
    ...inputs.endNonRevertibleData.publicCallStack,
    ...inputs.endNonRevertibleData.publicDataUpdateRequests,
    ...inputs.end.newNoteHashes,
    ...inputs.end.newNullifiers,
    ...inputs.end.noteEncryptedLogsHashes,
    ...inputs.end.encryptedLogsHashes,
    ...inputs.end.unencryptedLogsHashes,
    ...inputs.end.publicCallStack,
    ...inputs.end.publicDataUpdateRequests,
  ];

  let max = 0;
  for (const sideEffect of sideEffectCounters) {
    if ('startSideEffectCounter' in sideEffect) {
      // look at both start and end counters because for enqueued public calls start > 0 while end === 0
      max = Math.max(max, sideEffect.startSideEffectCounter.toNumber(), sideEffect.endSideEffectCounter.toNumber());
    } else if ('counter' in sideEffect) {
      max = Math.max(max, sideEffect.counter);
    } else {
      throw new Error('Unknown side effect type');
    }
  }

  return max;
}
