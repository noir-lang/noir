import { Abi, AbiParameter } from '@noir-lang/noirc_abi';

/**
 * A contract event.
 */
export interface EventAbi {
  /**
   * The event name.
   */
  name: string;
  /**
   * Fully qualified name of the event.
   */
  path: string;
  /**
   * The fields of the event.
   */
  fields: AbiParameter[];
}

/** The Noir function types. */
export type NoirFunctionType = 'Open' | 'Secret' | 'Unconstrained';

/**
 * The compilation result of an Noir function.
 */
export interface NoirFunctionEntry {
  /** The name of the function. */
  name: string;
  /** The type of the function. */
  function_type: NoirFunctionType;
  /** Whether the function is internal. */
  is_internal: boolean;
  /** The ABI of the function. */
  abi: Abi;
  /** The bytecode of the function in base64. */
  bytecode: string;
  /** The proving key. */
  proving_key: string;
  /** The verification key. */
  verification_key: string;
}

/**
 * The compilation result of an Noir contract.
 */
export interface NoirCompiledContract {
  /** The name of the contract. */
  name: string;
  /** Compilation backend. */
  backend: string;
  /** The functions of the contract. */
  functions: NoirFunctionEntry[];
  /** The events of the contract */
  events: EventAbi[];
}

/**
 * The compilation result of an Noir contract.
 */
export interface NoirCompiledCircuit {
  /** The hash of the circuit. */
  hash?: number;
  /** Compilation backend. */
  backend: string;
  /**
   * The ABI of the function.
   */
  abi: Abi;
  /** The bytecode of the circuit in base64. */
  bytecode: string;
}

/**
 * A file ID. It's assigned during compilation.
 */
export type FileId = number;

/**
 * A pointer to a specific section of the source code.
 */
export interface SourceCodeLocation {
  /**
   * The section of the source code.
   */
  span: {
    /**
     * The byte where the section starts.
     */
    start: number;
    /**
     * The byte where the section ends.
     */
    end: number;
  };
  /**
   * The source code file pointed to.
   */
  file: FileId;
}

/**
 * The location of an opcode in the bytecode.
 * It's a string of the form `{acirIndex}` or `{acirIndex}:{brilligIndex}`.
 */
export type OpcodeLocation = string;

/**
 * The debug information for a given function.
 */
export interface DebugInfo {
  /**
   * A map of the opcode location to the source code location.
   */
  locations: Record<OpcodeLocation, SourceCodeLocation[]>;
}

/**
 * Maps a file ID to its metadata for debugging purposes.
 */
export type DebugFileMap = Record<
  FileId,
  {
    /**
     * The source code of the file.
     */
    source: string;
    /**
     * The path of the file.
     */
    path: string;
  }
>;

/**
 * The debug metadata of an Noir contract.
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

  /**
   * The artifact that contains the debug metadata about the contract.
   */
  debug?: NoirDebugMetadata;
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

  /**
   * The artifact that contains the debug metadata about the contract.
   */
  debug?: NoirDebugMetadata;
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
