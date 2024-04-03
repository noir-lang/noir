import { type ArchiveSource, Archiver, KVArchiverDataStore, createArchiverClient } from '@aztec/archiver';
import {
  type AztecNode,
  type FromLogType,
  type GetUnencryptedLogsResponse,
  type L1ToL2MessageSource,
  type L2Block,
  type L2BlockL2Logs,
  type L2BlockNumber,
  type L2BlockSource,
  type L2LogsSource,
  type LogFilter,
  LogType,
  MerkleTreeId,
  NullifierMembershipWitness,
  type ProverClient,
  PublicDataWitness,
  type SequencerConfig,
  type SiblingPath,
  type Tx,
  type TxEffect,
  type TxHash,
  TxReceipt,
  TxStatus,
  partitionReverts,
} from '@aztec/circuit-types';
import {
  type ARCHIVE_HEIGHT,
  EthAddress,
  Fr,
  type Header,
  INITIAL_L2_BLOCK_NUM,
  type L1_TO_L2_MSG_TREE_HEIGHT,
  L2_TO_L1_MESSAGE_LENGTH,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  type NOTE_HASH_TREE_HEIGHT,
  type NULLIFIER_TREE_HEIGHT,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  type NullifierLeafPreimage,
  type PUBLIC_DATA_TREE_HEIGHT,
  type PublicDataTreeLeafPreimage,
} from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot } from '@aztec/circuits.js/hash';
import { type L1ContractAddresses, createEthereumChain } from '@aztec/ethereum';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { type AztecKVStore } from '@aztec/kv-store';
import { AztecLmdbStore } from '@aztec/kv-store/lmdb';
import { initStoreForRollup, openTmpStore } from '@aztec/kv-store/utils';
import { SHA256Trunc, StandardTree } from '@aztec/merkle-tree';
import { AztecKVTxPool, type P2P, createP2PClient } from '@aztec/p2p';
import { DummyProver, TxProver } from '@aztec/prover-client';
import {
  type GlobalVariableBuilder,
  PublicProcessorFactory,
  SequencerClient,
  getGlobalVariableBuilder,
} from '@aztec/sequencer-client';
import { WASMSimulator } from '@aztec/simulator';
import {
  type ContractClassPublic,
  type ContractDataSource,
  type ContractInstanceWithAddress,
} from '@aztec/types/contracts';
import {
  MerkleTrees,
  ServerWorldStateSynchronizer,
  type WorldStateConfig,
  type WorldStateSynchronizer,
  getConfigEnvVars as getWorldStateConfig,
} from '@aztec/world-state';

