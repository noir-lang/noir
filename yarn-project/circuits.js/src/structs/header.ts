import { pedersenHash } from '@aztec/foundation/crypto';
import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { GeneratorIndex, HEADER_LENGTH } from '../constants.gen.js';
import { ContentCommitment } from './content_commitment.js';
import { GlobalVariables } from './global_variables.js';
import { AppendOnlyTreeSnapshot } from './rollup/append_only_tree_snapshot.js';
import { StateReference } from './state_reference.js';

/** A header of an L2 block. */
export class Header {
  constructor(
    /** Snapshot of archive before the block is applied. */
    public lastArchive: AppendOnlyTreeSnapshot,
    /** Hash of the body of an L2 block. */
    public contentCommitment: ContentCommitment,
    /** State reference. */
    public state: StateReference,
    /** Global variables of an L2 block. */
    public globalVariables: GlobalVariables,
  ) {}

  toBuffer() {
    // Note: The order here must match the order in the HeaderLib solidity library.
    return serializeToBuffer(this.lastArchive, this.contentCommitment, this.state, this.globalVariables);
  }

  toFields(): Fr[] {
    // Note: The order here must match the order in header.nr
    const fields = [
      ...this.lastArchive.toFields(),
      ...this.contentCommitment.toFields(),
      ...this.state.toFields(),
      ...this.globalVariables.toFields(),
    ];
    if (fields.length !== HEADER_LENGTH) {
      throw new Error(`Invalid number of fields for Header. Expected ${HEADER_LENGTH}, got ${fields.length}`);
    }
    return fields;
  }

  static fromBuffer(buffer: Buffer | BufferReader): Header {
    const reader = BufferReader.asReader(buffer);

    return new Header(
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(ContentCommitment),
      reader.readObject(StateReference),
      reader.readObject(GlobalVariables),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): Header {
    const reader = FieldReader.asReader(fields);

    const lastArchive = new AppendOnlyTreeSnapshot(reader.readField(), Number(reader.readField().toBigInt()));
    const contentCommitment = ContentCommitment.fromFields(reader);
    const state = StateReference.fromFields(reader);
    const globalVariables = GlobalVariables.fromFields(reader);

    return new Header(lastArchive, contentCommitment, state, globalVariables);
  }

  static empty(): Header {
    return new Header(
      AppendOnlyTreeSnapshot.zero(),
      ContentCommitment.empty(),
      StateReference.empty(),
      GlobalVariables.empty(),
    );
  }

  isEmpty(): boolean {
    return (
      this.lastArchive.isZero() &&
      this.contentCommitment.isEmpty() &&
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
    return pedersenHash(this.toFields(), GeneratorIndex.BLOCK_HASH);
  }
}
