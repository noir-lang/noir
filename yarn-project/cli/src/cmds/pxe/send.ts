import { type AztecAddress, Contract, Fr } from '@aztec/aztec.js';
import { deriveSigningKey } from '@aztec/circuits.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';
import { type IFeeOpts, printGasEstimates } from '../../fees.js';
import { prepTx } from '../../utils/aztec.js';

export async function send(
  functionName: string,
  functionArgsIn: any[],
  contractArtifactPath: string,
  contractAddress: AztecAddress,
  encryptionPrivateKey: Fr,
  rpcUrl: string,
  wait: boolean,
  feeOpts: IFeeOpts,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const { functionArgs, contractArtifact } = await prepTx(contractArtifactPath, functionName, functionArgsIn, log);

  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const { getSchnorrAccount } = await import('@aztec/accounts/schnorr');

  const wallet = await getSchnorrAccount(
    client,
    encryptionPrivateKey,
    deriveSigningKey(encryptionPrivateKey),
    Fr.ZERO,
  ).getWallet();
  const contract = await Contract.at(contractAddress, contractArtifact, wallet);
  const call = contract.methods[functionName](...functionArgs);

  if (feeOpts.estimateOnly) {
    const gas = await call.estimateGas({ ...feeOpts.toSendOpts(wallet) });
    printGasEstimates(feeOpts, gas, log);
    return;
  }

  const tx = call.send({ ...feeOpts.toSendOpts(wallet) });
  log(`\nTransaction hash: ${(await tx.getTxHash()).toString()}`);
  if (wait) {
    await tx.wait();

    log('Transaction has been mined');

    const receipt = await tx.getReceipt();
    log(` Tx fee: ${receipt.transactionFee}`);
    log(` Status: ${receipt.status}`);
    log(` Block number: ${receipt.blockNumber}`);
    log(` Block hash: ${receipt.blockHash?.toString('hex')}`);
  } else {
    log('Transaction pending. Check status with get-tx-receipt');
  }
}
