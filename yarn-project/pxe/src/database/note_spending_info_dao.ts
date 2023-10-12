import { AztecAddress, Fr, PublicKey } from '@aztec/circuits.js';
import { Point } from '@aztec/foundation/fields';
import { NotePreimage } from '@aztec/types';

/**
 * Represents the data access object for auxiliary transaction data.
 * Contains properties from the decrypted note, computed properties, and information about
 * the public key used for encryption, as well as the location of the data in the tree.
 */
export interface NoteSpendingInfoDao {
  /**
   * The contract address this note is created in.
   */
  contractAddress: AztecAddress;
  /**
   * The nonce of the note.
   */
  nonce: Fr;
  /**
   * The specific storage location of the note on the contract.
   */
  storageSlot: Fr;
  /**
   * The preimage of the note, containing essential information about the note.
   */
  notePreimage: NotePreimage;
  /**
   * Inner note hash of the note. This is customizable by the app circuit.
   * We can use this value to compute siloedNoteHash and uniqueSiloedNoteHash.
   */
  innerNoteHash: Fr;
  /**
   * The nullifier of the note (siloed by contract address).
   */
  siloedNullifier: Fr;
  /**
   * The location of the relevant note in the private data tree.
   */
  index: bigint;
  /**
   * The public key that was used to encrypt the data.
   */
  publicKey: PublicKey;
}

export const createRandomNoteSpendingInfoDao = ({
  contractAddress = AztecAddress.random(),
  nonce = Fr.random(),
  storageSlot = Fr.random(),
  notePreimage = NotePreimage.random(),
  innerNoteHash = Fr.random(),
  siloedNullifier = Fr.random(),
  index = Fr.random().value,
  publicKey = Point.random(),
}: Partial<NoteSpendingInfoDao> = {}): NoteSpendingInfoDao => ({
  contractAddress,
  nonce,
  storageSlot,
  notePreimage,
  innerNoteHash,
  siloedNullifier,
  index,
  publicKey,
});

/**
 * Returns the size in bytes of a note spending info dao.
 * @param note - The note.
 * @returns - Its size in bytes.
 */
export function getNoteSpendingInfoDaoSize(note: NoteSpendingInfoDao) {
  // 7 fields + 1 bigint + 1 buffer size (4 bytes) + 1 buffer
  const indexSize = Math.ceil(Math.log2(Number(note.index)));
  return 7 * Fr.SIZE_IN_BYTES + indexSize + 4 + note.notePreimage.items.length * Fr.SIZE_IN_BYTES;
}
