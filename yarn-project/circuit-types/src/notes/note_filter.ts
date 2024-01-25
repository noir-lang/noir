import { AztecAddress, Fr } from '@aztec/circuits.js';

import { TxHash } from '../index.js';

/**
 * A filter used to fetch Notes.
 * @remarks This filter is applied as an intersection of all it's params.
 */
export type NoteFilter = {
  /** Hash of a transaction from which to fetch the notes. */
  txHash?: TxHash;
  /** The contract address the note belongs to. */
  contractAddress?: AztecAddress;
  /** The specific storage location of the note on the contract. */
  storageSlot?: Fr;
  /** The owner of the note (whose public key was used to encrypt the note). */
  owner?: AztecAddress;
};

/**
 * The comparator to use to compare.
 */
export enum Comparator {
  EQ = 1,
  NEQ = 2,
  LT = 3,
  LTE = 4,
  GT = 5,
  GTE = 6,
}
