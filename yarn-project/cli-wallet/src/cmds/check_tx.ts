import { type PXE, type TxHash } from '@aztec/aztec.js';
import { inspectTx } from '@aztec/cli/utils';
import { type LogFn } from '@aztec/foundation/log';

export async function checkTx(client: PXE, txHash: TxHash, statusOnly: boolean, log: LogFn) {
  if (statusOnly) {
    const receipt = await client.getTxReceipt(txHash);
    return receipt.status;
  } else {
    await inspectTx(client, txHash, log, { includeBlockInfo: true });
  }
}
