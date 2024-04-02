import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { GrumpkinScalar } from '@aztec/aztec.js';
import { type Fq, Fr } from '@aztec/foundation/fields';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function createAccount(
  rpcUrl: string,
  privateKey: Fq,
  wait: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const actualPrivateKey = privateKey ?? GrumpkinScalar.random();

  const account = getSchnorrAccount(client, actualPrivateKey, actualPrivateKey, Fr.ZERO);
  const { address, publicKey, partialAddress } = account.getCompleteAddress();
  const tx = await account.deploy();
  const txHash = await tx.getTxHash();
  debugLogger(`Account contract tx sent with hash ${txHash}`);
  if (wait) {
    log(`\nWaiting for account contract deployment...`);
    await tx.wait();
  } else {
    log(`\nAccount deployment transaction hash: ${txHash}\n`);
  }

  log(`\nNew account:\n`);
  log(`Address:         ${address.toString()}`);
  log(`Public key:      ${publicKey.toString()}`);
  if (!privateKey) {
    log(`Private key:     ${actualPrivateKey.toString()}`);
  }
  log(`Partial address: ${partialAddress.toString()}`);
}
