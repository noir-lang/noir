import { ContractDeployer, type DeployMethod, Fr } from '@aztec/aztec.js';
import { type PublicKeys, deriveSigningKey } from '@aztec/circuits.js';
import { getInitializer } from '@aztec/foundation/abi';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';
import { encodeArgs } from '../../encoding.js';
import { type IFeeOpts, printGasEstimates } from '../../fees.js';
import { GITHUB_TAG_PREFIX } from '../../github.js';
import { getContractArtifact } from '../../utils/aztec.js';

export async function deploy(
  artifactPath: string,
  json: boolean,
  rpcUrl: string,
  publicKeys: PublicKeys | undefined,
  rawArgs: any[],
  salt: Fr | undefined,
  privateKey: Fr,
  initializer: string | undefined,
  skipPublicDeployment: boolean,
  skipClassRegistration: boolean,
  skipInitialization: boolean | undefined,
  universalDeploy: boolean | undefined,
  wait: boolean,
  feeOpts: IFeeOpts,
  debugLogger: DebugLogger,
  log: LogFn,
  logJson: (output: any) => void,
) {
  salt ??= Fr.random();
  const contractArtifact = await getContractArtifact(artifactPath, log);
  const constructorArtifact = getInitializer(contractArtifact, initializer);

  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const nodeInfo = await client.getNodeInfo();
  const expectedAztecNrVersion = `${GITHUB_TAG_PREFIX}-v${nodeInfo.nodeVersion}`;
  if (contractArtifact.aztecNrVersion && contractArtifact.aztecNrVersion !== expectedAztecNrVersion) {
    log(
      `\nWarning: Contract was compiled with a different version of Aztec.nr: ${contractArtifact.aztecNrVersion}. Consider updating Aztec.nr to ${expectedAztecNrVersion}\n`,
    );
  }
  const { getSchnorrAccount } = await import('@aztec/accounts/schnorr');

  const wallet = await getSchnorrAccount(client, privateKey, deriveSigningKey(privateKey), Fr.ZERO).getWallet();
  const deployer = new ContractDeployer(contractArtifact, wallet, publicKeys?.hash() ?? Fr.ZERO, initializer);

  let args = [];
  if (rawArgs.length > 0) {
    if (!constructorArtifact) {
      throw new Error(`Cannot process constructor arguments as no constructor was found`);
    }
    debugLogger.debug(`Input arguments: ${rawArgs.map((x: any) => `"${x}"`).join(', ')}`);
    args = encodeArgs(rawArgs, constructorArtifact!.parameters);
    debugLogger.debug(`Encoded arguments: ${args.join(', ')}`);
  }

  const deploy = deployer.deploy(...args);
  const deployOpts: Parameters<DeployMethod['send']>[0] = {
    ...feeOpts.toSendOpts(wallet),
    contractAddressSalt: salt,
    universalDeploy,
    skipClassRegistration,
    skipInitialization,
    skipPublicDeployment,
  };

  if (feeOpts.estimateOnly) {
    const gas = await deploy.estimateGas(deployOpts);
    printGasEstimates(feeOpts, gas, log);
    return;
  }

  await deploy.create(deployOpts);
  const tx = deploy.send(deployOpts);

  const txHash = await tx.getTxHash();
  debugLogger.debug(`Deploy tx sent with hash ${txHash}`);
  if (wait) {
    const deployed = await tx.wait();
    const { address, partialAddress, instance } = deployed.contract;
    if (json) {
      logJson({
        address: address.toString(),
        partialAddress: partialAddress.toString(),
        initializationHash: instance.initializationHash.toString(),
        salt: salt.toString(),
        transactionFee: deployed.transactionFee,
      });
    } else {
      log(`Contract deployed at ${address.toString()}`);
      log(`Contract partial address ${partialAddress.toString()}`);
      log(`Contract init hash ${instance.initializationHash.toString()}`);
      log(`Deployment tx hash: ${txHash.toString()}`);
      log(`Deployment salt: ${salt.toString()}`);
      log(`Deployment fee: ${deployed.transactionFee}`);
    }
  } else {
    const { address, partialAddress } = deploy;
    const instance = deploy.getInstance();
    if (json) {
      logJson({
        address: address?.toString() ?? 'N/A',
        partialAddress: partialAddress?.toString() ?? 'N/A',
        txHash: txHash.toString(),
        initializationHash: instance.initializationHash.toString(),
        salt: salt.toString(),
        deployer: instance.deployer.toString(),
      });
    } else {
      log(`Contract deployed at ${address?.toString()}`);
      log(`Contract partial address ${partialAddress?.toString()}`);
      log(`Contract init hash ${instance.initializationHash.toString()}`);
      log(`Deployment tx hash: ${txHash.toString()}`);
      log(`Deployment salt: ${salt.toString()}`);
      log(`Deployer: ${instance.deployer.toString()}`);
    }
  }
}
