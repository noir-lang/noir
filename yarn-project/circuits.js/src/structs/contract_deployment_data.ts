import { pedersenHash } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import { CONTRACT_DEPLOYMENT_DATA_LENGTH, GeneratorIndex } from '../constants.gen.js';
import { PublicKey } from '../types/public_key.js';

/**
 * Contract deployment data in a TxContext
 * Not to be confused with NewContractData.
 */
export class ContractDeploymentData {
  constructor(
    /** Public key of the contract. */
    public publicKey: PublicKey,
    /** Hash of the initialization payload. */
    public initializationHash: Fr,
    /** Contract class identifier. */
    public contractClassId: Fr,
    /** Contract address salt (used when deriving a contract address). */
    public contractAddressSalt: Fr,
    /** Ethereum address of the portal contract on L1. */
    public portalContractAddress: EthAddress,
  ) {}

  static getFields(fields: FieldsOf<ContractDeploymentData>) {
    return [
      fields.publicKey,
      fields.initializationHash,
      fields.contractClassId,
      fields.contractAddressSalt,
      fields.portalContractAddress,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...ContractDeploymentData.getFields(this));
  }

  toFields(): Fr[] {
    const fields = serializeToFields(...ContractDeploymentData.getFields(this));
    if (fields.length !== CONTRACT_DEPLOYMENT_DATA_LENGTH) {
      throw new Error(
        `Invalid number of fields for ContractDeploymentData. Expected ${CONTRACT_DEPLOYMENT_DATA_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  /**
   * Returns an empty ContractDeploymentData.
   * @returns The empty ContractDeploymentData.
   */
  public static empty(): ContractDeploymentData {
    return new ContractDeploymentData(Point.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, EthAddress.ZERO);
  }

  isEmpty() {
    return ContractDeploymentData.getFields(this).every(f => f.isZero());
  }

  /**
   * Deserializes contract deployment data rom a buffer or reader.
   * @param buffer - Buffer to read from.
   * @returns The deserialized ContractDeploymentData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): ContractDeploymentData {
    const reader = BufferReader.asReader(buffer);
    return new ContractDeploymentData(
      reader.readObject(Point),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readObject(EthAddress),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): ContractDeploymentData {
    const reader = FieldReader.asReader(fields);
    return new ContractDeploymentData(
      reader.readObject(Point),
      reader.readField(),
      reader.readField(),
      reader.readField(),
      reader.readObject(EthAddress),
    );
  }

  hash(): Fr {
    return pedersenHash(
      this.toFields().map(f => f.toBuffer()),
      GeneratorIndex.CONTRACT_DEPLOYMENT_DATA,
    );
  }
}
