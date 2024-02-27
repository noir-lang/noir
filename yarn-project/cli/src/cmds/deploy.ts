import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { ContractDeployer, EthAddress, Fq, Fr, Point } from '@aztec/aztec.js';
import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';
import { encodeArgs } from '../encoding.js';
import { GITHUB_TAG_PREFIX } from '../github.js';
import { getContractArtifact, getFunctionArtifact } from '../utils.js';

export async function deploy(
  artifactPath: string,
  json: boolean,
  rpcUrl: string,
  publicKey: Point | undefined,
  rawArgs: any[],
  portalAddress: EthAddress,
  salt: Fr,
  privateKey: Fq,
  wait: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
  logJson: (output: any) => void,
) {
  const contractArtifact = await getContractArtifact(artifactPath, log);
  const constructorArtifact = contractArtifact.functions.find(({ name }) => name === 'constructor');

  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const nodeInfo = await client.getNodeInfo();
  const expectedAztecNrVersion = `${GITHUB_TAG_PREFIX}-v${nodeInfo.nodeVersion}`;
  if (contractArtifact.aztecNrVersion && contractArtifact.aztecNrVersion !== expectedAztecNrVersion) {
    log(
      `\nWarning: Contract was compiled with a different version of Aztec.nr: ${contractArtifact.aztecNrVersion}. Consider updating Aztec.nr to ${expectedAztecNrVersion}\n`,
    );
  }

  const wallet = await getSchnorrAccount(client, privateKey, privateKey, Fr.ZERO).getWallet();
  const deployer = new ContractDeployer(contractArtifact, wallet, publicKey);

  const constructor = getFunctionArtifact(contractArtifact, 'constructor');
  if (!constructor) {
    throw new Error(`Constructor not found in contract ABI`);
  }

  debugLogger(`Input arguments: ${rawArgs.map((x: any) => `"${x}"`).join(', ')}`);
  const args = encodeArgs(rawArgs, constructorArtifact!.parameters);
  debugLogger(`Encoded arguments: ${args.join(', ')}`);

  const deploy = deployer.deploy(...args);

  await deploy.create({ contractAddressSalt: salt, portalContract: portalAddress });
  const tx = deploy.send({ contractAddressSalt: salt, portalContract: portalAddress });
  const txHash = await tx.getTxHash();
  debugLogger(`Deploy tx sent with hash ${txHash}`);
  if (wait) {
    const deployed = await tx.wait();
    const { address, partialAddress } = deployed.contract;
    if (json) {
      logJson({ address: address.toString(), partialAddress: partialAddress.toString() });
    } else {
      log(`\nContract deployed at ${address.toString()}\n`);
      log(`Contract partial address ${partialAddress.toString()}\n`);
    }
  } else {
    const { address, partialAddress } = deploy;
    if (json) {
      logJson({
        address: address?.toString() ?? 'N/A',
        partialAddress: partialAddress?.toString() ?? 'N/A',
        txHash: txHash.toString(),
      });
    } else {
      log(`\nContract Address: ${address?.toString() ?? 'N/A'}`);
      log(`Contract Partial Address: ${partialAddress?.toString() ?? 'N/A'}`);
      log(`Deployment transaction hash: ${txHash}\n`);
    }
  }
}
