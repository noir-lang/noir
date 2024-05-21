import { AztecAddress, type Fr, FunctionData } from '@aztec/circuits.js';

/** A request to call a function on a contract from a given address. */
export type FunctionCall = {
  /** The recipient contract */
  to: AztecAddress;
  /** The function being called */
  functionData: FunctionData;
  /** Whether this call can make modifications to state or not */
  isStatic: boolean;
  /** The encoded args */
  args: Fr[];
};

/**
 * Creates an empty function call.
 * @returns an empty function call.
 */
export function emptyFunctionCall() {
  return {
    to: AztecAddress.ZERO,
    functionData: FunctionData.empty(),
    isStatic: false,
    args: [],
  };
}
