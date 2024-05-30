import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
  type PrivateKernelCircuitPublicInputs,
  PrivateKernelTailHints,
  sortByCounterGetSortedHints,
} from '@aztec/circuits.js';

export function buildPrivateKernelTailHints(publicInputs: PrivateKernelCircuitPublicInputs): PrivateKernelTailHints {
  const [sortedNoteHashes, sortedNoteHashesIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.newNoteHashes,
    MAX_NEW_NOTE_HASHES_PER_TX,
  );

  const [sortedNullifiers, sortedNullifiersIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.newNullifiers,
    MAX_NEW_NULLIFIERS_PER_TX,
  );

  const [sortedNoteEncryptedLogHashes, sortedNoteEncryptedLogHashesIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.noteEncryptedLogsHashes,
    MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  );

  const [sortedEncryptedLogHashes, sortedEncryptedLogHashesIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.encryptedLogsHashes,
    MAX_ENCRYPTED_LOGS_PER_TX,
  );

  const [sortedUnencryptedLogHashes, sortedUnencryptedLogHashesIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.unencryptedLogsHashes,
    MAX_UNENCRYPTED_LOGS_PER_TX,
  );

  const [sortedCallRequests, sortedCallRequestsIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.publicCallStack,
    MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
    {
      ascending: false,
      hintIndexesBy: 'sorted',
    },
  );

  return new PrivateKernelTailHints(
    sortedNoteHashes,
    sortedNoteHashesIndexes,
    sortedNullifiers,
    sortedNullifiersIndexes,
    sortedNoteEncryptedLogHashes,
    sortedNoteEncryptedLogHashesIndexes,
    sortedEncryptedLogHashes,
    sortedEncryptedLogHashesIndexes,
    sortedUnencryptedLogHashes,
    sortedUnencryptedLogHashesIndexes,
    sortedCallRequests,
    sortedCallRequestsIndexes,
  );
}
