import { AztecAddress, BufferReader, EthAddress, RunningPromise, createDebugLogger } from '@aztec/foundation';
import { INITIAL_L2_BLOCK_NUM } from '@aztec/l1-contracts';
import { RollupAbi, UnverifiedDataEmitterAbi } from '@aztec/l1-contracts/viem';
import {
  ContractData,
  ContractPublicData,
  ContractDataSource,
  EncodedContractFunction,
  L2Block,
  L2BlockSource,
  UnverifiedData,
  UnverifiedDataSource,
} from '@aztec/types';
import {
  Chain,
  Hex,
  HttpTransport,
  Log,
  PublicClient,
  createPublicClient,
  decodeFunctionData,
  getAbiItem,
  getAddress,
  hexToBytes,
  http,
} from 'viem';
import { localhost } from 'viem/chains';
import { ArchiverConfig } from './config.js';

/**
 * Pulls L2 blocks in a non-blocking manner and provides interface for their retrieval.
 * Responsible for handling robust L1 polling so that other components do not need to
 * concern themselves with it.
 */
export class Archiver implements L2BlockSource, UnverifiedDataSource, ContractDataSource {
  /**
   * A promise in which we will be continually fetching new L2 blocks.
   */
  private runningPromise?: RunningPromise;

  /**
   * An array containing all the L2 blocks that have been fetched so far.
   */
  private l2Blocks: L2Block[] = [];

  /**
   * An array containing all the `unverifiedData` that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private unverifiedData: UnverifiedData[] = [];

  /**
   * A sparse array containing all the contract data that have been fetched so far.
   */
  private contractPublicData: (ContractPublicData[] | undefined)[] = [];

  /**
   * Next L1 block number to fetch `L2BlockProcessed` logs from (i.e. `fromBlock` in eth_getLogs).
   */
  private nextL2BlockFromBlock = 0n;

  /**
   * Next L1 block number to fetch `UnverifiedData` logs from (i.e. `fromBlock` in eth_getLogs)
   */
  private nextUnverifiedDataFromBlock = 0n;

  /**
   * Next L1 block number to fetch `ContractPublicData` logs from (i.e. `fromBlock` in eth_getLogs)
   */
  private nextContractDataFromBlock = 0n;

  /**
   * Creates a new instance of the Archiver.
   * @param publicClient - A client for interacting with the Ethereum node.
   * @param rollupAddress - Ethereum address of the rollup contract.
   * @param unverifiedDataEmitterAddress - Ethereum address of the unverifiedDataEmitter contract.
   * @param pollingInterval - The interval for polling for rollup logs.
   * @param log - A logger.
   */
  constructor(
    private readonly publicClient: PublicClient<HttpTransport, Chain>,
    private readonly rollupAddress: EthAddress,
    private readonly unverifiedDataEmitterAddress: EthAddress,
    private readonly pollingIntervalMs = 10_000,
    private readonly log = createDebugLogger('aztec:archiver'),
  ) {}

  /**
   * Creates a new instance of the Archiver and blocks until it syncs from chain.
   * @param config - The archiver's desired configuration.
   * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
   * @returns - An instance of the archiver.
   */
  public static async createAndSync(config: ArchiverConfig, blockUntilSynced = true): Promise<Archiver> {
    const publicClient = createPublicClient({
      chain: localhost,
      transport: http(config.rpcUrl),
    });
    const archiver = new Archiver(
      publicClient,
      config.rollupContract,
      config.unverifiedDataEmitterContract,
      config.archiverPollingInterval,
    );
    await archiver.start(blockUntilSynced);
    return archiver;
  }

  /**
   * Starts sync process.
   * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
   */
  public async start(blockUntilSynced: boolean): Promise<void> {
    if (this.runningPromise) {
      throw new Error('Archiver is already running');
    }

    if (blockUntilSynced) {
      await this.sync(blockUntilSynced);
    }

    this.runningPromise = new RunningPromise(() => this.sync(false), this.pollingIntervalMs);
    this.runningPromise.start();
  }

