import { FunctionSelector, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP } from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import {
  ContractData,
  ContractDataSource,
  EncodedContractFunction,
  ExtendedContractData,
  GetUnencryptedLogsResponse,
  L1ToL2Message,
  L1ToL2MessageSource,
  L2Block,
  L2BlockL2Logs,
  L2BlockSource,
  L2LogsSource,
  L2Tx,
  LogFilter,
  LogType,
  TxHash,
} from '@aztec/types';

import omit from 'lodash.omit';
import { Chain, HttpTransport, PublicClient, createPublicClient, http } from 'viem';

import { ArchiverDataStore } from './archiver_store.js';
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
   * Use this to track logged block in order to avoid repeating the same message.
   */
  private lastLoggedL1BlockNumber = 0n;

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
    private readonly store: ArchiverDataStore,
    private readonly pollingIntervalMs = 10_000,
    private readonly log: DebugLogger = createDebugLogger('aztec:archiver'),
  ) {}

  /**
   * Creates a new instance of the Archiver and blocks until it syncs from chain.
   * @param config - The archiver's desired configuration.
   * @param archiverStore - The backing store for the archiver.
   * @param blockUntilSynced - If true, blocks until the archiver has fully synced.
   * @returns - An instance of the archiver.
   */
  public static async createAndSync(
    config: ArchiverConfig,
    archiverStore: ArchiverDataStore,
    blockUntilSynced = true,
  ): Promise<Archiver> {
    const chain = createEthereumChain(config.rpcUrl, config.apiKey);
    const publicClient = createPublicClient({
      chain: chain.chainInfo,
      transport: http(chain.rpcUrl),
      pollingInterval: config.viemPollingIntervalMS,
    });

    const archiver = new Archiver(
      publicClient,
      config.l1Contracts.rollupAddress,
      config.l1Contracts.inboxAddress,
      config.l1Contracts.registryAddress,
      config.l1Contracts.contractDeploymentEmitterAddress,
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
    const currentL1BlockNumber = await this.publicClient.getBlockNumber();
    // this makes the archiver more resilient to eventually-consistent eth providers like Infura
    // it _will_ process the same L1 blocks over and over again until the L2 chain advances
    // one thing to handle now is that we will process the same L1 to L2 messages over and over again
    // so the store needs to account for that.
    const lastProcessedL1BlockNumber = await this.store.getL1BlockNumber();
    if (currentL1BlockNumber <= lastProcessedL1BlockNumber) {
      // reducing logs, otherwise this gets triggered on every loop (1s)
      if (currentL1BlockNumber !== this.lastLoggedL1BlockNumber) {
        this.log(`No new blocks to process, current block number: ${currentL1BlockNumber}`);
        this.lastLoggedL1BlockNumber = currentL1BlockNumber;
      }
      return;
    }

    // ********** Ensuring Consistency of data pulled from L1 **********

    /**
     * There are a number of calls in this sync operation to L1 for retrieving
     * events and transaction data. There are a couple of things we need to bear in mind
     * to ensure that data is read exactly once.
     *
     * The first is the problem of eventually consistent ETH service providers like Infura.
     * We currently read from the last L1 block that we saw emit an L2 block. This could mean
     * that the archiver ends up looking at the same L1 block multiple times (e.g. if we last saw
     * an L2 block emitted at L1 block 10, we'd constantly ask for L1 blocks from 11 onwards until
     * we see another L2 block). For this to work message and block processing need to be idempotent.
     * We should re-visit this before mainnet launch.
     *
     * The second is that in between the various calls to L1, the block number can move meaning some
     * of the following calls will return data for blocks that were not present during earlier calls.
     * It's possible that we actually received messages in block currentBlockNumber + 1 meaning the next time
     * we do this sync we get the same message again. Additionally, the call to get cancelled L1 to L2 messages
     * could read from a block not present when retrieving pending messages. If a message was added and cancelled
     * in the same eth block then we could try and cancel a non-existent pending message.
     *
     * To combat this for the time being we simply ensure that all data retrieval methods only retrieve
     * data up to the currentBlockNumber captured at the top of this function. We might want to improve on this
     * in future but for the time being it should give us the guarantees that we need
     *
     */

    // ********** Events that are processed in between blocks **********

    // Process l1ToL2Messages, these are consumed as time passes, not each block
    const retrievedPendingL1ToL2Messages = await retrieveNewPendingL1ToL2Messages(
      this.publicClient,
      this.inboxAddress,
      blockUntilSynced,
      lastProcessedL1BlockNumber + 1n, // + 1 to prevent re-including messages from the last processed block
      currentL1BlockNumber,
    );
    const retrievedCancelledL1ToL2Messages = await retrieveNewCancelledL1ToL2Messages(
      this.publicClient,
      this.inboxAddress,
      blockUntilSynced,
      lastProcessedL1BlockNumber + 1n,
      currentL1BlockNumber,
    );

    // TODO (#717): optimize this - there could be messages in confirmed that are also in pending.
    // Or messages in pending that are also cancelled in the same block. No need to modify storage for them.

    if (retrievedPendingL1ToL2Messages.retrievedData.length) {
      // Store l1 to l2 messages
      this.log(`Adding ${retrievedPendingL1ToL2Messages.retrievedData.length} pending l1 to l2 messages to store`);
      await this.store.addPendingL1ToL2Messages(retrievedPendingL1ToL2Messages.retrievedData);
    }

    if (retrievedCancelledL1ToL2Messages.retrievedData.length) {
      // remove cancelled messages from the pending message store:
      this.log(
        `Removing ${retrievedCancelledL1ToL2Messages.retrievedData.length} pending l1 to l2 messages from store where messages were cancelled`,
      );
      await this.store.cancelPendingL1ToL2Messages(retrievedCancelledL1ToL2Messages.retrievedData);
    }

    // ********** Events that are processed per block **********

    // Read all data from chain and then write to our stores at the end
    const nextExpectedL2BlockNum = BigInt((await this.store.getBlockNumber()) + 1);
    const retrievedBlocks = await retrieveBlocks(
      this.publicClient,
      this.rollupAddress,
      blockUntilSynced,
      lastProcessedL1BlockNumber + 1n,
      currentL1BlockNumber,
      nextExpectedL2BlockNum,
    );

    if (retrievedBlocks.retrievedData.length === 0) {
      return;
    } else {
      this.log(
        `Retrieved ${retrievedBlocks.retrievedData.length} new L2 blocks between L1 blocks ${
          lastProcessedL1BlockNumber + 1n
        } and ${currentL1BlockNumber}.`,
      );
    }

    // create the block number -> block hash mapping to ensure we retrieve the appropriate events
    const blockHashMapping: { [key: number]: Buffer | undefined } = {};
    retrievedBlocks.retrievedData.forEach((block: L2Block) => {
      blockHashMapping[block.number] = block.getCalldataHash();
    });
    const retrievedContracts = await retrieveNewContractData(
      this.publicClient,
      this.contractDeploymentEmitterAddress,
      blockUntilSynced,
      lastProcessedL1BlockNumber + 1n,
      currentL1BlockNumber,
      blockHashMapping,
    );

    this.log(`Retrieved ${retrievedBlocks.retrievedData.length} block(s) from chain`);

    await Promise.all(
      retrievedBlocks.retrievedData.map(block =>
        this.store.addLogs(block.newEncryptedLogs, block.newUnencryptedLogs, block.number),
      ),
    );

    // store contracts for which we have retrieved L2 blocks
    const lastKnownL2BlockNum = retrievedBlocks.retrievedData[retrievedBlocks.retrievedData.length - 1].number;
    await Promise.all(
      retrievedContracts.retrievedData.map(async ([contracts, l2BlockNum]) => {
        this.log(`Retrieved extended contract data for l2 block number: ${l2BlockNum}`);
        if (l2BlockNum <= lastKnownL2BlockNum) {
          await this.store.addExtendedContractData(contracts, l2BlockNum);
        }
      }),
    );

    // from retrieved L2Blocks, confirm L1 to L2 messages that have been published
    // from each l2block fetch all messageKeys in a flattened array:
    const messageKeysToRemove = retrievedBlocks.retrievedData.map(l2block => l2block.newL1ToL2Messages).flat();
    this.log(`Confirming l1 to l2 messages in store`);
    await this.store.confirmL1ToL2Messages(messageKeysToRemove);

    // store retrieved L2 blocks after removing new logs information.
    // remove logs to serve "lightweight" block information. Logs can be fetched separately if needed.
    await this.store.addBlocks(
      retrievedBlocks.retrievedData.map(block => {
        // Ensure we pad the L1 to L2 message array to the full size before storing.
        block.newL1ToL2Messages = padArrayEnd(block.newL1ToL2Messages, Fr.ZERO, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);
        return L2Block.fromFields(
          omit(block, ['newEncryptedLogs', 'newUnencryptedLogs']),
          block.getBlockHash(),
          block.getL1BlockNumber(),
        );
      }),
    );
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
  public getBlocks(from: number, limit: number): Promise<L2Block[]> {
    return this.store.getBlocks(from, limit);
  }

  /**
   * Gets an l2 block.
   * @param number - The block number to return (inclusive).
   * @returns The requested L2 block.
   */
  public async getBlock(number: number): Promise<L2Block | undefined> {
    // If the number provided is -ve, then return the latest block.
    if (number < 0) {
      number = await this.store.getBlockNumber();
    }
    const blocks = await this.store.getBlocks(number, 1);
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
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   */
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse> {
    return this.store.getUnencryptedLogs(filter);
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
