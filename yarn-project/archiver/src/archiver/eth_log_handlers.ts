import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { ContractDeploymentEmitterAbi, InboxAbi, RollupAbi } from '@aztec/l1-artifacts';
import {
  BufferReader,
  ContractData,
  ContractDataAndBytecode,
  EncodedContractFunction,
  L1Actor,
  L1ToL2Message,
  L2Actor,
  L2Block,
} from '@aztec/types';

import { Hex, Log, PublicClient, decodeFunctionData, getAbiItem, getAddress, hexToBytes } from 'viem';

/**
 * Processes newly received MessageAdded (L1 to L2) logs.
 * @param logs - MessageAdded logs.
 * @returns Array of all Pending L1 to L2 messages that were processed
 */
export function processPendingL1ToL2MessageAddedLogs(
  logs: Log<bigint, number, undefined, true, typeof InboxAbi, 'MessageAdded'>[],
): L1ToL2Message[] {
  const l1ToL2Messages: L1ToL2Message[] = [];
  for (const log of logs) {
    const { sender, senderChainId, recipient, recipientVersion, content, secretHash, deadline, fee, entryKey } =
      log.args;
    l1ToL2Messages.push(
      new L1ToL2Message(
        new L1Actor(EthAddress.fromString(sender), Number(senderChainId)),
        new L2Actor(AztecAddress.fromString(recipient), Number(recipientVersion)),
        Fr.fromString(content),
        Fr.fromString(secretHash),
        deadline,
        Number(fee),
        Fr.fromString(entryKey),
      ),
    );
  }
  return l1ToL2Messages;
}

/**
 * Process newly received L1ToL2MessageCancelled logs.
 * @param logs - L1ToL2MessageCancelled logs.
 * @returns Array of message keys of the L1 to L2 messages that were cancelled
 */
export function processCancelledL1ToL2MessagesLogs(
  logs: Log<bigint, number, undefined, true, typeof InboxAbi, 'L1ToL2MessageCancelled'>[],
): Fr[] {
  const cancelledL1ToL2Messages: Fr[] = [];
  for (const log of logs) {
    cancelledL1ToL2Messages.push(Fr.fromString(log.args.entryKey));
  }
  return cancelledL1ToL2Messages;
}

/**
 * Processes newly received L2BlockProcessed logs.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param expectedL2BlockNumber - The next expected L2 block number.
 * @param logs - L2BlockProcessed logs.
 */
export async function processBlockLogs(
  publicClient: PublicClient,
  expectedL2BlockNumber: bigint,
  logs: Log<bigint, number, undefined, true, typeof RollupAbi, 'L2BlockProcessed'>[],
) {
  const retrievedBlocks: L2Block[] = [];
  for (const log of logs) {
    const blockNum = log.args.blockNum;
    if (blockNum !== expectedL2BlockNumber) {
      throw new Error('Block number mismatch. Expected: ' + expectedL2BlockNumber + ' but got: ' + blockNum + '.');
    }
    // TODO: Fetch blocks from calldata in parallel
    const newBlock = await getBlockFromCallData(publicClient, log.transactionHash!, log.args.blockNum);
    retrievedBlocks.push(newBlock);
    expectedL2BlockNumber++;
  }
  return retrievedBlocks;
}

/**
 * Builds an L2 block out of calldata from the tx that published it.
 * Assumes that the block was published from an EOA.
 * TODO: Add retries and error management.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param txHash - Hash of the tx that published it.
 * @param l2BlockNum - L2 block number.
 * @returns An L2 block deserialized from the calldata.
 */
async function getBlockFromCallData(
  publicClient: PublicClient,
  txHash: `0x${string}`,
  l2BlockNum: bigint,
): Promise<L2Block> {
  const { input: data } = await publicClient.getTransaction({ hash: txHash });
  // TODO: File a bug in viem who complains if we dont remove the ctor from the abi here
  const { functionName, args } = decodeFunctionData({
    abi: RollupAbi.filter(item => item.type.toString() !== 'constructor'),
    data,
  });
  if (functionName !== 'process') throw new Error(`Unexpected method called ${functionName}`);
  const [, l2BlockHex] = args! as [Hex, Hex];
  const block = L2Block.decode(Buffer.from(hexToBytes(l2BlockHex)));
  if (BigInt(block.number) !== l2BlockNum) {
    throw new Error(`Block number mismatch: expected ${l2BlockNum} but got ${block.number}`);
  }
  return block;
}

/**
 * Gets relevant `L2BlockProcessed` logs from chain.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param rollupAddress - The address of the rollup contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @returns An array of `L2BlockProcessed` logs.
 */
