import { AztecAddress, isContractDeployed } from '@aztec/aztec.js';
import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function checkDeploy(rpcUrl: string, contractAddress: AztecAddress, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const isDeployed = await isContractDeployed(client, contractAddress);
  if (isDeployed) {
    log(`\nContract found at ${contractAddress.toString()}\n`);
  } else {
    log(`\nNo contract found at ${contractAddress.toString()}\n`);
  }
}
