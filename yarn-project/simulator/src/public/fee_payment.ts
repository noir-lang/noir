import { GAS_TOKEN_ADDRESS } from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot, deriveStorageSlotInMap } from '@aztec/circuits.js/hash';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { GasTokenArtifact } from '@aztec/protocol-contracts/gas-token';

/**
 * Computes the storage slot within the gas token contract for the balance of the fee payer.
 */
export function computeFeePayerBalanceStorageSlot(feePayer: AztecAddress) {
  return deriveStorageSlotInMap(GasTokenArtifact.storageLayout.balances.slot, feePayer);
}

/**
 * Computes the leaf slot in the public data tree for the balance of the fee payer in the gas token.
 */
export function computeFeePayerBalanceLeafSlot(feePayer: AztecAddress): Fr {
  if (feePayer.isZero()) {
    return Fr.ZERO;
  }
  const gasToken = AztecAddress.fromBigInt(GAS_TOKEN_ADDRESS);
  const balanceSlot = computeFeePayerBalanceStorageSlot(feePayer);
  return computePublicDataTreeLeafSlot(gasToken, balanceSlot);
}
