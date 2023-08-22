import { ABIParameter, ABIType, DebugFileMap, DebugInfo } from '@aztec/foundation/abi';

/** The noir function types. */
type NoirFunctionType = 'Open' | 'Secret' | 'Unconstrained';

/** The ABI of a noir function. */
interface NoirFunctionAbi {
  /** The parameters of the function. */
  parameters: ABIParameter[];
  /** The witness indices of the parameters. Indexed by parameter name. */
  param_witnesses: Record<string, number[]>;
  /** The return type of the function. */
  return_type: ABIType;
  /** The witness indices of the return type. */
  return_witnesses: number[];
}

/**
 * The compilation result of a noir function.
 */
export interface NoirFunctionEntry {
  /** The name of the function. */
  name: string;
  /** The type of the function. */
  function_type: NoirFunctionType;
  /** Whether the function is internal. */
  is_internal: boolean;
  /** The ABI of the function. */
  abi: NoirFunctionAbi;
  /** The bytecode of the function in base64. */
  bytecode: string;
  /** The proving key. */
  proving_key: string;
  /** The verification key. */
  verification_key: string;
}

/**
 * The compilation result of a noir contract.
 */
export interface NoirCompiledContract {
  /** The name of the contract. */
  name: string;
  /** Compilation backend. */
  backend: string;
  /** The functions of the contract. */
  functions: NoirFunctionEntry[];
}

/**
 * The debug metadata of a noir contract.
 */
export interface NoirDebugMetadata {
  /**
   * The debug information for each function.
   */
  debug_symbols: DebugInfo[];
  /**
   * The map of file ID to the source code and path of the file.
   */
  file_map: DebugFileMap;
}

/**
 * The compilation artifacts of a given contract.
 */
export interface NoirCompilationArtifacts {
  /**
   * The compiled contract.
   */
  contract: NoirCompiledContract;
  /**
   * The artifact that contains the debug metadata about the contract.
   */
  debug?: NoirDebugMetadata;
}
