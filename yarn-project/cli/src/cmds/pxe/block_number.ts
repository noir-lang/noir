import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';

export async function blockNumber(rpcUrl: string, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const [latestNum, provenNum] = await Promise.all([client.getBlockNumber(), client.getProvenBlockNumber()]);
  log(`Latest block: ${latestNum}`);
  log(`Proven block: ${provenNum}\n`);
}
