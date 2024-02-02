import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, from2Fields, serializeToBuffer, to2Fields } from '@aztec/foundation/serialize';

import { GeneratorIndex, HEADER_LENGTH } from '../constants.gen.js';
import { GlobalVariables } from './global_variables.js';
import { AppendOnlyTreeSnapshot } from './rollup/append_only_tree_snapshot.js';
import { StateReference } from './state_reference.js';

export const NUM_BYTES_PER_SHA256 = 32;

/** A header of an L2 block. */
export class Header {
  constructor(
    /** Snapshot of archive before the block is applied. */
    public lastArchive: AppendOnlyTreeSnapshot,
    /** Hash of the body of an L2 block. */
    public bodyHash: Buffer,
    /** State reference. */
    public state: StateReference,
    /** Global variables of an L2 block. */
    public globalVariables: GlobalVariables,
  ) {
    if (bodyHash.length !== 32) {
      throw new Error('Body hash buffer must be 32 bytes');
    }
  }

  toBuffer() {
    // Note: The order here must match the order in the HeaderLib solidity library.
    return serializeToBuffer(this.lastArchive, this.bodyHash, this.state, this.globalVariables);
  }

  toFields(): Fr[] {
    // Note: The order here must match the order in header.nr
    const serialized = [
      ...this.lastArchive.toFields(),
      ...to2Fields(this.bodyHash),
      ...this.state.toFields(),
      ...this.globalVariables.toFields(),
    ];
    if (serialized.length !== HEADER_LENGTH) {
      throw new Error(`Expected header to have ${HEADER_LENGTH} fields, but it has ${serialized.length} fields`);
    }
    return serialized;
  }

  static fromBuffer(buffer: Buffer | BufferReader): Header {
    const reader = BufferReader.asReader(buffer);

    return new Header(
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readBytes(NUM_BYTES_PER_SHA256),
      reader.readObject(StateReference),
      reader.readObject(GlobalVariables),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): Header {
    const reader = FieldReader.asReader(fields);

    const lastArchive = new AppendOnlyTreeSnapshot(reader.readField(), Number(reader.readField().toBigInt()));
    const bodyHash = from2Fields(reader.readField(), reader.readField());
    const state = StateReference.fromFields(reader);
    const globalVariables = GlobalVariables.fromFields(reader);

    return new Header(lastArchive, bodyHash, state, globalVariables);
  }

  static empty(): Header {
    return new Header(
      AppendOnlyTreeSnapshot.zero(),
      Buffer.alloc(NUM_BYTES_PER_SHA256),
      StateReference.empty(),
      GlobalVariables.empty(),
    );
  }

  isEmpty(): boolean {
    return (
      this.lastArchive.isZero() &&
      this.bodyHash.equals(Buffer.alloc(NUM_BYTES_PER_SHA256)) &&
      this.state.isEmpty() &&
      this.globalVariables.isEmpty()
    );
  }

  /**
   * Serializes this instance into a string.
   * @returns Encoded string.
   */
  public toString(): string {
    return this.toBuffer().toString('hex');
  }

  static fromString(str: string): Header {
    const buffer = Buffer.from(str.replace(/^0x/i, ''), 'hex');
    return Header.fromBuffer(buffer);
  }

  hash(): Fr {
    return Fr.fromBuffer(
      pedersenHash(
        this.toFields().map(f => f.toBuffer()),
        GeneratorIndex.BLOCK_HASH,
      ),
    );
  }
}
