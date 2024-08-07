import { FEE_JUICE_ADDRESS } from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot, deriveStorageSlotInMap } from '@aztec/circuits.js/hash';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { FeeJuiceArtifact } from '@aztec/protocol-contracts/fee-juice';

/**
 * Computes the storage slot within the Fee Juice contract for the balance of the fee payer.
 */
export function computeFeePayerBalanceStorageSlot(feePayer: AztecAddress) {
  return deriveStorageSlotInMap(FeeJuiceArtifact.storageLayout.balances.slot, feePayer);
}

/**
 * Computes the leaf slot in the public data tree for the balance of the fee payer in the Fee Juice.
 */
export function computeFeePayerBalanceLeafSlot(feePayer: AztecAddress): Fr {
  if (feePayer.isZero()) {
    return Fr.ZERO;
  }
  const feeJuice = AztecAddress.fromBigInt(FEE_JUICE_ADDRESS);
  const balanceSlot = computeFeePayerBalanceStorageSlot(feePayer);
  return computePublicDataTreeLeafSlot(feeJuice, balanceSlot);
}
