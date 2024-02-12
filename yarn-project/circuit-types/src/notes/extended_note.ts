import { AztecAddress, Fr } from '@aztec/circuits.js';
import { BufferReader } from '@aztec/foundation/serialize';

import { Note } from '../logs/l1_note_payload/note.js';
import { TxHash } from '../tx/tx_hash.js';

/**
 * A note with contextual data.
 */
export class ExtendedNote {
  constructor(
    /** The note as emitted from the Noir contract. */
    public note: Note,
    /** The owner whose public key was used to encrypt the note. */
    public owner: AztecAddress,
    /** The contract address this note is created in. */
    public contractAddress: AztecAddress,
    /** The specific storage location of the note on the contract. */
    public storageSlot: Fr,
    /** The type identifier of the note on the contract. */
    public noteTypeId: Fr,
    /** The hash of the tx the note was created in. */
    public txHash: TxHash,
  ) {}

  toBuffer(): Buffer {
    return Buffer.concat([
      this.note.toBuffer(),
      this.owner.toBuffer(),
      this.contractAddress.toBuffer(),
      this.storageSlot.toBuffer(),
      this.noteTypeId.toBuffer(),
      this.txHash.buffer,
    ]);
  }
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);

    const note = Note.fromBuffer(reader);
    const owner = AztecAddress.fromBuffer(reader);
    const contractAddress = AztecAddress.fromBuffer(reader);
    const storageSlot = Fr.fromBuffer(reader);
    const noteTypeId = Fr.fromBuffer(reader);
    const txHash = new TxHash(reader.readBytes(TxHash.SIZE));

    return new this(note, owner, contractAddress, storageSlot, noteTypeId, txHash);
  }

  toString() {
    return '0x' + this.toBuffer().toString('hex');
  }

  static fromString(str: string) {
    const hex = str.replace(/^0x/, '');
    return ExtendedNote.fromBuffer(Buffer.from(hex, 'hex'));
  }
}
