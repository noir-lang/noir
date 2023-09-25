import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { FieldsOf } from '../index.js';
import { serializeToBuffer } from '../utils/index.js';

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
    return new GlobalVariables(Fr.zero(), Fr.zero(), Fr.zero(), Fr.zero());
  }

  static fromBuffer(buffer: Buffer | BufferReader): GlobalVariables {
    const reader = BufferReader.asReader(buffer);
    return new GlobalVariables(reader.readFr(), reader.readFr(), reader.readFr(), reader.readFr());
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
