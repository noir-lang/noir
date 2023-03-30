import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { AztecAddress, createDebugLogger, toBigIntBE } from '@aztec/foundation';
import { RollupAbi, YeeterAbi } from '@aztec/l1-contracts/viem';
import { ContractData, L2Block, L2BlockSource, UnverifiedData, UnverifiedDataSource } from '@aztec/l2-block';
import { createPublicClient, decodeFunctionData, getAddress, Hex, hexToBytes, http, Log, PublicClient } from 'viem';
import { localhost } from 'viem/chains';
import { ArchiverConfig } from './config.js';
import { INITIAL_L2_BLOCK_NUM } from '@aztec/l1-contracts';

/**
 * Pulls L2 blocks in a non-blocking manner and provides interface for their retrieval.
 */
export class Archiver implements L2BlockSource, UnverifiedDataSource {
  /**
   * An array containing all the L2 blocks that have been fetched so far.
   */
  private l2Blocks: L2Block[] = [];

  /**
   * An array containing all the `unverifiedData` that have been fetched so far.
   * Note: Index in the "outer" array equals to (corresponding L2 block's number - INITIAL_L2_BLOCK_NUM).
   */
  private unverifiedDatas: UnverifiedData[] = [];

  private unwatchBlocks: (() => void) | undefined;
  private unwatchYeets: (() => void) | undefined;

  /**
   * Creates a new instance of the Archiver.
   * @param publicClient - A client for interacting with the Ethereum node.
   * @param rollupAddress - Ethereum address of the rollup contract.
   * @param yeeterAddress - Ethereum address of the yeeter contract.
   * @param pollingInterval - The interval for polling for rollup events.
   * @param log - A logger.
   */
  constructor(
    private readonly publicClient: PublicClient,
    private readonly rollupAddress: EthAddress,
    private readonly yeeterAddress: EthAddress,
    private readonly pollingInterval = 10_000,
    private readonly log = createDebugLogger('aztec:archiver'),
  ) {}

  /**
   * Creates a new instance of the Archiver and blocks until it syncs from chain.
   * @param config - The archiver's desired configuration.
   * @returns - An instance of the archiver.
   */
  public static async createAndSync(config: ArchiverConfig) {
    const publicClient = createPublicClient({
      chain: localhost,
      transport: http(config.rpcUrl),
    });
    const archiver = new Archiver(
      publicClient,
      config.rollupContract,
      config.yeeterContract,
      config.archiverPollingInterval,
    );
    await archiver.start();
    return archiver;
  }

  /**
   * Starts sync process.
   */
  public async start() {
    this.log('Starting initial sync...');
    await this.runInitialSync();
    this.log('Initial sync finished.');
    // TODO: Any logs emitted between the initial sync and the start watching will be lost
    // We should start watching before we start the initial sync
    this.startWatchingEvents();
    this.log('Watching for new data...');
  }

  /**
   * Fetches all the L2BlockProcessed and Yeet events since genesis and processes them.
   */
  private async runInitialSync() {
    const blockFilter = await this.publicClient.createContractEventFilter({
      address: getAddress(this.rollupAddress.toString()),
      fromBlock: 0n,
      abi: RollupAbi,
      eventName: 'L2BlockProcessed',
    });

    const yeetFilter = await this.publicClient.createContractEventFilter({
      address: getAddress(this.yeeterAddress.toString()),
      abi: YeeterAbi,
      eventName: 'Yeet',
      fromBlock: 0n,
    });

    const blockLogs = await this.publicClient.getFilterLogs({ filter: blockFilter });
    const yeetLogs = await this.publicClient.getFilterLogs({ filter: yeetFilter });

    await this.processBlockLogs(blockLogs);
    this.processYeetLogs(yeetLogs);
  }

  /**
   * Starts a polling loop in the background which watches for new events and passes them to the respective handlers.
   * TODO: Handle reorgs, consider using github.com/ethereumjs/ethereumjs-blockstream.
   */
  private startWatchingEvents() {
    this.unwatchBlocks = this.publicClient.watchContractEvent({
      address: getAddress(this.rollupAddress.toString()),
      abi: RollupAbi,
      eventName: 'L2BlockProcessed',
      onLogs: logs => this.processBlockLogs(logs),
      pollingInterval: this.pollingInterval,
    });

    this.unwatchYeets = this.publicClient.watchContractEvent({
      address: getAddress(this.yeeterAddress.toString()),
      abi: YeeterAbi,
      eventName: 'Yeet',
      onLogs: logs => this.processYeetLogs(logs),
      pollingInterval: this.pollingInterval,
    });
  }