  /**
   * Fetches `L2BlockProcessed` and `UnverifiedData` logs from `nextL2BlockFromBlock` and
   * `nextUnverifiedDataFromBlock` and processes them.
   * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
   */
  private async sync(blockUntilSynced: boolean) {
    const currentBlockNumber = await this.publicClient.getBlockNumber();

    await this.syncBlocks(blockUntilSynced, currentBlockNumber);
    await this.syncUnverifiedData(blockUntilSynced, currentBlockNumber);
    await this.syncNewContractData(blockUntilSynced, currentBlockNumber);
  }

  private async syncBlocks(blockUntilSynced: boolean, currentBlockNumber: bigint) {
    do {
      if (this.nextL2BlockFromBlock > currentBlockNumber) {
        break;
      }

      this.log(`Synching L2BlockProcessed logs from block ${this.nextL2BlockFromBlock}`);
      const l2BlockProcessedLogs = await this.getL2BlockProcessedLogs(this.nextL2BlockFromBlock);

      if (l2BlockProcessedLogs.length === 0) {
        break;
      }

      await this.processBlockLogs(l2BlockProcessedLogs);

      // Setting `nextL2BlockFromBlock` to the block number of the last log + 1 because last log's block is the only
      // block we can be sure was synced to by the ETH node.
      this.nextL2BlockFromBlock = l2BlockProcessedLogs[l2BlockProcessedLogs.length - 1].blockNumber! + 1n;
    } while (blockUntilSynced && this.nextL2BlockFromBlock <= currentBlockNumber);
  }

  private async syncUnverifiedData(blockUntilSynced: boolean, currentBlockNumber: bigint) {
    do {
      if (this.nextUnverifiedDataFromBlock > currentBlockNumber) {
        break;
      }

      this.log(`Synching UnverifiedData logs from block ${this.nextUnverifiedDataFromBlock}`);
      const unverifiedDataLogs = await this.getUnverifiedDataLogs(this.nextUnverifiedDataFromBlock);

      if (unverifiedDataLogs.length === 0) {
        break;
      }

      this.processUnverifiedDataLogs(unverifiedDataLogs);

      this.nextUnverifiedDataFromBlock = unverifiedDataLogs[unverifiedDataLogs.length - 1].blockNumber + 1n;
    } while (blockUntilSynced && this.nextUnverifiedDataFromBlock <= currentBlockNumber);
  }

  private async syncNewContractData(blockUntilSynced: boolean, currentBlockNumber: bigint) {
    do {
      if (this.nextContractDataFromBlock > currentBlockNumber) {
        break;
      }

      this.log(`Syncing ContractData logs from block ${this.nextContractDataFromBlock}`);
      const contractDataLogs = await this.getContractDataLogs(this.nextContractDataFromBlock);

      this.processContractDataLogs(contractDataLogs);
      this.nextContractDataFromBlock =
        (contractDataLogs.findLast(cd => !!cd)?.blockNumber || this.nextContractDataFromBlock) + 1n;
    } while (blockUntilSynced && this.nextContractDataFromBlock <= currentBlockNumber);
  }

  /**
   * Gets relevant `L2BlockProcessed` logs from chain.
   * @param fromBlock - First block to get logs from (inclusive).
   * @returns An array of `L2BlockProcessed` logs.
   */
  private async getL2BlockProcessedLogs(fromBlock: bigint) {
    // Note: For some reason the return type of `getLogs` would not get correctly derived if I didn't set the abiItem
    //       as a standalone constant.
    const abiItem = getAbiItem({
      abi: RollupAbi,
      name: 'L2BlockProcessed',
    });
    return await this.publicClient.getLogs({
      address: getAddress(this.rollupAddress.toString()),
      event: abiItem,
      fromBlock,
    });
  }

  /**
   * Gets relevant `UnverifiedData` logs from chain.
   * @param fromBlock - First block to get logs from (inclusive).
   * @returns An array of `UnverifiedData` logs.
   */
  private async getUnverifiedDataLogs(fromBlock: bigint): Promise<any[]> {
    // Note: For some reason the return type of `getLogs` would not get correctly derived if I didn't set the abiItem
    //       as a standalone constant.
    const abiItem = getAbiItem({
      abi: UnverifiedDataEmitterAbi,
      name: 'UnverifiedData',
    });
    return await this.publicClient.getLogs({
      address: getAddress(this.unverifiedDataEmitterAddress.toString()),
      event: abiItem,
      fromBlock,
    });
  }

