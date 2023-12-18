import {
  ARCHIVE_HEIGHT,
  CONTRACT_TREE_HEIGHT,
  Fr,
  L1_TO_L2_MSG_TREE_HEIGHT,
  NOTE_HASH_TREE_HEIGHT,
  NULLIFIER_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';

import { L1ToL2MessageAndIndex } from '../l1_to_l2_message.js';
import { L2Block } from '../l2_block.js';
import { MerkleTreeId } from '../merkle_tree_id.js';
import { SiblingPath } from '../sibling_path.js';
import { NullifierMembershipWitness } from './nullifier_tree.js';
import { PublicDataWitness } from './public_data_tree.js';

/** Helper type for a specific L2 block number or the latest block number */
type BlockNumber = number | 'latest';

/**
 * Interface providing methods for retrieving information about content of the state trees.
 */
export interface StateInfoProvider {
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
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
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
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
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
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
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
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
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
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
   */
  getArchiveSiblingPath(blockNumber: BlockNumber, leafIndex: bigint): Promise<SiblingPath<typeof ARCHIVE_HEIGHT>>;

  /**
   * Returns a sibling path for a leaf in the committed public data tree.
   * @param blockNumber - The block number at which to get the data.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
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
}
