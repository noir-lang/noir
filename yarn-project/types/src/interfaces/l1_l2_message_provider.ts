import { Fr, L1_TO_L2_MSG_TREE_HEIGHT } from '@aztec/circuits.js';

import { L1ToL2MessageAndIndex } from '../l1_to_l2_message.js';
import { SiblingPath } from '../sibling_path.js';

/**
 * Interface for providing information about L1 to L2 messages within the tree.
 */
export interface L1ToL2MessageProvider {
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
  getL1ToL2MessagesTreePath(leafIndex: bigint): Promise<SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>>;
}
