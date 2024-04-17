import { AztecAddress } from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';
import { randomBytes, sha256Trunc } from '@aztec/foundation/crypto';
import { BufferReader, prefixBufferWithLength } from '@aztec/foundation/serialize';

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
    return EventSelector.SIZE + this.data.length + AztecAddress.SIZE_IN_BYTES + 4;
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
   * Outputs the log data as ascii if all bytes are valid ascii characters between 32 and 126, or as hex otherwise.
   * @returns A human readable representation of the log.
   */
  public toHumanReadable(): string {
    const payload = this.data.every(byte => byte >= 32 && byte <= 126)
      ? this.data.toString('ascii')
      : `0x` + this.data.toString('hex');
    return `UnencryptedL2Log(contractAddress: ${this.contractAddress.toString()}, selector: ${this.selector.toString()}, data: ${payload})`;
  }

  /** Returns a JSON-friendly representation of the log. */
  public toJSON(): object {
    return {
      contractAddress: this.contractAddress.toString(),
      selector: this.selector.toString(),
      data: this.data.toString('hex'),
    };
  }

  /** Converts a plain JSON object into an instance. */
  public static fromJSON(obj: any) {
    return new UnencryptedL2Log(
      AztecAddress.fromString(obj.contractAddress),
      EventSelector.fromString(obj.selector),
      Buffer.from(obj.data, 'hex'),
    );
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
   * Calculates hash of serialized logs.
   * @returns Buffer containing 248 bits of information of sha256 hash.
   */
  public hash(): Buffer {
    const preimage = this.toBuffer();
    return sha256Trunc(preimage);
  }

  /**
   * Crates a random log.
   * @returns A random log.
   */
  public static random(): UnencryptedL2Log {
    const contractAddress = AztecAddress.random();
    const selector = EventSelector.random();
    const dataLength = EventSelector.SIZE + randomBytes(1)[0];
    const data = randomBytes(dataLength);
    return new UnencryptedL2Log(contractAddress, selector, data);
  }
}
