import { AztecAddress, CompleteAddress, EthAddress, Fr, Point } from '@aztec/aztec.js';
import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';
import { getContractArtifact } from '../utils.js';

/**
 *
 */
export async function addContract(
  rpcUrl: string,
  contractArtifactPath: string,
  contractAddress: AztecAddress,
  partialAddress: Fr,
  publicKey: Point,
  portalContract: EthAddress | undefined,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const artifact = await getContractArtifact(contractArtifactPath, log);
  const completeAddress = new CompleteAddress(contractAddress, publicKey ?? Fr.ZERO, partialAddress);
  const portalContractAddress: EthAddress = portalContract ?? EthAddress.ZERO;
  const client = await createCompatibleClient(rpcUrl, debugLogger);

  await client.addContracts([{ artifact, completeAddress, portalContract: portalContractAddress }]);
  log(`\nContract added to PXE at ${contractAddress.toString()}\n`);
}
