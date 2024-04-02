import { Note, TxHash } from '@aztec/circuit-types';
import { AztecAddress, Fr, Point, type PublicKey, Vector } from '@aztec/circuits.js';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

/**
 * A note that is intended for us, but we cannot decode it yet because the contract is not yet in our database.
 *
 * So keep the state that we need to decode it later.
 */
export class DeferredNoteDao {
  constructor(
    /** The public key associated with this note */
    public publicKey: PublicKey,
    /** The note as emitted from the Noir contract. */
    public note: Note,
    /** The contract address this note is created in. */
    public contractAddress: AztecAddress,
    /** The specific storage location of the note on the contract. */
    public storageSlot: Fr,
    /** The type ID of the note on the contract. */
    public noteTypeId: Fr,
    /** The hash of the tx the note was created in. Equal to the first nullifier */
    public txHash: TxHash,
    /** New note hashes in this transaction, one of which belongs to this note */
    public newNoteHashes: Fr[],
    /** The next available leaf index for the note hash tree for this transaction */
    public dataStartIndexForTx: number,
  ) {}

  toBuffer(): Buffer {
    return serializeToBuffer(
      this.publicKey,
      this.note,
      this.contractAddress,
      this.storageSlot,
      this.noteTypeId,
      this.txHash,
      new Vector(this.newNoteHashes),
      this.dataStartIndexForTx,
    );
  }
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new DeferredNoteDao(
      reader.readObject(Point),
      reader.readObject(Note),
      reader.readObject(AztecAddress),
      reader.readObject(Fr),
      reader.readObject(Fr),
      reader.readObject(TxHash),
      reader.readVector(Fr),
      reader.readNumber(),
    );
  }
}
