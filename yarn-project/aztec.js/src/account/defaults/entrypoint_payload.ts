import { CircuitsWasm, Fr, GeneratorIndex } from '@aztec/circuits.js';
import { pedersenPlookupCompressWithHashIndex } from '@aztec/circuits.js/barretenberg';
import { padArrayEnd } from '@aztec/foundation/collection';
import { FunctionCall, PackedArguments, emptyFunctionCall } from '@aztec/types';

// These must match the values defined in yarn-project/aztec-nr/aztec/src/entrypoint.nr
export const ACCOUNT_MAX_CALLS = 4;

/** Encoded function call for account contract entrypoint */
export type EntrypointFunctionCall = {
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
export type EntrypointPayload = {
  // eslint-disable-next-line camelcase
  /** Encoded function calls to execute */
  function_calls: EntrypointFunctionCall[];
  /** A nonce for replay protection */
  nonce: Fr;
};

/** Assembles an entrypoint payload from a set of private and public function calls */
export async function buildPayload(calls: FunctionCall[]): Promise<{
  /** The payload for the entrypoint function */
  payload: EntrypointPayload;
  /** The packed arguments of functions called */
  packedArguments: PackedArguments[];
}> {
  const nonce = Fr.random();

  const paddedCalls = padArrayEnd(calls, emptyFunctionCall(), ACCOUNT_MAX_CALLS);
  const packedArguments: PackedArguments[] = [];
  for (const call of paddedCalls) {
    packedArguments.push(await PackedArguments.fromArgs(call.args));
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

/** Compresses an entrypoint payload to a 32-byte buffer (useful for signing) */
export async function hashPayload(payload: EntrypointPayload) {
  return pedersenPlookupCompressWithHashIndex(
    await CircuitsWasm.get(),
    flattenPayload(payload).map(fr => fr.toBuffer()),
    GeneratorIndex.SIGNATURE_PAYLOAD,
  );
}

/** Flattens an entrypoint payload */
export function flattenPayload(payload: EntrypointPayload) {
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
