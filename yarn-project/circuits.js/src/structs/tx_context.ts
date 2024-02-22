import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import { GeneratorIndex, TX_CONTEXT_DATA_LENGTH } from '../constants.gen.js';
import { ContractDeploymentData } from '../structs/contract_deployment_data.js';

/**
 * Transaction context.
 */
export class TxContext {
  constructor(
    /**
     * Whether this is a fee paying tx. If not other tx in a bundle will pay the fee.
     * TODO(#3417): Remove fee and rebate payment fields.
     */
    public isFeePaymentTx: boolean,
    /**
     * Indicates whether this a gas rebate payment tx.
     *
     * NOTE: The following is a WIP and it is likely to change in the future.
     * Explanation: Each tx is actually 3 txs in one: a fee-paying tx, the actual tx you want to execute, and a rebate
     * tx. The fee-paying tx pays some `max_fee = gas_price * gas_limit`. Then the actual tx will cost an amount of gas
     * to execute (actual_fee = gas_price * gas_used). Then the rebate tx returns `max_fee - actual_fee` back to
     * the user.
     */
    public isRebatePaymentTx: boolean,
    /**
     * Whether this is a contract deployment tx.
     */
    public isContractDeploymentTx: boolean,
    /**
     * Contract deployment data.
     */
    public contractDeploymentData: ContractDeploymentData,
    /**
     * Chain ID of the transaction. Here for replay protection.
     */
    public chainId: Fr,
    /**
     * Version of the transaction. Here for replay protection.
     */
    public version: Fr,
  ) {}

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(...TxContext.getFields(this));
  }

  toFields(): Fr[] {
    const fields = serializeToFields(...TxContext.getFields(this));
    if (fields.length !== TX_CONTEXT_DATA_LENGTH) {
      throw new Error(
        `Invalid number of fields for TxContext. Expected ${TX_CONTEXT_DATA_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  /**
   * Deserializes TxContext from a buffer or reader.
   * @param buffer - Buffer to read from.
   * @returns The TxContext.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxContext {
    const reader = BufferReader.asReader(buffer);
    return new TxContext(
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readObject(ContractDeploymentData),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  static empty(chainId: Fr | number = 0, version: Fr | number = 0) {
    return new TxContext(false, false, false, ContractDeploymentData.empty(), new Fr(chainId), new Fr(version));
  }

  isEmpty(): boolean {
    return (
      !this.isFeePaymentTx &&
      !this.isRebatePaymentTx &&
      !this.isContractDeploymentTx &&
      this.contractDeploymentData.isEmpty() &&
      this.chainId.isZero() &&
      this.version.isZero()
    );
  }

  /**
   * Create a new instance from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A new instance.
   */
  static from(fields: FieldsOf<TxContext>): TxContext {
    return new TxContext(...TxContext.getFields(fields));
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<TxContext>) {
    return [
      fields.isFeePaymentTx,
      fields.isRebatePaymentTx,
      fields.isContractDeploymentTx,
      fields.contractDeploymentData,
      fields.chainId,
      fields.version,
    ] as const;
  }

  hash(): Fr {
    return pedersenHash(
      this.toFields().map(f => f.toBuffer()),
      GeneratorIndex.TX_CONTEXT,
    );
  }
}