  private async getContractDataLogs(fromBlock: bigint) {
    const abiItem = getAbiItem({
      abi: UnverifiedDataEmitterAbi,
      name: 'ContractDeployment',
    });
    return await this.publicClient.getLogs({
      address: getAddress(this.unverifiedDataEmitterAddress.toString()),
      event: abiItem,
      fromBlock,
    });
  }

  /**
   * Processes newly received L2BlockProcessed logs.
   * @param logs - L2BlockProcessed logs.
   */
  private async processBlockLogs(logs: Log<bigint, number, undefined, typeof RollupAbi, 'L2BlockProcessed'>[]) {
    for (const log of logs) {
      const blockNum = log.args.blockNum;
      if (blockNum !== BigInt(this.l2Blocks.length + INITIAL_L2_BLOCK_NUM)) {
        throw new Error(
          'Block number mismatch. Expected: ' +
            (this.l2Blocks.length + INITIAL_L2_BLOCK_NUM) +
            ' but got: ' +
            blockNum +
            '.',
        );
      }
      // TODO: Fetch blocks from calldata in parallel
      const newBlock = await this.getBlockFromCallData(log.transactionHash!, log.args.blockNum);
      this.l2Blocks.push(newBlock);
      this.log(`Processed block ${newBlock.number}.`);
    }
  }

  /**
   * Processes newly received UnverifiedData logs.
   * @param logs - UnverifiedData logs.
   */
  private processUnverifiedDataLogs(
    logs: Log<bigint, number, undefined, typeof UnverifiedDataEmitterAbi, 'UnverifiedData'>[],
  ) {
    for (const log of logs) {
      const l2BlockNum = log.args.l2BlockNum;
      if (l2BlockNum !== BigInt(this.unverifiedData.length + INITIAL_L2_BLOCK_NUM)) {
        throw new Error(
          'Block number mismatch. Expected: ' +
            (this.unverifiedData.length + INITIAL_L2_BLOCK_NUM) +
            ' but got: ' +
            l2BlockNum +
            '.',
        );
      }
      const unverifiedDataBuf = Buffer.from(hexToBytes(log.args.data));
      const unverifiedData = UnverifiedData.fromBuffer(unverifiedDataBuf);
      this.unverifiedData.push(unverifiedData);
    }
    this.log('Processed unverifiedData corresponding to ' + logs.length + ' blocks.');
  }

  private processContractDataLogs(
    logs: Log<bigint, number, undefined, typeof UnverifiedDataEmitterAbi, 'ContractDeployment'>[],
  ) {
    for (const log of logs) {
      const l2BlockNum = log.args.l2BlockNum;
      const publicFnsReader = BufferReader.asReader(Buffer.from(log.args.acir.slice(2), 'hex'));
      const contractData = new ContractPublicData(
        new ContractData(AztecAddress.fromString(log.args.aztecAddress), EthAddress.fromString(log.args.portalAddress)),
        publicFnsReader.readVector(EncodedContractFunction),
      );
      (this.contractPublicData[Number(l2BlockNum)] || []).push(contractData);
    }
    this.log('Processed contractData corresponding to ' + logs.length + ' blocks.');
  }

