import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { EthAddress } from '@aztec/foundation/eth-address';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { INITIAL_L2_BLOCK_NUM, L1ToL2Message, L1ToL2MessageSource } from '@aztec/types';
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
import { Chain, HttpTransport, PublicClient, createPublicClient, http } from 'viem';
import { ArchiverConfig } from './config.js';
import { createEthereumChain } from '@aztec/ethereum';
import {
  retrieveBlocks,
  retrieveNewContractData,
  retrieveUnverifiedData,
  retrieveNewPendingL1ToL2Messages,
  retrieveNewCancelledL1ToL2Messages,
} from './data_retrieval.js';
import { ArchiverDataStore, MemoryArchiverStore } from './archiver_store.js';
import { Fr } from '@aztec/foundation/fields';

/**
 * Pulls L2 blocks in a non-blocking manner and provides interface for their retrieval.
 * Responsible for handling robust L1 polling so that other components do not need to
 * concern themselves with it.
 */
export class Archiver implements L2BlockSource, UnverifiedDataSource, ContractDataSource, L1ToL2MessageSource {
  /**
   * A promise in which we will be continually fetching new L2 blocks.
   */
  private runningPromise?: RunningPromise;

  /**
   * Next L1 block number to fetch `L2BlockProcessed` logs from (i.e. `fromBlock` in eth_getLogs).
   */
  private nextL2BlockFromBlock = 0n;

  /**
   * Last Processed Block Number
   */
  private lastProcessedBlockNumber = 0n;

  /**
   * Creates a new instance of the Archiver.
   * @param publicClient - A client for interacting with the Ethereum node.
   * @param rollupAddress - Ethereum address of the rollup contract.
   * @param inboxAddress - Ethereum address of the inbox contract.
   * @param unverifiedDataEmitterAddress - Ethereum address of the unverifiedDataEmitter contract.
   * @param searchStartBlock - The eth block from which to start searching for new blocks.
   * @param pollingIntervalMs - The interval for polling for rollup logs (in milliseconds).
   * @param store - An archiver data store for storage & retrieval of blocks, unverified data & contract data.
   * @param log - A logger.
   */
  constructor(
    private readonly publicClient: PublicClient<HttpTransport, Chain>,
    private readonly rollupAddress: EthAddress,
    private readonly inboxAddress: EthAddress,
    private readonly unverifiedDataEmitterAddress: EthAddress,
    searchStartBlock: number,
    private readonly store: ArchiverDataStore,
    private readonly pollingIntervalMs = 10_000,
    private readonly log: DebugLogger = createDebugLogger('aztec:archiver'),
  ) {
    this.nextL2BlockFromBlock = BigInt(searchStartBlock);
  }

