import { type FunctionCall, PackedValues, emptyFunctionCall } from '@aztec/circuit-types';
import { Fr, GeneratorIndex } from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { pedersenHash } from '@aztec/foundation/crypto';
import { type Tuple } from '@aztec/foundation/serialize';

import { type FeePaymentMethod } from '../fee/fee_payment_method.js';

/**
 * Fee payment options for a transaction.
 */
export type FeeOptions = {
  /** The fee payment method to use */
  paymentMethod: FeePaymentMethod;
  /** The fee limit to pay */
  maxFee: bigint | number | Fr;
};

// These must match the values defined in:
// - noir-projects/aztec-nr/aztec/src/entrypoint/app.nr
const APP_MAX_CALLS = 4;
// - and noir-projects/aztec-nr/aztec/src/entrypoint/fee.nr
const FEE_MAX_CALLS = 2;

/* eslint-disable camelcase */
/** Encoded function call for account contract entrypoint */
type EncodedFunctionCall = {
  /** Arguments hash for the call */
  args_hash: Fr;
  /** Selector of the function to call */
  function_selector: Fr;
  /** Address of the contract to call */
  target_address: Fr;
  /** Whether the function is public or private */
  is_public: boolean;
};
/* eslint-enable camelcase */

/** Assembles an entrypoint payload */
export class EntrypointPayload {
  #packedArguments: PackedValues[] = [];
  #functionCalls: EncodedFunctionCall[] = [];
  #nonce = Fr.random();
  #generatorIndex: number;

  private constructor(functionCalls: FunctionCall[], generatorIndex: number) {
    for (const call of functionCalls) {
      this.#packedArguments.push(PackedValues.fromValues(call.args));
    }

    /* eslint-disable camelcase */
    this.#functionCalls = functionCalls.map((call, index) => ({
      args_hash: this.#packedArguments[index].hash,
      function_selector: call.functionData.selector.toField(),
      target_address: call.to.toField(),
      is_public: !call.functionData.isPrivate,
    }));
    /* eslint-enable camelcase */

    this.#generatorIndex = generatorIndex;
  }

  /* eslint-disable camelcase */
  /**
   * The function calls to execute. This uses snake_case naming so that it is compatible with Noir encoding
   * @internal
   */
  get function_calls() {
    return this.#functionCalls;
  }
  /* eslint-enable camelcase */

  /**
   * The nonce
   * @internal
   */
  get nonce() {
    return this.#nonce;
  }

  /**
   * The packed arguments for the function calls
   */
  get packedArguments() {
    return this.#packedArguments;
  }

  /**
   * Serializes the payload to an array of fields
   * @returns The fields of the payload
   */
  toFields(): Fr[] {
    return [
      ...this.#functionCalls.flatMap(call => [
        call.args_hash,
        call.function_selector,
        call.target_address,
        new Fr(call.is_public),
      ]),
      this.#nonce,
    ];
  }

  /**
   * Hashes the payload
   * @returns The hash of the payload
   */
  hash() {
    return pedersenHash(this.toFields(), this.#generatorIndex);
  }

  /**
   * Creates an execution payload from a set of function calls
   * @param functionCalls - The function calls to execute
   * @returns The execution payload
   */
  static fromFunctionCalls(functionCalls: FunctionCall[]) {
    return new EntrypointPayload(functionCalls, 0);
  }

  /**
   * Creates an execution payload for the app-portion of a transaction from a set of function calls
   * @param functionCalls - The function calls to execute
   * @returns The execution payload
   */
  static fromAppExecution(functionCalls: FunctionCall[] | Tuple<FunctionCall, 4>) {
    if (functionCalls.length > APP_MAX_CALLS) {
      throw new Error(`Expected at most ${APP_MAX_CALLS} function calls, got ${functionCalls.length}`);
    }
    const paddedCalls = padArrayEnd(functionCalls, emptyFunctionCall(), APP_MAX_CALLS);
    return new EntrypointPayload(paddedCalls, GeneratorIndex.SIGNATURE_PAYLOAD);
  }

  /**
   * Creates an execution payload to pay the fee for a transaction
   * @param feeOpts - The fee payment options
   * @returns The execution payload
   */
  static async fromFeeOptions(feeOpts?: FeeOptions) {
    const calls = feeOpts ? await feeOpts.paymentMethod.getFunctionCalls(new Fr(feeOpts.maxFee)) : [];
    const paddedCalls = padArrayEnd(calls, emptyFunctionCall(), FEE_MAX_CALLS);
    return new EntrypointPayload(paddedCalls, GeneratorIndex.FEE_PAYLOAD);
  }
}
