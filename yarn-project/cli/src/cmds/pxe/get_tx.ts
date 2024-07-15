import { type TxHash } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../../client.js';
import { inspectTx } from '../../inspect.js';

export async function getTx(rpcUrl: string, txHash: TxHash, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  await inspectTx(client, txHash, log, { includeBlockInfo: true });
}
