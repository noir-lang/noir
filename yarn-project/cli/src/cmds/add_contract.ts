import {
  AztecAddress,
  type ContractInstanceWithAddress,
  EthAddress,
  type Fr,
  type Point,
  getContractClassFromArtifact,
} from '@aztec/aztec.js';
import { computeContractAddressFromInstance, computePublicKeysHash } from '@aztec/circuits.js/contract';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';
import { getContractArtifact } from '../utils.js';

export async function addContract(
  rpcUrl: string,
  contractArtifactPath: string,
  address: AztecAddress,
  initializationHash: Fr,
  salt: Fr,
  publicKey: Point | undefined,
  portalContract: EthAddress | undefined,
  deployer: AztecAddress | undefined,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const artifact = await getContractArtifact(contractArtifactPath, log);
  const instance: ContractInstanceWithAddress = {
    version: 1,
    salt,
    initializationHash,
    contractClassId: getContractClassFromArtifact(artifact).id,
    portalContractAddress: portalContract ?? EthAddress.ZERO,
    publicKeysHash: computePublicKeysHash(publicKey),
    address,
    deployer: deployer ?? AztecAddress.ZERO,
  };
  const computed = computeContractAddressFromInstance(instance);
  if (!computed.equals(address)) {
    throw new Error(`Contract address ${address.toString()} does not match computed address ${computed.toString()}`);
  }

  const client = await createCompatibleClient(rpcUrl, debugLogger);

  await client.registerContract({ artifact, instance });
  log(`\nContract added to PXE at ${address.toString()} with class ${instance.contractClassId.toString()}\n`);
}
