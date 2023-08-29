import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { ExtendedContractData, L1ToL2Message, L2Block } from '@aztec/types';

import { PublicClient } from 'viem';

import {
  getContractDeploymentLogs,
  getL1ToL2MessageCancelledLogs,
  getL2BlockProcessedLogs,
  getPendingL1ToL2MessageLogs,
  processBlockLogs,
  processCancelledL1ToL2MessagesLogs,
  processContractDeploymentLogs,
  processPendingL1ToL2MessageAddedLogs,
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
 * Fetches new L2 Blocks.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param rollupAddress - The address of the rollup contract.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param currentL1BlockNum - Latest available block number in the ETH node.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param expectedNextL2BlockNum - The next L2 block number that we expect to find.
 * @returns An array of L2 Blocks and the next eth block to search from
 */
export async function retrieveBlocks(
  publicClient: PublicClient,
  rollupAddress: EthAddress,
  blockUntilSynced: boolean,
  currentL1BlockNum: bigint,
  searchStartBlock: bigint,
  expectedNextL2BlockNum: bigint,
): Promise<DataRetrieval<L2Block>> {
  const retrievedBlocks: L2Block[] = [];
  do {
    if (searchStartBlock > currentL1BlockNum) {
      break;
    }
    const l2BlockProcessedLogs = await getL2BlockProcessedLogs(publicClient, rollupAddress, searchStartBlock);
    if (l2BlockProcessedLogs.length === 0) {
      break;
    }

    const newBlocks = await processBlockLogs(publicClient, expectedNextL2BlockNum, l2BlockProcessedLogs);
    retrievedBlocks.push(...newBlocks);
    searchStartBlock = l2BlockProcessedLogs[l2BlockProcessedLogs.length - 1].blockNumber! + 1n;
    expectedNextL2BlockNum += BigInt(newBlocks.length);
  } while (blockUntilSynced && searchStartBlock <= currentL1BlockNum);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedBlocks };
}

/**
 * Fetches new contract data.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param contractDeploymentEmitterAddress - The address of the contract deployment emitter contract.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param currentBlockNumber - Latest available block number in the ETH node.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param blockHashMapping - A mapping from block number to relevant block hash.
 * @returns An array of ExtendedContractData and their equivalent L2 Block number along with the next eth block to search from..
 */
export async function retrieveNewContractData(
  publicClient: PublicClient,
  contractDeploymentEmitterAddress: EthAddress,
  blockUntilSynced: boolean,
  currentBlockNumber: bigint,
  searchStartBlock: bigint,
  blockHashMapping: { [key: number]: Buffer | undefined },
): Promise<DataRetrieval<[ExtendedContractData[], number]>> {
  let retrievedNewContracts: [ExtendedContractData[], number][] = [];
  do {
    if (searchStartBlock > currentBlockNumber) {
      break;
    }
    const contractDataLogs = await getContractDeploymentLogs(
      publicClient,
      contractDeploymentEmitterAddress,
      searchStartBlock,
    );
    if (contractDataLogs.length === 0) {
      break;
    }
    const newContracts = processContractDeploymentLogs(blockHashMapping, contractDataLogs);
    retrievedNewContracts = retrievedNewContracts.concat(newContracts);
    searchStartBlock = (contractDataLogs.findLast(cd => !!cd)?.blockNumber || searchStartBlock) + 1n;
  } while (blockUntilSynced && searchStartBlock <= currentBlockNumber);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedNewContracts };
}

/**
 * Fetch new pending L1 to L2 messages.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param inboxAddress - The address of the inbox contract to fetch messages from.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param currentBlockNumber - Latest available block number in the ETH node.
 * @param searchStartBlock - The block number to use for starting the search.
 * @returns An array of L1ToL2Message and next eth block to search from.
 */
export async function retrieveNewPendingL1ToL2Messages(
  publicClient: PublicClient,
  inboxAddress: EthAddress,
  blockUntilSynced: boolean,
  currentBlockNumber: bigint,
  searchStartBlock: bigint,
): Promise<DataRetrieval<L1ToL2Message>> {
  const retrievedNewL1ToL2Messages: L1ToL2Message[] = [];
  do {
    if (searchStartBlock > currentBlockNumber) {
      break;
    }
    const newL1ToL2MessageLogs = await getPendingL1ToL2MessageLogs(publicClient, inboxAddress, searchStartBlock);
    const newL1ToL2Messages = processPendingL1ToL2MessageAddedLogs(newL1ToL2MessageLogs);
    retrievedNewL1ToL2Messages.push(...newL1ToL2Messages);
    // handles the case when there are no new messages:
    searchStartBlock = (newL1ToL2MessageLogs.findLast(msgLog => !!msgLog)?.blockNumber || searchStartBlock) + 1n;
  } while (blockUntilSynced && searchStartBlock <= currentBlockNumber);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedNewL1ToL2Messages };
}

/**
 * Fetch newly cancelled L1 to L2 messages.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param inboxAddress - The address of the inbox contract to fetch messages from.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param currentBlockNumber - Latest available block number in the ETH node.
 * @param searchStartBlock - The block number to use for starting the search.
 * @returns An array of message keys that were cancelled and next eth block to search from.
 */
export async function retrieveNewCancelledL1ToL2Messages(
  publicClient: PublicClient,
  inboxAddress: EthAddress,
  blockUntilSynced: boolean,
  currentBlockNumber: bigint,
  searchStartBlock: bigint,
): Promise<DataRetrieval<Fr>> {
  const retrievedNewCancelledL1ToL2Messages: Fr[] = [];
  do {
    if (searchStartBlock > currentBlockNumber) {
      break;
    }
    const newL1ToL2MessageCancelledLogs = await getL1ToL2MessageCancelledLogs(
      publicClient,
      inboxAddress,
      searchStartBlock,
    );
    const newCancelledL1ToL2Messages = processCancelledL1ToL2MessagesLogs(newL1ToL2MessageCancelledLogs);
    retrievedNewCancelledL1ToL2Messages.push(...newCancelledL1ToL2Messages);
    // handles the case when there are no new messages:
    searchStartBlock =
      (newL1ToL2MessageCancelledLogs.findLast(msgLog => !!msgLog)?.blockNumber || searchStartBlock) + 1n;
  } while (blockUntilSynced && searchStartBlock <= currentBlockNumber);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedNewCancelledL1ToL2Messages };
}
