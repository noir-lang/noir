import { type AbiType } from './abi.js';

/**
 * Returns whether the ABI type is an Aztec or Ethereum Address defined in Aztec.nr.
 * @param abiType - Type to check.
 * @returns Boolean.
 */
export function isAddressStruct(abiType: AbiType) {
  return isEthAddressStruct(abiType) || isAztecAddressStruct(abiType);
}

/**
 * Returns whether the ABI type is an Ethereum Address defined in Aztec.nr.
 * @param abiType - Type to check.
 * @returns Boolean.
 */
export function isEthAddressStruct(abiType: AbiType) {
  return abiType.kind === 'struct' && abiType.path.endsWith('address::EthAddress');
}

/**
 * Returns whether the ABI type is an Aztec Address defined in Aztec.nr.
 * @param abiType - Type to check.
 * @returns Boolean.
 */
export function isAztecAddressStruct(abiType: AbiType) {
  return abiType.kind === 'struct' && abiType.path.endsWith('address::AztecAddress');
}

/**
 * Returns whether the ABI type is an Function Selector defined in Aztec.nr.
 * @param abiType - Type to check.
 * @returns Boolean.
 */
export function isFunctionSelectorStruct(abiType: AbiType) {
  return abiType.kind === 'struct' && abiType.path.endsWith('types::abis::function_selector::FunctionSelector');
}

/**
 * Returns whether the ABI type is a struct with a single `inner` field.
 * @param abiType - Type to check.
 */
export function isWrappedFieldStruct(abiType: AbiType) {
  return (
    abiType.kind === 'struct' &&
    abiType.fields.length === 1 &&
    abiType.fields[0].name === 'inner' &&
    abiType.fields[0].type.kind === 'field'
  );
}
