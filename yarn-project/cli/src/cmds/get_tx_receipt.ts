import { type TxHash } from '@aztec/aztec.js';
import { JsonStringify } from '@aztec/foundation/json-rpc';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { createCompatibleClient } from '../client.js';

export async function getTxReceipt(rpcUrl: string, txHash: TxHash, debugLogger: DebugLogger, log: LogFn) {
  const client = await createCompatibleClient(rpcUrl, debugLogger);
  const receipt = await client.getTxReceipt(txHash);
  if (!receipt) {
    log(`No receipt found for transaction hash ${txHash.toString()}`);
  } else {
    log(`\nTransaction receipt: \n${JsonStringify(receipt, true)}\n`);
  }
}
