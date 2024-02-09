import { Abi, AbiType } from '@noir-lang/types';

/**
 * A named type.
 */
export interface ABIVariable {
  /**
   * The name of the variable.
   */
  name: string;
  /**
   * The type of the variable.
   */
  type: AbiType;
}

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
  fields: ABIVariable[];
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
  /** The debug information, compressed and base64 encoded. */
  debug_symbols: string;
}

/**
 * The compilation result of an Noir contract.
 */
export interface ContractArtifact {
  /** The name of the contract. */
  name: string;
  /** Version of noir used for the build. */
  noir_version: string;
  /** The functions of the contract. */
  functions: NoirFunctionEntry[];
  /** The events of the contract */
  events: EventAbi[];
  /** The map of file ID to the source code and path of the file. */
  file_map: DebugFileMap;
}

/**
 * The compilation result of an Noir contract.
 */
export interface ProgramArtifact {
  /** The hash of the circuit. */
  hash?: number;
  /** * The ABI of the function. */
  abi: Abi;
  /** The bytecode of the circuit in base64. */
  bytecode: string;
  /** The debug information, compressed and base64 encoded. */
  debug_symbols: string;
  /** The map of file ID to the source code and path of the file. */
  file_map: DebugFileMap;
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

/** Compilation warning */
export type Warning = unknown;

/**
 * The compilation artifacts of a given contract.
 */
export interface ContractCompilationArtifacts {
  /**
   * The compiled contract.
   */
  contract: ContractArtifact;

  /** Compilation warnings. */
  warnings: Warning[];
}

/**
 * The compilation artifacts of a given program.
 */
export interface ProgramCompilationArtifacts {
  /**
   * not part of the compilation output, injected later
   */
  name: string;
  /**
   * The compiled contract.
   */
  program: ProgramArtifact;

  /** Compilation warnings. */
  warnings: Warning[];
}

/**
 * output of Noir Wasm compilation, can be for a contract or lib/binary
 */
export type CompilationResult = ContractCompilationArtifacts | ProgramCompilationArtifacts;

/**
 * Check if it has Contract unique property
 */
export function isContractCompilationArtifacts(artifact: CompilationResult): artifact is ContractCompilationArtifacts {
  return (artifact as ContractCompilationArtifacts).contract !== undefined;
}

/**
 * Check if it has Contract unique property
 */
export function isProgramCompilationArtifacts(artifact: CompilationResult): artifact is ProgramCompilationArtifacts {
  return (artifact as ProgramCompilationArtifacts).program !== undefined;
}