  /**
   * Builds an L2 block out of calldata from the tx that published it.
   * Assumes that the block was published from an EOA.
   * TODO: Add retries and error management.
   * @param txHash - Hash of the tx that published it.
   * @param l2BlockNum - L2 block number.
   * @returns An L2 block deserialized from the calldata.
   */
  private async getBlockFromCallData(txHash: `0x${string}`, l2BlockNum: bigint): Promise<L2Block> {
    const { input: data } = await this.publicClient.getTransaction({ hash: txHash });
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
   * Stops the archiver.
   * @returns A promise signalling completion of the stop process.
   */
  public async stop(): Promise<void> {
    this.log('Stopping...');
    await this.runningPromise?.stop();

    this.log('Stopped.');
    return Promise.resolve();
  }

  /**
   * Gets the `take` amount of L2 blocks starting from `from`.
   * @param from - Number of the first block to return (inclusive).
   * @param take - The number of blocks to return.
   * @returns The requested L2 blocks.
   */
  public getL2Blocks(from: number, take: number): Promise<L2Block[]> {
    if (from < INITIAL_L2_BLOCK_NUM) {
      throw new Error(`Invalid block range ${from}`);
    }
    if (from > this.l2Blocks.length) {
      return Promise.resolve([]);
    }
    const startIndex = from - INITIAL_L2_BLOCK_NUM;
    const endIndex = startIndex + take;
    return Promise.resolve(this.l2Blocks.slice(startIndex, endIndex));
  }

  /**
   * Lookup the L2 contract data for this contract.
   * Contains information such as the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns The contract data.
   */
  public getL2ContractPublicData(contractAddress: AztecAddress): Promise<ContractPublicData | undefined> {
    // TODO: perhaps store contract data by address as well? to make this more efficient
    let result;
    for (let i = INITIAL_L2_BLOCK_NUM; i < this.contractPublicData.length; i++) {
      const contracts = this.contractPublicData[i];
      const contract = contracts?.find(c => c.contractData.contractAddress.equals(contractAddress));
      if (contract) {
        result = contract;
        break;
      }
    }
    return Promise.resolve(result);
  }

  /**
   * Lookup all contract data in an L2 block.
   * @param blockNumber - The block number to get all contract data from.
   * @returns All new contract data in the block (if found)
   */
  public getL2ContractPublicDataInBlock(blockNum: number): Promise<ContractPublicData[]> {
    if (blockNum > this.l2Blocks.length) {
      return Promise.resolve([]);
    }
    const contractData = this.contractPublicData[blockNum];
    return Promise.resolve(contractData || []);
  }

  /**
   * Lookup the L2 contract info for this contract.
   * Contains contract address & the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  public getL2ContractInfo(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    for (const block of this.l2Blocks) {
      for (const contractData of block.newContractData) {
        if (contractData.contractAddress.equals(contractAddress)) {
          return Promise.resolve(contractData);
        }
      }
    }
    return Promise.resolve(undefined);
  }

  /**
   * Lookup the L2 contract info inside a block.
   * Contains contract address & the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  public getL2ContractInfoInBlock(blockNum: number): Promise<ContractData[] | undefined> {
    if (blockNum > this.l2Blocks.length) {
      return Promise.resolve([]);
    }
    const block = this.l2Blocks[blockNum];
    return Promise.resolve(block.newContractData);
  }

  public async getPublicFunction(
    address: AztecAddress,
    functionSelector: Buffer,
  ): Promise<EncodedContractFunction | undefined> {
    const contractData = await this.getL2ContractPublicData(address);
    const result = contractData?.publicFunctions?.find(fn => fn.functionSelector.equals(functionSelector));
    return result;
  }

  /**
   * Gets the `take` amount of unverified data starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first `unverifiedData` to be returned.
   * @param take - The number of `unverifiedData` to return.
   * @returns The requested `unverifiedData`.
   */
  public getUnverifiedData(from: number, take: number): Promise<UnverifiedData[]> {
    if (from < INITIAL_L2_BLOCK_NUM) {
      throw new Error(`Invalid block range ${from}`);
    }
    if (from > this.unverifiedData.length) {
      return Promise.resolve([]);
    }
    const startIndex = from - INITIAL_L2_BLOCK_NUM;
    const endIndex = startIndex + take;
    return Promise.resolve(this.unverifiedData.slice(startIndex, endIndex));
  }

  /**
   * Gets the number of the latest L2 block processed by the block source implementation.
   * @returns The number of the latest L2 block processed by the block source implementation.
   */
  public getBlockHeight(): Promise<number> {
    if (this.l2Blocks.length === 0) return Promise.resolve(INITIAL_L2_BLOCK_NUM - 1);
    return Promise.resolve(this.l2Blocks[this.l2Blocks.length - 1].number);
  }

  /**
   * Gets the L2 block number associated with the latest unverified data.
   * @returns The L2 block number associated with the latest unverified data.
   */
  public getLatestUnverifiedDataBlockNum(): Promise<number> {
    if (this.unverifiedData.length === 0) return Promise.resolve(INITIAL_L2_BLOCK_NUM - 1);
    return Promise.resolve(this.unverifiedData.length + INITIAL_L2_BLOCK_NUM - 1);
  }
}
