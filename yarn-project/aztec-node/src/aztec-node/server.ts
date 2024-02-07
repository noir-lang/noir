import { ArchiveSource, Archiver, KVArchiverDataStore, createArchiverClient } from '@aztec/archiver';
import {
  AztecNode,
  ContractData,
  ContractDataSource,
  ExtendedContractData,
  GetUnencryptedLogsResponse,
  INITIAL_L2_BLOCK_NUM,
  L1ToL2MessageAndIndex,
  L1ToL2MessageSource,
  L2Block,
  L2BlockL2Logs,
  L2BlockSource,
  L2LogsSource,
  L2Tx,
  LogFilter,
  LogType,
  MerkleTreeId,
  NullifierMembershipWitness,
  PublicDataWitness,
  SequencerConfig,
  SiblingPath,
  Tx,
  TxHash,
} from '@aztec/circuit-types';
import {
  ARCHIVE_HEIGHT,
  CONTRACT_TREE_HEIGHT,
  EthAddress,
  Fr,
  Header,
  L1_TO_L2_MSG_TREE_HEIGHT,
  NOTE_HASH_TREE_HEIGHT,
  NULLIFIER_TREE_HEIGHT,
  NullifierLeafPreimage,
  PUBLIC_DATA_TREE_HEIGHT,
  PublicDataTreeLeafPreimage,
} from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot } from '@aztec/circuits.js/abis';
import { L1ContractAddresses, createEthereumChain } from '@aztec/ethereum';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';
import { AztecKVTxPool, P2P, createP2PClient } from '@aztec/p2p';
import {
  GlobalVariableBuilder,
  PublicProcessorFactory,
  SequencerClient,
  getGlobalVariableBuilder,
} from '@aztec/sequencer-client';
import { ContractClassPublic, ContractInstanceWithAddress } from '@aztec/types/contracts';
import {
  MerkleTrees,
  ServerWorldStateSynchronizer,
  WorldStateConfig,
  WorldStateSynchronizer,
  getConfigEnvVars as getWorldStateConfig,
} from '@aztec/world-state';

import { AztecNodeConfig } from './config.js';

/**
 * The aztec node.
 */
export class AztecNodeService implements AztecNode {
  constructor(
    protected readonly config: AztecNodeConfig,
    protected readonly p2pClient: P2P,
    protected readonly blockSource: L2BlockSource,
    protected readonly encryptedLogsSource: L2LogsSource,
    protected readonly unencryptedLogsSource: L2LogsSource,
    protected readonly contractDataSource: ContractDataSource,
    protected readonly l1ToL2MessageSource: L1ToL2MessageSource,
    protected readonly worldStateSynchronizer: WorldStateSynchronizer,
    protected readonly sequencer: SequencerClient | undefined,
    protected readonly chainId: number,
    protected readonly version: number,
    protected readonly globalVariableBuilder: GlobalVariableBuilder,
    protected readonly merkleTreesDb: AztecKVStore,
    private log = createDebugLogger('aztec:node'),
  ) {
    const message =
      `Started Aztec Node against chain 0x${chainId.toString(16)} with contracts - \n` +
      `Rollup: ${config.l1Contracts.rollupAddress.toString()}\n` +
      `Registry: ${config.l1Contracts.registryAddress.toString()}\n` +
      `Inbox: ${config.l1Contracts.inboxAddress.toString()}\n` +
      `Outbox: ${config.l1Contracts.outboxAddress.toString()}\n` +
      `Contract Emitter: ${config.l1Contracts.contractDeploymentEmitterAddress.toString()}`;
    this.log(message);
  }

