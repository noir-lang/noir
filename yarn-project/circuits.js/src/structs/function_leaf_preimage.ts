import { FunctionSelector } from '@aztec/foundation/abi';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import { FUNCTION_LEAF_PREIMAGE_LENGTH, GeneratorIndex } from '../constants.gen.js';

/**
 * A class representing the "preimage" of a function tree leaf.
 */
export class FunctionLeafPreimage {
  constructor(
    /**
     * Function selector.
     */
    public functionSelector: FunctionSelector,
    /**
     * Indicates whether the function is only callable by self or not.
     */
    public isInternal: boolean,
    /**
     * Indicates whether the function is private or public.
     */
    public isPrivate: boolean,
    /**
     * Verification key hash of the function.
     */
    public vkHash: Fr,
    /**
     * Hash of the ACIR of the function.
     */
    public acirHash: Fr,
  ) {}

  static getFields(fields: FieldsOf<FunctionLeafPreimage>) {
    return [fields.functionSelector, fields.isInternal, fields.isPrivate, fields.vkHash, fields.acirHash] as const;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(...FunctionLeafPreimage.getFields(this));
  }

  toFields(): Fr[] {
    const fields = serializeToFields(...FunctionLeafPreimage.getFields(this));
    if (fields.length !== FUNCTION_LEAF_PREIMAGE_LENGTH) {
      throw new Error(
        `Invalid number of fields for FunctionLeafPreimage. Expected ${FUNCTION_LEAF_PREIMAGE_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of FunctionLeafPreimage.
   */
  static fromBuffer(buffer: Buffer | BufferReader): FunctionLeafPreimage {
    const reader = BufferReader.asReader(buffer);
    return new FunctionLeafPreimage(
      reader.readObject(FunctionSelector),
      reader.readBoolean(),
      reader.readBoolean(),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  hash(): Fr {
    return pedersenHash(
      this.toFields().map(field => field.toBuffer()),
      GeneratorIndex.FUNCTION_LEAF,
    );
  }
}
