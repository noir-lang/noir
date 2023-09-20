import { ABIType } from './abi.js';

/**
 * Returns whether the ABI type is an Aztec or Ethereum Address defined in Aztec.nr.
 * @param abiType - Type to check.
 * @returns Boolean.
 */
export function isAddressStruct(abiType: ABIType) {
  return isEthereumAddressStruct(abiType) || isAztecAddressStruct(abiType);
}

/**
 * Returns whether the ABI type is an Ethereum Address defined in Aztec.nr.
 * @param abiType - Type to check.
 * @returns Boolean.
 */
export function isEthereumAddressStruct(abiType: ABIType) {
  return abiType.kind === 'struct' && abiType.path.endsWith('::types::address::EthereumAddress');
}

/**
 * Returns whether the ABI type is an Aztec Address defined in Aztec.nr.
 * @param abiType - Type to check.
 * @returns Boolean.
 */
export function isAztecAddressStruct(abiType: ABIType) {
  return abiType.kind === 'struct' && abiType.path.endsWith('::types::address::AztecAddress');
}
