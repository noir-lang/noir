import { FunctionSelector } from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import {
  ContractData,
  ContractDataSource,
  EncodedContractFunction,
  ExtendedContractData,
  INITIAL_L2_BLOCK_NUM,
  L1ToL2Message,
  L1ToL2MessageSource,
  L2Block,
  L2BlockL2Logs,
  L2BlockSource,
  L2LogsSource,
  L2Tx,
  LogType,
  TxHash,
} from '@aztec/types';

import omit from 'lodash.omit';
import { Chain, HttpTransport, PublicClient, createPublicClient, http } from 'viem';

import { ArchiverDataStore, MemoryArchiverStore } from './archiver_store.js';
import { ArchiverConfig } from './config.js';
import {
  retrieveBlocks,
  retrieveNewCancelledL1ToL2Messages,
  retrieveNewContractData,
  retrieveNewPendingL1ToL2Messages,
} from './data_retrieval.js';

/**
 * Pulls L2 blocks in a non-blocking manner and provides interface for their retrieval.
 * Responsible for handling robust L1 polling so that other components do not need to
 * concern themselves with it.
 */
export class Archiver implements L2BlockSource, L2LogsSource, ContractDataSource, L1ToL2MessageSource {
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
   * Use this to track logged block in order to avoid repeating the same message.
   */
  private lastLoggedBlockNumber = 0n;

