import { type AccountWalletWithSecretKey } from '@aztec/aztec.js/wallet';
import { type PXE } from '@aztec/circuit-types';
import { Fr, deriveSigningKey } from '@aztec/circuits.js';

import { getSchnorrAccount } from '../schnorr/index.js';

/**
 * Deploys and registers a new account using random private keys and returns the associated Schnorr account wallet. Useful for testing.
 * @param pxe - PXE.
 * @returns - A wallet for a fresh account.
 */
export function createAccount(pxe: PXE): Promise<AccountWalletWithSecretKey> {
  const secretKey = Fr.random();
  const signingKey = deriveSigningKey(secretKey);
  return getSchnorrAccount(pxe, secretKey, signingKey).waitSetup();
}

/**
 * Creates a given number of random accounts using the Schnorr account wallet.
 * @param pxe - PXE.
 * @param numberOfAccounts - How many accounts to create.
 * @param secrets - Optional array of secrets to use for the accounts. If empty, random secrets will be generated.
 * @throws If the secrets array is not empty and does not have the same length as the number of accounts.
 * @returns The created account wallets.
 */
export async function createAccounts(
  pxe: PXE,
  numberOfAccounts = 1,
  secrets: Fr[] = [],
): Promise<AccountWalletWithSecretKey[]> {
  const accounts = [];

  if (secrets.length == 0) {
    secrets = Array.from({ length: numberOfAccounts }, () => Fr.random());
  } else if (secrets.length > 0 && secrets.length !== numberOfAccounts) {
    throw new Error('Secrets array must be empty or have the same length as the number of accounts');
  }

  // Prepare deployments
  for (const secret of secrets) {
    const signingKey = deriveSigningKey(secret);
    const account = getSchnorrAccount(pxe, secret, signingKey);
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
