import { GrumpkinScalar } from '@aztec/circuits.js';
import { PXE } from '@aztec/types';

import { getSchnorrAccount } from '../account_manager/index.js';
import { AccountWalletWithPrivateKey } from '../wallet/account_wallet_with_private_key.js';

/**
 * Deploys and registers a new account using random private keys and returns the associated Schnorr account wallet. Useful for testing.
 * @param pxe - PXE.
 * @returns - A wallet for a fresh account.
 */
export function createAccount(pxe: PXE): Promise<AccountWalletWithPrivateKey> {
  return getSchnorrAccount(pxe, GrumpkinScalar.random(), GrumpkinScalar.random()).waitDeploy();
}

/**
 * Creates a given number of random accounts using the Schnorr account wallet.
 * @param pxe - PXE.
 * @param numberOfAccounts - How many accounts to create.
 * @returns The created account wallets.
 */
export async function createAccounts(pxe: PXE, numberOfAccounts = 1): Promise<AccountWalletWithPrivateKey[]> {
  const accounts = [];

  // Prepare deployments
  for (let i = 0; i < numberOfAccounts; ++i) {
    const account = getSchnorrAccount(pxe, GrumpkinScalar.random(), GrumpkinScalar.random());
    await account.getDeployMethod().then(d => d.simulate({ contractAddressSalt: account.salt }));
    accounts.push(account);
  }

  // Send them and await them to be mined
  const txs = await Promise.all(accounts.map(account => account.deploy()));
  await Promise.all(txs.map(tx => tx.wait({ interval: 0.1 })));
  return Promise.all(accounts.map(account => account.getWallet()));
}
