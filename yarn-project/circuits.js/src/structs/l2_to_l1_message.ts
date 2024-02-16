import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { L2_TO_L1_MESSAGE_LENGTH } from '../constants.gen.js';

export class L2ToL1Message {
  constructor(public recipient: EthAddress, public content: Fr) {}

  /**
   * Creates an empty L2ToL1Message with default values.
   * @returns An instance of L2ToL1Message with empty fields.
   */
  static empty(): L2ToL1Message {
    return new L2ToL1Message(EthAddress.ZERO, Fr.zero());
  }

  /**
   * Checks if another L2ToL1Message is equal to this instance.
   * @param other Another L2ToL1Message instance to compare with.
   * @returns True if both recipient and content are equal.
   */
  equals(other: L2ToL1Message): boolean {
    return this.recipient.equals(other.recipient) && this.content.equals(other.content);
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(this.recipient, this.content);
  }

  /**
   * Serializes the L2ToL1Message into an array of fields.
   * @returns An array of fields representing the serialized message.
   */
  toFields(): Fr[] {
    const fields = [this.recipient.toField(), this.content];
    if (fields.length !== L2_TO_L1_MESSAGE_LENGTH) {
      throw new Error(
        `Invalid number of fields for L2ToL1Message. Expected ${L2_TO_L1_MESSAGE_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  /**
   * Deserializes an array of fields into an L2ToL1Message instance.
   * @param fields An array of fields to deserialize from.
   * @returns An instance of L2ToL1Message.
   */
  static fromFields(fields: Fr[] | FieldReader): L2ToL1Message {
    const reader = FieldReader.asReader(fields);
    return new L2ToL1Message(reader.readObject(EthAddress), reader.readField());
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of L2ToL1Message.
   */
  static fromBuffer(buffer: Buffer | BufferReader): L2ToL1Message {
    const reader = BufferReader.asReader(buffer);
    return new L2ToL1Message(reader.readObject(EthAddress), reader.readObject(Fr));
  }

  /**
   * Convenience method to check if the message is empty.
   * @returns True if both recipient and content are zero.
   */
  isEmpty(): boolean {
    return this.recipient.isZero() && this.content.isZero();
  }
}
