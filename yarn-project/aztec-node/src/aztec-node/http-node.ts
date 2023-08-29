import {
  AztecAddress,
  CONTRACT_TREE_HEIGHT,
  EthAddress,
  Fr,
  HistoricBlockData,
  L1_TO_L2_MSG_TREE_HEIGHT,
  PRIVATE_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { DebugLogger, createDebugLogger } from '@aztec/foundation/log';
import {
  AztecNode,
  ContractData,
  ExtendedContractData,
  L1ToL2Message,
  L1ToL2MessageAndIndex,
  L2Block,
  L2BlockL2Logs,
  L2Tx,
  LogType,
  MerkleTreeId,
  SiblingPath,
  Tx,
  TxHash,
} from '@aztec/types';

/**
 * A Http client based implementation of Aztec Node.
 */
export class HttpNode implements AztecNode {
  private baseUrl: string;
  private log: DebugLogger;

  constructor(baseUrl: string, log = createDebugLogger('aztec:http-node')) {
    this.baseUrl = baseUrl.toString().replace(/\/$/, '');
    this.log = log;
  }
  /**
   * Method to determine if the node is ready to accept transactions.
   * @returns - Flag indicating the readiness for tx submission.
   */
  public async isReady(): Promise<boolean> {
    const url = new URL(this.baseUrl);
    const response = await fetch(url.toString());
    const respJson = await response.json();
    return respJson.isReady;
  }

  /**
   * Method to request a block at the provided block number.
   * @param number - The block number to request.
   * @returns The block requested. Or undefined if it does not exist.
   */
  async getBlock(number: number): Promise<L2Block | undefined> {
    const url = new URL(`${this.baseUrl}/get-block`);
    url.searchParams.append('number', number.toString());
    const response = await (await fetch(url.toString())).json();
    const { block } = response;
    return Promise.resolve(block ? L2Block.decode(Buffer.from(block, 'hex')) : block);
  }

  /**
   * Method to request blocks. Will attempt to return all requested blocks but will return only those available.
   * @param from - The start of the range of blocks to return.
   * @param limit - Maximum number of blocks to obtain.
   * @returns The blocks requested.
   */
  async getBlocks(from: number, limit: number): Promise<L2Block[]> {
    const url = new URL(`${this.baseUrl}/get-blocks`);
    url.searchParams.append('from', from.toString());
    if (limit !== undefined) {
      url.searchParams.append('limit', limit.toString());
    }
    const response = await (await fetch(url.toString())).json();
    const blocks = response.blocks as string[];
    if (!blocks) {
      return Promise.resolve([]);
    }
    return Promise.resolve(blocks.map(x => L2Block.decode(Buffer.from(x, 'hex'))));
  }

  /**
   * Method to fetch the current block number.
   * @returns The current block number.
   */
  async getBlockNumber(): Promise<number> {
    const url = new URL(`${this.baseUrl}/get-block-number`);
    const response = await fetch(url.toString());
    const respJson = await response.json();
    return respJson.blockNumber;
  }

  /**
   * Method to fetch the version of the rollup the node is connected to.
   * @returns The rollup version.
   */
  public async getVersion(): Promise<number> {
    const url = new URL(`${this.baseUrl}/get-version`);
    const response = await fetch(url.toString());
    const respJson = await response.json();
    return respJson.version;
  }

  public async getRollupAddress(): Promise<EthAddress> {
    const url = new URL(`${this.baseUrl}/get-rollup-address`);
    const response = await fetch(url.toString());
    const respJson = await response.json();
    return EthAddress.fromString(respJson.rollupAddress);
  }

  /**
   * Method to fetch the chain id of the base-layer for the rollup.
   * @returns The chain id.
   */
  public async getChainId(): Promise<number> {
    const url = new URL(`${this.baseUrl}/get-chain-id`);
    const response = await fetch(url.toString());
    const respJson = await response.json();
    return respJson.chainId;
  }

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  async getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined> {
    const url = new URL(`${this.baseUrl}/contract-data-and-bytecode`);
    url.searchParams.append('address', contractAddress.toString());
    const response = await (await fetch(url.toString())).json();
    if (!response || !response.contractData) {
      return undefined;
    }
    const contract = response.contractData as string;
    return Promise.resolve(ExtendedContractData.fromBuffer(Buffer.from(contract, 'hex')));
  }

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The maximum number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  public async getLogs(from: number, limit: number, logType: LogType): Promise<L2BlockL2Logs[]> {
    const url = new URL(`${this.baseUrl}/get-logs`);

    url.searchParams.append('from', from.toString());
    url.searchParams.append('limit', limit.toString());
    url.searchParams.append('logType', logType.toString());

    const response = await (await fetch(url.toString())).json();
    const logs = response.logs as string[];

    if (!logs) {
      return Promise.resolve([]);
    }
    return Promise.resolve(logs.map(x => L2BlockL2Logs.fromBuffer(Buffer.from(x, 'hex'))));
  }

  /**
   * Lookup the contract data for this contract.
   * Contains the ethereum portal address.
   * @param contractAddress - The contract data address.
   * @returns The contract's address & portal address.
   */
  async getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined> {
    const url = new URL(`${this.baseUrl}/contract-data`);
    url.searchParams.append('address', contractAddress.toString());
    const response = await (await fetch(url.toString())).json();
    if (!response || !response.contractData) {
      return undefined;
    }
    const contract = response.contractData as string;
    return Promise.resolve(ContractData.fromBuffer(Buffer.from(contract, 'hex')));
  }

  /**
   * Method to submit a transaction to the p2p pool.
   * @param tx - The transaction to be submitted.
   */
  async sendTx(tx: Tx): Promise<void> {
    const url = new URL(`${this.baseUrl}/tx`);
    const init: RequestInit = {};
    init['method'] = 'POST';
    init['body'] = tx.toBuffer();
    await fetch(url, init);
  }

  /**
   * Gets an l2 tx.
   * @param txHash - The txHash of the l2 tx.
   * @returns The requested L2 tx.
   */
  async getTx(txHash: TxHash): Promise<L2Tx | undefined> {
    const url = new URL(`${this.baseUrl}/get-tx`);
    url.searchParams.append('hash', txHash.toString());
    const response = await fetch(url.toString());
    if (response.status === 404) {
      this.log.info(`Tx ${txHash.toString()} not found`);
      return undefined;
    }
    const txBuffer = Buffer.from(await response.arrayBuffer());
    const tx = L2Tx.fromBuffer(txBuffer);
    return Promise.resolve(tx);
  }

  /**
   * Method to retrieve pending txs.
   * @returns - The pending txs.
   */
  getPendingTxs(): Promise<Tx[]> {
    return Promise.resolve([]);
  }

  /**
   * Method to retrieve a single pending tx.
   * @param txHash - The transaction hash to return.
   * @returns - The pending tx if it exists.
   */
  async getPendingTxByHash(txHash: TxHash): Promise<Tx | undefined> {
    const url = new URL(`${this.baseUrl}/get-pending-tx`);
    url.searchParams.append('hash', txHash.toString());
    const response = await fetch(url.toString());
    if (response.status === 404) {
      this.log.info(`Tx ${txHash.toString()} not found`);
      return undefined;
    }
    const txBuffer = Buffer.from(await response.arrayBuffer());
    const tx = Tx.fromBuffer(txBuffer);
    return Promise.resolve(tx);
  }

  /**
   * Find the index of the given contract.
   * @param leafValue - The value to search for.
   * @returns The index of the given leaf in the contracts tree or undefined if not found.
   */
  async findContractIndex(leafValue: Buffer): Promise<bigint | undefined> {
    const url = new URL(`${this.baseUrl}/contract-index`);
    url.searchParams.append('leaf', leafValue.toString('hex'));
    const response = await (await fetch(url.toString())).json();
    if (!response || !response.index) {
      return undefined;
    }
    const index = response.index as string;
    return Promise.resolve(BigInt(index));
  }

  /**
   * Returns the sibling path for the given index in the contract tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  async getContractPath(leafIndex: bigint): Promise<SiblingPath<typeof CONTRACT_TREE_HEIGHT>> {
    const url = new URL(`${this.baseUrl}/contract-path`);
    url.searchParams.append('leaf', leafIndex.toString());
    const response = await (await fetch(url.toString())).json();
    const path = response.path as string;
    return Promise.resolve(SiblingPath.fromString(path));
  }

  /**
   * Find the index of the given piece of data.
   * @param leafValue - The value to search for.
   * @returns The index of the given leaf in the data tree or undefined if not found.
   */
  async findCommitmentIndex(leafValue: Buffer): Promise<bigint | undefined> {
    const url = new URL(`${this.baseUrl}/commitment-index`);
    url.searchParams.append('leaf', leafValue.toString('hex'));
    const response = await (await fetch(url.toString())).json();
    if (!response || !response.index) {
      return undefined;
    }
    const index = response.index as string;
    return Promise.resolve(BigInt(index));
  }

  /**
   * Returns the sibling path for the given index in the data tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  async getDataTreePath(leafIndex: bigint): Promise<SiblingPath<typeof PRIVATE_DATA_TREE_HEIGHT>> {
    const url = new URL(`${this.baseUrl}/data-path`);
    url.searchParams.append('leaf', leafIndex.toString());
    const response = await (await fetch(url.toString())).json();
    const path = response.path as string;
    return Promise.resolve(SiblingPath.fromString(path));
  }

  /**
   * Gets a consumed/confirmed L1 to L2 message for the given message key and its index in the merkle tree.
   * @param messageKey - The message key.
   * @returns the message (or throws if not found)
   */
  async getL1ToL2MessageAndIndex(messageKey: Fr): Promise<L1ToL2MessageAndIndex> {
    const url = new URL(`${this.baseUrl}/l1-l2-message`);
    url.searchParams.append('messageKey', messageKey.toString());
    const response = await (await fetch(url.toString())).json();
    return Promise.resolve({
      message: L1ToL2Message.fromBuffer(Buffer.from(response.message as string, 'hex')),
      index: BigInt(response.index as string),
    });
  }

  /**
   * Returns the sibling path for a leaf in the committed l1 to l2 data tree.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  async getL1ToL2MessagesTreePath(leafIndex: bigint): Promise<SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>> {
    const url = new URL(`${this.baseUrl}/l1-l2-path`);
    url.searchParams.append('leaf', leafIndex.toString());
    const response = await (await fetch(url.toString())).json();
    const path = response.path as string;
    return Promise.resolve(SiblingPath.fromString(path));
  }

  /**
   * Gets the storage value at the given contract slot.
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @returns Storage value at the given contract slot (or undefined if not found).
   * Note: Aztec's version of `eth_getStorageAt`.
   */
  async getPublicStorageAt(contract: AztecAddress, slot: bigint): Promise<Buffer | undefined> {
    const url = new URL(`${this.baseUrl}/public-storage-at`);
    url.searchParams.append('address', contract.toString());
    url.searchParams.append('slot', slot.toString());
    const response = await (await fetch(url.toString())).json();
    if (!response || !response.value) {
      return undefined;
    }
    const value = response.value as string;
    return Promise.resolve(Buffer.from(value, 'hex'));
  }

  /**
   * Returns the current committed roots for the data trees.
   * @returns The current committed roots for the data trees.
   */
  async getTreeRoots(): Promise<Record<MerkleTreeId, Fr>> {
    const url = new URL(`${this.baseUrl}/tree-roots`);
    const response = await (await fetch(url.toString())).json();

    const extractRoot = (treeId: MerkleTreeId) => {
      // Buffer.from(...) returns an empty buffer when a hex string is prefixed with "0x"
      const rootHexString = response.roots[treeId].replace(/^0x/, '');
      return Fr.fromBuffer(Buffer.from(rootHexString, 'hex'));
    };

    return {
      [MerkleTreeId.CONTRACT_TREE]: extractRoot(MerkleTreeId.CONTRACT_TREE),
      [MerkleTreeId.PRIVATE_DATA_TREE]: extractRoot(MerkleTreeId.PRIVATE_DATA_TREE),
      [MerkleTreeId.NULLIFIER_TREE]: extractRoot(MerkleTreeId.NULLIFIER_TREE),
      [MerkleTreeId.PUBLIC_DATA_TREE]: extractRoot(MerkleTreeId.PUBLIC_DATA_TREE),
      [MerkleTreeId.L1_TO_L2_MESSAGES_TREE]: extractRoot(MerkleTreeId.L1_TO_L2_MESSAGES_TREE),
      [MerkleTreeId.BLOCKS_TREE]: extractRoot(MerkleTreeId.BLOCKS_TREE),
    };
  }

  /**
   * Returns the currently committed historic block data.
   * @returns The current committed block data.
   */
  public async getHistoricBlockData(): Promise<HistoricBlockData> {
    const url = new URL(`${this.baseUrl}/historic-block-data`);
    const response = await (await fetch(url.toString())).json();
    return response.blockData;
  }
}
