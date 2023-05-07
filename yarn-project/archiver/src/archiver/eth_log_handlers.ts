import { Hex, Log, PublicClient, decodeFunctionData, getAbiItem, getAddress, hexToBytes } from 'viem';
import { RollupAbi, UnverifiedDataEmitterAbi } from '@aztec/l1-artifacts';
import {
  BufferReader,
  ContractData,
  ContractPublicData,
  EncodedContractFunction,
  L2Block,
  UnverifiedData,
} from '@aztec/types';
import { EthAddress } from '@aztec/foundation/eth-address';
import { AztecAddress } from '@aztec/foundation/aztec-address';

/**
 * Processes newly received UnverifiedData logs.
 * @param logs - ContractDeployment logs.
 * @returns The set of retrieved contract public data items.
 */
export function processContractDeploymentLogs(
  logs: Log<bigint, number, undefined, typeof UnverifiedDataEmitterAbi, 'ContractDeployment'>[],
) {
  const contractPublicData: (ContractPublicData[] | undefined)[] = [];
  for (const log of logs) {
    const l2BlockNum = log.args.l2BlockNum;
    const publicFnsReader = BufferReader.asReader(Buffer.from(log.args.acir.slice(2), 'hex'));
    const contractData = new ContractPublicData(
      new ContractData(AztecAddress.fromString(log.args.aztecAddress), EthAddress.fromString(log.args.portalAddress)),
      publicFnsReader.readVector(EncodedContractFunction),
    );
    if (contractPublicData[Number(l2BlockNum)]) {
      contractPublicData[Number(l2BlockNum)]?.push(contractData);
    } else {
      contractPublicData[Number(l2BlockNum)] = [contractData];
    }
  }
  return contractPublicData;
}

/**
 * Processes newly received UnverifiedData logs.
 * @param expectedRollupNumber - The next expected rollup number.
 * @param logs - UnverifiedData logs.
 */
export function processUnverifiedDataLogs(
  expectedRollupNumber: bigint,
  logs: Log<bigint, number, undefined, typeof UnverifiedDataEmitterAbi, 'UnverifiedData'>[],
) {
  const unverifiedDataChunks: UnverifiedData[] = [];
  for (const log of logs) {
    const l2BlockNum = log.args.l2BlockNum;
    if (l2BlockNum !== expectedRollupNumber) {
      throw new Error('Block number mismatch. Expected: ' + expectedRollupNumber + ' but got: ' + l2BlockNum + '.');
    }
    const unverifiedDataBuf = Buffer.from(hexToBytes(log.args.data));
    const unverifiedData = UnverifiedData.fromBuffer(unverifiedDataBuf);
    unverifiedDataChunks.push(unverifiedData);
    expectedRollupNumber++;
  }
  return unverifiedDataChunks;
}

/**
 * Processes newly received L2BlockProcessed logs.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param expectedRollupNumber - The next expected rollup number.
 * @param logs - L2BlockProcessed logs.
 */
export async function processBlockLogs(
  publicClient: PublicClient,
  expectedRollupNumber: bigint,
  logs: Log<bigint, number, undefined, typeof RollupAbi, 'L2BlockProcessed'>[],
) {
  const retrievedBlocks: L2Block[] = [];
  for (const log of logs) {
    const blockNum = log.args.blockNum;
    if (blockNum !== expectedRollupNumber) {
      throw new Error('Block number mismatch. Expected: ' + expectedRollupNumber + ' but got: ' + blockNum + '.');
    }
    // TODO: Fetch blocks from calldata in parallel
    const newBlock = await getBlockFromCallData(publicClient, log.transactionHash!, log.args.blockNum);
    retrievedBlocks.push(newBlock);
    expectedRollupNumber++;
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
  return await publicClient.getLogs({
    address: getAddress(rollupAddress.toString()),
    event: abiItem,
    fromBlock,
  });
}

/**
 * Gets relevant `UnverifiedData` logs from chain.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param unverifiedDataEmitterAddress - The address of the unverified data emitter contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @returns An array of `UnverifiedData` logs.
 */
export async function getUnverifiedDataLogs(
  publicClient: PublicClient,
  unverifiedDataEmitterAddress: EthAddress,
  fromBlock: bigint,
): Promise<any[]> {
  // Note: For some reason the return type of `getLogs` would not get correctly derived if I didn't set the abiItem
  //       as a standalone constant.
  const abiItem = getAbiItem({
    abi: UnverifiedDataEmitterAbi,
    name: 'UnverifiedData',
  });
  return await publicClient.getLogs({
    address: getAddress(unverifiedDataEmitterAddress.toString()),
    event: abiItem,
    fromBlock,
  });
}

/**
 * Gets relevant `ContractDeployment` logs from chain.
 * @param publicClient - The viem public client to use for transaction retrieval.
 * @param unverifiedDataEmitterAddress - The address of the unverified data emitter contract.
 * @param fromBlock - First block to get logs from (inclusive).
 * @returns An array of `ContractDeployment` logs.
 */
export async function getContractDeploymentLogs(
  publicClient: PublicClient,
  unverifiedDataEmitterAddress: EthAddress,
  fromBlock: bigint,
): Promise<any[]> {
  const abiItem = getAbiItem({
    abi: UnverifiedDataEmitterAbi,
    name: 'ContractDeployment',
  });
  return await publicClient.getLogs({
    address: getAddress(unverifiedDataEmitterAddress.toString()),
    event: abiItem,
    fromBlock,
  });
}
