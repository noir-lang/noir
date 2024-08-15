import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH } from '../constants.gen.js';
import { CallContext } from './call_context.js';
import { Gas } from './gas.js';
import { RevertCode } from './revert_code.js';

/**
 * Compressed call stack item on a public call.
 */
export class PublicCallStackItemCompressed {
  constructor(
    public contractAddress: AztecAddress,
    public callContext: CallContext,
    public argsHash: Fr,
    public returnsHash: Fr,
    public revertCode: RevertCode,
    /** How much gas was available for execution. */
    public startGasLeft: Gas,
    /** How much gas was left after execution. */
    public endGasLeft: Gas,
  ) {}

  static getFields(fields: FieldsOf<PublicCallStackItemCompressed>) {
    return [
      fields.contractAddress,
      fields.callContext,
      fields.argsHash,
      fields.returnsHash,
      fields.revertCode,
      fields.startGasLeft,
      fields.endGasLeft,
    ] as const;
  }

  toFields(): Fr[] {
    const fields = serializeToFields(...PublicCallStackItemCompressed.getFields(this));
    if (fields.length !== PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH) {
      throw new Error(
        `Invalid number of fields for PublicCallStackItemCompressed. Expected ${PUBLIC_CALL_STACK_ITEM_COMPRESSED_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  toBuffer() {
    return serializeToBuffer(...PublicCallStackItemCompressed.getFields(this));
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PublicCallStackItemCompressed {
    const reader = BufferReader.asReader(buffer);
    return new PublicCallStackItemCompressed(
      reader.readObject(AztecAddress),
      reader.readObject(CallContext),
      reader.readObject(Fr),
      reader.readObject(Fr),
      reader.readObject(RevertCode),
      reader.readObject(Gas),
      reader.readObject(Gas),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): PublicCallStackItemCompressed {
    const reader = FieldReader.asReader(fields);

    return new PublicCallStackItemCompressed(
      AztecAddress.fromFields(reader),
      CallContext.fromFields(reader),
      reader.readField(),
      reader.readField(),
      RevertCode.fromFields(reader),
      Gas.fromFields(reader),
      Gas.fromFields(reader),
    );
  }

  /**
   * Returns a new instance of PublicCallStackItem with zero contract address, function data and public inputs.
   * @returns A new instance of PublicCallStackItem with zero contract address, function data and public inputs.
   */
  public static empty(): PublicCallStackItemCompressed {
    return new PublicCallStackItemCompressed(
      AztecAddress.ZERO,
      CallContext.empty(),
      Fr.ZERO,
      Fr.ZERO,
      RevertCode.OK,
      Gas.empty(),
      Gas.empty(),
    );
  }

  isEmpty() {
    return (
      this.contractAddress.isZero() &&
      this.callContext.isEmpty() &&
      this.argsHash.isEmpty() &&
      this.returnsHash.isEmpty() &&
      this.revertCode.equals(RevertCode.OK) &&
      this.startGasLeft.isEmpty() &&
      this.endGasLeft.isEmpty()
    );
  }
}
