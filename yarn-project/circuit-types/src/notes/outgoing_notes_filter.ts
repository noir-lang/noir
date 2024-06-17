import { type AztecAddress, type Fr } from '@aztec/circuits.js';

import { type TxHash } from '../tx/tx_hash.js';

/**
 * A filter used to fetch outgoing notes.
 * @remarks This filter is applied as an intersection of all its params.
 */
export type OutgoingNotesFilter = {
  /** Hash of a transaction from which to fetch the notes. */
  txHash?: TxHash;
  /** The contract address the note belongs to. */
  contractAddress?: AztecAddress;
  /** The specific storage location of the note on the contract. */
  storageSlot?: Fr;
  /** The owner of the note (whose public key was used to encrypt the note). */
  owner?: AztecAddress;
};
