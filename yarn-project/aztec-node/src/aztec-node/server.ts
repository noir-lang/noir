import { Archiver } from '@aztec/archiver';
import {
  CONTRACT_TREE_HEIGHT,
  CircuitsWasm,
  EthAddress,
  Fr,
  HistoricBlockData,
  L1_TO_L2_MSG_TREE_HEIGHT,
  PRIVATE_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { createDebugLogger } from '@aztec/foundation/log';
import { InMemoryTxPool, P2P, createP2PClient } from '@aztec/p2p';
import { SequencerClient } from '@aztec/sequencer-client';
import {
  AztecNode,
  ContractData,
  ContractDataAndBytecode,
  ContractDataSource,
  L1ToL2MessageAndIndex,
  L1ToL2MessageSource,
  L2Block,
  L2BlockL2Logs,
  L2BlockSource,
  L2LogsSource,
  LogType,
  MerkleTreeId,
  SiblingPath,
  Tx,
  TxHash,
} from '@aztec/types';
import {
  MerkleTrees,
  ServerWorldStateSynchroniser,
  WorldStateConfig,
  WorldStateSynchroniser,
  computePublicDataTreeLeafIndex,
  getConfigEnvVars as getWorldStateConfig,
} from '@aztec/world-state';

import { default as levelup } from 'levelup';
import { MemDown, default as memdown } from 'memdown';

import { AztecNodeConfig } from './config.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;

/**
 * The aztec node.
 */
export class AztecNodeService implements AztecNode {
  constructor(
    protected p2pClient: P2P,
    protected blockSource: L2BlockSource,
    protected encryptedLogsSource: L2LogsSource,
    protected unencryptedLogsSource: L2LogsSource,
    protected contractDataSource: ContractDataSource,
    protected l1ToL2MessageSource: L1ToL2MessageSource,
    protected worldStateSynchroniser: WorldStateSynchroniser,
    protected sequencer: SequencerClient,
    protected chainId: number,
    protected version: number,
    private log = createDebugLogger('aztec:node'),
  ) {}

  /**
   * Initialises the Aztec Node, wait for component to sync.
   * @param config - The configuration to be used by the aztec node.
   * @returns - A fully synced Aztec Node for use in development/testing.
   */
  public static async createAndSync(config: AztecNodeConfig) {
    // first create and sync the archiver
    const archiver = await Archiver.createAndSync(config);

    // we identify the P2P transaction protocol by using the rollup contract address.
    // this may well change in future
    config.transactionProtocol = `/aztec/tx/${config.rollupContract.toString()}`;

    // create the tx pool and the p2p client, which will need the l2 block source
    const p2pClient = await createP2PClient(config, new InMemoryTxPool(), archiver);

    // now create the merkle trees and the world state syncher
    const merkleTreeDB = await MerkleTrees.new(levelup(createMemDown()), await CircuitsWasm.get());
    const worldStateConfig: WorldStateConfig = getWorldStateConfig();
    const worldStateSynchroniser = new ServerWorldStateSynchroniser(merkleTreeDB, archiver, worldStateConfig);

    // start both and wait for them to sync from the block source
    await Promise.all([p2pClient.start(), worldStateSynchroniser.start()]);

    // now create the sequencer
    const sequencer = await SequencerClient.new(
      config,
      p2pClient,
      worldStateSynchroniser,
      archiver,
      archiver,
      archiver,
    );
    return new AztecNodeService(
      p2pClient,
      archiver,
      archiver,
      archiver,
      archiver,
      archiver,
      worldStateSynchroniser,
      sequencer,
      config.chainId,
      config.version,
    );
  }

  /**
   * Method to determine if the node is ready to accept transactions.
   * @returns - Flag indicating the readiness for tx submission.
   */
  public async isReady() {
    return (await this.p2pClient.isReady()) ?? false;
  }

  /**
   * Get the a given block.
   * @param number - The block number being requested.
   * @returns The blocks requested.
   */
  public async getBlock(number: number): Promise<L2Block | undefined> {
    return await this.blockSource.getL2Block(number);
  }

  /**
   * Method to request blocks. Will attempt to return all requested blocks but will return only those available.
   * @param from - The start of the range of blocks to return.
   * @param limit - The maximum number of blocks to obtain.
   * @returns The blocks requested.
   */
  public async getBlocks(from: number, limit: number): Promise<L2Block[]> {
    return (await this.blockSource.getL2Blocks(from, limit)) ?? [];
  }

  /**
   * Method to fetch the current block height.
   * @returns The block height as a number.
   */
  public async getBlockHeight(): Promise<number> {
    return await this.blockSource.getBlockHeight();
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
   * Method to fetch the rollup contract address at the base-layer.
   * @returns The rollup address.
   */
  public getRollupAddress(): Promise<EthAddress> {
    return this.blockSource.getRollupAddress();
  }

  /**
   * Lookup the L2 contract data for this contract.
   * Contains the ethereum portal address and bytecode.
   * @param contractAddress - The contract data address.
   * @returns The complete contract data including portal address & bytecode (if we didn't throw an error).
   */
  public async getContractDataAndBytecode(contractAddress: AztecAddress): Promise<ContractDataAndBytecode | undefined> {
    return await this.contractDataSource.getContractDataAndBytecode(contractAddress);
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
   * Method to submit a transaction to the p2p pool.
   * @param tx - The transaction to be submitted.
   */
  public async sendTx(tx: Tx) {
    this.log.info(`Received tx ${await tx.getTxHash()}`);
    await this.p2pClient!.sendTx(tx);
  }

  /**
   * Method to stop the aztec node.
   */
  public async stop() {
    await this.sequencer.stop();
    await this.p2pClient.stop();
    await this.worldStateSynchroniser.stop();
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
    return await this.p2pClient!.getTxByhash(txHash);
  }

  /**
   * Find the index of the given contract.
   * @param leafValue - The value to search for.
   * @returns The index of the given leaf in the contracts tree or undefined if not found.
   */
  public async findContractIndex(leafValue: Buffer): Promise<bigint | undefined> {
    const committedDb = await this.getWorldState();
    return committedDb.findLeafIndex(MerkleTreeId.CONTRACT_TREE, leafValue);
  }

  /**
   * Returns the sibling path for the given index in the contract tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  public async getContractPath(leafIndex: bigint): Promise<SiblingPath<typeof CONTRACT_TREE_HEIGHT>> {
    const committedDb = await this.getWorldState();
    return committedDb.getSiblingPath(MerkleTreeId.CONTRACT_TREE, leafIndex);
  }

  /**
   * Find the index of the given commitment.
   * @param leafValue - The value to search for.
   * @returns The index of the given leaf in the private data tree or undefined if not found.
   */
  public async findCommitmentIndex(leafValue: Buffer): Promise<bigint | undefined> {
    const committedDb = await this.getWorldState();
    return committedDb.findLeafIndex(MerkleTreeId.PRIVATE_DATA_TREE, leafValue);
  }

  /**
   * Returns the sibling path for the given index in the data tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  public async getDataTreePath(leafIndex: bigint): Promise<SiblingPath<typeof PRIVATE_DATA_TREE_HEIGHT>> {
    const committedDb = await this.getWorldState();
    return committedDb.getSiblingPath(MerkleTreeId.PRIVATE_DATA_TREE, leafIndex);
  }

  /**
   * Gets a confirmed/consumed L1 to L2 message for the given message key
   * and its index in the merkle tree.
   * @param messageKey - The message key.
   * @returns The map containing the message and index.
   */
  public async getL1ToL2MessageAndIndex(messageKey: Fr): Promise<L1ToL2MessageAndIndex> {
    // todo: #697 - make this one lookup.
    const committedDb = await this.getWorldState();
    const message = await this.l1ToL2MessageSource.getConfirmedL1ToL2Message(messageKey);
    const index = (await committedDb.findLeafIndex(MerkleTreeId.L1_TO_L2_MESSAGES_TREE, messageKey.toBuffer()))!;
    return Promise.resolve({ message, index });
  }

  /**
   * Returns the sibling path for a leaf in the committed l1 to l2 data tree.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  public async getL1ToL2MessagesTreePath(leafIndex: bigint): Promise<SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>> {
    const committedDb = await this.getWorldState();
    return committedDb.getSiblingPath(MerkleTreeId.L1_TO_L2_MESSAGES_TREE, leafIndex);
  }

  /**
   * Gets the storage value at the given contract slot.
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @returns Storage value at the given contract slot (or undefined if not found).
   * Note: Aztec's version of `eth_getStorageAt`.
   */
  public async getPublicStorageAt(contract: AztecAddress, slot: bigint): Promise<Buffer | undefined> {
    const committedDb = await this.getWorldState();
    const leafIndex = computePublicDataTreeLeafIndex(contract, new Fr(slot), await CircuitsWasm.get());
    return committedDb.getLeafValue(MerkleTreeId.PUBLIC_DATA_TREE, leafIndex);
  }

  /**
   * Returns the current committed roots for the data trees.
   * @returns The current committed roots for the data trees.
   */
  public async getTreeRoots(): Promise<Record<MerkleTreeId, Fr>> {
    const committedDb = await this.getWorldState();
    const getTreeRoot = async (id: MerkleTreeId) => Fr.fromBuffer((await committedDb.getTreeInfo(id)).root);

    const [privateDataTree, nullifierTree, contractTree, l1ToL2MessagesTree, blocksTree, publicDataTree] =
      await Promise.all([
        getTreeRoot(MerkleTreeId.PRIVATE_DATA_TREE),
        getTreeRoot(MerkleTreeId.NULLIFIER_TREE),
        getTreeRoot(MerkleTreeId.CONTRACT_TREE),
        getTreeRoot(MerkleTreeId.L1_TO_L2_MESSAGES_TREE),
        getTreeRoot(MerkleTreeId.BLOCKS_TREE),
        getTreeRoot(MerkleTreeId.PUBLIC_DATA_TREE),
      ]);

    return {
      [MerkleTreeId.CONTRACT_TREE]: contractTree,
      [MerkleTreeId.PRIVATE_DATA_TREE]: privateDataTree,
      [MerkleTreeId.NULLIFIER_TREE]: nullifierTree,
      [MerkleTreeId.PUBLIC_DATA_TREE]: publicDataTree,
      [MerkleTreeId.L1_TO_L2_MESSAGES_TREE]: l1ToL2MessagesTree,
      [MerkleTreeId.BLOCKS_TREE]: blocksTree,
    };
  }

  /**
   * Returns the currently committed historic block data.
   * @returns The current committed block data.
   */
  public async getHistoricBlockData(): Promise<HistoricBlockData> {
    const committedDb = await this.getWorldState();
    const [roots, globalsHash] = await Promise.all([this.getTreeRoots(), committedDb.getLatestGlobalVariablesHash()]);

    return new HistoricBlockData(
      roots[MerkleTreeId.PRIVATE_DATA_TREE],
      roots[MerkleTreeId.NULLIFIER_TREE],
      roots[MerkleTreeId.CONTRACT_TREE],
      roots[MerkleTreeId.L1_TO_L2_MESSAGES_TREE],
      roots[MerkleTreeId.BLOCKS_TREE],
      Fr.ZERO,
      roots[MerkleTreeId.PUBLIC_DATA_TREE],
      globalsHash,
    );
  }

  /**
   * Returns an instance of MerkleTreeOperations having first ensured the world state is fully synched
   * @returns An instance of a committed MerkleTreeOperations
   */
  private async getWorldState() {
    try {
      // Attempt to sync the world state if necessary
      await this.syncWorldState();
    } catch (err) {
      this.log.error(`Error getting world state: ${err}`);
    }
    return this.worldStateSynchroniser.getCommitted();
  }

  /**
   * Ensure we fully sync the world state
   * @returns A promise that fulfils once the world state is synced
   */
  private async syncWorldState() {
    const blockSourceHeight = await this.blockSource.getBlockHeight();
    await this.worldStateSynchroniser.syncImmediate(blockSourceHeight);
  }
}
