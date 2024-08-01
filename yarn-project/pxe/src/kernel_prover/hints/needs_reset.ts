import {
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  type PrivateKernelCircuitPublicInputs,
  getNonEmptyItems,
} from '@aztec/circuits.js';
import { type ExecutionResult } from '@aztec/simulator';

export function needsReset(publicInputs: PrivateKernelCircuitPublicInputs, executionStack: ExecutionResult[]) {
  const nextIteration = executionStack[executionStack.length - 1];
  return (
    getNonEmptyItems(nextIteration.callStackItem.publicInputs.noteHashes).length +
      getNonEmptyItems(publicInputs.end.noteHashes).length >
      MAX_NOTE_HASHES_PER_TX ||
    getNonEmptyItems(nextIteration.callStackItem.publicInputs.nullifiers).length +
      getNonEmptyItems(publicInputs.end.nullifiers).length >
      MAX_NULLIFIERS_PER_TX ||
    getNonEmptyItems(nextIteration.callStackItem.publicInputs.noteEncryptedLogsHashes).length +
      getNonEmptyItems(publicInputs.end.noteEncryptedLogsHashes).length >
      MAX_NOTE_ENCRYPTED_LOGS_PER_TX ||
    getNonEmptyItems(nextIteration.callStackItem.publicInputs.noteHashReadRequests).length +
      getNonEmptyItems(publicInputs.validationRequests.noteHashReadRequests).length >
      MAX_NOTE_HASH_READ_REQUESTS_PER_TX ||
    getNonEmptyItems(nextIteration.callStackItem.publicInputs.nullifierReadRequests).length +
      getNonEmptyItems(publicInputs.validationRequests.nullifierReadRequests).length >
      MAX_NULLIFIER_READ_REQUESTS_PER_TX ||
    getNonEmptyItems(nextIteration.callStackItem.publicInputs.keyValidationRequestsAndGenerators).length +
      getNonEmptyItems(publicInputs.validationRequests.scopedKeyValidationRequestsAndGenerators).length >
      MAX_KEY_VALIDATION_REQUESTS_PER_TX
  );
}

function hasTransientNullifier(publicInputs: PrivateKernelCircuitPublicInputs) {
  const noteHashSet = new Set();
  publicInputs.end.noteHashes.forEach(n => noteHashSet.add(n.noteHash.value.toBigInt()));
  noteHashSet.delete(0n);
  return publicInputs.end.nullifiers.some(n => noteHashSet.has(n.nullifiedNoteHash.toBigInt()));
}

export function somethingToReset(publicInputs: PrivateKernelCircuitPublicInputs) {
  return (
    getNonEmptyItems(publicInputs.validationRequests.noteHashReadRequests).length > 0 ||
    getNonEmptyItems(publicInputs.validationRequests.nullifierReadRequests).length > 0 ||
    getNonEmptyItems(publicInputs.validationRequests.scopedKeyValidationRequestsAndGenerators).length > 0 ||
    hasTransientNullifier(publicInputs)
  );
}