import { type AztecNodeConfig } from './config.js';
import { getSimulationProvider } from './simulator-factory.js';

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
    private readonly prover: ProverClient,
    private log = createDebugLogger('aztec:node'),
  ) {
    const message =
      `Started Aztec Node against chain 0x${chainId.toString(16)} with contracts - \n` +
      `Rollup: ${config.l1Contracts.rollupAddress.toString()}\n` +
      `Registry: ${config.l1Contracts.registryAddress.toString()}\n` +
      `Inbox: ${config.l1Contracts.inboxAddress.toString()}\n` +
      `Outbox: ${config.l1Contracts.outboxAddress.toString()}\n` +
      `Availability Oracle: ${config.l1Contracts.availabilityOracleAddress.toString()}`;
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
    const storeLog = createDebugLogger('aztec:node:lmdb');
    const store = await initStoreForRollup(
      AztecLmdbStore.open(config.dataDirectory, false, storeLog),
      config.l1Contracts.rollupAddress,
      storeLog,
    );

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

    // start the prover if we have been told to
    const simulationProvider = await getSimulationProvider(config, log);
    const prover = config.disableProver
      ? await DummyProver.new()
      : await TxProver.new(config, worldStateSynchronizer, simulationProvider);

    // now create the sequencer
    const sequencer = config.disableSequencer
      ? undefined
      : await SequencerClient.new(
          config,
          p2pClient,
          worldStateSynchronizer,
          archiver,
          archiver,
          archiver,
          prover,
          simulationProvider,
        );

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
      prover,
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
  public getLogs<TLogType extends LogType>(
    from: number,
    limit: number,
    logType: LogType,
  ): Promise<L2BlockL2Logs<FromLogType<TLogType>>[]> {
    const logSource = logType === LogType.ENCRYPTED ? this.encryptedLogsSource : this.unencryptedLogsSource;
    return logSource.getLogs(from, limit, logType) as Promise<L2BlockL2Logs<FromLogType<TLogType>>[]>;
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
    this.log.info(`Received tx ${tx.getTxHash()}`);
    await this.p2pClient!.sendTx(tx);
  }

  public async getTxReceipt(txHash: TxHash): Promise<TxReceipt> {
    let txReceipt = new TxReceipt(txHash, TxStatus.DROPPED, 'Tx dropped by P2P node.');

    // We first check if the tx is in pending (instead of first checking if it is mined) because if we first check
    // for mined and then for pending there could be a race condition where the tx is mined between the two checks
    // and we would incorrectly return a TxReceipt with status DROPPED
    const pendingTx = await this.getPendingTxByHash(txHash);
    if (pendingTx) {
      txReceipt = new TxReceipt(txHash, TxStatus.PENDING, '');
    }

    const settledTxReceipt = await this.blockSource.getSettledTxReceipt(txHash);
    if (settledTxReceipt) {
      txReceipt = settledTxReceipt;
    }

    return txReceipt;
  }

  public getTxEffect(txHash: TxHash): Promise<TxEffect | undefined> {
    return this.blockSource.getTxEffect(txHash);
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
    await this.prover.stop();
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
    blockNumber: L2BlockNumber,
    treeId: MerkleTreeId,
    leafValue: Fr,
  ): Promise<bigint | undefined> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.findLeafIndex(treeId, leafValue.toBuffer());
  }

  /**
   * Returns a sibling path for the given index in the nullifier tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  public async getNullifierSiblingPath(
    blockNumber: L2BlockNumber,
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
    blockNumber: L2BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof NOTE_HASH_TREE_HEIGHT>> {
    const committedDb = await this.#getWorldState(blockNumber);
    return committedDb.getSiblingPath(MerkleTreeId.NOTE_HASH_TREE, leafIndex);
  }

  /**
   * Returns the index and a sibling path for a leaf in the committed l1 to l2 data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param l1ToL2Message - The l1ToL2Message to get the index / sibling path for.
   * @param startIndex - The index to start searching from (used when skipping nullified messages)
   * @returns A tuple of the index and the sibling path of the L1ToL2Message (undefined if not found).
   */
  public async getL1ToL2MessageMembershipWitness(
    blockNumber: L2BlockNumber,
    l1ToL2Message: Fr,
    startIndex = 0n,
  ): Promise<[bigint, SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>] | undefined> {
    const index = await this.l1ToL2MessageSource.getL1ToL2MessageIndex(l1ToL2Message, startIndex);
    if (index === undefined) {
      return undefined;
    }
    const committedDb = await this.#getWorldState(blockNumber);
    const siblingPath = await committedDb.getSiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>(
      MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
      index,
    );
    return [index, siblingPath];
  }

  /**
   * Returns whether an L1 to L2 message is synced by archiver and if it's ready to be included in a block.
   * @param l1ToL2Message - The L1 to L2 message to check.
   * @param startL2BlockNumber - The block number after which we are interested in checking if the message was
   * included.
   * @remarks We pass in the minL2BlockNumber because there can be duplicate messages and the block number allow us
   * to skip the duplicates (we know after which block a given message is to be included).
   * @returns Whether the message is synced and ready to be included in a block.
   */
  public async isL1ToL2MessageSynced(l1ToL2Message: Fr, startL2BlockNumber = INITIAL_L2_BLOCK_NUM): Promise<boolean> {
    const startIndex = BigInt(startL2BlockNumber - INITIAL_L2_BLOCK_NUM) * BigInt(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP);
    return (await this.l1ToL2MessageSource.getL1ToL2MessageIndex(l1ToL2Message, startIndex)) !== undefined;
  }

  /**
   * Returns the index of a l2ToL1Message in a ephemeral l2 to l1 data tree as well as its sibling path.
   * @remarks This tree is considered ephemeral because it is created on-demand by: taking all the l2ToL1 messages
   * in a single block, and then using them to make a variable depth append-only tree with these messages as leaves.
   * The tree is discarded immediately after calculating what we need from it.
   * @param blockNumber - The block number at which to get the data.
   * @param l2ToL1Message - The l2ToL1Message get the index / sibling path for.
   * @returns A tuple of the index and the sibling path of the L2ToL1Message.
   */
  public async getL2ToL1MessageMembershipWitness(
    blockNumber: L2BlockNumber,
    l2ToL1Message: Fr,
  ): Promise<[bigint, SiblingPath<number>]> {
    const block = await this.blockSource.getBlock(blockNumber === 'latest' ? await this.getBlockNumber() : blockNumber);

    if (block === undefined) {
      throw new Error('Block is not defined');
    }

    // We multiply the number of messages per block by the length of each message because each message occupies
    // 2 leaves in the tree!
    const l2ToL1Messages = block.body.txEffects.flatMap(txEffect =>
      padArrayEnd(txEffect.l2ToL1Msgs, Fr.ZERO, MAX_NEW_L2_TO_L1_MSGS_PER_TX * L2_TO_L1_MESSAGE_LENGTH),
    );

    const indexOfL2ToL1Message = BigInt(
      l2ToL1Messages.findIndex(l2ToL1MessageInBlock => l2ToL1MessageInBlock.equals(l2ToL1Message)),
    );

    if (indexOfL2ToL1Message === -1n) {
      throw new Error('The L2ToL1Message you are trying to prove inclusion of does not exist');
    }

    const treeHeight = Math.ceil(Math.log2(l2ToL1Messages.length));
    // The root of this tree is the out_hash calculated in Noir => we truncate to match Noir's SHA
    const tree = new StandardTree(
      openTmpStore(true),
      new SHA256Trunc(),
      'temp_outhash_sibling_path',
      treeHeight,
      0n,
      Fr,
    );
    await tree.appendLeaves(l2ToL1Messages);

    return [indexOfL2ToL1Message, await tree.getSiblingPath(indexOfL2ToL1Message, true)];
  }

  /**
   * Returns a sibling path for a leaf in the committed blocks tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  public async getArchiveSiblingPath(
    blockNumber: L2BlockNumber,
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
    blockNumber: L2BlockNumber,
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
    blockNumber: L2BlockNumber,
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
    blockNumber: L2BlockNumber,
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

  async getPublicDataTreeWitness(blockNumber: L2BlockNumber, leafSlot: Fr): Promise<PublicDataWitness | undefined> {
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
    this.log.info(`Simulating tx ${tx.getTxHash()}`);
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
      new WASMSimulator(),
    );
    const processor = await publicProcessorFactory.create(prevHeader, newGlobalVariables);
    const [processedTxs, failedTxs, returns] = await processor.process([tx]);
    if (failedTxs.length) {
      this.log.warn(`Simulated tx ${tx.getTxHash()} fails: ${failedTxs[0].error}`);
      throw failedTxs[0].error;
    }
    const { reverted } = partitionReverts(processedTxs);
    if (reverted.length) {
      this.log.warn(`Simulated tx ${tx.getTxHash()} reverts: ${reverted[0].revertReason}`);
      throw reverted[0].revertReason;
    }
    this.log.info(`Simulated tx ${tx.getTxHash()} succeeds`);
    return returns;
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
  async #getWorldState(blockNumber: L2BlockNumber) {
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
