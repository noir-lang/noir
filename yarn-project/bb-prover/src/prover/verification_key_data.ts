import { type Fr, type VERIFICATION_KEY_LENGTH_IN_FIELDS } from '@aztec/circuits.js';
import { type Tuple } from '@aztec/foundation/serialize';

export const AGGREGATION_OBJECT_SIZE = 16;
export const CIRCUIT_SIZE_INDEX = 3;
export const CIRCUIT_PUBLIC_INPUTS_INDEX = 4;
export const CIRCUIT_RECURSIVE_INDEX = 5;

export type VerificationKeyData = {
  hash: Fr;
  keyAsFields: Tuple<Fr, typeof VERIFICATION_KEY_LENGTH_IN_FIELDS>;
  keyAsBytes: Buffer;
  numPublicInputs: number;
  circuitSize: number;
  isRecursive: boolean;
};
