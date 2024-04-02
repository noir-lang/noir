import { type FeeOptions } from '@aztec/aztec.js/entrypoint';
import { Fr } from '@aztec/aztec.js/fields';
import { type FunctionCall, PackedArguments, emptyFunctionCall } from '@aztec/circuit-types';
import { type AztecAddress } from '@aztec/circuits.js';
import { padArrayEnd } from '@aztec/foundation/collection';
import { pedersenHash } from '@aztec/foundation/crypto';

// These must match the values defined in:
// - noir-projects/aztec-nr/aztec/src/entrypoint/app.nr
const ACCOUNT_MAX_CALLS = 4;
// - and noir-projects/aztec-nr/aztec/src/entrypoint/fee.nr
const FEE_MAX_CALLS = 2;

/** Encoded function call for account contract entrypoint */
type EntrypointFunctionCall = {
  // eslint-disable-next-line camelcase
  /** Arguments hash for the call */
  args_hash: Fr;
  // eslint-disable-next-line camelcase
  /** Selector of the function to call */
  function_selector: Fr;
  // eslint-disable-next-line camelcase
  /** Address of the contract to call */
  target_address: Fr;
  // eslint-disable-next-line camelcase
  /** Whether the function is public or private */
  is_public: boolean;
};

/** Encoded payload for the account contract entrypoint */
type EntrypointPayload = {
  // eslint-disable-next-line camelcase
  /** Encoded function calls to execute */
  function_calls: EntrypointFunctionCall[];
  /** A nonce for replay protection */
  nonce: Fr;
};

/** Represents a generic payload to be executed in the context of an account contract */
export type PayloadWithArguments = {
  /** The payload to be run */
  payload: EntrypointPayload;
  /** The packed arguments for the function calls */
  packedArguments: PackedArguments[];
};

/**
 * Builds a payload to be sent to the account contract
 * @param calls - The function calls to run
 * @param maxCalls - The maximum number of call expected to be run. Used for padding
 * @returns A payload object and packed arguments
 */
function buildPayload(calls: FunctionCall[], maxCalls: number): PayloadWithArguments {
  const nonce = Fr.random();

  const paddedCalls = padArrayEnd(calls, emptyFunctionCall(), maxCalls);
  const packedArguments: PackedArguments[] = [];
  for (const call of paddedCalls) {
    packedArguments.push(PackedArguments.fromArgs(call.args));
  }

  const formattedCalls: EntrypointFunctionCall[] = paddedCalls.map((call, index) => ({
    // eslint-disable-next-line camelcase
    args_hash: packedArguments[index].hash,
    // eslint-disable-next-line camelcase
    function_selector: call.functionData.selector.toField(),
    // eslint-disable-next-line camelcase
    target_address: call.to.toField(),
    // eslint-disable-next-line camelcase
    is_public: !call.functionData.isPrivate,
  }));

  return {
    payload: {
      // eslint-disable-next-line camelcase
      function_calls: formattedCalls,
      nonce,
    },
    packedArguments,
  };
}

/** builds the payload for a Dapp entrypoint */
export function buildDappPayload(call: FunctionCall): PayloadWithArguments {
  return buildPayload([call], 1);
}

/** Assembles an entrypoint app payload from a set of private and public function calls */
export function buildAppPayload(calls: FunctionCall[]): PayloadWithArguments {
  return buildPayload(calls, ACCOUNT_MAX_CALLS);
}

/** Creates the payload for paying the fee for a transaction */
export async function buildFeePayload(feeOpts?: FeeOptions): Promise<PayloadWithArguments> {
  const calls = feeOpts ? await feeOpts.paymentMethod.getFunctionCalls(new Fr(feeOpts.maxFee)) : [];
  return buildPayload(calls, FEE_MAX_CALLS);
}

// TODO (dogfooding) change all of these names app/dapp/fee/payload and generator indices for all of them
/** Hashes a payload to a 32-byte buffer */
export function hashPayload(payload: EntrypointPayload, generatorIndex: number) {
  return pedersenHash(flattenPayload(payload), generatorIndex);
}

/** Hash the payload for a dapp */
export function hashDappPayload(payload: EntrypointPayload, userAddress: AztecAddress, generatorIndex: number) {
  return pedersenHash([...flattenPayload(payload), userAddress], generatorIndex);
}

/** Flattens an payload */
function flattenPayload(payload: EntrypointPayload) {
  return [
    ...payload.function_calls.flatMap(call => [
      call.args_hash,
      call.function_selector,
      call.target_address,
      new Fr(call.is_public),
    ]),
    payload.nonce,
  ];
}
