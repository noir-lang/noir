import { type AccountWalletWithSecretKey, type AztecAddress, Contract, Fr } from '@aztec/aztec.js';
import { GasSettings } from '@aztec/circuits.js';
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
  cancellable: boolean,
  feeOpts: IFeeOpts,
  log: LogFn,
) {
  const { functionArgs, contractArtifact } = await prepTx(contractArtifactPath, functionName, functionArgsIn, log);

  const contract = await Contract.at(contractAddress, contractArtifact, wallet);
  const call = contract.methods[functionName](...functionArgs);

  const gasLimits = await call.estimateGas({ ...(await feeOpts.toSendOpts(wallet)) });
  printGasEstimates(feeOpts, gasLimits, log);

  if (feeOpts.estimateOnly) {
    return;
  }

  const nonce = Fr.random();
  const tx = call.send({ ...(await feeOpts.toSendOpts(wallet)), nonce, cancellable });
  const txHash = await tx.getTxHash();
  log(`\nTransaction hash: ${txHash.toString()}`);
  if (wait) {
    try {
      await tx.wait();

      log('Transaction has been mined');

      const receipt = await tx.getReceipt();
      log(` Tx fee: ${receipt.transactionFee}`);
      log(` Status: ${receipt.status}`);
      log(` Block number: ${receipt.blockNumber}`);
      log(` Block hash: ${receipt.blockHash?.toString('hex')}`);
    } catch (err: any) {
      log(`Transaction failed\n ${err.message}`);
    }
  } else {
    log('Transaction pending. Check status with check-tx');
  }
  const gasSettings = GasSettings.from({
    ...gasLimits,
    maxFeesPerGas: feeOpts.gasSettings.maxFeesPerGas,
    inclusionFee: feeOpts.gasSettings.inclusionFee,
  });
  return {
    txHash,
    nonce,
    cancellable,
    gasSettings,
  };
}
