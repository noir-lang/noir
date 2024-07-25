import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { inspect } from 'util';

import { PublicCallStackItemCompressed } from './public_call_stack_item_compressed.js';

/**
 * Represents a request to call a public function.
 */
export class PublicCallRequest {
  constructor(
    public item: PublicCallStackItemCompressed,
    /**
     * The counter for this call request.
     */
    public counter: number,
  ) {}

  getSize() {
    return this.isEmpty() ? 0 : this.toBuffer().length;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.item, this.counter);
  }

  /**
   * Deserialize this from a buffer.
   * @param buffer - The bufferable type from which to deserialize.
   * @returns The deserialized instance of PublicCallRequest.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicCallRequest(PublicCallStackItemCompressed.fromBuffer(reader), reader.readNumber());
  }

  /**
   * Create PublicCallRequest from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A PublicCallRequest object.
   */
  static from(fields: FieldsOf<PublicCallRequest>): PublicCallRequest {
    return new PublicCallRequest(...PublicCallRequest.getFields(fields));
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<PublicCallRequest>) {
    return [fields.item, fields.counter] as const;
  }

  toFields(): Fr[] {
    return serializeToFields([this.item, this.counter]);
  }

  static fromFields(fields: Fr[] | FieldReader): PublicCallRequest {
    const reader = FieldReader.asReader(fields);
    return new PublicCallRequest(PublicCallStackItemCompressed.fromFields(reader), reader.readU32());
  }

  static empty() {
    return new PublicCallRequest(PublicCallStackItemCompressed.empty(), 0);
  }

  isEmpty(): boolean {
    return this.item.isEmpty() && this.counter == 0;
  }

  [inspect.custom]() {
    return `PublicCallRequest {
      item: ${this.item}
      counter: ${this.counter}
    }`;
  }
}
