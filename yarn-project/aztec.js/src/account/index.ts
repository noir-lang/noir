/**
 * The `account` module provides utilities for managing accounts. The most common methods to use
 * are {@link getEcdsaAccount} and {@link getSchnorrAccount}, which return {@link AccountManager} instances
 * using the default ECDSA or Schnorr account implementation respectively. The {@link AccountManager} class then
 * allows to deploy and register a fresh account, or to obtain a `Wallet` instance out of an account already deployed.
 *
 * ```ts
 * const encryptionPrivateKey = GrumpkinScalar.random();
 * const signingPrivateKey = GrumpkinScalar.random();
 * const wallet = getSchnorrAccount(pxe, encryptionPrivateKey, signingPrivateKey).waitDeploy();
 * ```
 *
 * For testing purposes, consider using the {@link createAccount} and {@link createAccounts} methods,
 * which create, register, and deploy random accounts, and return their associated `Wallet`s.
 *
 * For implementing your own account contract, the recommended way is to extend from the base
 * {@link BaseAccountContract} class.
 * Read more in {@link https://docs.aztec.network/dev_docs/wallets/writing_an_account_contract | Writing an account contract}.
 *
 * @packageDocumentation
 */
import { CompleteAddress, GrumpkinPrivateKey, PXE } from '@aztec/types';

import { AccountContract, AccountWallet, AztecAddress, Fr } from '../index.js';
import { EcdsaAccountContract } from './contract/ecdsa_account_contract.js';
import { SchnorrAccountContract } from './contract/schnorr_account_contract.js';
import { SingleKeyAccountContract } from './contract/single_key_account_contract.js';
import { AccountManager } from './manager/index.js';

export * from './contract/index.js';
export * from './defaults/index.js';
export * from './utils.js';
export { AccountInterface, AuthWitnessProvider } from './interface.js';
export { AccountManager, CompleteAddress };

/** A contract deployment salt. */
export type Salt = Fr | number | bigint;

/**
 * Creates an Account that relies on an ECDSA signing key for authentication.
 * @param pxe - An PXE server instance.
 * @param encryptionPrivateKey - Grumpkin key used for note encryption.
 * @param signingPrivateKey - Secp256k1 key used for signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getEcdsaAccount(
  pxe: PXE,
  encryptionPrivateKey: GrumpkinPrivateKey,
  signingPrivateKey: Buffer,
  saltOrAddress?: Salt | CompleteAddress,
): AccountManager {
  return new AccountManager(pxe, encryptionPrivateKey, new EcdsaAccountContract(signingPrivateKey), saltOrAddress);
}

/**
 * Creates an Account that relies on a Grumpkin signing key for authentication.
 * @param pxe - An PXE server instance.
 * @param encryptionPrivateKey - Grumpkin key used for note encryption.
 * @param signingPrivateKey - Grumpkin key used for signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getSchnorrAccount(
  pxe: PXE,
  encryptionPrivateKey: GrumpkinPrivateKey,
  signingPrivateKey: GrumpkinPrivateKey,
  saltOrAddress?: Salt | CompleteAddress,
): AccountManager {
  return new AccountManager(pxe, encryptionPrivateKey, new SchnorrAccountContract(signingPrivateKey), saltOrAddress);
}

/**
 * Creates an Account that uses the same Grumpkin key for encryption and authentication.
 * @param pxe - An PXE server instance.
 * @param encryptionAndSigningPrivateKey - Grumpkin key used for note encryption and signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getUnsafeSchnorrAccount(
  pxe: PXE,
  encryptionAndSigningPrivateKey: GrumpkinPrivateKey,
  saltOrAddress?: Salt | CompleteAddress,
): AccountManager {
  return new AccountManager(
    pxe,
    encryptionAndSigningPrivateKey,
    new SingleKeyAccountContract(encryptionAndSigningPrivateKey),
    saltOrAddress,
  );
}

/**
 * Gets a wallet for an already registered account using Schnorr signatures with a single key for encryption and authentication.
 * @param pxe - An PXE server instance.
 * @param address - Address for the account.
 * @param signingPrivateKey - Grumpkin key used for note encryption and signing transactions.
 * @returns A wallet for this account that can be used to interact with a contract instance.
 */
export function getUnsafeSchnorrWallet(
  pxe: PXE,
  address: AztecAddress,
  signingKey: GrumpkinPrivateKey,
): Promise<AccountWallet> {
  return getWallet(pxe, address, new SingleKeyAccountContract(signingKey));
}

/**
 * Gets a wallet for an already registered account.
 * @param pxe - PXE Service instance.
 * @param address - Address for the account.
 * @param accountContract - Account contract implementation.
 * @returns A wallet for this account that can be used to interact with a contract instance.
 */
export async function getWallet(
  pxe: PXE,
  address: AztecAddress,
  accountContract: AccountContract,
): Promise<AccountWallet> {
  const completeAddress = await pxe.getRegisteredAccount(address);
  if (!completeAddress) {
    throw new Error(`Account ${address} not found`);
  }
  const nodeInfo = await pxe.getNodeInfo();
  const entrypoint = await accountContract.getInterface(completeAddress, nodeInfo);
  return new AccountWallet(pxe, entrypoint);
}
