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
import { Fr } from '@aztec/circuits.js';

export { CompleteAddress } from '@aztec/types';

export * from './defaults/index.js';
export { AccountInterface, AuthWitnessProvider } from './interface.js';
export * from './wallet.js';

/** A contract deployment salt. */
export type Salt = Fr | number | bigint;
