import { Fr } from '@aztec/foundation/fields';
import { serializeToBuffer } from '../utils/serialize.js';
import { EthAddress } from '@aztec/foundation/eth-address';
import { BufferReader } from '@aztec/foundation/serialize';

/**
 * Contract deployment data in a TxContext
 * cpp/src/aztec3/circuits/abis/contract_deployment_data.hpp.
 *
 * Not to be confused with NewContractData.
 */
export class ContractDeploymentData {
  constructor(
    /**
     * Hash of the constuctor verification key.
     */
    public constructorVkHash: Fr,
    /**
     * Function tree root.
     */
    public functionTreeRoot: Fr,
    /**
     * Contract address salt (used when deriving a contract address).
     */
    public contractAddressSalt: Fr,
    /**
     * Ethereum address of the portal contract on L1.
     */
    public portalContractAddress: EthAddress,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.constructorVkHash,
      this.functionTreeRoot,
      this.contractAddressSalt,
      this.portalContractAddress,
    );
  }

  /**
   * Returns an empty ContractDeploymentData.
   * @returns The empty ContractDeploymentData.
   */
  public static empty(): ContractDeploymentData {
    return new ContractDeploymentData(Fr.ZERO, Fr.ZERO, Fr.ZERO, EthAddress.ZERO);
  }
  /**
   * Deserializes contract deployment data rom a buffer or reader.
   * @param buffer - Buffer to read from.
   * @returns The deserialized ContractDeploymentData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): ContractDeploymentData {
    const reader = BufferReader.asReader(buffer);
    return new ContractDeploymentData(
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      new EthAddress(reader.readBytes(32)),
    );
  }
}

/**
 * Transaction context.
 * @see cpp/src/aztec3/circuits/abis/tx_context.hpp.
 */
export class TxContext {
  constructor(
    /**
     * Whether this is a fee paying tx. If not other tx in a bundle will pay the fee.
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
    public isContractDeployment: boolean,
    /**
     * Contract deployment data.
     */
    public contractDeploymentData: ContractDeploymentData,
  ) {}

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.isFeePaymentTx,
      this.isRebatePaymentTx,
      this.isContractDeployment,
      this.contractDeploymentData,
    );
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
    );
  }

  static empty() {
    return new TxContext(false, false, true, ContractDeploymentData.empty());
  }
}
