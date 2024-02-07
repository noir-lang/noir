import { FUNCTION_SELECTOR_NUM_BYTES, Fr, FunctionSelector } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { randomBytes } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import {
  BufferReader,
  numToInt32BE,
  serializeBufferArrayToVector,
  serializeToBuffer,
} from '@aztec/foundation/serialize';
import { ContractClassPublic, ContractInstanceWithAddress } from '@aztec/types/contracts';

/**
 * Used for retrieval of contract data (A3 address, portal contract address, bytecode).
 */
export interface ContractDataSource {
  /**
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined>;

  /**
   * Lookup the L2 contract base info for this contract.
   * NOTE: This works for all Aztec contracts and will only return contractAddress / portalAddress.
   * @param contractAddress - The contract data address.
   * @returns The aztec & ethereum portal address (if found).
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Gets extended contract data for all contracts deployed in L2 block.
   * @param blockNumber - The block number.
   * @returns Extended contract data of contracts deployed in L2 block.
   */
  getExtendedContractDataInBlock(blockNumber: number): Promise<ExtendedContractData[]>;

  /**
   * Lookup contract data in an L2 block.
   * @param blockNumber - The block number.
   * @returns Portal contract address info of contracts deployed in L2 block.
   */
  getContractDataInBlock(blockNumber: number): Promise<ContractData[] | undefined>;

  /**
   * Returns a contract's encoded public function, given its function selector.
   * @param address - The contract aztec address.
   * @param selector - The function's selector.
   * @returns The function's data.
   */
  getPublicFunction(address: AztecAddress, selector: FunctionSelector): Promise<EncodedContractFunction | undefined>;

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
}

/**
 * Represents encoded contract function.
 */
export class EncodedContractFunction {
  constructor(
    /**
     * The function selector.
     */
    public selector: FunctionSelector,
    /**
     * Whether the function is internal.
     */
    public isInternal: boolean,
    /**
     * The function bytecode.
     */
    public bytecode: Buffer,
  ) {}

  /**
   * Serializes this instance into a buffer.
   * @returns Encoded buffer.
   */
  toBuffer(): Buffer {
    const bytecodeBuf = Buffer.concat([numToInt32BE(this.bytecode.length), this.bytecode]);
    return serializeToBuffer(this.selector, this.isInternal, bytecodeBuf);
  }

  /**
   * Deserializes a contract function object from an encoded buffer.
   * @param buffer - The encoded buffer.
   * @returns The deserialized contract function.
   */
  static fromBuffer(buffer: Buffer | BufferReader): EncodedContractFunction {
    const reader = BufferReader.asReader(buffer);
    const fnSelector = FunctionSelector.fromBuffer(reader.readBytes(FUNCTION_SELECTOR_NUM_BYTES));
    const isInternal = reader.readBoolean();
    return new EncodedContractFunction(fnSelector, isInternal, reader.readBuffer());
  }

  /**
   * Serializes this instance into a string.
   * @returns Encoded string.
   */
  toString(): string {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes a contract function object from an encoded string.
   * @param data - The encoded string.
   * @returns The deserialized contract function.
   */
  static fromString(data: string): EncodedContractFunction {
    return EncodedContractFunction.fromBuffer(Buffer.from(data, 'hex'));
  }

  /**
   * Creates a random contract function.
   * @returns A random contract function.
   */
  static random(): EncodedContractFunction {
    return new EncodedContractFunction(FunctionSelector.fromBuffer(randomBytes(4)), false, randomBytes(64));
  }
}

/**
 * A contract data blob, containing L1 and L2 addresses, public functions' bytecode, partial address and public key.
 */
export class ExtendedContractData {
  /** The contract's encoded ACIR code. This should become Brillig code once implemented. */
  public bytecode: Buffer;

  constructor(
    /** The base contract data: aztec & portal addresses. */
    public contractData: ContractData,
    /** Artifacts of public functions. */
    public readonly publicFunctions: EncodedContractFunction[],
    /** Contract class id */
    public readonly contractClassId: Fr,
    /** Salted init hash. */
    public readonly saltedInitializationHash: Fr,
    /** Public key hash of the contract. */
    public readonly publicKeyHash: Fr,
  ) {
    this.bytecode = serializeBufferArrayToVector(publicFunctions.map(fn => fn.toBuffer()));
  }

  /**
   * Gets the public function data or undefined.
   * @param selector - The function selector of the function to fetch.
   * @returns The public function data (if found).
   */
  public getPublicFunction(selector: FunctionSelector): EncodedContractFunction | undefined {
    return this.publicFunctions.find(fn => fn.selector.equals(selector));
  }

  /**
   * Serializes this instance into a buffer, using 20 bytes for the eth address.
   * @returns Encoded buffer.
   */
  public toBuffer(): Buffer {
    const contractDataBuf = this.contractData.toBuffer();
    return serializeToBuffer(
      contractDataBuf,
      this.bytecode,
      this.contractClassId,
      this.saltedInitializationHash,
      this.publicKeyHash,
    );
  }

  /**
   * Serializes this instance into a string.
   * @returns Encoded string.
   */
  public toString(): string {
    return this.toBuffer().toString('hex');
  }

  /** True if this represents an empty instance. */
  public isEmpty(): boolean {
    return (
      this.contractData.isEmpty() &&
      this.publicFunctions.length === 0 &&
      this.contractClassId.isZero() &&
      this.publicKeyHash.isZero() &&
      this.saltedInitializationHash.isZero()
    );
  }

  /**
   * Deserializes a contract data object from an encoded buffer, using 20 bytes for the eth address.
   * @param buffer - Byte array resulting from calling toBuffer.
   * @returns Deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    const contractData = reader.readObject(ContractData);
    const publicFns = reader.readVector(EncodedContractFunction);
    const contractClassId = reader.readObject(Fr);
    const saltedInitializationHash = reader.readObject(Fr);
    const publicKeyHash = reader.readObject(Fr);
    return new ExtendedContractData(contractData, publicFns, contractClassId, saltedInitializationHash, publicKeyHash);
  }

  /**
   * Deserializes a contract data object from an encoded string, using 20 bytes for the eth address.
   * @param str - String resulting from calling toString.
   * @returns Deserialized instance.
   */
  static fromString(str: string) {
    return ExtendedContractData.fromBuffer(Buffer.from(str, 'hex'));
  }

  /**
   * Generate ContractData with random addresses.
   * @param contractData - Optional contract data to use.
   * @returns A random ExtendedContractData object.
   */
  static random(contractData?: ContractData): ExtendedContractData {
    return new ExtendedContractData(
      contractData ?? ContractData.random(),
      [EncodedContractFunction.random(), EncodedContractFunction.random()],
      Fr.random(),
      Fr.random(),
      Fr.random(),
    );
  }

  /** Generates empty extended contract data. */
  static empty(): ExtendedContractData {
    return new ExtendedContractData(ContractData.empty(), [], Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }
}

/**
 * A contract data blob, containing L1 and L2 addresses.
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
