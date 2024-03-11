import { Body, L1ToL2Message, NewInboxLeaf } from '@aztec/circuit-types';
import { AppendOnlyTreeSnapshot, Fr, Header } from '@aztec/circuits.js';
import { EthAddress } from '@aztec/foundation/eth-address';

import { PublicClient } from 'viem';

import {
  getL1ToL2MessageCancelledLogs,
  getL2BlockProcessedLogs,
  getLeafInsertedLogs,
  getPendingL1ToL2MessageLogs,
  getTxsPublishedLogs,
  processCancelledL1ToL2MessagesLogs,
  processL2BlockProcessedLogs,
  processLeafInsertedLogs,
  processPendingL1ToL2MessageAddedLogs,
  processTxsPublishedLogs,
} from './eth_log_handlers.js';

/**
 * Data retrieved from logs
 */
type DataRetrieval<T> = {
  /**
   * The next block number.
   */
  nextEthBlockNumber: bigint;
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
 * @returns An array of tuples representing block metadata including the header, archive tree snapshot, and associated l1 block number; as well as the next eth block to search from.
 */
export async function retrieveBlockMetadataFromRollup(
  publicClient: PublicClient,
  rollupAddress: EthAddress,
  blockUntilSynced: boolean,
  searchStartBlock: bigint,
  searchEndBlock: bigint,
  expectedNextL2BlockNum: bigint,
): Promise<DataRetrieval<[Header, AppendOnlyTreeSnapshot, bigint]>> {
  const retrievedBlockMetadata: [Header, AppendOnlyTreeSnapshot, bigint][] = [];
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
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedBlockMetadata };
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
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedBlockBodies };
}

/**
 * Fetch new pending L1 to L2 messages.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param inboxAddress - The address of the inbox contract to fetch messages from.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param searchEndBlock - The highest block number that we should search up to.
 * @returns An array of L1ToL2Message and next eth block to search from.
 */
export async function retrieveNewPendingL1ToL2Messages(
  publicClient: PublicClient,
  inboxAddress: EthAddress,
  blockUntilSynced: boolean,
  searchStartBlock: bigint,
  searchEndBlock: bigint,
): Promise<DataRetrieval<[L1ToL2Message, bigint]>> {
  const retrievedNewL1ToL2Messages: [L1ToL2Message, bigint][] = [];
  do {
    if (searchStartBlock > searchEndBlock) {
      break;
    }
    const newL1ToL2MessageLogs = await getPendingL1ToL2MessageLogs(
      publicClient,
      inboxAddress,
      searchStartBlock,
      searchEndBlock,
    );
    if (newL1ToL2MessageLogs.length === 0) {
      break;
    }
    const newL1ToL2Messages = processPendingL1ToL2MessageAddedLogs(newL1ToL2MessageLogs);
    retrievedNewL1ToL2Messages.push(...newL1ToL2Messages);
    // handles the case when there are no new messages:
    searchStartBlock = (newL1ToL2MessageLogs.findLast(msgLog => !!msgLog)?.blockNumber || searchStartBlock) + 1n;
  } while (blockUntilSynced && searchStartBlock <= searchEndBlock);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedNewL1ToL2Messages };
}

/**
 * Fetch new L1 to L2 messages.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param newInboxAddress - The address of the inbox contract to fetch messages from.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param searchEndBlock - The highest block number that we should search up to.
 * @returns An array of NewInboxLeaf and next eth block to search from.
 */
export async function retrieveNewL1ToL2Messages(
  publicClient: PublicClient,
  newInboxAddress: EthAddress,
  blockUntilSynced: boolean,
  searchStartBlock: bigint,
  searchEndBlock: bigint,
): Promise<DataRetrieval<NewInboxLeaf>> {
  const retrievedNewL1ToL2Messages: NewInboxLeaf[] = [];
  do {
    if (searchStartBlock > searchEndBlock) {
      break;
    }
    const leafInsertedLogs = await getLeafInsertedLogs(publicClient, newInboxAddress, searchStartBlock, searchEndBlock);
    if (leafInsertedLogs.length === 0) {
      break;
    }
    const newL1ToL2Messages = processLeafInsertedLogs(leafInsertedLogs);
    retrievedNewL1ToL2Messages.push(...newL1ToL2Messages);
    // handles the case when there are no new messages:
    searchStartBlock = (leafInsertedLogs.findLast(msgLog => !!msgLog)?.blockNumber || searchStartBlock) + 1n;
  } while (blockUntilSynced && searchStartBlock <= searchEndBlock);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedNewL1ToL2Messages };
}

/**
 * Fetch newly cancelled L1 to L2 messages.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param inboxAddress - The address of the inbox contract to fetch messages from.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param searchEndBlock - The highest block number that we should search up to.
 * @returns An array of entry keys that were cancelled and next eth block to search from.
 * TODO(#4492): Nuke the following when purging the old inbox
 */
export async function retrieveNewCancelledL1ToL2Messages(
  publicClient: PublicClient,
  inboxAddress: EthAddress,
  blockUntilSynced: boolean,
  searchStartBlock: bigint,
  searchEndBlock: bigint,
): Promise<DataRetrieval<[Fr, bigint]>> {
  const retrievedNewCancelledL1ToL2Messages: [Fr, bigint][] = [];
  do {
    if (searchStartBlock > searchEndBlock) {
      break;
    }
    const newL1ToL2MessageCancelledLogs = await getL1ToL2MessageCancelledLogs(
      publicClient,
      inboxAddress,
      searchStartBlock,
      searchEndBlock,
    );
    if (newL1ToL2MessageCancelledLogs.length === 0) {
      break;
    }
    const newCancelledL1ToL2Messages = processCancelledL1ToL2MessagesLogs(newL1ToL2MessageCancelledLogs);
    retrievedNewCancelledL1ToL2Messages.push(...newCancelledL1ToL2Messages);
    // handles the case when there are no new messages:
    searchStartBlock =
      (newL1ToL2MessageCancelledLogs.findLast(msgLog => !!msgLog)?.blockNumber || searchStartBlock) + 1n;
  } while (blockUntilSynced && searchStartBlock <= searchEndBlock);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedNewCancelledL1ToL2Messages };
}
