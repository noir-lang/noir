import { AztecAddress } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';
import { TokenContractArtifact } from '@aztec/noir-contracts.js';
import { GasTokenAddress, GasTokenArtifact } from '@aztec/protocol-contracts/gas-token';
import { computeSlotForMapping } from '@aztec/simulator';

import { createCompatibleClient } from '../client.js';

export async function getBalance(
  address: AztecAddress,
  maybeTokenAddress: string | undefined,
  rpcUrl: string,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const tokenAddress = maybeTokenAddress ? AztecAddress.fromString(maybeTokenAddress) : GasTokenAddress;

  // Get private balance
  if (!tokenAddress.equals(GasTokenAddress)) {
    const result = await client.simulateUnconstrained(`balance_of_private`, [address], tokenAddress);
    log(`\nPrivate balance: ${result.toString()}`);
  }

  // TODO(#6707): For public balance, we cannot directly simulate a public function call, so we read directly from storage as a workaround
  const balancesStorageSlot = tokenAddress.equals(GasTokenAddress)
    ? GasTokenArtifact.storageLayout.balances.slot
    : TokenContractArtifact.storageLayout.public_balances.slot;
  const slot = computeSlotForMapping(balancesStorageSlot, address);
  const result = await client.getPublicStorageAt(tokenAddress, slot);
  log(`Public balance: ${result.toBigInt()}`);
}
