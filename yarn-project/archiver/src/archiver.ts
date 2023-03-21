import { Address, PublicClient } from 'viem';
import { rollupAbi } from './abis/rollup.js';
import { yeeterAbi } from './abis/yeeter.js';
import { ContractData, L2Block } from './l2_block/l2_block.js';
import { randomAppendOnlyTreeSnapshot, randomBytes, randomContractData } from './l2_block/mocks.js';
import { L2BlockSource, SyncStatus } from './l2_block_source.js';
import { createLogger } from './movetofoundation/log/console.js';

/**
 * Pulls L2 blocks in a non-blocking manner and provides interface for their retrieval.
 */
export class Archiver implements L2BlockSource {
  /**
   * An array containing all the L2 blocks that have been fetched so far.
   */
  private l2Blocks: L2Block[] = [];

  /**
   * An array of yeets that have been fetched but not yet added as a property to L2 blocks.
   * Note: Can happen when Yeet event is received before L2BlockProcessed event for whatever reason.
   */
  private pendingYeets: Buffer[] = [];

  private unwatchBlocks: (() => void) | undefined;
  private unwatchYeets: (() => void) | undefined;

  /**
   * Creates a new instance of the Archiver.
   * @param publicClient - A client for interacting with the Ethereum node.
   * @param rollupAddress - Ethereum address of the rollup contract.
   * @param yeeterAddress - Ethereum address of the yeeter contract.
   * @param log - A logger.
   */
  constructor(
    private readonly publicClient: PublicClient,
    private readonly rollupAddress: Address,
    private readonly yeeterAddress: Address,
    private readonly log = createLogger('Archiver'),
  ) {}

  /**
   * {@inheritDoc L2BlockSource.getSyncStatus}
   */
  public async getSyncStatus(): Promise<SyncStatus> {
    const nextBlockNum = await this.publicClient.readContract({
      address: this.rollupAddress,
      abi: rollupAbi,
      functionName: 'nextBlockNum',
    });

    return {
      syncedToBlock: this.getLatestBlockNum(),
      latestBlock: Number(nextBlockNum) - 1,
    };
  }

  /**
   * Starts sync process.
   */
  public async start() {
    this.log('Starting initial sync...');
    await this.runInitialSync();
    this.log('Initial sync finished.');
    this.startWatchingEvents();
    this.log('Watching for new data...');
  }

  /**
   * Fetches all the L2BlockProcessed and Yeet events since genesis and processes them.
   */
  private async runInitialSync() {
    const blockFilter = await this.publicClient.createEventFilter({
      address: this.rollupAddress,
      fromBlock: 0n,
      event: rollupAbi[0],
    });

    const yeetFilter = await this.publicClient.createEventFilter({
      address: this.yeeterAddress,
      event: yeeterAbi[0],
      fromBlock: 0n,
    });

    const blockLogs = await this.publicClient.getFilterLogs({ filter: blockFilter });
    const yeetLogs = await this.publicClient.getFilterLogs({ filter: yeetFilter });

    this.processBlockLogs(blockLogs);
    this.processYeetLogs(yeetLogs);
  }

  /**
   * Starts a polling loop in the background which watches for new events and passes them to the respective handlers.
   */
  private startWatchingEvents() {
    this.unwatchBlocks = this.publicClient.watchEvent({
      address: this.rollupAddress,
      event: rollupAbi[0],
      onLogs: logs => this.processBlockLogs(logs),
    });

    this.unwatchYeets = this.publicClient.watchEvent({
      address: this.yeeterAddress,
      event: yeeterAbi[0],
      onLogs: logs => this.processYeetLogs(logs),
    });
  }

  /**
   * Processes newly received L2BlockProcessed events.
   * @param logs - L2BlockProcessed event logs.
   */
  private processBlockLogs(logs: any[]) {
    this.log('Processed ' + logs.length + ' L2 blocks...');
    for (const log of logs) {
      const blockNum = log.args.blockNum;
      if (blockNum !== BigInt(this.l2Blocks.length)) {
        throw new Error('Block number mismatch. Expected: ' + this.l2Blocks.length + ' but got: ' + blockNum + '.');
      }
      const newBlock = mockRandomL2Block(log.args.blockNum);
      const yeet = this.pendingYeets.find(yeet => yeet.readUInt32BE(0) === blockNum);
      if (yeet !== undefined) {
        newBlock.setYeet(yeet);
        // Remove yeet from pending
        this.pendingYeets = this.pendingYeets.filter(yeet => yeet.readUInt32BE(0) !== blockNum);
      }
      this.l2Blocks.push(newBlock);
    }
  }

  /**
   * Processes newly received Yeet events.
   * @param logs - Yeet event logs.
   */
  private processYeetLogs(logs: any[]) {
    for (const log of logs) {
      const blockNum = log.args.blockNum;
      if (blockNum < BigInt(this.l2Blocks.length)) {
        const block = this.l2Blocks[blockNum];
        block.setYeet(log.args.blabber);
        this.log('Enriched block ' + blockNum + ' with yeet.');
      } else {
        this.pendingYeets.push(log.args.blabber);
        this.log('Added yeet with blockNum ' + blockNum + ' to pending list.');
      }
    }
    this.log('Processed ' + logs.length + ' yeets...');
  }

  /**
   * Stops the event polling loop.
   */
  public stop() {
    this.log('Stopping...');
    if (this.unwatchBlocks === undefined || this.unwatchYeets === undefined) {
      throw new Error('Archiver is not running.');
    }

    this.unwatchBlocks();
    this.unwatchYeets();

    this.log('Stopped.');
  }

  /**
   * {@inheritDoc L2BlockSource.getL2Blocks}
   */
  public getL2Blocks(from: number, take: number): L2Block[] {
    if (from > this.l2Blocks.length) {
      return [];
    }
    if (from + take > this.l2Blocks.length) {
      return this.l2Blocks.slice(from);
    }

    return this.l2Blocks.slice(from, from + take);
  }

  /**
   * {@inheritDoc L2BlockSource.getLatestBlockNum}
   */
  public getLatestBlockNum(): number {
    return this.l2Blocks.length - 1;
  }
}

/**
 * Creates a random L2Block with the given block number.
 * @param l2BlockNum - Block number.
 * @returns Random L2Block.
 */
function mockRandomL2Block(l2BlockNum: number): L2Block {
  const newNullifiers = [randomBytes(32), randomBytes(32), randomBytes(32), randomBytes(32)];
  const newCommitments = [randomBytes(32), randomBytes(32), randomBytes(32), randomBytes(32)];
  const newContracts: Buffer[] = [randomBytes(32)];
  const newContractsData: ContractData[] = [randomContractData()];

  return new L2Block(
    l2BlockNum,
    randomAppendOnlyTreeSnapshot(0),
    randomAppendOnlyTreeSnapshot(0),
    randomAppendOnlyTreeSnapshot(0),
    randomAppendOnlyTreeSnapshot(0),
    randomAppendOnlyTreeSnapshot(0),
    randomAppendOnlyTreeSnapshot(newCommitments.length),
    randomAppendOnlyTreeSnapshot(newNullifiers.length),
    randomAppendOnlyTreeSnapshot(newContracts.length),
    randomAppendOnlyTreeSnapshot(1),
    randomAppendOnlyTreeSnapshot(1),
    newCommitments,
    newNullifiers,
    newContracts,
    newContractsData,
  );
}
