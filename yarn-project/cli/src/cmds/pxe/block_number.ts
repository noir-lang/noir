import { createCompatibleClient } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

export async function blockNumber(rpcUrl: string, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const [latestNum, provenNum] = await Promise.all([client.getBlockNumber(), client.getProvenBlockNumber()]);
  log(`Latest block: ${latestNum}`);
  log(`Proven block: ${provenNum}\n`);
}
