import { PublicClient } from 'viem';
import {
  getContractDeploymentLogs,
  getL2BlockProcessedLogs,
  getUnverifiedDataLogs,
  processBlockLogs,
  processContractDeploymentLogs,
  processUnverifiedDataLogs,
} from './eth_log_handlers.js';
import { EthAddress } from '@aztec/foundation/eth-address';
import { ContractPublicData, L2Block, UnverifiedData } from '@aztec/types';

/**
 * Fetches new L2 Blocks.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param rollupAddress - The address of the rollup contract.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param currentBlockNumber - Latest available block number in the ETH node.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param expectedNextRollupNumber - The next rollup id that we expect to find.
 */
export async function retrieveBlocks(
  publicClient: PublicClient,
  rollupAddress: EthAddress,
  blockUntilSynced: boolean,
  currentBlockNumber: bigint,
  searchStartBlock: bigint,
  expectedNextRollupNumber: bigint,
) {
  const retrievedBlocks: L2Block[] = [];
  do {
    if (searchStartBlock > currentBlockNumber) {
      break;
    }

    const l2BlockProcessedLogs = await getL2BlockProcessedLogs(publicClient, rollupAddress, searchStartBlock);

    if (l2BlockProcessedLogs.length === 0) {
      break;
    }

    const newBlocks = await processBlockLogs(publicClient, expectedNextRollupNumber, l2BlockProcessedLogs);
    retrievedBlocks.push(...newBlocks);
    searchStartBlock = l2BlockProcessedLogs[l2BlockProcessedLogs.length - 1].blockNumber! + 1n;
    expectedNextRollupNumber += BigInt(newBlocks.length);
  } while (blockUntilSynced && searchStartBlock <= currentBlockNumber);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedBlocks };
}

/**
 * Fetches new unverified data.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param unverifiedDataEmitterAddress - The address of the unverified data emitter contract.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param currentBlockNumber - Latest available block number in the ETH node.
 * @param searchStartBlock - The block number to use for starting the search.
 * @param expectedNextRollupNumber - The next rollup id that we expect to find.
 */
export async function retrieveUnverifiedData(
  publicClient: PublicClient,
  unverifiedDataEmitterAddress: EthAddress,
  blockUntilSynced: boolean,
  currentBlockNumber: bigint,
  searchStartBlock: bigint,
  expectedNextRollupNumber: bigint,
) {
  const newUnverifiedDataChunks: UnverifiedData[] = [];
  do {
    if (searchStartBlock > currentBlockNumber) {
      break;
    }

    const unverifiedDataLogs = await getUnverifiedDataLogs(
      publicClient,
      unverifiedDataEmitterAddress,
      searchStartBlock,
    );

    if (unverifiedDataLogs.length === 0) {
      break;
    }

    const newChunks = processUnverifiedDataLogs(expectedNextRollupNumber, unverifiedDataLogs);
    newUnverifiedDataChunks.push(...newChunks);
    searchStartBlock = unverifiedDataLogs[unverifiedDataLogs.length - 1].blockNumber + 1n;
    expectedNextRollupNumber += BigInt(newChunks.length);
  } while (blockUntilSynced && searchStartBlock <= currentBlockNumber);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: newUnverifiedDataChunks };
}

/**
 * Fetches new contract data.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param unverifiedDataEmitterAddress - The address of the unverified data emitter contract.
 * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
 * @param currentBlockNumber - Latest available block number in the ETH node.
 * @param searchStartBlock - The block number to use for starting the search.
 */
export async function retrieveNewContractData(
  publicClient: PublicClient,
  unverifiedDataEmitterAddress: EthAddress,
  blockUntilSynced: boolean,
  currentBlockNumber: bigint,
  searchStartBlock: bigint,
) {
  let retrievedNewContracts: (ContractPublicData[] | undefined)[] = [];
  do {
    if (searchStartBlock > currentBlockNumber) {
      break;
    }

    const contractDataLogs = await getContractDeploymentLogs(
      publicClient,
      unverifiedDataEmitterAddress,
      searchStartBlock,
    );
    const newContracts = processContractDeploymentLogs(contractDataLogs);
    retrievedNewContracts = retrievedNewContracts.concat(newContracts);

    searchStartBlock = (contractDataLogs.findLast(cd => !!cd)?.blockNumber || searchStartBlock) + 1n;
  } while (blockUntilSynced && searchStartBlock <= currentBlockNumber);
  return { nextEthBlockNumber: searchStartBlock, retrievedData: retrievedNewContracts };
}
