import { type AccountManager, type DeployAccountOptions } from '@aztec/aztec.js';
import { prettyPrintJSON } from '@aztec/cli/cli-utils';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type IFeeOpts, printGasEstimates } from '../utils/options/fees.js';

export async function deployAccount(
  account: AccountManager,
  wait: boolean,
  feeOpts: IFeeOpts,
  json: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const out: Record<string, any> = {};
  const { address, partialAddress, publicKeys } = account.getCompleteAddress();
  const { initializationHash, deployer, salt } = account.getInstance();
  const wallet = await account.getWallet();
  const secretKey = wallet.getSecretKey();

  if (json) {
    out.address = address;
    out.partialAddress = partialAddress;
    out.salt = salt;
    out.initHash = initializationHash;
    out.deployer = deployer;
  } else {
    log(`\nNew account:\n`);
    log(`Address:         ${address.toString()}`);
    log(`Public key:      0x${publicKeys.toString()}`);
    if (secretKey) {
      log(`Secret key:     ${secretKey.toString()}`);
    }
    log(`Partial address: ${partialAddress.toString()}`);
    log(`Salt:            ${salt.toString()}`);
    log(`Init hash:       ${initializationHash.toString()}`);
    log(`Deployer:        ${deployer.toString()}`);
  }

  let tx;
  let txReceipt;

  const sendOpts: DeployAccountOptions = {
    ...feeOpts.toSendOpts(wallet),
    skipInitialization: false,
  };
  if (feeOpts.estimateOnly) {
    const gas = await (await account.getDeployMethod()).estimateGas({ ...sendOpts });
    if (json) {
      out.fee = {
        gasLimits: {
          da: gas.gasLimits.daGas,
          l2: gas.gasLimits.l2Gas,
        },
        teardownGasLimits: {
          da: gas.teardownGasLimits.daGas,
          l2: gas.teardownGasLimits,
        },
      };
    } else {
      printGasEstimates(feeOpts, gas, log);
    }
  } else {
    tx = account.deploy({ ...sendOpts });
    const txHash = await tx.getTxHash();
    debugLogger.debug(`Account contract tx sent with hash ${txHash}`);
    out.txHash = txHash;
    if (wait) {
      if (!json) {
        log(`\nWaiting for account contract deployment...`);
      }
      txReceipt = await tx.wait();
      out.txReceipt = {
        status: txReceipt.status,
        transactionFee: txReceipt.transactionFee,
      };
    }
  }

  if (json) {
    log(prettyPrintJSON(out));
  } else {
    if (tx) {
      log(`Deploy tx hash:  ${await tx.getTxHash()}`);
    }
    if (txReceipt) {
      log(`Deploy tx fee:   ${txReceipt.transactionFee}`);
    }
  }

  return { address, secretKey, salt };
}
