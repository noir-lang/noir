import { Abi, AbiType } from '@noir-lang/types';

/**
 * A basic value.
 */
export interface BasicValue<T extends string, V> {
  /**
   * The kind of the value.
   */
  kind: T;
  value: V;
}

/**
 * An exported value.
 */
export type AbiValue =
  | BasicValue<'boolean', boolean>
  | BasicValue<'string', string>
  | BasicValue<'array', AbiValue[]>
  | TupleValue
  | IntegerValue
  | StructValue;

export type TypedStructFieldValue<T> = { name: string; value: T };

export interface StructValue {
  kind: 'struct';
  fields: TypedStructFieldValue<AbiValue>[];
}

export interface TupleValue {
  kind: 'tuple';
  fields: AbiValue[];
}

export interface IntegerValue extends BasicValue<'integer', string> {
  sign: boolean;
}

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
 * The compilation result of an Noir function.
 */
export interface NoirFunctionEntry {
  /** The name of the function. */
  name: string;
  /** Whether the function is unconstrained. */
  is_unconstrained: boolean;
  /** The custom attributes applied to the function. */
  custom_attributes: string[];
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

  outputs: {
    structs: Record<string, AbiType[]>;
    globals: Record<string, AbiValue[]>;
  };
  /** The map of file ID to the source code and path of the file. */
  file_map: DebugFileMap;
}

/**
 * The compilation result of an Noir contract.
 */
export interface ProgramArtifact {
  /** Version of noir used for the build. */
  noir_version: string;
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
 */
export type OpcodeLocation = number;

export type BrilligFunctionId = number;

export type OpcodeToLocationsMap = Record<OpcodeLocation, SourceCodeLocation[]>;

export interface LocationNodeDebugInfo {
  value: SourceCodeLocation;
}

export interface LocationTree {
  locations: Array<LocationNodeDebugInfo>;
}

/**
 * The debug information for a given function.
 */
export interface DebugInfo {
  /**
   * A map of the opcode location to the source code location.
   */
  location_tree: LocationTree;
  /**
   * For each Brillig function, we have a map of the opcode location to the source code location.
   */
  brillig_locations: Record<BrilligFunctionId, OpcodeToLocationsMap>;
}

/**
 * The debug information for a given program.
 */
export interface ProgramDebugInfo {
  /**
   * An array that maps to each function of a program.
   */
  debug_infos: Array<DebugInfo>;
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
