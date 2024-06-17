import { Note, TxHash } from '@aztec/circuit-types';
import { AztecAddress, Fr, Point, type PublicKey } from '@aztec/circuits.js';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

/**
 * A note with contextual data which was decrypted as outgoing.
 */
export class OutgoingNoteDao {
  constructor(
    /** The note as emitted from the Noir contract. */
    public note: Note,
    /** The contract address this note is created in. */
    public contractAddress: AztecAddress,
    /** The specific storage location of the note on the contract. */
    public storageSlot: Fr,
    /** The note type identifier for the contract. */
    public noteTypeId: Fr,
    /** The hash of the tx the note was created in. */
    public txHash: TxHash,
    /** The nonce of the note. */
    public nonce: Fr,
    /**
     * Inner note hash of the note. This is customizable by the app circuit.
     * We can use this value to compute siloedNoteHash and uniqueSiloedNoteHash.
     */
    public innerNoteHash: Fr,
    /** The location of the relevant note in the note hash tree. */
    public index: bigint,
    /** The public key with which the note was encrypted. */
    public ovpkM: PublicKey,
  ) {}

  toBuffer(): Buffer {
    return serializeToBuffer([
      this.note,
      this.contractAddress,
      this.storageSlot,
      this.noteTypeId,
      this.txHash.buffer,
      this.nonce,
      this.innerNoteHash,
      this.index,
      this.ovpkM,
    ]);
  }
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);

    const note = Note.fromBuffer(reader);
    const contractAddress = AztecAddress.fromBuffer(reader);
    const storageSlot = Fr.fromBuffer(reader);
    const noteTypeId = Fr.fromBuffer(reader);
    const txHash = new TxHash(reader.readBytes(TxHash.SIZE));
    const nonce = Fr.fromBuffer(reader);
    const innerNoteHash = Fr.fromBuffer(reader);
    const index = toBigIntBE(reader.readBytes(32));
    const publicKey = Point.fromBuffer(reader);

    return new OutgoingNoteDao(
      note,
      contractAddress,
      storageSlot,
      noteTypeId,
      txHash,
      nonce,
      innerNoteHash,
      index,
      publicKey,
    );
  }

  toString() {
    return '0x' + this.toBuffer().toString('hex');
  }

  static fromString(str: string) {
    const hex = str.replace(/^0x/, '');
    return OutgoingNoteDao.fromBuffer(Buffer.from(hex, 'hex'));
  }

  /**
   * Returns the size in bytes of the Note Dao.
   * @returns - Its size in bytes.
   */
  public getSize() {
    const noteSize = 4 + this.note.items.length * Fr.SIZE_IN_BYTES;
    return noteSize + AztecAddress.SIZE_IN_BYTES + Fr.SIZE_IN_BYTES * 2 + TxHash.SIZE + Point.SIZE_IN_BYTES;
  }
}
