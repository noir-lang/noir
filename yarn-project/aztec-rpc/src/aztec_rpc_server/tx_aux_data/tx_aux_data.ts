import { Fr } from '@aztec/foundation/fields';
import { AztecAddress } from '@aztec/circuits.js';
import { BufferReader } from '@aztec/foundation/serialize';
import { NotePreimage } from './note_preimage.js';
import { serializeToBuffer } from '@aztec/circuits.js/utils';
import { decryptBuffer, encryptBuffer } from './encrypt_buffer.js';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';

export class TxAuxData {
  constructor(public notePreImage: NotePreimage, public contractAddress: AztecAddress, public storageSlot: Fr) {}

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new TxAuxData(reader.readObject(NotePreimage), reader.readObject(AztecAddress), reader.readFr());
  }

  toBuffer() {
    return serializeToBuffer([this.notePreImage, this.contractAddress, this.storageSlot]);
  }

  public toEncryptedBuffer(ownerPubKey: AztecAddress, ephPrivKey: Buffer, grumpkin: Grumpkin) {
    return encryptBuffer(this.toBuffer(), ownerPubKey, ephPrivKey, grumpkin);
  }

  static fromEncryptedBuffer(data: Buffer, ownerPrivKey: Buffer, grumpkin: Grumpkin) {
    const buf = decryptBuffer(data, ownerPrivKey, grumpkin);
    if (!buf) {
      return;
    }
    return TxAuxData.fromBuffer(buf);
  }
}