  /**
   * Processes newly received L2BlockProcessed events.
   * @param logs - L2BlockProcessed event logs.
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
      this.log(`Retrieved block ${newBlock.number} from chain`);
      this.l2Blocks.push(newBlock);
    }
  }

  /**
   * Processes newly received Yeet events.
   * @param logs - Yeet event logs.
   */
  private processYeetLogs(logs: Log<bigint, number, undefined, typeof YeeterAbi, 'Yeet'>[]) {
    for (const log of logs) {
      const blockNum = log.args.l2blockNum;
      if (blockNum !== BigInt(this.unverifiedDatas.length + INITIAL_L2_BLOCK_NUM)) {
        throw new Error(
          'Block number mismatch. Expected: ' +
            (this.unverifiedDatas.length + INITIAL_L2_BLOCK_NUM) +
            ' but got: ' +
            blockNum +
            '.',
        );
      }
      const unverifiedDataBuf = Buffer.from(hexToBytes(log.args.blabber));
      let currIndex = 0;
      const chunks: Buffer[] = [];
      while (currIndex < unverifiedDataBuf.length) {
        const nextChunkLength = Number(toBigIntBE(unverifiedDataBuf.slice(currIndex, currIndex + 4)));
        currIndex += 4;
        const nextChunk = unverifiedDataBuf.slice(currIndex, currIndex + nextChunkLength);
        currIndex += nextChunkLength;
        chunks.push(nextChunk);
      }
      if (currIndex !== unverifiedDataBuf.length) {
        console.error(
          `Unverified data buffer was not fully consumed. Consumed ${currIndex + 1} bytes. Total length: ${
            unverifiedDataBuf.length
          } bytes.`,
        );
      }
      this.unverifiedDatas.push(new UnverifiedData(chunks));
      this.log(`Added ${chunks.length} chunks of unverifiedData corresponding to block ${blockNum}`);
    }
    this.log('Processed unverifiedData corresponding to ' + logs.length + ' blocks.');
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
      abi: RollupAbi.filter(item => item.type !== 'constructor'),
      data,
    });
    if (functionName !== 'process') throw new Error(`Unexpected method called ${functionName}`);
    const [, l2blockHex] = args! as [Hex, Hex];
    const block = L2Block.decode(Buffer.from(hexToBytes(l2blockHex)));
    if (BigInt(block.number) !== l2BlockNum) {
      throw new Error(`Block number mismatch: expected ${l2BlockNum} but got ${block.number}`);
    }
    return block;
  }

  /**
   * Stops the archiver.
   * @returns A promise signalling completion of the stop process.
   */
  public stop(): Promise<void> {
    this.log('Stopping...');
    if (this.unwatchBlocks === undefined || this.unwatchYeets === undefined) {
      throw new Error('Archiver is not running.');
    }

    this.unwatchBlocks();
    this.unwatchYeets();

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
   * @returns The portal address (if we didn't throw an error).
   */
  public getL2ContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    for (const block of this.l2Blocks) {
      for (const contractData of block.newContractData) {
        if (contractData.aztecAddress.equals(contractAddress)) {
          return Promise.resolve(contractData);
        }
      }
    }
    return Promise.resolve(undefined);
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
    if (from > this.unverifiedDatas.length) {
      return Promise.resolve([]);
    }
    const startIndex = from - INITIAL_L2_BLOCK_NUM;
    const endIndex = startIndex + take;
    return Promise.resolve(this.unverifiedDatas.slice(startIndex, endIndex));
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
    if (this.unverifiedDatas.length === 0) return Promise.resolve(INITIAL_L2_BLOCK_NUM - 1);
    return Promise.resolve(this.unverifiedDatas.length + INITIAL_L2_BLOCK_NUM - 1);
  }
}
