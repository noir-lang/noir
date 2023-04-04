import { assertLength, FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { ARGS_LENGTH } from './constants.js';
import { FunctionData } from './function_data.js';
import { EcdsaSignature } from './shared.js';
import { AztecAddress, BufferReader, EthAddress, Fr } from '@aztec/foundation';

/**
 * Contract deployment data in a TxContext
 * cpp/src/aztec3/circuits/abis/contract_deployment_data.hpp
 */
export class ContractDeploymentData {
  constructor(
    public constructorVkHash: Fr,
    public functionTreeRoot: Fr,
    public contractAddressSalt: Fr,
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

  public static empty() {
    return new ContractDeploymentData(Fr.ZERO, Fr.ZERO, Fr.ZERO, EthAddress.ZERO);
  }
  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
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
    public isFeePaymentTx: boolean,
    public isRebatePaymentTx: boolean,
    public isContractDeployment: boolean,
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
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
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
}

/**
 * Signed transaction request.
 * @see cpp/src/aztec3/circuits/abis/signed_tx_request.hpp.
 */
export class SignedTxRequest {
  constructor(public txRequest: TxRequest, public signature: EcdsaSignature) {}

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.txRequest, this.signature);
  }
}

/**
 * Transaction request.
 * @see cpp/src/aztec3/circuits/abis/tx_request.hpp.
 */
export class TxRequest {
  constructor(
    public from: AztecAddress,
    public to: AztecAddress,
    public functionData: FunctionData,
    public args: Fr[],
    public nonce: Fr,
    public txContext: TxContext,
    public chainId: Fr,
  ) {}

  static getFields(fields: FieldsOf<TxRequest>) {
    return [
      fields.from,
      fields.to,
      fields.functionData,
      fields.args,
      fields.nonce,
      fields.txContext,
      fields.chainId,
    ] as const;
  }

  static from(fields: FieldsOf<TxRequest>): TxRequest {
    return new TxRequest(...TxRequest.getFields(fields));
  }

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(...TxRequest.getFields(this));
  }
}
