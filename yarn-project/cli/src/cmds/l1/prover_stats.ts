import { retrieveL2ProofVerifiedEvents } from '@aztec/archiver';
import { createAztecNodeClient } from '@aztec/circuit-types';
import { EthAddress } from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { type LogFn, createDebugLogger } from '@aztec/foundation/log';

import groupBy from 'lodash.groupby';
import { createPublicClient, http } from 'viem';

export async function proverStats(opts: {
  l1RpcUrl: string;
  chainId: number;
  l1RollupAddress: string | undefined;
  nodeUrl: string | undefined;
  log: LogFn;
  startBlock: bigint;
  batchSize: bigint;
}) {
  const debugLog = createDebugLogger('aztec:cli:prover_stats');
  const { startBlock, chainId, l1RpcUrl, l1RollupAddress, batchSize, nodeUrl, log } = opts;
  if (!l1RollupAddress && !nodeUrl) {
    throw new Error('Either L1 rollup address or node URL must be set');
  }
  const rollup = l1RollupAddress
    ? EthAddress.fromString(l1RollupAddress)
    : await createAztecNodeClient(nodeUrl!)
        .getL1ContractAddresses()
        .then(a => a.rollupAddress);

  const chain = createEthereumChain(l1RpcUrl, chainId).chainInfo;
  const publicClient = createPublicClient({ chain, transport: http(l1RpcUrl) });
  const lastBlockNum = await publicClient.getBlockNumber();
  debugLog.verbose(`Querying events on rollup at ${rollup.toString()} from ${startBlock} up to ${lastBlockNum}`);

  let blockNum = startBlock;
  const events = [];
  while (blockNum <= lastBlockNum) {
    const end = blockNum + batchSize > lastBlockNum + 1n ? lastBlockNum + 1n : blockNum + batchSize;
    debugLog.verbose(`Querying events from block ${blockNum} to ${end}`);
    const newEvents = await retrieveL2ProofVerifiedEvents(publicClient, rollup, blockNum, end);
    events.push(...newEvents);
    debugLog.verbose(`Got ${newEvents.length} events`);
    blockNum += batchSize;
  }

  const stats = groupBy(events, 'proverId');
  for (const proverId in stats) {
    log(`${proverId}, ${stats[proverId].length}`);
  }
}
