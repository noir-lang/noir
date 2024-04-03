import { type AccountWalletWithPrivateKey } from '@aztec/aztec.js/wallet';
import { type PXE } from '@aztec/circuit-types';
import { GrumpkinScalar } from '@aztec/circuits.js';

import { getSchnorrAccount } from '../schnorr/index.js';

/**
 * Deploys and registers a new account using random private keys and returns the associated Schnorr account wallet. Useful for testing.
 * @param pxe - PXE.
 * @returns - A wallet for a fresh account.
 */
export function createAccount(pxe: PXE): Promise<AccountWalletWithPrivateKey> {
  return getSchnorrAccount(pxe, GrumpkinScalar.random(), GrumpkinScalar.random()).waitSetup();
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
    // Unfortunately the function below is not stateless and we call it here because it takes a long time to run and
    // the results get stored within the account object. By calling it here we increase the probability of all the
    // accounts being deployed in the same block because it makes the deploy() method basically instant.
    await account.getDeployMethod().then(d =>
      d.prove({
        contractAddressSalt: account.salt,
        skipClassRegistration: true,
        skipPublicDeployment: true,
        universalDeploy: true,
      }),
    );
    accounts.push(account);
  }

  // Send them and await them to be mined
  const txs = await Promise.all(accounts.map(account => account.deploy()));
  await Promise.all(txs.map(tx => tx.wait({ interval: 0.1 })));
  return Promise.all(accounts.map(account => account.getWallet()));
}
