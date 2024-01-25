import { FunctionSelector } from '@aztec/foundation/abi';
import { randomBytes } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, numToUInt8, serializeToBuffer } from '@aztec/foundation/serialize';

const VERSION = 1 as const;

export interface ContractClass {
  /** Version of the contract class. */
  version: typeof VERSION;
  /** Hash of the contract artifact. The specification of this hash is not enforced by the protocol. Should include commitments to unconstrained code and compilation metadata. Intended to be used by clients to verify that an off-chain fetched artifact matches a registered class. */
  artifactHash: Fr;
  /** List of individual private functions, constructors included. */
  privateFunctions: PrivateFunction[];
  /** List of individual public functions. Should be removed once we switch to the AVM where all public bytecode is bundled together. */
  publicFunctions: PublicFunction[];
  /** Packed bytecode representation of the AVM bytecode for all public functions in this contract. Unused for now, see `publicFunctions`. */
  packedBytecode: Buffer;
}

/** Serializable implementation of the contract class interface. */
export class SerializableContractClass implements ContractClass {
  /** Version identifier. Initially one, bumped for any changes to the contract class struct. */
  public readonly version = VERSION;

  public readonly artifactHash: Fr;
  public readonly packedBytecode: Buffer;
  public readonly privateFunctions: SerializablePrivateFunction[];
  public readonly publicFunctions: SerializablePublicFunction[];

  constructor(contractClass: ContractClass) {
    if (contractClass.version !== VERSION) {
      throw new Error(`Unexpected contract class version ${contractClass.version}`);
    }
    this.privateFunctions = contractClass.privateFunctions.map(x => new SerializablePrivateFunction(x));
    this.publicFunctions = contractClass.publicFunctions.map(x => new SerializablePublicFunction(x));
    this.artifactHash = contractClass.artifactHash;
    this.packedBytecode = contractClass.packedBytecode;
  }

  /** Returns a copy of this object with its id included. */
  withId(id: Fr): ContractClassWithId {
    return { ...this, id };
  }

  public toBuffer() {
    return serializeToBuffer(
      numToUInt8(this.version),
      this.artifactHash,
      this.privateFunctions.length,
      this.privateFunctions,
      this.publicFunctions.length,
      this.publicFunctions,
      this.packedBytecode.length,
      this.packedBytecode,
    );
  }

  static fromBuffer(bufferOrReader: BufferReader | Buffer) {
    const reader = BufferReader.asReader(bufferOrReader);
    return new SerializableContractClass({
      version: reader.readUInt8() as typeof VERSION,
      artifactHash: reader.readObject(Fr),
      privateFunctions: reader.readVector(SerializablePrivateFunction),
      publicFunctions: reader.readVector(SerializablePublicFunction),
      packedBytecode: reader.readBuffer(),
    });
  }

  static random() {
    return new SerializableContractClass({
      version: VERSION,
      artifactHash: Fr.random(),
      privateFunctions: [SerializablePrivateFunction.random()],
      publicFunctions: [SerializablePublicFunction.random()],
      packedBytecode: randomBytes(32),
    });
  }
}

export interface PrivateFunction {
  /** Selector of the function. Calculated as the hash of the method name and parameters. The specification of this is not enforced by the protocol. */
  selector: FunctionSelector;
  /** Hash of the verification key associated to this private function. */
  vkHash: Fr;
  /**
   * Whether the function is internal.
   * @deprecated To be reimplemented as an app-level macro.
   */
  isInternal: boolean;
}

/** Private function in a Contract Class. */
export class SerializablePrivateFunction {
  public readonly selector: FunctionSelector;
  public readonly vkHash: Fr;
  public readonly isInternal: boolean;

  constructor(privateFunction: PrivateFunction) {
    this.selector = privateFunction.selector;
    this.vkHash = privateFunction.vkHash;
    this.isInternal = privateFunction.isInternal;
  }

  public toBuffer() {
    return serializeToBuffer(this.selector, this.vkHash, this.isInternal);
  }

  static fromBuffer(bufferOrReader: BufferReader | Buffer): PrivateFunction {
    const reader = BufferReader.asReader(bufferOrReader);
    return new SerializablePrivateFunction({
      selector: reader.readObject(FunctionSelector),
      vkHash: reader.readObject(Fr),
      isInternal: reader.readBoolean(),
    });
  }

  static random() {
    return new SerializablePrivateFunction({
      selector: FunctionSelector.random(),
      vkHash: Fr.random(),
      isInternal: false,
    });
  }
}

export interface PublicFunction {
  /** Selector of the function. Calculated as the hash of the method name and parameters. The specification of this is not enforced by the protocol. */
  selector: FunctionSelector;
  /** Public bytecode. */
  bytecode: Buffer;
  /**
   * Whether the function is internal.
   * @deprecated To be reimplemented as an app-level macro.
   */
  isInternal: boolean;
}

/**
 * Public function in a Contract Class. Use `packedBytecode` in the parent class once supported.
 */
export class SerializablePublicFunction {
  public readonly selector: FunctionSelector;
  public readonly bytecode: Buffer;
  public readonly isInternal: boolean;

  constructor(publicFunction: PublicFunction) {
    this.selector = publicFunction.selector;
    this.bytecode = publicFunction.bytecode;
    this.isInternal = publicFunction.isInternal;
  }

  public toBuffer() {
    return serializeToBuffer(this.selector, this.bytecode.length, this.bytecode, this.isInternal);
  }

  static fromBuffer(bufferOrReader: BufferReader | Buffer): PublicFunction {
    const reader = BufferReader.asReader(bufferOrReader);
    return new SerializablePublicFunction({
      selector: reader.readObject(FunctionSelector),
      bytecode: reader.readBuffer(),
      isInternal: reader.readBoolean(),
    });
  }

  static random() {
    return new SerializablePublicFunction({
      selector: FunctionSelector.random(),
      bytecode: randomBytes(32),
      isInternal: false,
    });
  }
}

export type ContractClassWithId = ContractClass & { id: Fr };
