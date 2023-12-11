import { ABIParameter, ABIType } from './types/abi';

/** The Aztec.nr function types. */
type NoirFunctionType = 'Open' | 'Secret' | 'Unconstrained';

/** The ABI of an Aztec.nr function. */
export interface NoirFunctionAbi {
  /** The parameters of the function. */
  parameters: ABIParameter[];
  /** The witness indices of the parameters. Indexed by parameter name. */
  param_witnesses: {
    /** input */
    input: {
      /** start */
      start: number;
      /** end */
      end: number;
    }[];
  };
  /** The return type of the function. */
  return_type: ABIType;
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
  proving_key: string;
  /** The verification key. */
  verification_key: string;
}

/**
 * The compilation result of a Noir circuit.
 */
export interface NoirCompiledCircuit {
  /** The hash of the circuit. */
  hash?: number;
  /** Compilation backend. */
  backend: string;
  /**
   * The ABI of the function.
   */
  abi: NoirFunctionAbi;
  /** The bytecode of the circuit in base64. */
  bytecode: string;
}

/**
 * Defines artifact of a contract.
 */
export interface ProgramArtifact {
  /**
   * version of noir used to compile
   */
  noir_version?: string;
  /**
   * the name of the project, read from Nargo.toml
   */
  name?: string;
  /**
   * The hash of the contract.
   */
  hash?: number;

  /**
   * The compilation backend of the artifact.
   */
  backend: string;

  /**
   * The abi of the program.
   */
  abi: any; // TODO: type
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
   * The compiled circuit.
   */
  program: NoirCompiledCircuit;
}

/**
 * Check if it has Program unique property
 */
export function isNoirProgramCompilationArtifacts(artifact: NoirProgramCompilationArtifacts) {
  return artifact.program !== undefined;
}
