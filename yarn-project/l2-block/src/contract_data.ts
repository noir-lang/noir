import { AztecAddress, EthAddress } from '@aztec/circuits.js';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';

export { EthAddress } from '@aztec/circuits.js';
export { BufferReader } from '@aztec/circuits.js/utils';

/**
 * A contract data blob, containing L1 and L2 addresses.
 */
export class ContractData {
  constructor(
    /**
     * The L2 address of the contract, as a field element (32 bytes).
     */
    public aztecAddress: AztecAddress,
    /**
     * The L1 address of the contract, (20 bytes).
     */
    public ethAddress: EthAddress,
  ) {}

  /**
   * Serializes this instance into a buffer, using 20 bytes for the eth address.
   * @returns Encoded buffer.
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(this.aztecAddress, this.ethAddress.toBuffer());
  }

  /**
   * Deserializes a contract data object from an encoded buffer, using 20 bytes for the eth address.
   * @param buffer - Byte array resulting from calling toBuffer.
   * @returns Deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ContractData(
      AztecAddress.fromBuffer(reader),
      new EthAddress(reader.readBytes(EthAddress.SIZE_IN_BYTES)),
    );
  }
}
