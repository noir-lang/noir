import { CONTRACT_TREE_HEIGHT, L1_TO_L2_MESSAGES_TREE_HEIGHT, PRIVATE_DATA_TREE_HEIGHT } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { ContractPublicData, ContractData, L2Block, MerkleTreeId, L1ToL2MessageAndIndex } from '@aztec/types';
import { SiblingPath } from '@aztec/merkle-tree';
import { Tx, TxHash } from '@aztec/types';
import { NoirLogs } from '@aztec/types';
import { Fr } from '@aztec/foundation/fields';

/**
 * The aztec node.
 */
export interface AztecNode {
  /**
   * Method to determine if the node is ready to accept transactions.
   * @returns - Flag indicating the readiness for tx submission.
   */
  isReady(): Promise<boolean>;

  /**
   * Method to request blocks. Will attempt to return all requested blocks but will return only those available.
   * @param from - The start of the range of blocks to return.
   * @param take - The number of blocks desired.
   * @returns The blocks requested.
   */
  getBlocks(from: number, take: number): Promise<L2Block[]>;

  /**
   * Method to fetch the current block height.
   * @returns The block height as a number.
   */
  getBlockHeight(): Promise<number>;

  /**
   * Lookup the L2 contract data for this contract.
   * Contains the ethereum portal address and bytecode.
   * @param contractAddress - The contract data address.
   * @returns The complete contract data including portal address & bytecode (if we didn't throw an error).
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractPublicData | undefined>;

  /**
   * Lookup the L2 contract info for this contract.
   * Contains the ethereum portal address .
   * @param contractAddress - The contract data address.
   * @returns The contract's address & portal address.
   */
  getContractInfo(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Gets the `take` amount of encrypted logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first encrypted logs to be returned.
   * @param take - The number of encrypted logs to return.
   * @returns The requested encrypted logs.
   */
  getEncryptedLogs(from: number, take: number): Promise<NoirLogs[]>;

  /**
   * Method to submit a transaction to the p2p pool.
   * @param tx - The transaction to be submitted.
   */
  sendTx(tx: Tx): Promise<void>;

  /**
   * Method to retrieve pending txs.
   * @returns The pending txs.
   */
  getPendingTxs(): Promise<Tx[]>;

  /**
   * Method to retrieve a single pending tx.
   * @param txHash - The transaction hash to return.
   * @returns The pending tx if it exists.
   */
  getPendingTxByHash(txHash: TxHash): Promise<Tx | undefined>;

  /**
   * Find the index of the given contract.
   * @param leafValue - The value to search for.
   * @returns The index of the given leaf in the contracts tree or undefined if not found.
   */
  findContractIndex(leafValue: Buffer): Promise<bigint | undefined>;

  /**
   * Returns the sibling path for the given index in the contract tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getContractPath(leafIndex: bigint): Promise<SiblingPath<typeof CONTRACT_TREE_HEIGHT>>;

  /**
   * Find the index of the given commitment.
   * @param leafValue - The value to search for.
   * @returns The index of the given leaf of undefined if not found.
   */
  findCommitmentIndex(leafValue: Buffer): Promise<bigint | undefined>;

  /**
   * Returns the sibling path for the given index in the data tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getDataTreePath(leafIndex: bigint): Promise<SiblingPath<typeof PRIVATE_DATA_TREE_HEIGHT>>;

  /**
   * Gets a confirmed/consumed L1 to L2 message for the given message key (throws if not found).
   * and its index in the merkle tree
   * @param messageKey - The message key.
   * @returns The map containing the message and index.
   */
  getL1ToL2MessageAndIndex(messageKey: Fr): Promise<L1ToL2MessageAndIndex>;

  /**
   * Returns the sibling path for a leaf in the committed l1 to l2 data tree.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  getL1ToL2MessagesTreePath(leafIndex: bigint): Promise<SiblingPath<typeof L1_TO_L2_MESSAGES_TREE_HEIGHT>>;

  /**
   * Gets the storage value at the given contract slot. Our version of eth_getStorageAt.
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @returns Storage value at the given contract slot (or undefined if not found).
   */
  getStorageAt(contract: AztecAddress, slot: bigint): Promise<Buffer | undefined>;

  /**
   * Returns the current committed roots for the data trees.
   * @returns The current committed roots for the data trees.
   */
  getTreeRoots(): Promise<Record<MerkleTreeId, Fr>>;
}
