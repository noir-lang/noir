import { type AztecAddress, type Fr } from '@aztec/circuits.js';

import { type TxHash } from '../tx/tx_hash.js';
import { type NoteStatus } from './note_status.js';

/**
 * A filter used to fetch incoming notes.
 * @remarks This filter is applied as an intersection of all its params.
 */
export type IncomingNotesFilter = {
  /** Hash of a transaction from which to fetch the notes. */
  txHash?: TxHash;
  /** The contract address the note belongs to. */
  contractAddress?: AztecAddress;
  /** The specific storage location of the note on the contract. */
  storageSlot?: Fr;
  /** The owner of the note (whose public key was used to encrypt the note). */
  owner?: AztecAddress;
  /** The status of the note. Defaults to 'ACTIVE'. */
  status?: NoteStatus;
  /** The siloed nullifier for the note. */
  siloedNullifier?: Fr;
  /** The scopes in which to get incoming notes from. This defaults to all scopes. */
  scopes?: AztecAddress[];
};
