import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import { ContractDeployer, type EthAddress, Fr } from '@aztec/aztec.js';
import { deriveKeys } from '@aztec/circuits.js';
import { getInitializer } from '@aztec/foundation/abi';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';
import { encodeArgs } from '../encoding.js';
import { GITHUB_TAG_PREFIX } from '../github.js';
import { getContractArtifact } from '../utils.js';

export async function deploy(
  artifactPath: string,
  json: boolean,
  rpcUrl: string,
  rawArgs: any[],
  portalAddress: EthAddress,
  salt: Fr,
  secretKey: Fr,
  initializer: string | undefined,
  wait: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
  logJson: (output: any) => void,
) {
  const contractArtifact = await getContractArtifact(artifactPath);
  const constructorArtifact = getInitializer(contractArtifact, initializer);

  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const nodeInfo = await client.getNodeInfo();
  const expectedAztecNrVersion = `${GITHUB_TAG_PREFIX}-v${nodeInfo.nodeVersion}`;
  if (contractArtifact.aztecNrVersion && contractArtifact.aztecNrVersion !== expectedAztecNrVersion) {
    log(
      `\nWarning: Contract was compiled with a different version of Aztec.nr: ${contractArtifact.aztecNrVersion}. Consider updating Aztec.nr to ${expectedAztecNrVersion}\n`,
    );
  }

  const keys = deriveKeys(secretKey);
  const wallet = await getSchnorrAccount(client, secretKey, keys.masterIncomingViewingSecretKey, Fr.ZERO).getWallet();
  const deployer = new ContractDeployer(contractArtifact, wallet, keys.publicKeysHash, initializer);

  let args = [];
  if (rawArgs.length > 0) {
    if (!constructorArtifact) {
      throw new Error(`Cannot process constructor arguments as no constructor was found`);
    }
    debugLogger.verbose(`Input arguments: ${rawArgs.map((x: any) => `"${x}"`).join(', ')}`);
    args = encodeArgs(rawArgs, constructorArtifact!.parameters);
    debugLogger.verbose(`Encoded arguments: ${args.join(', ')}`);
  }

  const deploy = deployer.deploy(...args);

  await deploy.create({ contractAddressSalt: salt, portalContract: portalAddress });
  const tx = deploy.send({ contractAddressSalt: salt, portalContract: portalAddress });
  const txHash = await tx.getTxHash();
  debugLogger.verbose(`Deploy tx sent with hash ${txHash}`);
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
