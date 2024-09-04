import { getL2BlockProposedLogs, retrieveL2ProofVerifiedEvents } from '@aztec/archiver';
import { createAztecNodeClient } from '@aztec/circuit-types';
import { EthAddress } from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { compactArray, mapValues, unique } from '@aztec/foundation/collection';
import { type LogFn, type Logger, createDebugLogger } from '@aztec/foundation/log';

import chunk from 'lodash.chunk';
import groupBy from 'lodash.groupby';
import { type PublicClient, createPublicClient, http } from 'viem';

export async function proverStats(opts: {
  l1RpcUrl: string;
  chainId: number;
  l1RollupAddress: string | undefined;
  nodeUrl: string | undefined;
  log: LogFn;
  startBlock: bigint;
  endBlock: bigint | undefined;
  batchSize: bigint;
  provingTimeout: bigint | undefined;
  rawLogs: boolean;
}) {
  const debugLog = createDebugLogger('aztec:cli:prover_stats');
  const { startBlock, chainId, l1RpcUrl, l1RollupAddress, batchSize, nodeUrl, provingTimeout, endBlock, rawLogs, log } =
    opts;
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
  const lastBlockNum = endBlock ?? (await publicClient.getBlockNumber());
  debugLog.verbose(`Querying events on rollup at ${rollup.toString()} from ${startBlock} up to ${lastBlockNum}`);

  // Get all events for L2 proof submissions
  const events = await getL2ProofVerifiedEvents(startBlock, lastBlockNum, batchSize, debugLog, publicClient, rollup);

  // If we only care for raw logs, output them
  if (rawLogs) {
    log(`l1_block_number, l2_block_number, prover_id, tx_hash`);
    for (const event of events) {
      const { l1BlockNumber, l2BlockNumber, proverId, txHash } = event;
      log(`${l1BlockNumber}, ${l2BlockNumber}, ${proverId}, ${txHash}`);
    }
    return;
  }

  // If we don't have a proving timeout, we can just count the number of unique blocks per prover
  if (!provingTimeout) {
    const stats = groupBy(events, 'proverId');
    log(`prover_id, total_blocks_proven`);
    for (const proverId in stats) {
      const uniqueBlocks = new Set(stats[proverId].map(e => e.l2BlockNumber));
      log(`${proverId}, ${uniqueBlocks.size}`);
    }
    return;
  }

  // But if we do, fetch the events for each block submitted, so we can look up their timestamp
  const blockEvents = await getL2BlockEvents(startBlock, lastBlockNum, batchSize, debugLog, publicClient, rollup);
  debugLog.verbose(
    `First L2 block within range is ${blockEvents[0]?.args.blockNumber} at L1 block ${blockEvents[0]?.blockNumber}`,
  );

  // Get the timestamps for every block on every log, both for proof and block submissions
  const l1BlockNumbers = unique([...events.map(e => e.l1BlockNumber), ...blockEvents.map(e => e.blockNumber)]);
  const l1BlockTimestamps: Record<string, bigint> = {};
  for (const l1Batch of chunk(l1BlockNumbers, Number(batchSize))) {
    const blocks = await Promise.all(
      l1Batch.map(blockNumber => publicClient.getBlock({ includeTransactions: false, blockNumber })),
    );
    debugLog.verbose(`Queried ${blocks.length} L1 blocks between ${l1Batch[0]} and ${l1Batch[l1Batch.length - 1]}`);
    for (const block of blocks) {
      l1BlockTimestamps[block.number.toString()] = block.timestamp;
    }
  }

  // Map from l2 block number to the l1 block in which it was submitted
  const l2BlockSubmissions: Record<string, bigint> = {};
  for (const blockEvent of blockEvents) {
    l2BlockSubmissions[blockEvent.args.blockNumber.toString()] = blockEvent.blockNumber;
  }

  // Now calculate stats
  const stats = mapValues(groupBy(events, 'proverId'), (blocks, proverId) =>
    compactArray(
      blocks.map(e => {
        const provenTimestamp = l1BlockTimestamps[e.l1BlockNumber.toString()];
        const uploadedBlockNumber = l2BlockSubmissions[e.l2BlockNumber.toString()];
        if (!uploadedBlockNumber) {
          debugLog.verbose(
            `Skipping ${proverId}'s proof for L2 block ${e.l2BlockNumber} as it was before the start block`,
          );
          return undefined;
        }
        const uploadedTimestamp = l1BlockTimestamps[uploadedBlockNumber.toString()];
        const provingTime = provenTimestamp - uploadedTimestamp;
        debugLog.debug(
          `prover=${e.proverId} blockNumber=${e.l2BlockNumber} uploaded=${uploadedTimestamp} proven=${provenTimestamp} time=${provingTime}`,
        );
        return { provenTimestamp, uploadedTimestamp, provingTime, ...e };
      }),
    ),
  );

  log(`prover_id, blocks_proven_within_timeout, total_blocks_proven, avg_proving_time`);
  for (const proverId in stats) {
    const blocks = stats[proverId];
    const withinTimeout = blocks.filter(b => b.provingTime <= provingTimeout);
    const uniqueBlocksWithinTimeout = new Set(withinTimeout.map(e => e.l2BlockNumber));
    const uniqueBlocks = new Set(blocks.map(e => e.l2BlockNumber));
    const avgProvingTime =
      blocks.length === 0 ? 0 : Math.ceil(Number(blocks.reduce((acc, b) => acc + b.provingTime, 0n)) / blocks.length);

    log(`${proverId}, ${uniqueBlocksWithinTimeout.size}, ${uniqueBlocks.size}, ${avgProvingTime}`);
  }
  return;
}

async function getL2ProofVerifiedEvents(
  startBlock: bigint,
  lastBlockNum: bigint,
  batchSize: bigint,
  debugLog: Logger,
  publicClient: PublicClient,
  rollup: EthAddress,
) {
  let blockNum = startBlock;
  const events = [];
  while (blockNum <= lastBlockNum) {
    const end = blockNum + batchSize > lastBlockNum + 1n ? lastBlockNum + 1n : blockNum + batchSize;
    const newEvents = await retrieveL2ProofVerifiedEvents(publicClient, rollup, blockNum, end);
    events.push(...newEvents);
    debugLog.verbose(`Got ${newEvents.length} events querying l2 proof verified from block ${blockNum} to ${end}`);
    blockNum += batchSize;
  }
  return events;
}

async function getL2BlockEvents(
  startBlock: bigint,
  lastBlockNum: bigint,
  batchSize: bigint,
  debugLog: Logger,
  publicClient: PublicClient,
  rollup: EthAddress,
) {
  let blockNum = startBlock;
  const events = [];
  while (blockNum <= lastBlockNum) {
    const end = blockNum + batchSize > lastBlockNum + 1n ? lastBlockNum + 1n : blockNum + batchSize;
    const newEvents = await getL2BlockProposedLogs(publicClient, rollup, blockNum, end);
    events.push(...newEvents);
    debugLog.verbose(`Got ${newEvents.length} events querying l2 block submitted from block ${blockNum} to ${end}`);
    blockNum += batchSize;
  }
  return events;
}
