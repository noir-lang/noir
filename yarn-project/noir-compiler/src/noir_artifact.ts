import { ABIParameter, ABIType } from '@aztec/foundation/abi';

/** The noir function types. */
type NoirFunctionType = 'Open' | 'Secret' | 'Unconstrained';

/** The ABI of a noir function. */
interface NoirFunctionAbi {
  /** The parameters of the function. */
  parameters: ABIParameter[];
  /**
   * The witness indices of the parameters. Indexed by parameter name.
   */
  param_witnesses: Record<string, number[]>;
  /** The return type of the function. */
  return_type: ABIType;
  /** The witness indices of the return type. */
  return_witnesses: number[];
}

/**
 * The compilation result of a noir function.
 */
interface NoirFunctionEntry {
  /** The name of the function. */
  name: string;
  /** The type of the function. */
  function_type: NoirFunctionType;
  /**
   * Whether the function is internal.
   */
  is_internal: boolean;
  /** The ABI of the function. */
  abi: NoirFunctionAbi;
  /** The bytecode of the function. */
  bytecode: Uint8Array;
}

/**
 * The compilation result of a noir contract.
 */
export interface NoirCompiledContract {
  /** The name of the contract. */
  name: string;
  /** The functions of the contract. */
  functions: NoirFunctionEntry[];
}
