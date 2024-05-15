import {
  type MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  type MAX_UNENCRYPTED_LOGS_PER_TX,
  type PrivateKernelCircuitPublicInputs,
  PrivateKernelTailHints,
  type SideEffect,
  type SideEffectType,
  sortByCounterGetSortedHints,
} from '@aztec/circuits.js';
import { type Tuple } from '@aztec/foundation/serialize';

/** @deprecated Use sortByCounterGetSortedHints instead */
function sortSideEffects<T extends SideEffectType, K extends number>(
  sideEffects: Tuple<T, K>,
): [Tuple<T, K>, Tuple<number, K>] {
  const sorted = sideEffects
    .map((sideEffect, index) => ({ sideEffect, index }))
    .sort((a, b) => {
      // Empty ones go to the right
      if (a.sideEffect.isEmpty()) {
        return 1;
      }
      return Number(a.sideEffect.counter.toBigInt() - b.sideEffect.counter.toBigInt());
    });

  const originalToSorted = sorted.map(() => 0);
  sorted.forEach(({ index }, i) => {
    originalToSorted[index] = i;
  });

  return [sorted.map(({ sideEffect }) => sideEffect) as Tuple<T, K>, originalToSorted as Tuple<number, K>];
}

export function buildPrivateKernelTailHints(publicInputs: PrivateKernelCircuitPublicInputs) {
  const [sortedNoteHashes, sortedNoteHashesIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.newNoteHashes,
    MAX_NEW_NOTE_HASHES_PER_TX,
  );

  const [sortedNullifiers, sortedNullifiersIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.newNullifiers,
    MAX_NEW_NULLIFIERS_PER_TX,
  );

  const [sortedEncryptedLogHashes, sortedEncryptedLogHashesIndexes] = sortSideEffects<
    SideEffect,
    typeof MAX_ENCRYPTED_LOGS_PER_TX
  >(publicInputs.end.encryptedLogsHashes);

  const [sortedUnencryptedLogHashes, sortedUnencryptedLogHashesIndexes] = sortSideEffects<
    SideEffect,
    typeof MAX_UNENCRYPTED_LOGS_PER_TX
  >(publicInputs.end.unencryptedLogsHashes);

  return new PrivateKernelTailHints(
    sortedNoteHashes,
    sortedNoteHashesIndexes,
    sortedNullifiers,
    sortedNullifiersIndexes,
    sortedEncryptedLogHashes,
    sortedEncryptedLogHashesIndexes,
    sortedUnencryptedLogHashes,
    sortedUnencryptedLogHashesIndexes,
  );
}
