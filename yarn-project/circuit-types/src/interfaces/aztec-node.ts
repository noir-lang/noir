import {
  type ARCHIVE_HEIGHT,
  type Header,
  type L1_TO_L2_MSG_TREE_HEIGHT,
  type NOTE_HASH_TREE_HEIGHT,
  type NULLIFIER_TREE_HEIGHT,
  type PUBLIC_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { type L1ContractAddresses } from '@aztec/ethereum';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { type Fr } from '@aztec/foundation/fields';
import {
  type ContractClassPublic,
  type ContractInstanceWithAddress,
  type ProtocolContractAddresses,
} from '@aztec/types/contracts';

import { type L2Block } from '../l2_block.js';
import {
  type FromLogType,
  type GetUnencryptedLogsResponse,
  type L2BlockL2Logs,
  type LogFilter,
  type LogType,
} from '../logs/index.js';
import { type MerkleTreeId } from '../merkle_tree_id.js';
import { type PublicDataWitness } from '../public_data_witness.js';
import { type SiblingPath } from '../sibling_path/index.js';
import { type PublicSimulationOutput, type Tx, type TxHash, type TxReceipt } from '../tx/index.js';
import { type TxEffect } from '../tx_effect.js';
import { type SequencerConfig } from './configs.js';
import { type L2BlockNumber } from './l2_block_number.js';
import { type NullifierMembershipWitness } from './nullifier_tree.js';
import { type ProverConfig } from './prover-client.js';

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
  findLeafIndex(blockNumber: L2BlockNumber, treeId: MerkleTreeId, leafValue: Fr): Promise<bigint | undefined>;

  /**
   * Returns a sibling path for the given index in the nullifier tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getNullifierSiblingPath(
    blockNumber: L2BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof NULLIFIER_TREE_HEIGHT>>;

  /**
   * Returns a sibling path for the given index in the note hash tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   */
  getNoteHashSiblingPath(
    blockNumber: L2BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof NOTE_HASH_TREE_HEIGHT>>;

  /**
   * Returns the index and a sibling path for a leaf in the committed l1 to l2 data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param l1ToL2Message - The l1ToL2Message to get the index / sibling path for.
   * @param startIndex - The index to start searching from.
   * @returns A tuple of the index and the sibling path of the L1ToL2Message (undefined if not found).
   */
  getL1ToL2MessageMembershipWitness(
    blockNumber: L2BlockNumber,
    l1ToL2Message: Fr,
    startIndex: bigint,
  ): Promise<[bigint, SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>] | undefined>;

  /**
   * Returns whether an L1 to L2 message is synced by archiver and if it's ready to be included in a block.
   * @param l1ToL2Message - The L1 to L2 message to check.
   * @param startL2BlockNumber - The block number after which we are interested in checking if the message was
   * included.
   * @remarks We pass in the minL2BlockNumber because there can be duplicate messages and the block number allow us
   * to skip the duplicates (we know after which block a given message is to be included).
   * @returns Whether the message is synced and ready to be included in a block.
   */
  isL1ToL2MessageSynced(l1ToL2Message: Fr, startL2BlockNumber: number): Promise<boolean>;

  /**
   * Returns a membership witness of an l2ToL1Message in an ephemeral l2 to l1 message tree.
   * @dev Membership witness is a consists of the index and the sibling path of the l2ToL1Message.
   * @remarks This tree is considered ephemeral because it is created on-demand by: taking all the l2ToL1 messages
   * in a single block, and then using them to make a variable depth append-only tree with these messages as leaves.
   * The tree is discarded immediately after calculating what we need from it.
   * @param blockNumber - The block number at which to get the data.
   * @param l2ToL1Message - The l2ToL1Message to get the membership witness for.
   * @returns A tuple of the index and the sibling path of the L2ToL1Message.
   */
  getL2ToL1MessageMembershipWitness(
    blockNumber: L2BlockNumber,
    l2ToL1Message: Fr,
  ): Promise<[bigint, SiblingPath<number>]>;

  /**
   * Returns a sibling path for a leaf in the committed historic blocks tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  getArchiveSiblingPath(blockNumber: L2BlockNumber, leafIndex: bigint): Promise<SiblingPath<typeof ARCHIVE_HEIGHT>>;

  /**
   * Returns a sibling path for a leaf in the committed public data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   */
  getPublicDataSiblingPath(
    blockNumber: L2BlockNumber,
    leafIndex: bigint,
  ): Promise<SiblingPath<typeof PUBLIC_DATA_TREE_HEIGHT>>;

  /**
   * Returns a nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the data.
   * @param nullifier - Nullifier we try to find witness for.
   * @returns The nullifier membership witness (if found).
   */
  getNullifierMembershipWitness(
    blockNumber: L2BlockNumber,
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
    blockNumber: L2BlockNumber,
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
  getPublicDataTreeWitness(blockNumber: L2BlockNumber, leafSlot: Fr): Promise<PublicDataWitness | undefined>;

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
   * Fetches the latest proven block number.
   * @returns The block number.
   */
  getProvenBlockNumber(): Promise<number>;

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
   * Method to fetch the version of the package.
   * @returns The node package version
   */
  getNodeVersion(): Promise<string>;

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
   * Method to fetch the protocol contract addresses.
   */
  getProtocolContractAddresses(): Promise<ProtocolContractAddresses>;

  /**
   * Method to add a contract artifact to the database.
   * @param aztecAddress
   * @param artifact
   */
  addContractArtifact(address: AztecAddress, artifact: ContractArtifact): Promise<void>;

  /**
   * Gets up to `limit` amount of logs starting from `from`.
   * @param from - Number of the L2 block to which corresponds the first logs to be returned.
   * @param limit - The maximum number of logs to return.
   * @param logType - Specifies whether to return encrypted or unencrypted logs.
   * @returns The requested logs.
   */
  getLogs<TLogType extends LogType>(
    from: number,
    limit: number,
    logType: TLogType,
  ): Promise<L2BlockL2Logs<FromLogType<TLogType>>[]>;

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
   * Fetches a transaction receipt for a given transaction hash. Returns a mined receipt if it was added
   * to the chain, a pending receipt if it's still in the mempool of the connected Aztec node, or a dropped
   * receipt if not found in the connected Aztec node.
   *
   * @param txHash - The transaction hash.
   * @returns A receipt of the transaction.
   */
  getTxReceipt(txHash: TxHash): Promise<TxReceipt>;

  /**
   * Get a tx effect.
   * @param txHash - The hash of a transaction which resulted in the returned tx effect.
   * @returns The requested tx effect.
   */
  getTxEffect(txHash: TxHash): Promise<TxEffect | undefined>;

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
  getTxByHash(txHash: TxHash): Promise<Tx | undefined>;

  /**
   * Gets the storage value at the given contract storage slot.
   *
   * @remarks The storage slot here refers to the slot as it is defined in Noir not the index in the merkle tree.
   * Aztec's version of `eth_getStorageAt`.
   *
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @param blockNumber - The block number at which to get the data or 'latest'.
   * @returns Storage value at the given contract slot.
   */
  getPublicStorageAt(contract: AztecAddress, slot: Fr, blockNumber: L2BlockNumber): Promise<Fr>;

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
  simulatePublicCalls(tx: Tx): Promise<PublicSimulationOutput>;

  /**
   * Updates the configuration of this node.
   * @param config - Updated configuration to be merged with the current one.
   */
  setConfig(config: Partial<SequencerConfig & ProverConfig>): Promise<void>;

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
