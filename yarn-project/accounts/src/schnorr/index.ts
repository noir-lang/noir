/**
 * The `@aztec/accounts/schnorr` export provides an account contract implementation that uses Schnorr signatures with a Grumpkin key for authentication, and a separate Grumpkin key for encryption.
 * This is the suggested account contract type for most use cases within Aztec.
 *
 * @packageDocumentation
 */
import { AccountManager, type Salt } from '@aztec/aztec.js/account';
import { type AccountWallet, getWallet } from '@aztec/aztec.js/wallet';
import { type GrumpkinScalar, type PXE } from '@aztec/circuit-types';
import { type AztecAddress, type Fr } from '@aztec/circuits.js';

import { SchnorrAccountContract } from './account_contract.js';

export { SchnorrAccountContract };

export { SchnorrAccountContractArtifact } from './artifact.js';

/**
 * Creates an Account Manager that relies on a Grumpkin signing key for authentication.
 * @param pxe - An PXE server instance.
 * @param secretKey - Secret key used to derive all the keystore keys.
 * @param signingPrivateKey - Grumpkin key used for signing transactions.
 * @param salt - Deployment salt.
 */
export function getSchnorrAccount(
  pxe: PXE,
  secretKey: Fr,
  signingPrivateKey: GrumpkinScalar,
  salt?: Salt,
): AccountManager {
  return new AccountManager(pxe, secretKey, new SchnorrAccountContract(signingPrivateKey), salt);
}

/**
 * Gets a wallet for an already registered account using Schnorr signatures.
 * @param pxe - An PXE server instance.
 * @param address - Address for the account.
 * @param signingPrivateKey - Grumpkin key used for signing transactions.
 * @returns A wallet for this account that can be used to interact with a contract instance.
 */
export function getSchnorrWallet(
  pxe: PXE,
  address: AztecAddress,
  signingPrivateKey: GrumpkinScalar,
): Promise<AccountWallet> {
  return getWallet(pxe, address, new SchnorrAccountContract(signingPrivateKey));
}
