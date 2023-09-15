import { CompleteAddress, GrumpkinScalar } from '@aztec/circuits.js';
import { AztecRPC } from '@aztec/types';

import { getSchnorrAccount } from '../index.js';
import { AccountWallet } from '../wallet/account_wallet.js';

/**
 * Deploys and registers a new account using random private keys and returns the associated wallet. Useful for testing.
 * @param rpc - RPC client.
 * @returns - A wallet for a fresh account.
 */
export function createAccount(rpc: AztecRPC): Promise<AccountWallet> {
  return getSchnorrAccount(rpc, GrumpkinScalar.random(), GrumpkinScalar.random()).waitDeploy();
}

/**
 * Creates a random address and registers it as a recipient on the RPC server. Useful for testing.
 * @param rpc - RPC client.
 * @returns Complete address of the registered recipient.
 */
export async function createRecipient(rpc: AztecRPC): Promise<CompleteAddress> {
  const completeAddress = await CompleteAddress.random();
  await rpc.registerRecipient(completeAddress);
  return completeAddress;
}

/**
 * Creates a given number of random accounts using the Schnorr account wallet.
 * @param rpc - RPC interface.
 * @param numberOfAccounts - How many accounts to create.
 * @returns The created account wallets.
 */
export async function createAccounts(rpc: AztecRPC, numberOfAccounts = 1): Promise<AccountWallet[]> {
  const accounts = [];

  // Prepare deployments
  for (let i = 0; i < numberOfAccounts; ++i) {
    const account = getSchnorrAccount(rpc, GrumpkinScalar.random(), GrumpkinScalar.random());
    await account.getDeployMethod().then(d => d.simulate({ contractAddressSalt: account.salt }));
    accounts.push(account);
  }

  // Send them and await them to be mined
  const txs = await Promise.all(accounts.map(account => account.deploy()));
  await Promise.all(txs.map(tx => tx.wait({ interval: 0.1 })));
  return Promise.all(accounts.map(account => account.getWallet()));
}
