import { type AztecAddress, type LogFilter, type LogId, type TxHash } from '@aztec/aztec.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';
import { sleep } from '@aztec/foundation/sleep';

import { createCompatibleClient } from '../../client.js';

export async function getLogs(
  txHash: TxHash,
  fromBlock: number,
  toBlock: number,
  afterLog: LogId,
  contractAddress: AztecAddress,
  rpcUrl: string,
  follow: boolean,
  debugLogger: DebugLogger,
  log: LogFn,
) {
  const pxe = await createCompatibleClient(rpcUrl, debugLogger);

  if (follow) {
    if (txHash) {
      throw Error('Cannot use --follow with --tx-hash');
    }
    if (toBlock) {
      throw Error('Cannot use --follow with --to-block');
    }
  }

  const filter: LogFilter = { txHash, fromBlock, toBlock, afterLog, contractAddress };

  const fetchLogs = async () => {
    const response = await pxe.getUnencryptedLogs(filter);
    const logs = response.logs;

    if (!logs.length) {
      const filterOptions = Object.entries(filter)
        .filter(([, value]) => value !== undefined)
        .map(([key, value]) => `${key}: ${value}`)
        .join(', ');
      if (!follow) {
        log(`No logs found for filter: {${filterOptions}}`);
      }
    } else {
      if (!follow && !filter.afterLog) {
        log('Logs found: \n');
      }
      logs.forEach(unencryptedLog => log(unencryptedLog.toHumanReadable()));
      // Set the continuation parameter for the following requests
      filter.afterLog = logs[logs.length - 1].id;
    }
    return response.maxLogsHit;
  };

  if (follow) {
    log('Fetching logs...');
    while (true) {
      const maxLogsHit = await fetchLogs();
      if (!maxLogsHit) {
        await sleep(1000);
      }
    }
  } else {
    while (await fetchLogs()) {
      // Keep fetching logs until we reach the end.
    }
  }
}
