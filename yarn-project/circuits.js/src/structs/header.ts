import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

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
    /** Total fees in the block, computed by the root rollup circuit */
    public totalFees: Fr,
  ) {}

  static getFields(fields: FieldsOf<Header>) {
    // Note: The order here must match the order in the HeaderLib solidity library.
    return [
      fields.lastArchive,
      fields.contentCommitment,
      fields.state,
      fields.globalVariables,
      fields.totalFees,
    ] as const;
  }

  static from(fields: FieldsOf<Header>) {
    return new Header(...Header.getFields(fields));
  }

  getSize() {
    return (
      this.lastArchive.getSize() +
      this.contentCommitment.getSize() +
      this.state.getSize() +
      this.globalVariables.getSize() +
      this.totalFees.size
    );
  }

  toBuffer() {
    return serializeToBuffer(...Header.getFields(this));
  }

  toFields(): Fr[] {
    const fields = serializeToFields(...Header.getFields(this));
    if (fields.length !== HEADER_LENGTH) {
      throw new Error(`Invalid number of fields for Header. Expected ${HEADER_LENGTH}, got ${fields.length}`);
    }
    return fields;
  }

  clone(): Header {
    return Header.fromBuffer(this.toBuffer());
  }

  static fromBuffer(buffer: Buffer | BufferReader): Header {
    const reader = BufferReader.asReader(buffer);

    return new Header(
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(ContentCommitment),
      reader.readObject(StateReference),
      reader.readObject(GlobalVariables),
      reader.readObject(Fr),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): Header {
    const reader = FieldReader.asReader(fields);

    return new Header(
      AppendOnlyTreeSnapshot.fromFields(reader),
      ContentCommitment.fromFields(reader),
      StateReference.fromFields(reader),
      GlobalVariables.fromFields(reader),
      reader.readField(),
    );
  }

  static empty(fields: Partial<FieldsOf<Header>> = {}): Header {
    return Header.from({
      lastArchive: AppendOnlyTreeSnapshot.zero(),
      contentCommitment: ContentCommitment.empty(),
      state: StateReference.empty(),
      globalVariables: GlobalVariables.empty(),
      totalFees: Fr.ZERO,
      ...fields,
    });
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
