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
  siloedNullifier = Fr.random(),
  index = Fr.random().value,
  publicKey = Point.random(),
}: Partial<NoteSpendingInfoDao> = {}): NoteSpendingInfoDao => ({
  contractAddress,
  nonce,
  storageSlot,
  notePreimage,
  siloedNullifier,
  index,
  publicKey,
});
