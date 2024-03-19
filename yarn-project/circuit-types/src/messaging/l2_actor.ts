import { AztecAddress } from '@aztec/foundation/aztec-address';
import { randomInt } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

/**
 * The recipient of an L2 message.
 */
export class L2Actor {
  constructor(
    /**
     * The recipient of the message.
     */
    public readonly recipient: AztecAddress,
    /**
     * The version of the protocol.
     */
    public readonly version: number,
  ) {}

  static empty() {
    return new L2Actor(AztecAddress.ZERO, 0);
  }

  toFields(): Fr[] {
    return [this.recipient.toField(), new Fr(BigInt(this.version))];
  }

  toBuffer(): Buffer {
    return serializeToBuffer(this.recipient, this.version);
  }

  static fromBuffer(buffer: Buffer | BufferReader): L2Actor {
    const reader = BufferReader.asReader(buffer);
    const aztecAddr = AztecAddress.fromBuffer(reader);
    const version = reader.readNumber();
    return new L2Actor(aztecAddr, version);
  }

  static random(): L2Actor {
    return new L2Actor(AztecAddress.random(), randomInt(1000));
  }
}
