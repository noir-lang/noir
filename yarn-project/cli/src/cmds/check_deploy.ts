import { type AztecAddress } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function checkDeploy(rpcUrl: string, contractAddress: AztecAddress, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const isPrivatelyDeployed = await client.getContractInstance(contractAddress);
  const isPubliclyDeployed = await client.isContractPubliclyDeployed(contractAddress);
  if (isPubliclyDeployed && isPrivatelyDeployed) {
    log(`\nContract is publicly deployed at ${contractAddress.toString()}\n`);
  } else if (isPrivatelyDeployed) {
    log(`\nContract is registered in the local pxe at ${contractAddress.toString()} but not publicly deployed\n`);
  } else if (isPubliclyDeployed) {
    log(`\nContract is publicly deployed at ${contractAddress.toString()} but not registered in the local pxe\n`);
  } else {
    log(`\nNo contract found at ${contractAddress.toString()}\n`);
  }
}
