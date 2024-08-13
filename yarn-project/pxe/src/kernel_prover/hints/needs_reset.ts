import {
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  type PrivateKernelCircuitPublicInputs,
  countAccumulatedItems,
} from '@aztec/circuits.js';
import { type ExecutionResult } from '@aztec/simulator';

export function needsReset(publicInputs: PrivateKernelCircuitPublicInputs, executionStack: ExecutionResult[]) {
  const nextIteration = executionStack[executionStack.length - 1];
  return (
    countAccumulatedItems(nextIteration.callStackItem.publicInputs.noteHashes) +
      countAccumulatedItems(publicInputs.end.noteHashes) >
      MAX_NOTE_HASHES_PER_TX ||
    countAccumulatedItems(nextIteration.callStackItem.publicInputs.nullifiers) +
      countAccumulatedItems(publicInputs.end.nullifiers) >
      MAX_NULLIFIERS_PER_TX ||
    countAccumulatedItems(nextIteration.callStackItem.publicInputs.noteEncryptedLogsHashes) +
      countAccumulatedItems(publicInputs.end.noteEncryptedLogsHashes) >
      MAX_NOTE_ENCRYPTED_LOGS_PER_TX ||
    countAccumulatedItems(nextIteration.callStackItem.publicInputs.noteHashReadRequests) +
      countAccumulatedItems(publicInputs.validationRequests.noteHashReadRequests) >
      MAX_NOTE_HASH_READ_REQUESTS_PER_TX ||
    countAccumulatedItems(nextIteration.callStackItem.publicInputs.nullifierReadRequests) +
      countAccumulatedItems(publicInputs.validationRequests.nullifierReadRequests) >
      MAX_NULLIFIER_READ_REQUESTS_PER_TX ||
    countAccumulatedItems(nextIteration.callStackItem.publicInputs.keyValidationRequestsAndGenerators) +
      countAccumulatedItems(publicInputs.validationRequests.scopedKeyValidationRequestsAndGenerators) >
      MAX_KEY_VALIDATION_REQUESTS_PER_TX
  );
}

function hasTransientNullifier(publicInputs: PrivateKernelCircuitPublicInputs) {
  const noteHashSet = new Set();
  publicInputs.end.noteHashes.forEach(n => noteHashSet.add(n.noteHash.value.toBigInt()));
  noteHashSet.delete(0n);
  return publicInputs.end.nullifiers.some(n => noteHashSet.has(n.nullifiedNoteHash.toBigInt()));
}

export function needsFinalReset(publicInputs: PrivateKernelCircuitPublicInputs) {
  return (
    countAccumulatedItems(publicInputs.end.noteHashes) > 0 ||
    countAccumulatedItems(publicInputs.end.nullifiers) > 0 ||
    countAccumulatedItems(publicInputs.end.encryptedLogsHashes) > 0 ||
    countAccumulatedItems(publicInputs.validationRequests.noteHashReadRequests) > 0 ||
    countAccumulatedItems(publicInputs.validationRequests.nullifierReadRequests) > 0 ||
    countAccumulatedItems(publicInputs.validationRequests.scopedKeyValidationRequestsAndGenerators) > 0 ||
    hasTransientNullifier(publicInputs)
  );
}
