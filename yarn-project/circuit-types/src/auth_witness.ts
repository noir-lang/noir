import { Vector } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

/**
 * An authentication witness. Used to authorize an action by a user.
 */
export class AuthWitness {
  /** Authentication witness for the hash  */
  public readonly witness: Fr[];

  constructor(
    /** Hash of the request to authorize */
    public readonly requestHash: Fr,
    /** Authentication witness for the hash  */
    witness: (Fr | number)[],
  ) {
    this.witness = witness.map(x => new Fr(x));
  }

  toBuffer() {
    return serializeToBuffer(this.requestHash, new Vector(this.witness));
  }

  static fromBuffer(buffer: Buffer | BufferReader): AuthWitness {
    const reader = BufferReader.asReader(buffer);
    return new AuthWitness(Fr.fromBuffer(reader), reader.readVector(Fr));
  }

  toString() {
    return '0x' + this.toBuffer().toString('hex');
  }

  static fromString(str: string) {
    const hex = str.replace(/^0x/, '');
    return AuthWitness.fromBuffer(Buffer.from(hex, 'hex'));
  }
}