  /**
   * Creates a new instance of the Archiver and blocks until it syncs from chain.
   * @param config - The archiver's desired configuration.
   * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
   * @returns - An instance of the archiver.
   */
  public static async createAndSync(config: ArchiverConfig, blockUntilSynced = true): Promise<Archiver> {
    const chain = createEthereumChain(config.rpcUrl, config.apiKey);
    const publicClient = createPublicClient({
      chain: chain.chainInfo,
      transport: http(chain.rpcUrl),
    });
    const archiverStore = new MemoryArchiverStore();
    const archiver = new Archiver(
      publicClient,
      config.rollupContract,
      config.inboxContract,
      config.unverifiedDataEmitterContract,
      config.searchStartBlock,
      archiverStore,
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
      this.log(`Performing initial chain sync...`);
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
    if (currentBlockNumber <= this.lastProcessedBlockNumber) {
      this.log(`No new blocks to process, current block number: ${currentBlockNumber}`);
      return;
    }

    // ********** Events that are processed inbetween blocks **********

    // Process l1ToL2Messages, these are consumed as time passes, not each block
    const retrievedPendingL1ToL2Messages = await retrieveNewPendingL1ToL2Messages(
      this.publicClient,
      this.inboxAddress,
      blockUntilSynced,
      currentBlockNumber,
      this.lastProcessedBlockNumber + 1n, // + 1 to prevent re including messages from the last processed block
    );
    const retrievedCancelledL1ToL2Messages = await retrieveNewCancelledL1ToL2Messages(
      this.publicClient,
      this.inboxAddress,
      blockUntilSynced,
      currentBlockNumber,
      this.lastProcessedBlockNumber + 1n,
    );

    // TODO (#717): optimise this - there could be messages in confirmed that are also in pending.
    // Or messages in pending that are also cancelled in the same block. No need to modify storage for them.
    // Store l1 to l2 messages
    this.log('Adding pending l1 to l2 messages to store');
    await this.store.addPendingL1ToL2Messages(retrievedPendingL1ToL2Messages.retrievedData);
    // remove cancelled messages from the pending message store:
    this.log('Removing pending l1 to l2 messages from store where messages were cancelled');
    await this.store.cancelPendingL1ToL2Messages(retrievedCancelledL1ToL2Messages.retrievedData);

    this.lastProcessedBlockNumber = currentBlockNumber;

    // ********** Events that are processed per block **********

    // The sequencer publishes unverified data first
    // Read all data from chain and then write to our stores at the end
    const nextExpectedRollupId = BigInt(this.store.getBlocksLength() + INITIAL_L2_BLOCK_NUM);
    this.log(
      `Retrieving chain state from eth block: ${this.nextL2BlockFromBlock}, next expected rollup id: ${nextExpectedRollupId}`,
    );
    const retrievedBlocks = await retrieveBlocks(
      this.publicClient,
      this.rollupAddress,
      blockUntilSynced,
      currentBlockNumber,
      this.nextL2BlockFromBlock,
      nextExpectedRollupId,
    );

    // create the block number -> block hash mapping to ensure we retrieve the appropriate events
    const blockHashMapping: { [key: number]: Buffer | undefined } = {};
    retrievedBlocks.retrievedData.forEach((block: L2Block) => {
      blockHashMapping[block.number] = block.getCalldataHash();
    });
    const retrievedUnverifiedData = await retrieveUnverifiedData(
      this.publicClient,
      this.unverifiedDataEmitterAddress,
      blockUntilSynced,
      currentBlockNumber,
      this.nextL2BlockFromBlock,
      nextExpectedRollupId,
      blockHashMapping,
    );
    const retrievedContracts = await retrieveNewContractData(
      this.publicClient,
      this.unverifiedDataEmitterAddress,
      blockUntilSynced,
      currentBlockNumber,
      this.nextL2BlockFromBlock,
      blockHashMapping,
    );
    if (retrievedBlocks.retrievedData.length === 0) {
      return;
    }

    this.log(`Retrieved ${retrievedBlocks.retrievedData.length} block(s) from chain`);

    // store unverified chunks for which we have retrieved rollups
    await this.store.addUnverifiedData(
      retrievedUnverifiedData.retrievedData.slice(0, retrievedBlocks.retrievedData.length),
    );

    // store contracts for which we have retrieved rollups
    const lastKnownRollupId = retrievedBlocks.retrievedData[retrievedBlocks.retrievedData.length - 1].number;
    retrievedContracts.retrievedData.forEach(async ([contracts, l2BlockNum], index) => {
      this.log(`Retrieved contract public data for rollup id: ${index}`);
      if (l2BlockNum <= lastKnownRollupId) {
        await this.store.addL2ContractPublicData(contracts, l2BlockNum);
      }
    });

    // from retrieved L2Blocks, confirm L1 to L2 messages that have been published
    // from each l2block fetch all messageKeys in a flattened array:
    const messageKeysToRemove = retrievedBlocks.retrievedData.map(l2block => l2block.newL1ToL2Messages).flat();
    this.log(`Confirming l1 to l2 messages in store`);
    await this.store.confirmL1ToL2Messages(messageKeysToRemove);

    // store retrieved rollup blocks
    await this.store.addL2Blocks(retrievedBlocks.retrievedData);

    // set the eth block for the next search
    this.nextL2BlockFromBlock = retrievedBlocks.nextEthBlockNumber;
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
    return this.store.getL2Blocks(from, take);
  }

  /**
   * Lookup the L2 contract data for this contract.
   * Contains the contract's public function bytecode.
   * @param contractAddress - The contract data address.
   * @returns The contract data.
   */
  public getL2ContractPublicData(contractAddress: AztecAddress): Promise<ContractPublicData | undefined> {
    return this.store.getL2ContractPublicData(contractAddress);
  }

  /**
   * Lookup all contract data in an L2 block.
   * @param blockNum - The block number to get all contract data from.
   * @returns All new contract data in the block (if found).
   */
  public getL2ContractPublicDataInBlock(blockNum: number): Promise<ContractPublicData[]> {
    return this.store.getL2ContractPublicDataInBlock(blockNum);
  }

  /**
   * Lookup the L2 contract info for this contract.
   * Contains contract address & the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  public getL2ContractInfo(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return this.store.getL2ContractInfo(contractAddress);
  }

  /**
   * Lookup the L2 contract info inside a block.
   * Contains contract address & the ethereum portal address.
   * @param l2BlockNum - The L2 block number to get the contract data from.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  public getL2ContractInfoInBlock(l2BlockNum: number): Promise<ContractData[] | undefined> {
    return this.store.getL2ContractInfoInBlock(l2BlockNum);
  }

  /**
   * Gets the public function data for a contract.
   * @param contractAddress - The contract address containing the function to fetch.
   * @param functionSelector - The function selector of the function to fetch.
   * @returns The public function data (if found).
   */
  public async getPublicFunction(
    contractAddress: AztecAddress,
    functionSelector: Buffer,
  ): Promise<EncodedContractFunction | undefined> {
    const contractData = await this.getL2ContractPublicData(contractAddress);
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
    return this.store.getUnverifiedData(from, take);
  }

  /**
   * Gets the number of the latest L2 block processed by the block source implementation.
   * @returns The number of the latest L2 block processed by the block source implementation.
   */
  public getBlockHeight(): Promise<number> {
    return this.store.getBlockHeight();
  }

  /**
   * Gets the L2 block number associated with the latest unverified data.
   * @returns The L2 block number associated with the latest unverified data.
   */
  public getLatestUnverifiedDataBlockNum(): Promise<number> {
    return this.store.getLatestUnverifiedDataBlockNum();
  }

  /**
   * Gets the `take` amount of pending L1 to L2 messages.
   * @param take - The number of messages to return.
   * @returns The requested L1 to L2 messages' keys.
   */
  getPendingL1ToL2Messages(take: number): Promise<Fr[]> {
    return this.store.getPendingL1ToL2MessageKeys(take);
  }

  /**
   * Gets the confirmed/consumed L1 to L2 message associated with the given message key
   * @param messageKey - The message key.
   * @returns The L1 to L2 message (throws if not found).
   */
  getConfirmedL1ToL2Message(messageKey: Fr): Promise<L1ToL2Message> {
    return this.store.getConfirmedL1ToL2Message(messageKey);
  }
}
