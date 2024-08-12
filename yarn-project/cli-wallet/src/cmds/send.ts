import { type AccountWalletWithSecretKey, type AztecAddress, Contract } from '@aztec/aztec.js';
import { prepTx } from '@aztec/cli/utils';
import { type LogFn } from '@aztec/foundation/log';

import { type IFeeOpts, printGasEstimates } from '../utils/options/fees.js';

export async function send(
  wallet: AccountWalletWithSecretKey,
  functionName: string,
  functionArgsIn: any[],
  contractArtifactPath: string,
  contractAddress: AztecAddress,
  wait: boolean,
  feeOpts: IFeeOpts,
  log: LogFn,
) {
  const { functionArgs, contractArtifact } = await prepTx(contractArtifactPath, functionName, functionArgsIn, log);

  const contract = await Contract.at(contractAddress, contractArtifact, wallet);
  const call = contract.methods[functionName](...functionArgs);

  if (feeOpts.estimateOnly) {
    const gas = await call.estimateGas({ ...(await feeOpts.toSendOpts(wallet)) });
    printGasEstimates(feeOpts, gas, log);
    return;
  }

  const tx = call.send({ ...(await feeOpts.toSendOpts(wallet)) });
  const txHash = (await tx.getTxHash()).toString();
  log(`\nTransaction hash: ${txHash}`);
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
  return txHash;
}