  /**
   * Creates a new instance of the Archiver.
   * @param publicClient - A client for interacting with the Ethereum node.
   * @param rollupAddress - Ethereum address of the rollup contract.
   * @param inboxAddress - Ethereum address of the inbox contract.
   * @param registryAddress - Ethereum address of the registry contract.
   * @param contractDeploymentEmitterAddress - Ethereum address of the contractDeploymentEmitter contract.
   * @param searchStartBlock - The L1 block from which to start searching for new blocks.
   * @param pollingIntervalMs - The interval for polling for L1 logs (in milliseconds).
   * @param store - An archiver data store for storage & retrieval of blocks, encrypted logs & contract data.
   * @param log - A logger.
   */
  constructor(
    private readonly publicClient: PublicClient<HttpTransport, Chain>,
    private readonly rollupAddress: EthAddress,
    private readonly inboxAddress: EthAddress,
    private readonly registryAddress: EthAddress,
    private readonly contractDeploymentEmitterAddress: EthAddress,
    searchStartBlock: number,
    private readonly store: ArchiverDataStore,
    private readonly pollingIntervalMs = 10_000,
    private readonly log: DebugLogger = createDebugLogger('aztec:archiver'),
  ) {
    this.nextL2BlockFromBlock = BigInt(searchStartBlock);
    this.lastProcessedBlockNumber = BigInt(searchStartBlock);
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
      pollingInterval: config.viemPollingIntervalMS,
    });
    const archiverStore = new MemoryArchiverStore();
    const archiver = new Archiver(
      publicClient,
      config.l1Contracts.rollupAddress,
      config.l1Contracts.inboxAddress,
      config.l1Contracts.registryAddress,
      config.l1Contracts.contractDeploymentEmitterAddress,
      config.searchStartBlock,
      archiverStore,
      config.archiverPollingIntervalMS,
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
   * Fetches `L2BlockProcessed` and `ContractDeployment` logs from `nextL2BlockFromBlock` and processes them.
   * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
   */
  private async sync(blockUntilSynced: boolean) {
    const currentBlockNumber = await this.publicClient.getBlockNumber();
    if (currentBlockNumber <= this.lastProcessedBlockNumber) {
      // reducing logs, otherwise this gets triggered on every loop (1s)
      if (currentBlockNumber !== this.lastLoggedBlockNumber) {
        this.log(`No new blocks to process, current block number: ${currentBlockNumber}`);
        this.lastLoggedBlockNumber = currentBlockNumber;
      }
      return;
    }

    // ********** Events that are processed in between blocks **********

    // Process l1ToL2Messages, these are consumed as time passes, not each block
    const retrievedPendingL1ToL2Messages = await retrieveNewPendingL1ToL2Messages(
      this.publicClient,
      this.inboxAddress,
      blockUntilSynced,
      currentBlockNumber,
      this.lastProcessedBlockNumber + 1n, // + 1 to prevent re-including messages from the last processed block
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

    // Read all data from chain and then write to our stores at the end
    const nextExpectedL2BlockNum = BigInt(this.store.getBlocksLength() + INITIAL_L2_BLOCK_NUM);
    this.log(
      `Retrieving chain state from L1 block: ${this.nextL2BlockFromBlock}, next expected l2 block number: ${nextExpectedL2BlockNum}`,
    );
    const retrievedBlocks = await retrieveBlocks(
      this.publicClient,
      this.rollupAddress,
      blockUntilSynced,
      currentBlockNumber,
      this.nextL2BlockFromBlock,
      nextExpectedL2BlockNum,
    );

    // create the block number -> block hash mapping to ensure we retrieve the appropriate events
    const blockHashMapping: { [key: number]: Buffer | undefined } = {};
    retrievedBlocks.retrievedData.forEach((block: L2Block) => {
      blockHashMapping[block.number] = block.getCalldataHash();
    });
    const retrievedContracts = await retrieveNewContractData(
      this.publicClient,
      this.contractDeploymentEmitterAddress,
      blockUntilSynced,
      currentBlockNumber,
      this.nextL2BlockFromBlock,
      blockHashMapping,
    );
    if (retrievedBlocks.retrievedData.length === 0) {
      return;
    }

    this.log(`Retrieved ${retrievedBlocks.retrievedData.length} block(s) from chain`);

    // store encrypted logs from L2 Blocks that we have retrieved
    const encryptedLogs = retrievedBlocks.retrievedData.map(block => {
      return block.newEncryptedLogs!;
    });
    await this.store.addLogs(encryptedLogs, LogType.ENCRYPTED);

    // store unencrypted logs from L2 Blocks that we have retrieved
    const unencryptedLogs = retrievedBlocks.retrievedData.map(block => {
      return block.newUnencryptedLogs!;
    });
    await this.store.addLogs(unencryptedLogs, LogType.UNENCRYPTED);

    // store contracts for which we have retrieved L2 blocks
    const lastKnownL2BlockNum = retrievedBlocks.retrievedData[retrievedBlocks.retrievedData.length - 1].number;
    retrievedContracts.retrievedData.forEach(async ([contracts, l2BlockNum], index) => {
      this.log(`Retrieved extended contract data for l2 block number: ${index}`);
      if (l2BlockNum <= lastKnownL2BlockNum) {
        await this.store.addExtendedContractData(contracts, l2BlockNum);
      }
    });

    // from retrieved L2Blocks, confirm L1 to L2 messages that have been published
    // from each l2block fetch all messageKeys in a flattened array:
    const messageKeysToRemove = retrievedBlocks.retrievedData.map(l2block => l2block.newL1ToL2Messages).flat();
    this.log(`Confirming l1 to l2 messages in store`);
    await this.store.confirmL1ToL2Messages(messageKeysToRemove);

    // store retrieved L2 blocks after removing new logs information.
    // remove logs to serve "lightweight" block information. Logs can be fetched separately if needed.
    await this.store.addL2Blocks(
      retrievedBlocks.retrievedData.map(block =>
        L2Block.fromFields(omit(block, ['newEncryptedLogs', 'newUnencryptedLogs']), block.getBlockHash()),
      ),
    );

    // set the L1 block for the next search
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

  public getRollupAddress(): Promise<EthAddress> {
    return Promise.resolve(this.rollupAddress);
  }

  public getRegistryAddress(): Promise<EthAddress> {
    return Promise.resolve(this.registryAddress);
  }

  /**
   * Gets up to `limit` amount of L2 blocks starting from `from`.
   * @param from - Number of the first block to return (inclusive).
   * @param limit - The number of blocks to return.
   * @returns The requested L2 blocks.
   */
  public getL2Blocks(from: number, limit: number): Promise<L2Block[]> {
    return this.store.getL2Blocks(from, limit);
  }

  /**
   * Gets an l2 block.
   * @param number - The block number to return (inclusive).
   * @returns The requested L2 block.
   */
  public async getL2Block(number: number): Promise<L2Block | undefined> {
    // If the number provided is -ve, then return the latest block.
    if (number < 0) {
      number = this.store.getBlocksLength();
    }
    const blocks = await this.store.getL2Blocks(number, 1);
    return blocks.length === 0 ? undefined : blocks[0];
  }

  public getL2Tx(txHash: TxHash): Promise<L2Tx | undefined> {
    return this.store.getL2Tx(txHash);
  }

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    return this.store.getExtendedContractData(contractAddress);
  }

  /**
   * Lookup all contract data in an L2 block.
   * @param blockNum - The block number to get all contract data from.
   * @returns All new contract data in the block (if found).
   */
  public getExtendedContractDataInBlock(blockNum: number): Promise<ExtendedContractData[]> {
    return this.store.getExtendedContractDataInBlock(blockNum);
  }

  /**
   * Lookup the contract data for this contract.
   * Contains contract address & the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  public getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return this.store.getContractData(contractAddress);
  }

  /**
   * Lookup the L2 contract data inside a block.
   * Contains contract address & the ethereum portal address.
   * @param l2BlockNum - The L2 block number to get the contract data from.
   * @returns ContractData with the portal address (if we didn't throw an error).
   */
  public getContractDataInBlock(l2BlockNum: number): Promise<ContractData[] | undefined> {
    return this.store.getContractDataInBlock(l2BlockNum);
  }

  /**
   * Gets the public function data for a contract.
   * @param contractAddress - The contract address containing the function to fetch.
   * @param selector - The function selector of the function to fetch.
   * @returns The public function data (if found).
   */
  public async getPublicFunction(
    contractAddress: AztecAddress,
    selector: FunctionSelector,
  ): Promise<EncodedContractFunction | undefined> {
    const contractData = await this.getExtendedContractData(contractAddress);
    return contractData?.getPublicFunction(selector);
  }

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  public getLogs(from: number, limit: number, logType: LogType): Promise<L2BlockL2Logs[]> {
    return this.store.getLogs(from, limit, logType);
  }

  /**
   * Gets the number of the latest L2 block processed by the block source implementation.
   * @returns The number of the latest L2 block processed by the block source implementation.
   */
  public getBlockNumber(): Promise<number> {
    return this.store.getBlockNumber();
  }

  /**
   * Gets up to `limit` amount of pending L1 to L2 messages.
   * @param limit - The number of messages to return.
   * @returns The requested L1 to L2 messages' keys.
   */
  getPendingL1ToL2Messages(limit: number): Promise<Fr[]> {
    return this.store.getPendingL1ToL2MessageKeys(limit);
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
