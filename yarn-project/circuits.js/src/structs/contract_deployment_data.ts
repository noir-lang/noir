import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

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

  toBuffer() {
    return serializeToBuffer(
      this.publicKey,
      this.initializationHash,
      this.contractClassId,
      this.contractAddressSalt,
      this.portalContractAddress,
    );
  }

  toFields(): Fr[] {
    return [
      ...this.publicKey.toFields(),
      this.initializationHash,
      this.contractClassId,
      this.contractAddressSalt,
      this.portalContractAddress.toField(),
    ];
  }

  /**
   * Returns an empty ContractDeploymentData.
   * @returns The empty ContractDeploymentData.
   */
  public static empty(): ContractDeploymentData {
    return new ContractDeploymentData(Point.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, EthAddress.ZERO);
  }

  isEmpty() {
    return (
      this.publicKey.isZero() &&
      this.initializationHash.isZero() &&
      this.contractClassId.isZero() &&
      this.contractAddressSalt.isZero() &&
      this.portalContractAddress.isZero()
    );
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

    const publicKey = reader.readObject(Point);
    const initializationHash = reader.readField();
    const contractClassId = reader.readField();
    const contractAddressSalt = reader.readField();
    const portalContractAddress = reader.readObject(EthAddress);

    return new ContractDeploymentData(
      publicKey,
      initializationHash,
      contractClassId,
      contractAddressSalt,
      portalContractAddress,
    );
  }
}
