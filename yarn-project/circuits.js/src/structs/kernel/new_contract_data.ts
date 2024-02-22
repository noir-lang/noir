import { AztecAddress } from '@aztec/foundation/aztec-address';
import { pedersenHash } from '@aztec/foundation/crypto';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import { GeneratorIndex, NEW_CONTRACT_DATA_LENGTH } from '../../constants.gen.js';

/**
 * The information assembled after the contract deployment was processed by the private kernel circuit.
 *
 * Note: Not to be confused with `ContractDeploymentData`.
 */
export class NewContractData {
  constructor(
    /**
     * Aztec address of the contract.
     */
    public contractAddress: AztecAddress,
    /**
     * Ethereum address of the portal contract on L1.
     */
    public portalContractAddress: EthAddress,
    /**
     * Contract class id.
     */
    public contractClassId: Fr,
  ) {}

  static getFields(fields: FieldsOf<NewContractData>) {
    return [fields.contractAddress, fields.portalContractAddress, fields.contractClassId] as const;
  }

  toBuffer() {
    return serializeToBuffer(...NewContractData.getFields(this));
  }

  toFields() {
    const fields = serializeToFields(...NewContractData.getFields(this));
    if (fields.length !== NEW_CONTRACT_DATA_LENGTH) {
      throw new Error(
        `Invalid number of fields for NewContractData. Expected ${NEW_CONTRACT_DATA_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  /**
   * Computes a hash of contract data which is a leaf in the contracts tree.
   * @param cd - The contract data of the deployed contract.
   * @returns The contract data hash/contract tree leaf.
   */
  hash(): Fr {
    if (this.isEmpty()) {
      return new Fr(0);
    }
    return pedersenHash(
      NewContractData.getFields(this).map(f => f.toBuffer()),
      GeneratorIndex.CONTRACT_LEAF,
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized `NewContractData`.
   */
  static fromBuffer(buffer: Buffer | BufferReader): NewContractData {
    const reader = BufferReader.asReader(buffer);
    return new NewContractData(reader.readObject(AztecAddress), reader.readObject(EthAddress), Fr.fromBuffer(reader));
  }

  static empty() {
    return new NewContractData(AztecAddress.ZERO, EthAddress.ZERO, Fr.ZERO);
  }

  /**
   * Checks if the data is empty.
   * @returns True if the data operation is empty, false otherwise.
   */
  isEmpty(): boolean {
    return this.contractAddress.isZero() && this.portalContractAddress.isZero() && this.contractClassId.isZero();
  }
}