  /**
   * initializes the Aztec Node, wait for component to sync.
   * @param config - The configuration to be used by the aztec node.
   * @returns - A fully synced Aztec Node for use in development/testing.
   */
  public static async createAndSync(config: AztecNodeConfig) {
    const ethereumChain = createEthereumChain(config.rpcUrl, config.apiKey);
    //validate that the actual chain id matches that specified in configuration
    if (config.chainId !== ethereumChain.chainInfo.id) {
      throw new Error(
        `RPC URL configured for chain id ${ethereumChain.chainInfo.id} but expected id ${config.chainId}`,
      );
    }

    const log = createDebugLogger('aztec:node');
    const store = await AztecLmdbStore.open(config.l1Contracts.rollupAddress, config.dataDirectory);

    let archiver: ArchiveSource;
    if (!config.archiverUrl) {
      // first create and sync the archiver
      const archiverStore = new KVArchiverDataStore(store, config.maxLogs);
      archiver = await Archiver.createAndSync(config, archiverStore, true);
    } else {
      archiver = createArchiverClient(config.archiverUrl);
    }

    // we identify the P2P transaction protocol by using the rollup contract address.
    // this may well change in future
    config.transactionProtocol = `/aztec/tx/${config.l1Contracts.rollupAddress.toString()}`;

    // create the tx pool and the p2p client, which will need the l2 block source
    const p2pClient = await createP2PClient(store, config, new AztecKVTxPool(store), archiver);

    // now create the merkle trees and the world state synchronizer
    const merkleTrees = await MerkleTrees.new(store);
    const worldStateConfig: WorldStateConfig = getWorldStateConfig();
    const worldStateSynchronizer = new ServerWorldStateSynchronizer(store, merkleTrees, archiver, worldStateConfig);

    // start both and wait for them to sync from the block source
    await Promise.all([p2pClient.start(), worldStateSynchronizer.start()]);

    // now create the sequencer
    const sequencer = config.disableSequencer
      ? undefined
      : await SequencerClient.new(config, p2pClient, worldStateSynchronizer, archiver, archiver, archiver);

    return new AztecNodeService(
      config,
      p2pClient,
      archiver,
      archiver,
      archiver,
      archiver,
      archiver,
      worldStateSynchronizer,
      sequencer,
      ethereumChain.chainInfo.id,
      config.version,
      getGlobalVariableBuilder(config),
      store,
      log,
    );
  }

  /**
   * Returns the sequencer client instance.
   * @returns The sequencer client instance.
   */
  public getSequencer(): SequencerClient | undefined {
    return this.sequencer;
  }

  /**
   * Method to return the currently deployed L1 contract addresses.
   * @returns - The currently deployed L1 contract addresses.
   */
  public getL1ContractAddresses(): Promise<L1ContractAddresses> {
    return Promise.resolve(this.config.l1Contracts);
  }

  /**
   * Method to determine if the node is ready to accept transactions.
   * @returns - Flag indicating the readiness for tx submission.
   */
  public async isReady() {
    return (await this.p2pClient.isReady()) ?? false;
  }

  /**
   * Get a block specified by its number.
   * @param number - The block number being requested.
   * @returns The requested block.
   */
  public async getBlock(number: number): Promise<L2Block | undefined> {
    return await this.blockSource.getBlock(number);
  }

  /**
   * Method to request blocks. Will attempt to return all requested blocks but will return only those available.
   * @param from - The start of the range of blocks to return.
   * @param limit - The maximum number of blocks to obtain.
   * @returns The blocks requested.
   */
  public async getBlocks(from: number, limit: number): Promise<L2Block[]> {
    return (await this.blockSource.getBlocks(from, limit)) ?? [];
  }

  /**
   * Method to fetch the current block number.
   * @returns The block number.
   */
  public async getBlockNumber(): Promise<number> {
    return await this.blockSource.getBlockNumber();
  }

  /**
   * Method to fetch the version of the rollup the node is connected to.
   * @returns The rollup version.
   */
  public getVersion(): Promise<number> {
    return Promise.resolve(this.version);
  }

