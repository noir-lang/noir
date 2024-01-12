import { AztecAddress } from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';
import { BufferReader, prefixBufferWithLength } from '@aztec/foundation/serialize';

import { randomBytes } from 'crypto';

/**
 * Represents an individual unencrypted log entry.
 */
export class UnencryptedL2Log {
  constructor(
    /**
     * Address of the contract that emitted the event
     * NOTE: It would make sense to have the address only in `FunctionL2Logs` because contract address is shared for all
     * function logs. I didn't do this because it would require us to have 2 FunctionL2Logs classes (one with contract
     * address and one without) for unencrypted and encrypted because encrypted logs can't expose the address in an
     * unencrypted form. For this reason separating the classes seems like a premature optimization.
     * TODO: Optimize this once it makes sense.
     */
    public readonly contractAddress: AztecAddress,
    /** Selector of the event/log topic. */
    public readonly selector: EventSelector,
    /** The data contents of the log. */
    public readonly data: Buffer,
  ) {}

  get length(): number {
    return EventSelector.SIZE + this.data.length;
  }

  /**
   * Serializes log to a buffer.
   * @returns A buffer containing the serialized log.
   */
  public toBuffer(): Buffer {
    return Buffer.concat([
      this.contractAddress.toBuffer(),
      this.selector.toBuffer(),
      prefixBufferWithLength(this.data),
    ]);
  }

  /**
   * Serializes log to a human readable string.
   * @returns A human readable representation of the log.
   */
  public toHumanReadable(): string {
    return `UnencryptedL2Log(contractAddress: ${this.contractAddress.toString()}, selector: ${this.selector.toString()}, data: ${this.data.toString(
      'ascii',
    )})`;
  }

  /**
   * Deserializes log from a buffer.
   * @param buffer - The buffer or buffer reader containing the log.
   * @returns Deserialized instance of `Log`.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): UnencryptedL2Log {
    const reader = BufferReader.asReader(buffer);
    const contractAddress = AztecAddress.fromBuffer(reader);
    const selector = EventSelector.fromBuffer(reader);
    const data = reader.readBuffer();
    return new UnencryptedL2Log(contractAddress, selector, data);
  }

  /**
   * Crates a random log.
   * @returns A random log.
   */
  public static random(): UnencryptedL2Log {
    const contractAddress = AztecAddress.random();
    const selector = new EventSelector(Math.floor(Math.random() * (2 ** (EventSelector.SIZE * 8) - 1)));
    const dataLength = EventSelector.SIZE + Math.floor(Math.random() * 200);
    const data = randomBytes(dataLength);
    return new UnencryptedL2Log(contractAddress, selector, data);
  }
}
