import { Fr, FunctionSelector } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { ContractClassPublic, ContractInstanceWithAddress, PublicFunction } from '@aztec/types/contracts';

/**
 * Used for retrieval of contract data (A3 address, portal contract address, bytecode).
 */
export interface ContractDataSource {
  /**
   * Lookup the L2 contract base info for this contract.
   * NOTE: This works for all Aztec contracts and will only return contractAddress / portalAddress.
   * @param contractAddress - The contract data address.
   * @returns The aztec & ethereum portal address (if found).
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Returns a contract's encoded public function, given its function selector.
   * @param address - The contract aztec address.
   * @param selector - The function's selector.
   * @returns The function's data.
   */
  getPublicFunction(address: AztecAddress, selector: FunctionSelector): Promise<PublicFunction | undefined>;

  /**
   * Gets the number of the latest L2 block processed by the implementation.
   * @returns The number of the latest L2 block processed by the implementation.
   */
  getBlockNumber(): Promise<number>;

  /**
   * Returns the contract class for a given contract class id, or undefined if not found.
   * @param id - Contract class id.
   */
  getContractClass(id: Fr): Promise<ContractClassPublic | undefined>;

  /**
   * Returns a publicly deployed contract instance given its address.
   * @param address - Address of the deployed contract.
   */
  getContract(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined>;

  /** Returns the list of all class ids known. */
  getContractClassIds(): Promise<Fr[]>;
}

/**
 * A contract data blob, containing L1 and L2 addresses.
 * TODO(palla/purge-old-contract-deploy): Delete me
 */
export class ContractData {
  constructor(
    /**
     * The L2 address of the contract, as a field element (32 bytes).
     */
    public contractAddress: AztecAddress,
    /**
     * The L1 address of the contract, (20 bytes).
     */
    public portalContractAddress: EthAddress,
  ) {}

  /**
   * Serializes this instance into a buffer, using 20 bytes for the eth address.
   * @returns Encoded buffer.
   */
  public toBuffer(): Buffer {
    return serializeToBuffer(this.contractAddress, this.portalContractAddress);
  }

  /**
   * Serializes this instance into a string, using 20 bytes for the eth address.
   * @returns Encoded string.
   */
  public toString(): string {
    return this.toBuffer().toString('hex');
  }

  /** True if all data is zero. */
  public isEmpty(): boolean {
    return this.contractAddress.isZero() && this.portalContractAddress.isZero();
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
    return new ContractData(aztecAddr, ethAddr);
  }

  /**
   * Deserializes a contract data object from an encoded string, using 20 bytes for the eth address.
   * @param str - String resulting from calling toString.
   * @returns Deserialized instance.
   */
  static fromString(str: string) {
    return ContractData.fromBuffer(Buffer.from(str, 'hex'));
  }

  /**
   * Generate ContractData with random addresses.
   * @returns ContractData.
   */
  static random(): ContractData {
    return new ContractData(AztecAddress.random(), EthAddress.random());
  }

  /** Generates an empty ContractData. */
  static empty(): ContractData {
    return new ContractData(AztecAddress.ZERO, EthAddress.ZERO);
  }
}
