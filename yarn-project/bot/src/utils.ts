import { type AztecAddress } from '@aztec/circuits.js';
import { type TokenContract } from '@aztec/noir-contracts.js/Token';

/**
 * Gets the private and public balance of the given token for the given address.
 * @param token - Token contract.
 * @param who - Address to get the balance for.
 * @returns - Private and public token balances as bigints.
 */
export async function getBalances(
  token: TokenContract,
  who: AztecAddress,
): Promise<{ privateBalance: bigint; publicBalance: bigint }> {
  const privateBalance = await token.methods.balance_of_private(who).simulate();
  const publicBalance = await token.methods.balance_of_public(who).simulate();
  return { privateBalance, publicBalance };
}
