import { sha256ToField } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { L1Actor } from './l1_actor.js';
import { L2Actor } from './l2_actor.js';

/**
 * The format of an L1 to L2 Message.
 */
export class L1ToL2Message {
  constructor(
    /**
     * The sender of the message on L1.
     */
    public readonly sender: L1Actor,
    /**
     * The recipient of the message on L2.
     */
    public readonly recipient: L2Actor,
    /**
     * The message content.
     */
    public readonly content: Fr,
    /**
     * The hash of the spending secret.
     */
    public readonly secretHash: Fr,
  ) {}

  /**
   * Returns each element within its own field so that it can be consumed by an acvm oracle call.
   * @returns The message as an array of fields (in order).
   */
  toFields(): Fr[] {
    return [...this.sender.toFields(), ...this.recipient.toFields(), this.content, this.secretHash];
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.sender, this.recipient, this.content, this.secretHash);
  }

  hash(): Fr {
    return sha256ToField(serializeToBuffer(...this.toFields()));
  }

  static fromBuffer(buffer: Buffer | BufferReader): L1ToL2Message {
    const reader = BufferReader.asReader(buffer);
    const sender = reader.readObject(L1Actor);
    const recipient = reader.readObject(L2Actor);
    const content = Fr.fromBuffer(reader);
    const secretHash = Fr.fromBuffer(reader);
    return new L1ToL2Message(sender, recipient, content, secretHash);
  }

  toString(): string {
    return this.toBuffer().toString('hex');
  }

  static fromString(data: string): L1ToL2Message {
    const buffer = Buffer.from(data, 'hex');
    return L1ToL2Message.fromBuffer(buffer);
  }

  static empty(): L1ToL2Message {
    return new L1ToL2Message(L1Actor.empty(), L2Actor.empty(), Fr.ZERO, Fr.ZERO);
  }

  static random(): L1ToL2Message {
    return new L1ToL2Message(L1Actor.random(), L2Actor.random(), Fr.random(), Fr.random());
  }
}
