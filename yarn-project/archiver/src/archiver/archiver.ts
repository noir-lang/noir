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
} from '@aztec/circuit-types';
import { FunctionSelector, NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP, getContractClassId } from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import {
  ContractClass,
  ContractClassWithId,
  ContractInstance,
  ContractInstanceWithAddress,
} from '@aztec/types/contracts';

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
 * Helper interface to combine all sources this archiver implementation provides.
 */
export type ArchiveSource = L2BlockSource & L2LogsSource & ContractDataSource & L1ToL2MessageSource;

/**
 * Pulls L2 blocks in a non-blocking manner and provides interface for their retrieval.
 * Responsible for handling robust L1 polling so that other components do not need to
 * concern themselves with it.
 */
export class Archiver implements ArchiveSource {
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
    /**
     * We keep track of three "pointers" to L1 blocks:
     * 1. the last L1 block that published an L2 block
     * 2. the last L1 block that added L1 to L2 messages
     * 3. the last L1 block that cancelled L1 to L2 messages
     *
     * We do this to deal with L1 data providers that are eventually consistent (e.g. Infura).
     * We guard against seeing block X with no data at one point, and later, the provider processes the block and it has data.
     * The archiver will stay back, until there's data on L1 that will move the pointers forward.
     *
     * This code does not handle reorgs.
     */
    const lastL1Blocks = await this.store.getL1BlockNumber();
    const currentL1BlockNumber = await this.publicClient.getBlockNumber();

    if (
      currentL1BlockNumber <= lastL1Blocks.addedBlock &&
      currentL1BlockNumber <= lastL1Blocks.addedMessages &&
      currentL1BlockNumber <= lastL1Blocks.cancelledMessages
    ) {
      // chain hasn't moved forward
      // or it's been rolled back
      return;
    }

    // ********** Ensuring Consistency of data pulled from L1 **********

    /**
     * There are a number of calls in this sync operation to L1 for retrieving
     * events and transaction data. There are a couple of things we need to bear in mind
     * to ensure that data is read exactly once.
     *
     * The first is the problem of eventually consistent ETH service providers like Infura.
     * Each L1 read operation will query data from the last L1 block that it saw emit its kind of data.
     * (so pending L1 to L2 messages will read from the last L1 block that emitted a message and so  on)
     * This will mean the archiver will lag behind L1 and will only advance when there's L2-relevant activity on the chain.
     *
     * The second is that in between the various calls to L1, the block number can move meaning some
     * of the following calls will return data for blocks that were not present during earlier calls.
     * To combat this for the time being we simply ensure that all data retrieval methods only retrieve
     * data up to the currentBlockNumber captured at the top of this function. We might want to improve on this
     * in future but for the time being it should give us the guarantees that we need
     */

    // ********** Events that are processed per L1 block **********

    // Process l1ToL2Messages, these are consumed as time passes, not each block
    const retrievedPendingL1ToL2Messages = await retrieveNewPendingL1ToL2Messages(
      this.publicClient,
      this.inboxAddress,
      blockUntilSynced,
      lastL1Blocks.addedMessages + 1n,
      currentL1BlockNumber,
    );
    const retrievedCancelledL1ToL2Messages = await retrieveNewCancelledL1ToL2Messages(
      this.publicClient,
      this.inboxAddress,
      blockUntilSynced,
      lastL1Blocks.cancelledMessages + 1n,
      currentL1BlockNumber,
    );

    // group pending messages and cancelled messages by their L1 block number
    const messagesByBlock = new Map<bigint, [L1ToL2Message[], Fr[]]>();
    for (const [message, blockNumber] of retrievedPendingL1ToL2Messages.retrievedData) {
      const messages = messagesByBlock.get(blockNumber) || [[], []];
      messages[0].push(message);
      messagesByBlock.set(blockNumber, messages);
    }

    for (const [messageKey, blockNumber] of retrievedCancelledL1ToL2Messages.retrievedData) {
      const messages = messagesByBlock.get(blockNumber) || [[], []];
      messages[1].push(messageKey);
      messagesByBlock.set(blockNumber, messages);
    }

    // process messages from each L1 block in sequence
    const l1BlocksWithMessages = Array.from(messagesByBlock.keys()).sort((a, b) => (a < b ? -1 : a === b ? 0 : 1));
    for (const l1Block of l1BlocksWithMessages) {
      const [newMessages, cancelledMessages] = messagesByBlock.get(l1Block)!;
      this.log(
        `Adding ${newMessages.length} new messages and ${cancelledMessages.length} cancelled messages in L1 block ${l1Block}`,
      );
      await this.store.addPendingL1ToL2Messages(newMessages, l1Block);
      await this.store.cancelPendingL1ToL2Messages(cancelledMessages, l1Block);
    }

    // ********** Events that are processed per L2 block **********

    // Read all data from chain and then write to our stores at the end
    const nextExpectedL2BlockNum = BigInt((await this.store.getBlockNumber()) + 1);
    const retrievedBlocks = await retrieveBlocks(
      this.publicClient,
      this.rollupAddress,
      blockUntilSynced,
      lastL1Blocks.addedBlock + 1n,
      currentL1BlockNumber,
      nextExpectedL2BlockNum,
    );

    if (retrievedBlocks.retrievedData.length === 0) {
      return;
    } else {
      this.log(
        `Retrieved ${retrievedBlocks.retrievedData.length} new L2 blocks between L1 blocks ${
          lastL1Blocks.addedBlock + 1n
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
      lastL1Blocks.addedBlock + 1n,
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
          await this.storeContractDataAsClassesAndInstances(contracts, l2BlockNum);
        }
      }),
    );

    // from retrieved L2Blocks, confirm L1 to L2 messages that have been published
    // from each l2block fetch all messageKeys in a flattened array:
    this.log(`Confirming l1 to l2 messages in store`);
    for (const block of retrievedBlocks.retrievedData) {
      await this.store.confirmL1ToL2Messages(block.newL1ToL2Messages);
    }

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
   * Stores extended contract data as classes and instances.
   * Temporary solution until we source this data from the contract class registerer and instance deployer.
   * @param contracts - The extended contract data to be stored.
   * @param l2BlockNum - The L2 block number to which the contract data corresponds.
   */
  async storeContractDataAsClassesAndInstances(contracts: ExtendedContractData[], l2BlockNum: number) {
    const classesAndInstances = contracts.map(extendedContractDataToContractClassAndInstance);
    await this.store.addContractClasses(
      classesAndInstances.map(([c, _]) => c),
      l2BlockNum,
    );
    await this.store.addContractInstances(
      classesAndInstances.map(([_, i]) => i),
      l2BlockNum,
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

/** Converts ExtendedContractData into contract classes and instances. */
function extendedContractDataToContractClassAndInstance(
  data: ExtendedContractData,
): [ContractClassWithId, ContractInstanceWithAddress] {
  const contractClass: ContractClass = {
    version: 1,
    artifactHash: Fr.ZERO,
    publicFunctions: data.publicFunctions.map(f => ({
      selector: f.selector,
      bytecode: f.bytecode,
      isInternal: f.isInternal,
    })),
    privateFunctions: [],
    packedBytecode: data.bytecode,
  };
  const contractClassId = getContractClassId(contractClass);
  const contractInstance: ContractInstance = {
    version: 1,
    salt: Fr.ZERO,
    contractClassId,
    initializationHash: Fr.ZERO,
    portalContractAddress: data.contractData.portalContractAddress,
    publicKeysHash: data.partialAddress,
  };
  const address = data.contractData.contractAddress;
  return [
    { ...contractClass, id: contractClassId },
    { ...contractInstance, address },
  ];
}
