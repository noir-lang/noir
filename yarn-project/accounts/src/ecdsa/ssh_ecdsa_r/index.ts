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

import { EcdsaRSSHAccountContract } from './account_contract.js';

export { EcdsaRAccountContractArtifact } from './artifact.js';
export { EcdsaRSSHAccountContract };

/**
 * Creates an Account that relies on an ECDSA signing key for authentication.
 * @param pxe - An PXE server instance.
 * @param secretKey - Secret key used to derive all the keystore keys.
 * @param signingPublicKey - Secp2561 key used to identify its corresponding private key in the SSH Agent.
 * @param salt - Deployment salt.
 */
export function getEcdsaRSSHAccount(pxe: PXE, secretKey: Fr, signingPublicKey: Buffer, salt?: Salt): AccountManager {
  return new AccountManager(pxe, secretKey, new EcdsaRSSHAccountContract(signingPublicKey), salt);
}

/**
 * Gets a wallet for an already registered account using ECDSA signatures.
 * @param pxe - An PXE server instance.
 * @param address - Address for the account.
 * @param signingPrivateKey - ECDSA key used for signing transactions.
 * @returns A wallet for this account that can be used to interact with a contract instance.
 */
export function getEcdsaRSSHWallet(pxe: PXE, address: AztecAddress, signingPublicKey: Buffer): Promise<AccountWallet> {
  return getWallet(pxe, address, new EcdsaRSSHAccountContract(signingPublicKey));
}
