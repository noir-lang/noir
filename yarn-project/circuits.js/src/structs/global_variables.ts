import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import { AztecAddress, EthAddress } from './index.js';

/**
 * Global variables of the L2 block.
 */
export class GlobalVariables {
  constructor(
    /** ChainId for the L2 block. */
    public chainId: Fr,
    /** Version for the L2 block. */
    public version: Fr,
    /** Block number of the L2 block. */
    public blockNumber: Fr,
    /** Timestamp of the L2 block. */
    public timestamp: Fr,
    /** Recipient of block reward. */
    public coinbase: EthAddress,
    /** Address to receive fees. */
    public feeRecipient: AztecAddress,
  ) {}

  static from(fields: FieldsOf<GlobalVariables>): GlobalVariables {
    return new GlobalVariables(...GlobalVariables.getFields(fields));
  }

  static empty(): GlobalVariables {
    return new GlobalVariables(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, EthAddress.ZERO, AztecAddress.ZERO);
  }

  static fromBuffer(buffer: Buffer | BufferReader): GlobalVariables {
    const reader = BufferReader.asReader(buffer);
    return new GlobalVariables(
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readObject(EthAddress),
      reader.readObject(AztecAddress),
    );
  }

  static fromJSON(obj: any): GlobalVariables {
    return new GlobalVariables(
      Fr.fromString(obj.chainId),
      Fr.fromString(obj.version),
      Fr.fromString(obj.blockNumber),
      Fr.fromString(obj.timestamp),
      EthAddress.fromString(obj.coinbase),
      AztecAddress.fromString(obj.feeRecipient),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): GlobalVariables {
    const reader = FieldReader.asReader(fields);

    return new GlobalVariables(
      reader.readField(),
      reader.readField(),
      reader.readField(),
      reader.readField(),
      EthAddress.fromField(reader.readField()),
      AztecAddress.fromField(reader.readField()),
    );
  }

  static getFields(fields: FieldsOf<GlobalVariables>) {
    // Note: The order here must match the order in the HeaderLib solidity library.
    return [
      fields.chainId,
      fields.version,
      fields.blockNumber,
      fields.timestamp,
      fields.coinbase,
      fields.feeRecipient,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...GlobalVariables.getFields(this));
  }

  toFields() {
    return serializeToFields(...GlobalVariables.getFields(this));
  }

  toJSON() {
    return {
      chainId: this.chainId.toString(),
      version: this.version.toString(),
      blockNumber: this.blockNumber.toString(),
      timestamp: this.timestamp.toString(),
      coinbase: this.coinbase.toString(),
      feeRecipient: this.feeRecipient.toString(),
    };
  }

  isEmpty(): boolean {
    return (
      this.chainId.isZero() &&
      this.version.isZero() &&
      this.blockNumber.isZero() &&
      this.timestamp.isZero() &&
      this.coinbase.isZero() &&
      this.feeRecipient.isZero()
    );
  }
}
