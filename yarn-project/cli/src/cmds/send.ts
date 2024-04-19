import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type AztecAddress, Contract, Fr } from '@aztec/aztec.js';
import { deriveKeys } from '@aztec/circuits.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';
import { prepTx } from '../utils.js';

export async function send(
  functionName: string,
  functionArgsIn: any[],
  contractArtifactPath: string,
  contractAddress: AztecAddress,
  secretKey: Fr,
  rpcUrl: string,
  wait: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const { functionArgs, contractArtifact } = await prepTx(contractArtifactPath, functionName, functionArgsIn);

  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const signingKey = deriveKeys(secretKey).masterIncomingViewingSecretKey;
  const wallet = await getSchnorrAccount(client, secretKey, signingKey, Fr.ZERO).getWallet();
  const contract = await Contract.at(contractAddress, contractArtifact, wallet);
  const tx = contract.methods[functionName](...functionArgs).send();
  log(`\nTransaction hash: ${(await tx.getTxHash()).toString()}`);
  if (wait) {
    await tx.wait();

    log('Transaction has been mined');

    const receipt = await tx.getReceipt();
    log(`Status: ${receipt.status}\n`);
    log(`Block number: ${receipt.blockNumber}`);
    log(`Block hash: ${receipt.blockHash?.toString('hex')}`);
  } else {
    log('Transaction pending. Check status with get-tx-receipt');
  }
}
