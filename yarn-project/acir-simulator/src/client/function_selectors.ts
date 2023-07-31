import { FUNCTION_SELECTOR_NUM_BYTES } from '@aztec/circuits.js';
import { computeFunctionSelector } from '@aztec/foundation/abi';

export const computeNoteHashAndNullifierSignature = 'compute_note_hash_and_nullifier(field,field,field,array)';

export const computeNoteHashAndNullifierSelector = computeFunctionSelector(
  computeNoteHashAndNullifierSignature,
  FUNCTION_SELECTOR_NUM_BYTES,
);
