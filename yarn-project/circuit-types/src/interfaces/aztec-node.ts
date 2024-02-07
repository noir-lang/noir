import {
  ARCHIVE_HEIGHT,
  CONTRACT_TREE_HEIGHT,
  Header,
  L1_TO_L2_MSG_TREE_HEIGHT,
  NOTE_HASH_TREE_HEIGHT,
  NULLIFIER_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { L1ContractAddresses } from '@aztec/ethereum';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { ContractClassPublic, ContractInstanceWithAddress } from '@aztec/types/contracts';

import { ContractData, ExtendedContractData } from '../contract_data.js';
import { L1ToL2MessageAndIndex } from '../l1_to_l2_message.js';
import { L2Block } from '../l2_block.js';
import { L2Tx } from '../l2_tx.js';
import { GetUnencryptedLogsResponse, L2BlockL2Logs, LogFilter, LogType } from '../logs/index.js';
import { MerkleTreeId } from '../merkle_tree_id.js';
import { SiblingPath } from '../sibling_path/index.js';
import { Tx, TxHash } from '../tx/index.js';
import { SequencerConfig } from './configs.js';
import { NullifierMembershipWitness } from './nullifier_tree.js';
import { PublicDataWitness } from './public_data_tree.js';

/** Helper type for a specific L2 block number or the latest block number */
type BlockNumber = number | 'latest';

/**
 * The aztec node.
 * We will probably implement the additional interfaces by means other than Aztec Node as it's currently a privacy leak
 */
export interface AztecNode {
  /**
   * Find the index of the given leaf in the given tree.
   * @param blockNumber - The block number at which to get the data or 'latest' for latest data
   * @param treeId - The tree to search in.
   * @param leafValue - The value to search for
   * @returns The index of the given leaf in the given tree or undefined if not found.
   */
  findLeafIndex(blockNumber: BlockNumber, treeId: MerkleTreeId, leafValue: Fr): Promise<bigint | undefined>;

  /**
   * Returns a sibling path for the given index in the contract tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getContractSiblingPath(
    blockNumber: BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof CONTRACT_TREE_HEIGHT>>;

  /**
   * Returns a sibling path for the given index in the nullifier tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getNullifierSiblingPath(
    blockNumber: BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof NULLIFIER_TREE_HEIGHT>>;

  /**
   * Returns a sibling path for the given index in the note hash tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getNoteHashSiblingPath(
    blockNumber: BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof NOTE_HASH_TREE_HEIGHT>>;

  /**
   * Gets a confirmed/consumed L1 to L2 message for the given message key (throws if not found).
   * and its index in the merkle tree
   * @param messageKey - The message key.
   * @returns The map containing the message and index.
   */
  getL1ToL2MessageAndIndex(messageKey: Fr): Promise<L1ToL2MessageAndIndex>;

  /**
   * Returns a sibling path for a leaf in the committed l1 to l2 data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  getL1ToL2MessageSiblingPath(
    blockNumber: BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>>;

  /**
   * Returns a sibling path for a leaf in the committed historic blocks tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  getArchiveSiblingPath(blockNumber: BlockNumber, leafIndex: bigint): Promise<SiblingPath<typeof ARCHIVE_HEIGHT>>;

  /**
   * Returns a sibling path for a leaf in the committed public data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  getPublicDataSiblingPath(
    blockNumber: BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof PUBLIC_DATA_TREE_HEIGHT>>;

  /**
   * Returns a nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the data.
   * @param nullifier - Nullifier we try to find witness for.
   * @returns The nullifier membership witness (if found).
   */
  getNullifierMembershipWitness(
    blockNumber: BlockNumber,
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined>;

  /**
   * Returns a low nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the data.
   * @param nullifier - Nullifier we try to find the low nullifier witness for.
   * @returns The low nullifier membership witness (if found).
   * @remarks Low nullifier witness can be used to perform a nullifier non-inclusion proof by leveraging the "linked
   * list structure" of leaves and proving that a lower nullifier is pointing to a bigger next value than the nullifier
   * we are trying to prove non-inclusion for.
   */
  getLowNullifierMembershipWitness(
    blockNumber: BlockNumber,
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined>;

  /**
   * Returns a public data tree witness for a given leaf slot at a given block.
   * @param blockNumber - The block number at which to get the data.
   * @param leafSlot - The leaf slot we try to find the witness for.
   * @returns The public data witness (if found).
   * @remarks The witness can be used to compute the current value of the public data tree leaf. If the low leaf preimage corresponds to an
   * "in range" slot, means that the slot doesn't exist and the value is 0. If the low leaf preimage corresponds to the exact slot, the current value
   * is contained in the leaf preimage.
   */
  getPublicDataTreeWitness(blockNumber: BlockNumber, leafSlot: Fr): Promise<PublicDataWitness | undefined>;

  /**
   * Get a block specified by its number.
   * @param number - The block number being requested.
   * @returns The requested block.
   */
  getBlock(number: number): Promise<L2Block | undefined>;

  /**
   * Fetches the current block number.
   * @returns The block number.
   */
  getBlockNumber(): Promise<number>;
  /**
   * Method to determine if the node is ready to accept transactions.
   * @returns - Flag indicating the readiness for tx submission.
   */
  isReady(): Promise<boolean>;

  /**
   * Method to request blocks. Will attempt to return all requested blocks but will return only those available.
   * @param from - The start of the range of blocks to return.
   * @param limit - The maximum number of blocks to return.
   * @returns The blocks requested.
   */
  getBlocks(from: number, limit: number): Promise<L2Block[]>;

  /**
   * Method to fetch the version of the rollup the node is connected to.
   * @returns The rollup version.
   */
  getVersion(): Promise<number>;

  /**
   * Method to fetch the chain id of the base-layer for the rollup.
   * @returns The chain id.
   */
  getChainId(): Promise<number>;

  /**
   * Method to fetch the currently deployed l1 contract addresses.
   * @returns The deployed contract addresses.
   */
  getL1ContractAddresses(): Promise<L1ContractAddresses>;

  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined>;

  /**
   * Lookup the contract data for this contract.
   * Contains the ethereum portal address .
   * @param contractAddress - The contract data address.
   * @returns The contract's address & portal address.
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The maximum number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  getLogs(from: number, limit: number, logType: LogType): Promise<L2BlockL2Logs[]>;

  /**
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   */
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse>;

  /**
   * Method to submit a transaction to the p2p pool.
   * @param tx - The transaction to be submitted.
   * @returns Nothing.
   */
  sendTx(tx: Tx): Promise<void>;

  /**
   * Get a settled tx.
   * @param txHash - The txHash being requested.
   * @returns The tx requested.
   */
  getTx(txHash: TxHash): Promise<L2Tx | undefined>;

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
   * Gets the storage value at the given contract storage slot.
   *
   * @remarks The storage slot here refers to the slot as it is defined in Noir not the index in the merkle tree.
   * Aztec's version of `eth_getStorageAt`.
   *
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @returns Storage value at the given contract slot.
   */
  getPublicStorageAt(contract: AztecAddress, slot: Fr): Promise<Fr>;

  /**
   * Returns the currently committed block header.
   * @returns The current committed block header.
   */
  getHeader(): Promise<Header>;

  /**
   * Simulates the public part of a transaction with the current state.
   * This currently just checks that the transaction execution succeeds.
   * @param tx - The transaction to simulate.
   **/
  simulatePublicCalls(tx: Tx): Promise<void>;

  /**
   * Updates the configuration of this node.
   * @param config - Updated configuration to be merged with the current one.
   */
  setConfig(config: Partial<SequencerConfig>): Promise<void>;

  /**
   * Returns a registered contract class given its id.
   * @param id - Id of the contract class.
   */
  getContractClass(id: Fr): Promise<ContractClassPublic | undefined>;

  /**
   * Returns a publicly deployed contract instance given its address.
   * @param address - Address of the deployed contract.
   */
  getContract(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined>;
}