export async function getL2BlockProcessedLogs(
  publicClient: PublicClient,
  rollupAddress: EthAddress,
  fromBlock: bigint,
) {
  // Note: For some reason the return type of `getLogs` would not get correctly derived if I didn't set the abiItem
  //       as a standalone constant.
  const abiItem = getAbiItem({
    abi: RollupAbi,
    name: 'L2BlockProcessed',
  });
  return await publicClient.getLogs<typeof abiItem, true>({
    address: getAddress(rollupAddress.toString()),
    event: abiItem,
    fromBlock,
  });
}

/**
 * Gets relevant `ContractDeployment` logs from chain.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param contractDeploymentEmitterAddress - The address of the L2 contract deployment emitter contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @returns An array of `ContractDeployment` logs.
 */
export async function getContractDeploymentLogs(
  publicClient: PublicClient,
  contractDeploymentEmitterAddress: EthAddress,
  fromBlock: bigint,
): Promise<Log<bigint, number, undefined, true, typeof ContractDeploymentEmitterAbi, 'ContractDeployment'>[]> {
  const abiItem = getAbiItem({
    abi: ContractDeploymentEmitterAbi,
    name: 'ContractDeployment',
  });
  return await publicClient.getLogs({
    address: getAddress(contractDeploymentEmitterAddress.toString()),
    event: abiItem,
    fromBlock,
  });
}

/**
 * Processes newly received ContractDeployment logs.
 * @param blockHashMapping - A mapping from block number to relevant block hash.
 * @param logs - ContractDeployment logs.
 * @returns The set of retrieved contract data and bytecode items.
 */
export function processContractDeploymentLogs(
  blockHashMapping: { [key: number]: Buffer | undefined },
  logs: Log<bigint, number, undefined, true, typeof ContractDeploymentEmitterAbi, 'ContractDeployment'>[],
): [ContractDataAndBytecode[], number][] {
  const contractDataAndBytecode: [ContractDataAndBytecode[], number][] = [];
  for (let i = 0; i < logs.length; i++) {
    const log = logs[i];
    const l2BlockNum = Number(log.args.l2BlockNum);
    const blockHash = Buffer.from(hexToBytes(log.args.l2BlockHash));
    const expectedBlockHash = blockHashMapping[l2BlockNum];
    if (expectedBlockHash === undefined || !blockHash.equals(expectedBlockHash)) {
      continue;
    }
    const publicFnsReader = BufferReader.asReader(Buffer.from(log.args.acir.slice(2), 'hex'));
    const contractData = new ContractDataAndBytecode(
      new ContractData(AztecAddress.fromString(log.args.aztecAddress), EthAddress.fromString(log.args.portalAddress)),
      publicFnsReader.readVector(EncodedContractFunction),
    );
    if (contractDataAndBytecode[i]) {
      contractDataAndBytecode[i][0].push(contractData);
    } else {
      contractDataAndBytecode[i] = [[contractData], l2BlockNum];
    }
  }
  return contractDataAndBytecode;
}

/**
 * Get relevant `MessageAdded` logs emitted by Inbox on chain.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param inboxAddress - The address of the inbox contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @returns An array of `MessageAdded` logs.
 */
export async function getPendingL1ToL2MessageLogs(
  publicClient: PublicClient,
  inboxAddress: EthAddress,
  fromBlock: bigint,
): Promise<Log<bigint, number, undefined, true, typeof InboxAbi, 'MessageAdded'>[]> {
  const abiItem = getAbiItem({
    abi: InboxAbi,
    name: 'MessageAdded',
  });
  return await publicClient.getLogs({
    address: getAddress(inboxAddress.toString()),
    event: abiItem,
    fromBlock,
  });
}

/**
 * Get relevant `L1ToL2MessageCancelled` logs emitted by Inbox on chain when pending messages are cancelled
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param inboxAddress - The address of the inbox contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @returns An array of `L1ToL2MessageCancelled` logs.
 */
export async function getL1ToL2MessageCancelledLogs(
  publicClient: PublicClient,
  inboxAddress: EthAddress,
  fromBlock: bigint,
): Promise<Log<bigint, number, undefined, true, typeof InboxAbi, 'L1ToL2MessageCancelled'>[]> {
  const abiItem = getAbiItem({
    abi: InboxAbi,
    name: 'L1ToL2MessageCancelled',
  });
  return await publicClient.getLogs({
    address: getAddress(inboxAddress.toString()),
    event: abiItem,
    fromBlock,
  });
}
