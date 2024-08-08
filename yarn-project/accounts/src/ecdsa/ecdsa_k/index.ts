/**
 * The `@aztec/accounts/ecdsa` export provides an ECDSA account contract implementation, that uses an ECDSA private key for authentication, and a Grumpkin key for encryption.
 * Consider using this account type when working with integrations with Ethereum wallets.
 *
 * @packageDocumentation
 */
import { AccountManager, type Salt } from '@aztec/aztec.js/account';
import { type AccountWallet, getWallet } from '@aztec/aztec.js/wallet';
import { type PXE } from '@aztec/circuit-types';
import { type AztecAddress, type Fr } from '@aztec/circuits.js';

import { EcdsaKAccountContract } from './account_contract.js';

export { EcdsaKAccountContractArtifact } from './artifact.js';
export { EcdsaKAccountContract };

/**
 * Creates an Account that relies on an ECDSA signing key for authentication.
 * @param pxe - An PXE server instance.
 * @param secretKey - Secret key used to derive all the keystore keys.
 * @param signingPrivateKey - Secp256k1 key used for signing transactions.
 * @param salt - Deployment salt.
 */
export function getEcdsaKAccount(pxe: PXE, secretKey: Fr, signingPrivateKey: Buffer, salt?: Salt): AccountManager {
  return new AccountManager(pxe, secretKey, new EcdsaKAccountContract(signingPrivateKey), salt);
}

/**
 * Gets a wallet for an already registered account using ECDSA signatures.
 * @param pxe - An PXE server instance.
 * @param address - Address for the account.
 * @param signingPrivateKey - ECDSA key used for signing transactions.
 * @returns A wallet for this account that can be used to interact with a contract instance.
 */
export function getEcdsaKWallet(pxe: PXE, address: AztecAddress, signingPrivateKey: Buffer): Promise<AccountWallet> {
  return getWallet(pxe, address, new EcdsaKAccountContract(signingPrivateKey));
}
