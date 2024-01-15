import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

/**
 * Global variables of the L2 block.
 */
export class GlobalVariables {
  constructor(
    /**
     * ChainId for the L2 block.
     */
    public chainId: Fr,
    /**
     * version for the L2 block.
     */
    public version: Fr,
    /**
     * Block number of the L2 block.
     */
    public blockNumber: Fr,
    /**
     * Timestamp of the L2 block.
     */
    public timestamp: Fr,
  ) {}

  static from(fields: FieldsOf<GlobalVariables>): GlobalVariables {
    return new GlobalVariables(...GlobalVariables.getFields(fields));
  }

  static empty(): GlobalVariables {
    return new GlobalVariables(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }

  static fromBuffer(buffer: Buffer | BufferReader): GlobalVariables {
    const reader = BufferReader.asReader(buffer);
    return new GlobalVariables(
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  static fromJSON(obj: any): GlobalVariables {
    return new GlobalVariables(
      Fr.fromString(obj.chainId),
      Fr.fromString(obj.version),
      Fr.fromString(obj.blockNumber),
      Fr.fromString(obj.timestamp),
    );
  }

  static getFields(fields: FieldsOf<GlobalVariables>) {
    // Note: The order here must match the order in the HeaderDecoder solidity library.
    return [fields.chainId, fields.version, fields.blockNumber, fields.timestamp] as const;
  }

  toBuffer() {
    return serializeToBuffer(...GlobalVariables.getFields(this));
  }

  toJSON() {
    return {
      chainId: this.chainId.toString(),
      version: this.version.toString(),
      blockNumber: this.blockNumber.toString(),
      timestamp: this.timestamp.toString(),
    };
  }
}
