import { CircuitsWasm, Fr, GeneratorIndex } from '@aztec/circuits.js';
import { pedersenPlookupCompressWithHashIndex } from '@aztec/circuits.js/barretenberg';
import { padArrayEnd } from '@aztec/foundation/collection';
import { FunctionCall, PackedArguments, emptyFunctionCall } from '@aztec/types';

import partition from 'lodash.partition';

// These must match the values defined in yarn-project/noir-libs/noir-aztec/src/entrypoint.nr
export const ACCOUNT_MAX_PRIVATE_CALLS = 2;
export const ACCOUNT_MAX_PUBLIC_CALLS = 2;

/** Encoded payload for the account contract entrypoint */
export type EntrypointPayload = {
  // eslint-disable-next-line camelcase
  /** Concatenated arguments for every call */
  flattened_args_hashes: Fr[];
  // eslint-disable-next-line camelcase
  /** Concatenated selectors for every call */
  flattened_selectors: Fr[];
  // eslint-disable-next-line camelcase
  /** Concatenated target addresses for every call */
  flattened_targets: Fr[];
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

  const [privateCalls, publicCalls] = partition(calls, call => call.functionData.isPrivate);

  const paddedCalls = [
    ...padArrayEnd(privateCalls, emptyFunctionCall(), ACCOUNT_MAX_PRIVATE_CALLS),
    ...padArrayEnd(publicCalls, emptyFunctionCall(), ACCOUNT_MAX_PUBLIC_CALLS),
  ];

  const packedArguments = [];
  const wasm = await CircuitsWasm.get();

  for (const call of paddedCalls) {
    packedArguments.push(await PackedArguments.fromArgs(call.args, wasm));
  }

  return {
    payload: {
      // eslint-disable-next-line camelcase
      flattened_args_hashes: packedArguments.map(args => args.hash),
      // eslint-disable-next-line camelcase
      flattened_selectors: paddedCalls.map(call => call.functionData.selector.toField()),
      // eslint-disable-next-line camelcase
      flattened_targets: paddedCalls.map(call => call.to.toField()),
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
    ...payload.flattened_args_hashes,
    ...payload.flattened_selectors,
    ...payload.flattened_targets,
    payload.nonce,
  ];
}
