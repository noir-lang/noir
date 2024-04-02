import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function getNodeInfo(rpcUrl: string, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const info = await client.getNodeInfo();
  log(`\nNode Info:\n`);
  log(`Node Version: ${info.nodeVersion}\n`);
  log(`Chain Id: ${info.chainId}\n`);
  log(`Protocol Version: ${info.protocolVersion}\n`);
  log(`Rollup Address: ${info.l1ContractAddresses.rollupAddress.toString()}`);
}
