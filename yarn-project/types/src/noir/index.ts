import {
  ABIParameter,
  ABIParameterVisibility,
  ABIType,
  DebugFileMap,
  DebugInfo,
  EventAbi,
} from '@aztec/foundation/abi';

/** The Aztec.nr function types. */
type NoirFunctionType = 'Open' | 'Secret' | 'Unconstrained';

/** The witness indices of the parameters. */
type ParamWitnessIndices = { /** Start */ start: number; /** End */ end: number };

/** The ABI of an Aztec.nr function. */
export interface NoirFunctionAbi {
  /** The parameters of the function. */
  parameters: ABIParameter[];
  /** The witness indices of the parameters. Indexed by parameter name. */
  param_witnesses: { [key: string]: undefined | ParamWitnessIndices[] };
  /** The return type of the function. */
  return_type: {
    /**
     * The type of the return value.
     */
    abi_type: ABIType;
    /**
     * The visibility of the return value.
     */
    visibility: ABIParameterVisibility;
  };
  /** The witness indices of the return type. */
  return_witnesses: number[];
}

/**
 * The compilation result of an Aztec.nr function.
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
  proving_key?: string;
  /** The verification key. */
  verification_key?: string;
  /** The debug information, compressed and base64 encoded. */
  debug_symbols: string;
}

/**
 * The compilation result of an Aztec.nr contract.
 */
export interface NoirCompiledContract {
  /** The name of the contract. */
  name: string;
  /** The functions of the contract. */
  functions: NoirFunctionEntry[];
  /** The events of the contract */
  events: EventAbi[];
  /** The map of file ID to the source code and path of the file. */
  file_map: DebugFileMap;
}

/**
 * The compilation result of an Aztec.nr contract.
 */
export interface NoirCompiledCircuit {
  /** The hash of the circuit. */
  hash?: number;
  /**
   * The ABI of the function.
   */
  abi: NoirFunctionAbi;
  /** The bytecode of the circuit in base64. */
  bytecode: string;
  /** The debug information, compressed and base64 encoded. */
  debug_symbols: string;
  /** The map of file ID to the source code and path of the file. */
  file_map: DebugFileMap;
}

/**
 * The debug metadata of an Aztec.nr contract.
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
export interface NoirContractCompilationArtifacts {
  /**
   * The compiled contract.
   */
  contract: NoirCompiledContract;
}

/**
 * The compilation artifacts of a given program.
 */
export interface NoirProgramCompilationArtifacts {
  /**
   * not part of the compilation output, injected later
   */
  name: string;
  /**
   * The compiled contract.
   */
  program: NoirCompiledCircuit;
}

/**
 * output of Noir Wasm compilation, can be for a contract or lib/binary
 */
export type NoirCompilationResult = NoirContractCompilationArtifacts | NoirProgramCompilationArtifacts;

/**
 * Check if it has Contract unique property
 */
export function isNoirContractCompilationArtifacts(
  artifact: NoirCompilationResult,
): artifact is NoirContractCompilationArtifacts {
  return (artifact as NoirContractCompilationArtifacts).contract !== undefined;
}

/**
 * Check if it has Contract unique property
 */
export function isNoirProgramCompilationArtifacts(
  artifact: NoirCompilationResult,
): artifact is NoirProgramCompilationArtifacts {
  return (artifact as NoirProgramCompilationArtifacts).program !== undefined;
}
