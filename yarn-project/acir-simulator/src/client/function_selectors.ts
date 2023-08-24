import { FunctionSelector } from '@aztec/circuits.js';

export const computeNoteHashAndNullifierSignature = 'compute_note_hash_and_nullifier(field,field,field,array)';

export const computeNoteHashAndNullifierSelector = FunctionSelector.fromSignature(computeNoteHashAndNullifierSignature);