  /**
   * Method to fetch the chain id of the base-layer for the rollup.
   * @returns The chain id.
   */
  public getChainId(): Promise<number> {
    return Promise.resolve(this.chainId);
  }

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  async getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    return await this.contractDataSource.getExtendedContractData(contractAddress);
  }

  /**
   * Lookup the contract data for this contract.
   * Contains the ethereum portal address .
   * @param contractAddress - The contract data address.
   * @returns The contract's address & portal address.
   */
  public async getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    return await this.contractDataSource.getContractData(contractAddress);
  }

  public getContractClass(id: Fr): Promise<ContractClassPublic | undefined> {
    return this.contractDataSource.getContractClass(id);
  }

  public getContract(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return this.contractDataSource.getContract(address);
  }

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The maximum number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  public getLogs(from: number, limit: number, logType: LogType): Promise<L2BlockL2Logs[]> {
    const logSource = logType === LogType.ENCRYPTED ? this.encryptedLogsSource : this.unencryptedLogsSource;
    return logSource.getLogs(from, limit, logType);
  }

  /**
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   */
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse> {
    return this.unencryptedLogsSource.getUnencryptedLogs(filter);
  }

  /**
   * Method to submit a transaction to the p2p pool.
   * @param tx - The transaction to be submitted.
   */
  public async sendTx(tx: Tx) {
    this.log.info(`Received tx ${await tx.getTxHash()}`);
    await this.p2pClient!.sendTx(tx);
  }

  public getTx(txHash: TxHash): Promise<L2Tx | undefined> {
    return this.blockSource.getL2Tx(txHash);
  }

  /**
   * Method to stop the aztec node.
   */
  public async stop() {
    this.log.info(`Stopping`);
    await this.sequencer?.stop();
    await this.p2pClient.stop();
    await this.worldStateSynchronizer.stop();
    await this.blockSource.stop();
    this.log.info(`Stopped`);
  }

  /**
   * Method to retrieve pending txs.
   * @returns - The pending txs.
   */
  public async getPendingTxs() {
    return await this.p2pClient!.getTxs();
  }

  /**
   * Method to retrieve a single pending tx.
   * @param txHash - The transaction hash to return.
   * @returns - The pending tx if it exists.
   */
  public async getPendingTxByHash(txHash: TxHash) {
    return await this.p2pClient!.getTxByHash(txHash);
  }

  /**
   * Find the index of the given leaf in the given tree.
   * @param blockNumber - The block number at which to get the data
   * @param treeId - The tree to search in.
   * @param leafValue - The value to search for
   * @returns The index of the given leaf in the given tree or undefined if not found.
   */
  public async findLeafIndex(
    blockNumber: number | 'latest',
    treeId: MerkleTreeId,
    leafValue: Fr,
  ): Promise<bigint | undefined> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.findLeafIndex(treeId, leafValue.toBuffer());
  }

  /**
   * Returns a sibling path for the given index in the contract tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  public async getContractSiblingPath(
    blockNumber: number | 'latest',
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof CONTRACT_TREE_HEIGHT>> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.getSiblingPath(MerkleTreeId.CONTRACT_TREE, leafIndex);
  }

  /**
   * Returns a sibling path for the given index in the nullifier tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  public async getNullifierSiblingPath(
    blockNumber: number | 'latest',
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof NULLIFIER_TREE_HEIGHT>> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.getSiblingPath(MerkleTreeId.NULLIFIER_TREE, leafIndex);
  }

  /**
   * Returns a sibling path for the given index in the data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  public async getNoteHashSiblingPath(
    blockNumber: number | 'latest',
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof NOTE_HASH_TREE_HEIGHT>> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.getSiblingPath(MerkleTreeId.NOTE_HASH_TREE, leafIndex);
  }

  /**
   * Gets a confirmed/consumed L1 to L2 message for the given message key
   * and its index in the merkle tree.
   * @param messageKey - The message key.
   * @returns The map containing the message and index.
   */
  public async getL1ToL2MessageAndIndex(messageKey: Fr): Promise<L1ToL2MessageAndIndex> {
    // todo: #697 - make this one lookup.
    const index = (await this.findLeafIndex('latest', MerkleTreeId.L1_TO_L2_MESSAGE_TREE, messageKey))!;
    const message = await this.l1ToL2MessageSource.getConfirmedL1ToL2Message(messageKey);
    return Promise.resolve(new L1ToL2MessageAndIndex(index, message));
  }

  /**
   * Returns a sibling path for a leaf in the committed l1 to l2 data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  public async getL1ToL2MessageSiblingPath(
    blockNumber: number | 'latest',
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.getSiblingPath(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, leafIndex);
  }

  /**
   * Returns a sibling path for a leaf in the committed blocks tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  public async getArchiveSiblingPath(
    blockNumber: number | 'latest',
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof ARCHIVE_HEIGHT>> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.getSiblingPath(MerkleTreeId.ARCHIVE, leafIndex);
  }

  /**
   * Returns a sibling path for a leaf in the committed public data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  public async getPublicDataSiblingPath(
    blockNumber: number | 'latest',
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof PUBLIC_DATA_TREE_HEIGHT>> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.getSiblingPath(MerkleTreeId.PUBLIC_DATA_TREE, leafIndex);
  }

  /**
   * Returns a nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param nullifier - Nullifier we try to find witness for.
   * @returns The nullifier membership witness (if found).
   */
  public async getNullifierMembershipWitness(
    blockNumber: number | 'latest',
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    const db = await this.#getWorldState(blockNumber);
    const index = await db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
    if (!index) {
      return undefined;
    }

    const leafPreimagePromise = db.getLeafPreimage(MerkleTreeId.NULLIFIER_TREE, index);
    const siblingPathPromise = db.getSiblingPath<typeof NULLIFIER_TREE_HEIGHT>(
      MerkleTreeId.NULLIFIER_TREE,
      BigInt(index),
    );

    const [leafPreimage, siblingPath] = await Promise.all([leafPreimagePromise, siblingPathPromise]);

    if (!leafPreimage) {
      return undefined;
    }

    return new NullifierMembershipWitness(BigInt(index), leafPreimage as NullifierLeafPreimage, siblingPath);
  }

  /**
   * Returns a low nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param nullifier - Nullifier we try to find the low nullifier witness for.
   * @returns The low nullifier membership witness (if found).
   * @remarks Low nullifier witness can be used to perform a nullifier non-inclusion proof by leveraging the "linked
   * list structure" of leaves and proving that a lower nullifier is pointing to a bigger next value than the nullifier
   * we are trying to prove non-inclusion for.
   *
   * Note: This function returns the membership witness of the nullifier itself and not the low nullifier when
   * the nullifier already exists in the tree. This is because the `getPreviousValueIndex` function returns the
   * index of the nullifier itself when it already exists in the tree.
   * TODO: This is a confusing behavior and we should eventually address that.
   */
  public async getLowNullifierMembershipWitness(
    blockNumber: number | 'latest',
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    const committedDb = await this.#getWorldState(blockNumber);
    const findResult = await committedDb.getPreviousValueIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBigInt());
    if (!findResult) {
      return undefined;
    }
    const { index, alreadyPresent } = findResult;
    if (alreadyPresent) {
      this.log.warn(`Nullifier ${nullifier.toBigInt()} already exists in the tree`);
    }
    const preimageData = (await committedDb.getLeafPreimage(MerkleTreeId.NULLIFIER_TREE, index))!;

    const siblingPath = await committedDb.getSiblingPath<typeof NULLIFIER_TREE_HEIGHT>(
      MerkleTreeId.NULLIFIER_TREE,
      BigInt(index),
    );
    return new NullifierMembershipWitness(BigInt(index), preimageData as NullifierLeafPreimage, siblingPath);
  }

  async getPublicDataTreeWitness(blockNumber: number | 'latest', leafSlot: Fr): Promise<PublicDataWitness | undefined> {
    const committedDb = await this.#getWorldState(blockNumber);
    const lowLeafResult = await committedDb.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot.toBigInt());
    if (!lowLeafResult) {
      return undefined;
    } else {
      const preimage = (await committedDb.getLeafPreimage(
        MerkleTreeId.PUBLIC_DATA_TREE,
        lowLeafResult.index,
      )) as PublicDataTreeLeafPreimage;
      const path = await committedDb.getSiblingPath<typeof PUBLIC_DATA_TREE_HEIGHT>(
        MerkleTreeId.PUBLIC_DATA_TREE,
        lowLeafResult.index,
      );
      return new PublicDataWitness(lowLeafResult.index, preimage, path);
    }
  }

  /**
   * Gets the storage value at the given contract storage slot.
   *
   * @remarks The storage slot here refers to the slot as it is defined in Noir not the index in the merkle tree.
   * Aztec's version of `eth_getStorageAt`.
   *
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @returns Storage value at the given contract slot.
   */
  public async getPublicStorageAt(contract: AztecAddress, slot: Fr): Promise<Fr> {
    const committedDb = await this.#getWorldState('latest');
    const leafSlot = computePublicDataTreeLeafSlot(contract, slot);

    const lowLeafResult = await committedDb.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot.toBigInt());
    if (!lowLeafResult || !lowLeafResult.alreadyPresent) {
      return Fr.ZERO;
    }
    const preimage = (await committedDb.getLeafPreimage(
      MerkleTreeId.PUBLIC_DATA_TREE,
      lowLeafResult.index,
    )) as PublicDataTreeLeafPreimage;
    return preimage.value;
  }

  /**
   * Returns the currently committed block header.
   * @returns The current committed block header.
   */
  public async getHeader(): Promise<Header> {
    const block = await this.getBlock(-1);
    if (block) {
      return block.header;
    }

    // No block was not found so we build the initial header.
    const committedDb = await this.#getWorldState('latest');
    return await committedDb.buildInitialHeader();
  }

  /**
   * Simulates the public part of a transaction with the current state.
   * @param tx - The transaction to simulate.
   **/
  public async simulatePublicCalls(tx: Tx) {
    this.log.info(`Simulating tx ${await tx.getTxHash()}`);
    const blockNumber = (await this.blockSource.getBlockNumber()) + 1;

    // If sequencer is not initialized, we just set these values to zero for simulation.
    const coinbase = this.sequencer?.coinbase || EthAddress.ZERO;
    const feeRecipient = this.sequencer?.feeRecipient || AztecAddress.ZERO;

    const newGlobalVariables = await this.globalVariableBuilder.buildGlobalVariables(
      new Fr(blockNumber),
      coinbase,
      feeRecipient,
    );
    const prevHeader = (await this.blockSource.getBlock(-1))?.header;

    // Instantiate merkle trees so uncommitted updates by this simulation are local to it.
    // TODO we should be able to remove this after https://github.com/AztecProtocol/aztec-packages/issues/1869
    // So simulation of public functions doesn't affect the merkle trees.
    const merkleTrees = await MerkleTrees.new(this.merkleTreesDb, this.log);

    const publicProcessorFactory = new PublicProcessorFactory(
      merkleTrees.asLatest(),
      this.contractDataSource,
      this.l1ToL2MessageSource,
    );
    const processor = await publicProcessorFactory.create(prevHeader, newGlobalVariables);
    const [, failedTxs] = await processor.process([tx]);
    if (failedTxs.length) {
      throw failedTxs[0].error;
    }
    this.log.info(`Simulated tx ${await tx.getTxHash()} succeeds`);
  }

  public setConfig(config: Partial<SequencerConfig>): Promise<void> {
    this.sequencer?.updateSequencerConfig(config);
    return Promise.resolve();
  }

  /**
   * Returns an instance of MerkleTreeOperations having first ensured the world state is fully synched
   * @param blockNumber - The block number at which to get the data.
   * @returns An instance of a committed MerkleTreeOperations
   */
  async #getWorldState(blockNumber: number | 'latest') {
    if (typeof blockNumber === 'number' && blockNumber < INITIAL_L2_BLOCK_NUM) {
      throw new Error('Invalid block number to get world state for: ' + blockNumber);
    }

    let blockSyncedTo: number = 0;
    try {
      // Attempt to sync the world state if necessary
      blockSyncedTo = await this.#syncWorldState();
    } catch (err) {
      this.log.error(`Error getting world state: ${err}`);
    }

    // using a snapshot could be less efficient than using the committed db
    if (blockNumber === 'latest' || blockNumber === blockSyncedTo) {
      this.log(`Using committed db for block ${blockNumber}, world state synced upto ${blockSyncedTo}`);
      return this.worldStateSynchronizer.getCommitted();
    } else if (blockNumber < blockSyncedTo) {
      this.log(`Using snapshot for block ${blockNumber}, world state synced upto ${blockSyncedTo}`);
      return this.worldStateSynchronizer.getSnapshot(blockNumber);
    } else {
      throw new Error(`Block ${blockNumber} not yet synced`);
    }
  }

  /**
   * Ensure we fully sync the world state
   * @returns A promise that fulfils once the world state is synced
   */
  async #syncWorldState(): Promise<number> {
    const blockSourceHeight = await this.blockSource.getBlockNumber();
    return this.worldStateSynchronizer.syncImmediate(blockSourceHeight);
  }
}
