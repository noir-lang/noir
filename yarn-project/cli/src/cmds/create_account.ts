import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { deriveKeys } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function createAccount(
  rpcUrl: string,
  secretKey: Fr,
  wait: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const actualSecretKey = secretKey ?? Fr.random();

  const signingKey = deriveKeys(actualSecretKey).masterIncomingViewingSecretKey;
  const account = getSchnorrAccount(client, actualSecretKey, signingKey, Fr.ZERO);
  const { address, publicKey, partialAddress } = account.getCompleteAddress();
  const tx = account.deploy();
  const txHash = await tx.getTxHash();
  debugLogger.verbose(`Account contract tx sent with hash ${txHash}`);
  if (wait) {
    log(`\nWaiting for account contract deployment...`);
    await tx.wait();
  } else {
    log(`\nAccount deployment transaction hash: ${txHash}\n`);
  }

  log(`\nNew account:\n`);
  log(`Address:         ${address.toString()}`);
  log(`Public key:      ${publicKey.toString()}`);
  if (!secretKey) {
    log(`Secret key:     ${actualSecretKey.toString()}`);
  }
  log(`Partial address: ${partialAddress.toString()}`);
}
