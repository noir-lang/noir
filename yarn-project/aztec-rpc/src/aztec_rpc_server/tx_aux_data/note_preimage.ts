import { Fr } from '@aztec/foundation/fields';
import { Vector } from '@aztec/circuits.js';
import { BufferReader } from '@aztec/foundation/serialize';

export class NotePreimage extends Vector<Fr> {
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NotePreimage(reader.readVector(Fr));
  }

  static random() {
    const numItems = Math.floor(Math.random() * 10) + 1;
    const items = Array.from({ length: numItems }, () => Fr.random());
    return new NotePreimage(items);
  }
}
