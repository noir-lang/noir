import {
  CONTRACT_TREE_HEIGHT,
  Fr,
  HISTORIC_BLOCKS_TREE_HEIGHT,
  L1_TO_L2_MSG_TREE_HEIGHT,
  NOTE_HASH_TREE_HEIGHT,
  NULLIFIER_TREE_HEIGHT,
  PUBLIC_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';

import { L1ToL2MessageAndIndex } from '../l1_to_l2_message.js';
import { L2Block } from '../l2_block.js';
import { MerkleTreeId } from '../merkle_tree_id.js';
import { SiblingPath } from '../sibling_path.js';
import { NullifierMembershipWitness } from './nullifier_witness.js';

/**
 * Interface providing methods for retrieving information about content of the state trees.
 */
export interface StateInfoProvider {
  /**
   * Find the index of the given leaf in the given tree.
   * @param treeId - The tree to search in.
   * @param leafValue - The value to search for
   * @returns The index of the given leaf in the given tree or undefined if not found.
   */
  findLeafIndex(treeId: MerkleTreeId, leafValue: Fr): Promise<bigint | undefined>;

  /**
   * Returns a sibling path for the given index in the contract tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
   */
  getContractSiblingPath(leafIndex: bigint): Promise<SiblingPath<typeof CONTRACT_TREE_HEIGHT>>;

  /**
   * Returns a sibling path for the given index in the nullifier tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
   */
  getNullifierTreeSiblingPath(leafIndex: bigint): Promise<SiblingPath<typeof NULLIFIER_TREE_HEIGHT>>;

  /**
   * Returns a sibling path for the given index in the note hash tree.
   * @param leafIndex - The index of the leaf for which the sibling path is required.
   * @returns The sibling path for the leaf index.
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
   */
  getNoteHashSiblingPath(leafIndex: bigint): Promise<SiblingPath<typeof NOTE_HASH_TREE_HEIGHT>>;

  /**
   * Gets a confirmed/consumed L1 to L2 message for the given message key (throws if not found).
   * and its index in the merkle tree
   * @param messageKey - The message key.
   * @returns The map containing the message and index.
   */
  getL1ToL2MessageAndIndex(messageKey: Fr): Promise<L1ToL2MessageAndIndex>;

  /**
   * Returns a sibling path for a leaf in the committed l1 to l2 data tree.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
   */
  getL1ToL2MessageSiblingPath(leafIndex: bigint): Promise<SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>>;

  /**
   * Returns a sibling path for a leaf in the committed historic blocks tree.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
   */
  getHistoricBlocksTreeSiblingPath(leafIndex: bigint): Promise<SiblingPath<typeof HISTORIC_BLOCKS_TREE_HEIGHT>>;

  /**
   * Returns a sibling path for a leaf in the committed public data tree.
   * @param leafIndex - Index of the leaf in the tree.
   * @returns The sibling path.
   * TODO: https://github.com/AztecProtocol/aztec-packages/issues/3414
   */
  getPublicDataTreeSiblingPath(leafIndex: bigint): Promise<SiblingPath<typeof PUBLIC_DATA_TREE_HEIGHT>>;

  /**
   * Returns a nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param nullifier - Nullifier we try to find witness for.
   * @returns The nullifier membership witness (if found).
   */
  getNullifierMembershipWitness(blockNumber: number, nullifier: Fr): Promise<NullifierMembershipWitness | undefined>;

  /**
   * Returns a low nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param nullifier - Nullifier we try to find the low nullifier witness for.
   * @returns The low nullifier membership witness (if found).
   * @remarks Low nullifier witness can be used to perform a nullifier non-inclusion proof by leveraging the "linked
   * list structure" of leaves and proving that a lower nullifier is pointing to a bigger next value than the nullifier
   * we are trying to prove non-inclusion for.
   */
  getLowNullifierMembershipWitness(blockNumber: number, nullifier: Fr): Promise<NullifierMembershipWitness | undefined>;

  /**
   * Get a block specified by its number.
   * @param number - The block number being requested.
   * @returns The requested block.
   */
  getBlock(number: number): Promise<L2Block | undefined>;
}
