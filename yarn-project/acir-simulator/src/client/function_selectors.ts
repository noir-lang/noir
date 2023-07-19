import { FUNCTION_SELECTOR_NUM_BYTES } from '@aztec/circuits.js';
import { computeFunctionSelector } from '@aztec/foundation/abi';

export const computeNoteHashAndNullifierSignature = 'stev(field,field,field,array)';

export const computeNoteHashAndNullifierSelector = computeFunctionSelector(
  computeNoteHashAndNullifierSignature,
  FUNCTION_SELECTOR_NUM_BYTES,
);
