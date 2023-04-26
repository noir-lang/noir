import { AztecAddress, EthAddress, numToInt32BE, randomBytes, serializeBufferArrayToVector } from '@aztec/foundation';
import { BufferReader, serializeToBuffer } from '@aztec/circuits.js/utils';
import { FUNCTION_SELECTOR_NUM_BYTES } from '@aztec/circuits.js';

export { BufferReader } from '@aztec/circuits.js/utils';

/**
 * Used for retrieval of contract data (A3 address, portal contract address, bytecode)
 */
export interface ContractDataSource {
  /**
   * Lookup the L2 contract data for this contract.
   * Contains information such as the ethereum portal address and bytecode.
   * NOTE: This method works only for contracts that have public function bytecode.
   * @param contractAddress - The contract data address.
   * @returns The full contract information (if found).
   */
  getL2ContractData(contractAddress: AztecAddress): Promise<CompleteContractData | undefined>;

  /**
   * Lookup the L2 contract base info for this contract.
   * NOTE: This works for all Aztec contracts and will only return contractAddres / portalAddress.
   * @param contractAddress - The contract data address.
   * @returns The aztec & etehereum portal address (if found).
   */
  getL2ContractInfo(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Lookup all contract data in an L2 block.
   * @param blockNumber - The block number
   */
  getL2ContractDataInBlock(blockNumber: number): Promise<ContractData[]>;

  /**
   * Returns a contract's encoded public function, given its function selector.
   * @param address - The contract aztec address.
   * @param functionSelector - The function's selector
   * @returns The function's data.
   */
  getPublicFunction(address: AztecAddress, functionSelector: Buffer): Promise<EncodedContractFunction | undefined>;
}

export class EncodedContractFunction {
  constructor(public functionSelector: Buffer, public bytecode: Buffer) {}

  toBuffer() {
    const bytecodeBuf = Buffer.concat([numToInt32BE(this.bytecode.length), this.bytecode]);
    return serializeToBuffer(this.functionSelector, bytecodeBuf);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    const fnSelector = reader.readBytes(FUNCTION_SELECTOR_NUM_BYTES);
    return new EncodedContractFunction(fnSelector, reader.readBuffer());
  }

  static random() {
    return new EncodedContractFunction(randomBytes(4), randomBytes(64));
  }
}

export type CompleteContractData = Required<ContractData>;

/**
 * A contract data blob, containing L1 and L2 addresses.
 */
export class ContractData {
  /**
   * The contract's encoded ACIR code. This should become Brilling code once implemented.
   */
  public bytecode?: Buffer;
  constructor(
    /**
     * The L2 address of the contract, as a field element (32 bytes).
     */
    public contractAddress: AztecAddress,
    /**
     * The L1 address of the contract, (20 bytes).
     */
    public portalContractAddress: EthAddress,

    /**
     * ABIs of public functions
     */
    public publicFunctions?: EncodedContractFunction[],
  ) {
    if (publicFunctions?.length) {
      this.bytecode = serializeBufferArrayToVector(publicFunctions.map(fn => fn.toBuffer()));
    }
  }

  /**
   * Serializes this instance into a buffer, using 20 bytes for the eth address.
   * @returns Encoded buffer.
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(
      this.contractAddress,
      this.portalContractAddress.toBuffer(),
      this.bytecode || Buffer.alloc(4, 0),
    );
  }

  /**
   * Deserializes a contract data object from an encoded buffer, using 20 bytes for the eth address.
   * @param buffer - Byte array resulting from calling toBuffer.
   * @returns Deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    const aztecAddr = AztecAddress.fromBuffer(reader);
    const ethAddr = new EthAddress(reader.readBytes(EthAddress.SIZE_IN_BYTES));
    const publicFns = reader.readVector(EncodedContractFunction);
    return new ContractData(aztecAddr, ethAddr, publicFns);
  }

  /**
   * Generate ContractData with random addresses.
   * @returns ContractData.
   */
  static random(): CompleteContractData {
    return new ContractData(AztecAddress.random(), EthAddress.random(), [
      EncodedContractFunction.random(),
      EncodedContractFunction.random(),
    ]) as CompleteContractData;
  }

  static createCompleteContractData(
    contractAddres: AztecAddress,
    portalContractAddress: EthAddress,
    publicFunctions: EncodedContractFunction[],
  ): CompleteContractData {
    return new ContractData(contractAddres, portalContractAddress, publicFunctions) as CompleteContractData;
  }
}
