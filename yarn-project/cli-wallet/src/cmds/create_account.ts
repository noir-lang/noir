import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { type DeployAccountOptions, createCompatibleClient } from '@aztec/aztec.js';
import { deriveSigningKey } from '@aztec/circuits.js';
import { prettyPrintJSON } from '@aztec/cli/cli-utils';
import { Fr } from '@aztec/foundation/fields';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type IFeeOpts, printGasEstimates } from '../utils/fees.js';

export async function createAccount(
  rpcUrl: string,
  privateKey: Fr | undefined,
  alias: string | undefined,
  registerOnly: boolean,
  publicDeploy: boolean,
  skipInitialization: boolean,
  wait: boolean,
  feeOpts: IFeeOpts,
  json: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const printPK = typeof privateKey === 'undefined';
  privateKey ??= Fr.random();

  const salt = Fr.ZERO;

  const account = getSchnorrAccount(client, privateKey, deriveSigningKey(privateKey), salt);
  const { address, publicKeys, partialAddress } = account.getCompleteAddress();

  const out: Record<string, any> = {};
  if (json) {
    out.address = address;
    out.publicKey = publicKeys;
    if (printPK) {
      out.privateKey = privateKey;
    }
    out.partialAddress = partialAddress;
    out.salt = salt;
    out.initHash = account.getInstance().initializationHash;
    out.deployer = account.getInstance().deployer;
  } else {
    log(`\nNew account:\n`);
    log(`Address:         ${address.toString()}`);
    log(`Public key:      0x${publicKeys.toString()}`);
    if (printPK) {
      log(`Private key:     ${privateKey.toString()}`);
    }
    log(`Partial address: ${partialAddress.toString()}`);
    log(`Salt:            ${salt.toString()}`);
    log(`Init hash:       ${account.getInstance().initializationHash.toString()}`);
    log(`Deployer:        ${account.getInstance().deployer.toString()}`);
  }

  let tx;
  let txReceipt;
  if (registerOnly) {
    await account.register();
  } else {
    const wallet = await account.getWallet();
    const sendOpts: DeployAccountOptions = {
      ...feeOpts.toSendOpts(wallet),
      skipClassRegistration: !publicDeploy,
      skipPublicDeployment: !publicDeploy,
      skipInitialization: skipInitialization,
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

  return { alias, address, privateKey, salt };
}
