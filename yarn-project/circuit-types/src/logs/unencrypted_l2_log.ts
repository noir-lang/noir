import { AztecAddress } from '@aztec/circuits.js';
import { randomBytes, sha256Trunc } from '@aztec/foundation/crypto';
import { BufferReader, prefixBufferWithLength, toHumanReadable } from '@aztec/foundation/serialize';

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
    /** The data contents of the log. */
    public readonly data: Buffer,
  ) {}

  get length(): number {
    // We want the length of the buffer output from function_l2_logs -> toBuffer to equal the stored log length in the kernels.
    // The kernels store the length of the processed log as 4 bytes; thus for this length value to match the log length stored in the kernels,
    // we need to add four to the length here.
    // https://github.com/AztecProtocol/aztec-packages/issues/6578#issuecomment-2125003435
    return this.data.length + AztecAddress.SIZE_IN_BYTES + 4;
  }

  /**
   * Serializes log to a buffer.
   * @returns A buffer containing the serialized log.
   */
  public toBuffer(): Buffer {
    return Buffer.concat([this.contractAddress.toBuffer(), prefixBufferWithLength(this.data)]);
  }

  /**
   * Serializes log to a human readable string.
   * Outputs the log data as ascii if all bytes are valid ascii characters between 32 and 126, or as hex otherwise.
   * @returns A human readable representation of the log.
   */
  public toHumanReadable(): string {
    const payload = toHumanReadable(this.data);
    return `UnencryptedL2Log(contractAddress: ${this.contractAddress.toString()}, data: ${payload})`;
  }

  /** Returns a JSON-friendly representation of the log. */
  public toJSON(): object {
    return {
      contractAddress: this.contractAddress.toString(),
      data: this.data.toString('hex'),
    };
  }

  /** Converts a plain JSON object into an instance. */
  public static fromJSON(obj: any) {
    return new UnencryptedL2Log(AztecAddress.fromString(obj.contractAddress), Buffer.from(obj.data, 'hex'));
  }

  /**
   * Deserializes log from a buffer.
   * @param buffer - The buffer or buffer reader containing the log.
   * @returns Deserialized instance of `Log`.
   */
  public static fromBuffer(buffer: Buffer | BufferReader): UnencryptedL2Log {
    const reader = BufferReader.asReader(buffer);
    const contractAddress = AztecAddress.fromBuffer(reader);
    const data = reader.readBuffer();
    return new UnencryptedL2Log(contractAddress, data);
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
   * Calculates siloed hash of serialized logs.
   * In the kernels, we use the storage contract address and not the one encoded here.
   * They should match, so it seems fine to use the existing info here.
   * @returns Buffer containing 248 bits of information of sha256 hash.
   */
  public getSiloedHash(): Buffer {
    const hash = this.hash();
    return sha256Trunc(Buffer.concat([this.contractAddress.toBuffer(), hash]));
  }

  /**
   * Crates a random log.
   * @returns A random log.
   */
  public static random(): UnencryptedL2Log {
    const contractAddress = AztecAddress.random();
    const dataLength = randomBytes(1)[0];
    const data = randomBytes(dataLength);
    return new UnencryptedL2Log(contractAddress, data);
  }
}
