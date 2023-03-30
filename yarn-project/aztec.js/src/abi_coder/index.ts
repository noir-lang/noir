import { ABIParameter } from '@aztec/noir-contracts';

export * from './hex_string.js';

function pack(parameter: ABIParameter, value: any) {
  return Buffer.alloc(32);
}

export function encodeParameters(parameters: ABIParameter[], args: any[]) {
  if (parameters.length !== args.length) {
    throw new Error(`Incorrect number of args. Expect ${parameters.length}. Got ${args.length}.`);
  }

  return parameters.map((p, i) => pack(p, args[i]));
}
