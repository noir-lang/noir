import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, numToUInt8, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

const VERSION = 1 as const;

/** A contract instance is a concrete deployment of a contract class. A contract instance always references a contract class, which dictates what code it executes when called. A contract instance has state (both private and public), as well as an address that acts as its identifier. A contract instance can be called into. */
export interface ContractInstance {
  /** Version identifier. Initially one, bumped for any changes to the contract instance struct. */
  version: typeof VERSION;
  /** User-generated pseudorandom value for uniqueness. */
  salt: Fr;
  /** Identifier of the contract class for this instance. */
  contractClassId: Fr;
  /** Hash of the selector and arguments to the constructor. */
  initializationHash: Fr;
  /** Optional address of the L1 portal contract. */
  portalContractAddress: EthAddress;
  /** Optional hash of the struct of public keys used for encryption and nullifying by this contract. */
  publicKeysHash: Fr;
  /** Optional deployer address or zero if this was a universal deploy. */
  deployer: AztecAddress;
}

export type ContractInstanceWithAddress = ContractInstance & { address: AztecAddress };

export class SerializableContractInstance {
  public readonly version = VERSION;
  public readonly salt: Fr;
  public readonly contractClassId: Fr;
  public readonly initializationHash: Fr;
  public readonly portalContractAddress: EthAddress;
  public readonly publicKeysHash: Fr;
  public readonly deployer: AztecAddress;

  constructor(instance: ContractInstance) {
    if (instance.version !== VERSION) {
      throw new Error(`Unexpected contract class version ${instance.version}`);
    }
    this.salt = instance.salt;
    this.contractClassId = instance.contractClassId;
    this.initializationHash = instance.initializationHash;
    this.portalContractAddress = instance.portalContractAddress;
    this.publicKeysHash = instance.publicKeysHash;
    this.deployer = instance.deployer;
  }

  public toBuffer() {
    return serializeToBuffer(
      numToUInt8(this.version),
      this.salt,
      this.contractClassId,
      this.initializationHash,
      this.portalContractAddress,
      this.publicKeysHash,
      this.deployer,
    );
  }

  /** Returns a copy of this object with its address included. */
  withAddress(address: AztecAddress): ContractInstanceWithAddress {
    return { ...this, address };
  }

  static fromBuffer(bufferOrReader: Buffer | BufferReader) {
    const reader = BufferReader.asReader(bufferOrReader);
    return new SerializableContractInstance({
      version: reader.readUInt8() as typeof VERSION,
      salt: reader.readObject(Fr),
      contractClassId: reader.readObject(Fr),
      initializationHash: reader.readObject(Fr),
      portalContractAddress: reader.readObject(EthAddress),
      publicKeysHash: reader.readObject(Fr),
      deployer: reader.readObject(AztecAddress),
    });
  }

  static random(opts: Partial<FieldsOf<ContractInstance>> = {}) {
    return new SerializableContractInstance({
      version: VERSION,
      salt: Fr.random(),
      contractClassId: Fr.random(),
      initializationHash: Fr.random(),
      portalContractAddress: EthAddress.random(),
      publicKeysHash: Fr.random(),
      deployer: AztecAddress.random(),
      ...opts,
    });
  }
}
