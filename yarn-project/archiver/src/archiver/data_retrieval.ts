import { type Body, type InboxLeaf } from '@aztec/circuit-types';
import { type AppendOnlyTreeSnapshot, type Header } from '@aztec/circuits.js';
import { type EthAddress } from '@aztec/foundation/eth-address';

import { type PublicClient } from 'viem';

import {
  getL2BlockProcessedLogs,
  getMessageSentLogs,
  getTxsPublishedLogs,
  processL2BlockProcessedLogs,
  processMessageSentLogs,
  processTxsPublishedLogs,
} from './eth_log_handlers.js';

/**
 * Data retrieved from logs
 */
export type DataRetrieval<T> = {
  /**
   * Blocknumber of the last L1 block from which we obtained data.
   */
  lastProcessedL1BlockNumber: bigint;
  /**
   * The data returned.
   */
  retrievedData: T[];
};

/**
 * Fetches new L2 block metadata (header, archive snapshot).
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param rollupAddress - The address of the rollup contract.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param searchEndBlock - The highest block number that we should search up to.
 * @param expectedNextL2BlockNum - The next L2 block number that we expect to find.
 * @returns An array of tuples representing block metadata including the header, archive tree snapshot; as well as the next eth block to search from.
 */
export async function retrieveBlockMetadataFromRollup(
  publicClient: PublicClient,
  rollupAddress: EthAddress,
  blockUntilSynced: boolean,
  searchStartBlock: bigint,
  searchEndBlock: bigint,
  expectedNextL2BlockNum: bigint,
): Promise<DataRetrieval<[Header, AppendOnlyTreeSnapshot]>> {
  const retrievedBlockMetadata: [Header, AppendOnlyTreeSnapshot][] = [];
  do {
    if (searchStartBlock > searchEndBlock) {
      break;
    }
    const l2BlockProcessedLogs = await getL2BlockProcessedLogs(
      publicClient,
      rollupAddress,
      searchStartBlock,
      searchEndBlock,
    );
    if (l2BlockProcessedLogs.length === 0) {
      break;
    }

    const newBlockMetadata = await processL2BlockProcessedLogs(
      publicClient,
      expectedNextL2BlockNum,
      l2BlockProcessedLogs,
    );
    retrievedBlockMetadata.push(...newBlockMetadata);
    searchStartBlock = l2BlockProcessedLogs[l2BlockProcessedLogs.length - 1].blockNumber! + 1n;
    expectedNextL2BlockNum += BigInt(newBlockMetadata.length);
  } while (blockUntilSynced && searchStartBlock <= searchEndBlock);
  return { lastProcessedL1BlockNumber: searchStartBlock - 1n, retrievedData: retrievedBlockMetadata };
}

/**
 * Fetches new L2 block bodies and their hashes.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param availabilityOracleAddress - The address of the availability oracle contract.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param searchEndBlock - The highest block number that we should search up to.
 * @returns A array of tuples of L2 block bodies and their associated hash as well as the next eth block to search from
 */
export async function retrieveBlockBodiesFromAvailabilityOracle(
  publicClient: PublicClient,
  availabilityOracleAddress: EthAddress,
  blockUntilSynced: boolean,
  searchStartBlock: bigint,
  searchEndBlock: bigint,
): Promise<DataRetrieval<[Body, Buffer]>> {
  const retrievedBlockBodies: [Body, Buffer][] = [];

  do {
    if (searchStartBlock > searchEndBlock) {
      break;
    }
    const l2TxsPublishedLogs = await getTxsPublishedLogs(
      publicClient,
      availabilityOracleAddress,
      searchStartBlock,
      searchEndBlock,
    );
    if (l2TxsPublishedLogs.length === 0) {
      break;
    }

    const newBlockBodies = await processTxsPublishedLogs(publicClient, l2TxsPublishedLogs);
    retrievedBlockBodies.push(...newBlockBodies);
    searchStartBlock = l2TxsPublishedLogs[l2TxsPublishedLogs.length - 1].blockNumber! + 1n;
  } while (blockUntilSynced && searchStartBlock <= searchEndBlock);
  return { lastProcessedL1BlockNumber: searchStartBlock - 1n, retrievedData: retrievedBlockBodies };
}

/**
 * Fetch L1 to L2 messages.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param inboxAddress - The address of the inbox contract to fetch messages from.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param searchEndBlock - The highest block number that we should search up to.
 * @returns An array of InboxLeaf and next eth block to search from.
 */
export async function retrieveL1ToL2Messages(
  publicClient: PublicClient,
  inboxAddress: EthAddress,
  blockUntilSynced: boolean,
  searchStartBlock: bigint,
  searchEndBlock: bigint,
): Promise<DataRetrieval<InboxLeaf>> {
  const retrievedL1ToL2Messages: InboxLeaf[] = [];
  do {
    if (searchStartBlock > searchEndBlock) {
      break;
    }
    const messageSentLogs = await getMessageSentLogs(publicClient, inboxAddress, searchStartBlock, searchEndBlock);
    if (messageSentLogs.length === 0) {
      break;
    }
    const l1ToL2Messages = processMessageSentLogs(messageSentLogs);
    retrievedL1ToL2Messages.push(...l1ToL2Messages);
    // handles the case when there are no new messages:
    searchStartBlock = (messageSentLogs.findLast(msgLog => !!msgLog)?.blockNumber || searchStartBlock) + 1n;
  } while (blockUntilSynced && searchStartBlock <= searchEndBlock);
  return { lastProcessedL1BlockNumber: searchStartBlock - 1n, retrievedData: retrievedL1ToL2Messages };
}
