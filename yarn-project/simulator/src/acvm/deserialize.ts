import { Fr } from '@aztec/foundation/fields';

import { type ACVMField, type ACVMWitness } from './acvm_types.js';

/**
 * Converts an ACVM field to a Fr.
 * @param field - The ACVM field to convert.
 * @returns The Fr.
 */
export function fromACVMField(field: ACVMField): Fr {
  return Fr.fromBuffer(Buffer.from(field.slice(2), 'hex'));
}

/**
 * Converts a field to a number.
 * @param fr - The field to convert.
 * @returns The number.
 * TODO(#4102): Nuke this once block number is big int.
 */
export function frToNumber(fr: Fr): number {
  return Number(fr.value);
}

/**
 * Converts a field to a boolean.
 * @param fr - The field to convert.
 */
export function frToBoolean(fr: Fr): boolean {
  return fr.toBigInt() === BigInt(1);
}

/**
 * Transforms a witness map to its field elements.
 * @param witness - The witness to extract from.
 * @returns The return values.
 */
export function witnessMapToFields(witness: ACVMWitness): Fr[] {
  const sortedKeys = [...witness.keys()].sort((a, b) => a - b);
  return sortedKeys.map(key => witness.get(key)!).map(fromACVMField);
}
