import { AztecRPC, CompleteAddress, PrivateKey } from '@aztec/types';

import { AccountContract, AccountWallet, AztecAddress, Fr } from '../index.js';
import { Account } from './account.js';
import { EcdsaAccountContract } from './contract/ecdsa_account_contract.js';
import { SchnorrAccountContract } from './contract/schnorr_account_contract.js';
import { SingleKeyAccountContract } from './contract/single_key_account_contract.js';

export { Account } from './account.js';
export * from './contract/index.js';
export * from './entrypoint/index.js';
export { CompleteAddress };

/** A contract deployment salt. */
export type Salt = Fr | number | bigint;

/**
 * Creates an Account that relies on an ECDSA signing key for authentication.
 * @param rpc - An AztecRPC server instance.
 * @param encryptionPrivateKey - Grumpkin key used for note encryption.
 * @param signingPrivateKey - Secp256k1 key used for signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getEcdsaAccount(
  rpc: AztecRPC,
  encryptionPrivateKey: PrivateKey,
  signingPrivateKey: PrivateKey,
  saltOrAddress?: Salt | CompleteAddress,
): Account {
  return new Account(rpc, encryptionPrivateKey, new EcdsaAccountContract(signingPrivateKey), saltOrAddress);
}

/**
 * Creates an Account that relies on a Grumpkin signing key for authentication.
 * @param rpc - An AztecRPC server instance.
 * @param encryptionPrivateKey - Grumpkin key used for note encryption.
 * @param signingPrivateKey - Grumpkin key used for signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getSchnorrAccount(
  rpc: AztecRPC,
  encryptionPrivateKey: PrivateKey,
  signingPrivateKey: PrivateKey,
  saltOrAddress?: Salt | CompleteAddress,
): Account {
  return new Account(rpc, encryptionPrivateKey, new SchnorrAccountContract(signingPrivateKey), saltOrAddress);
}

/**
 * Creates an Account that uses the same Grumpkin key for encryption and authentication.
 * @param rpc - An AztecRPC server instance.
 * @param encryptionAndSigningPrivateKey - Grumpkin key used for note encryption and signing transactions.
 * @param saltOrAddress - Deployment salt or complete address if account contract is already deployed.
 */
export function getUnsafeSchnorrAccount(
  rpc: AztecRPC,
  encryptionAndSigningPrivateKey: PrivateKey,
  saltOrAddress?: Salt | CompleteAddress,
): Account {
  return new Account(
    rpc,
    encryptionAndSigningPrivateKey,
    new SingleKeyAccountContract(encryptionAndSigningPrivateKey),
    saltOrAddress,
  );
}

/**
 * Gets a wallet for an already registered account using Schnorr signatures with a single key for encryption and authentication.
 * @param rpc - An AztecRPC server instance.
 * @param address - Address for the account.
 * @param signingPrivateKey - Grumpkin key used for note encryption and signing transactions.
 * @returns A wallet for this account that can be used to interact with a contract instance.
 */
export function getUnsafeSchnorrWallet(
  rpc: AztecRPC,
  address: AztecAddress,
  signingKey: PrivateKey,
): Promise<AccountWallet> {
  return getWallet(rpc, address, new SingleKeyAccountContract(signingKey));
}

/**
 * Gets a wallet for an already registered account.
 * @param rpc - An AztecRPC server instance.
 * @param address - Address for the account.
 * @param accountContract - Account contract implementation.
 * * @returns A wallet for this account that can be used to interact with a contract instance.
 */
export async function getWallet(
  rpc: AztecRPC,
  address: AztecAddress,
  accountContract: AccountContract,
): Promise<AccountWallet> {
  const completeAddress = await rpc.getAccount(address);
  if (!completeAddress) {
    throw new Error(`Account ${address} not found`);
  }
  const nodeInfo = await rpc.getNodeInfo();
  const entrypoint = await accountContract.getEntrypoint(completeAddress, nodeInfo);
  return new AccountWallet(rpc, entrypoint, completeAddress);
}
