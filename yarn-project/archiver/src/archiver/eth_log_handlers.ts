import { Body, InboxLeaf } from '@aztec/circuit-types';
import { AppendOnlyTreeSnapshot, Header } from '@aztec/circuits.js';
import { type EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { numToUInt32BE } from '@aztec/foundation/serialize';
import { AvailabilityOracleAbi, InboxAbi, RollupAbi } from '@aztec/l1-artifacts';

import { type Hex, type Log, type PublicClient, decodeFunctionData, getAbiItem, getAddress, hexToBytes } from 'viem';

/**
 * Processes newly received MessageSent (L1 to L2) logs.
 * @param logs - MessageSent logs.
 * @returns Array of all processed MessageSent logs
 */
export function processMessageSentLogs(
  logs: Log<bigint, number, false, undefined, true, typeof InboxAbi, 'MessageSent'>[],
): InboxLeaf[] {
  const leaves: InboxLeaf[] = [];
  for (const log of logs) {
    const { l2BlockNumber, index, hash } = log.args;
    leaves.push(new InboxLeaf(l2BlockNumber, index, Fr.fromString(hash)));
  }
  return leaves;
}

/**
 * Processes newly received L2BlockProcessed logs.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param expectedL2BlockNumber - The next expected L2 block number.
 * @param logs - L2BlockProcessed logs.
 * @returns - An array of tuples representing block metadata including the header, archive tree snapshot.
 */
export async function processL2BlockProcessedLogs(
  publicClient: PublicClient,
  expectedL2BlockNumber: bigint,
  logs: Log<bigint, number, false, undefined, true, typeof RollupAbi, 'L2BlockProcessed'>[],
): Promise<[Header, AppendOnlyTreeSnapshot][]> {
  const retrievedBlockMetadata: [Header, AppendOnlyTreeSnapshot][] = [];
  for (const log of logs) {
    const blockNum = log.args.blockNumber;
    if (blockNum !== expectedL2BlockNumber) {
      throw new Error('Block number mismatch. Expected: ' + expectedL2BlockNumber + ' but got: ' + blockNum + '.');
    }
    // TODO: Fetch blocks from calldata in parallel
    const [header, archive] = await getBlockMetadataFromRollupTx(
      publicClient,
      log.transactionHash!,
      log.args.blockNumber,
    );

    retrievedBlockMetadata.push([header, archive]);
    expectedL2BlockNumber++;
  }

  return retrievedBlockMetadata;
}

export async function processTxsPublishedLogs(
  publicClient: PublicClient,
  logs: Log<bigint, number, false, undefined, true, typeof AvailabilityOracleAbi, 'TxsPublished'>[],
): Promise<[Body, Buffer][]> {
  const retrievedBlockBodies: [Body, Buffer][] = [];
  for (const log of logs) {
    const newBlockBody = await getBlockBodiesFromAvailabilityOracleTx(publicClient, log.transactionHash!);
    retrievedBlockBodies.push([newBlockBody, Buffer.from(hexToBytes(log.args.txsEffectsHash))]);
  }

  return retrievedBlockBodies;
}

/**
 * Gets block metadata (header and archive snapshot) from the calldata of an L1 transaction.
 * Assumes that the block was published from an EOA.
 * TODO: Add retries and error management.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param txHash - Hash of the tx that published it.
 * @param l2BlockNum - L2 block number.
 * @returns L2 block metadata (header and archive) from the calldata, deserialized
 */
async function getBlockMetadataFromRollupTx(
  publicClient: PublicClient,
  txHash: `0x${string}`,
  l2BlockNum: bigint,
): Promise<[Header, AppendOnlyTreeSnapshot]> {
  const { input: data } = await publicClient.getTransaction({ hash: txHash });
  const { functionName, args } = decodeFunctionData({
    abi: RollupAbi,
    data,
  });

  if (functionName !== 'process') {
    throw new Error(`Unexpected method called ${functionName}`);
  }
  const [headerHex, archiveRootHex] = args! as readonly [Hex, Hex];

  const header = Header.fromBuffer(Buffer.from(hexToBytes(headerHex)));

  const blockNumberFromHeader = header.globalVariables.blockNumber.toBigInt();

  if (blockNumberFromHeader !== l2BlockNum) {
    throw new Error(`Block number mismatch: expected ${l2BlockNum} but got ${blockNumberFromHeader}`);
  }

  const archive = AppendOnlyTreeSnapshot.fromBuffer(
    Buffer.concat([
      Buffer.from(hexToBytes(archiveRootHex)), // L2Block.archive.root
      numToUInt32BE(Number(l2BlockNum)), // L2Block.archive.nextAvailableLeafIndex
    ]),
  );

  return [header, archive];
}

/**
 * Gets block bodies from calldata of an L1 transaction, and deserializes them into Body objects.
 * Assumes that the block was published from an EOA.
 * TODO: Add retries and error management.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param txHash - Hash of the tx that published it.
 * @returns An L2 block body from the calldata, deserialized
 */
async function getBlockBodiesFromAvailabilityOracleTx(
  publicClient: PublicClient,
  txHash: `0x${string}`,
): Promise<Body> {
  const { input: data } = await publicClient.getTransaction({ hash: txHash });
  const { functionName, args } = decodeFunctionData({
    abi: AvailabilityOracleAbi,
    data,
  });

  if (functionName !== 'publish') {
    throw new Error(`Unexpected method called ${functionName}`);
  }

  const [bodyHex] = args! as [Hex];

  const blockBody = Body.fromBuffer(Buffer.from(hexToBytes(bodyHex)));

  return blockBody;
}

/**
 * Gets relevant `L2BlockProcessed` logs from chain.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param rollupAddress - The address of the rollup contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @param toBlock - Last block to get logs from (inclusive).
 * @returns An array of `L2BlockProcessed` logs.
 */
export function getL2BlockProcessedLogs(
  publicClient: PublicClient,
  rollupAddress: EthAddress,
  fromBlock: bigint,
  toBlock: bigint,
): Promise<Log<bigint, number, false, undefined, true, typeof RollupAbi, 'L2BlockProcessed'>[]> {
  return publicClient.getLogs({
    address: getAddress(rollupAddress.toString()),
    event: getAbiItem({
      abi: RollupAbi,
      name: 'L2BlockProcessed',
    }),
    fromBlock,
    toBlock: toBlock + 1n, // the toBlock argument in getLogs is exclusive
  });
}

/**
 * Gets relevant `TxsPublished` logs from chain.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param dataAvailabilityOracleAddress - The address of the availability oracle contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @param toBlock - Last block to get logs from (inclusive).
 * @returns An array of `TxsPublished` logs.
 */
export function getTxsPublishedLogs(
  publicClient: PublicClient,
  dataAvailabilityOracleAddress: EthAddress,
  fromBlock: bigint,
  toBlock: bigint,
): Promise<Log<bigint, number, false, undefined, true, typeof AvailabilityOracleAbi, 'TxsPublished'>[]> {
  return publicClient.getLogs({
    address: getAddress(dataAvailabilityOracleAddress.toString()),
    event: getAbiItem({
      abi: AvailabilityOracleAbi,
      name: 'TxsPublished',
    }),
    fromBlock,
    toBlock: toBlock + 1n, // the toBlock argument in getLogs is exclusive
  });
}

/**
 * Get relevant `MessageSent` logs emitted by Inbox on chain.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param inboxAddress - The address of the inbox contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @param toBlock - Last block to get logs from (inclusive).
 * @returns An array of `MessageSent` logs.
 */
export function getMessageSentLogs(
  publicClient: PublicClient,
  inboxAddress: EthAddress,
  fromBlock: bigint,
  toBlock: bigint,
): Promise<Log<bigint, number, false, undefined, true, typeof InboxAbi, 'MessageSent'>[]> {
  return publicClient.getLogs({
    address: getAddress(inboxAddress.toString()),
    event: getAbiItem({
      abi: InboxAbi,
      name: 'MessageSent',
    }),
    fromBlock,
    toBlock: toBlock + 1n, // the toBlock argument in getLogs is exclusive
  });
}
