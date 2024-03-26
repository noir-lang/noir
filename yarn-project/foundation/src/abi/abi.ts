import { inflate } from 'pako';

import { type FunctionSelector } from './function_selector.js';

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
  type: ABIType;
}

/**
 * Indicates whether a parameter is public or secret/private.
 */
export enum ABIParameterVisibility {
  PUBLIC = 'public',
  SECRET = 'secret',
}

/**
 * A function parameter.
 */
export interface ABIParameter extends ABIVariable {
  /**
   * Indicates whether a parameter is public or secret/private.
   */
  visibility: ABIParameterVisibility;
}

/**
 * A basic type.
 */
export interface BasicType<T extends string> {
  /**
   * The kind of the type.
   */
  kind: T;
}

/**
 * A variable type.
 */
export type ABIType = BasicType<'field'> | BasicType<'boolean'> | IntegerType | ArrayType | StringType | StructType;

/**
 * An integer type.
 */
export interface IntegerType extends BasicType<'integer'> {
  /**
   * The sign of the integer.
   */
  sign: string;
  /**
   * The width of the integer in bits.
   */
  width: number;
}

/**
 * An array type.
 */
export interface ArrayType extends BasicType<'array'> {
  /**
   * The length of the array.
   */
  length: number;
  /**
   * The type of the array elements.
   */
  type: ABIType;
}

/**
 * A string type.
 */
export interface StringType extends BasicType<'string'> {
  /**
   * The length of the string.
   */
  length: number;
}

/**
 * A struct type.
 */
export interface StructType extends BasicType<'struct'> {
  /**
   * The fields of the struct.
   */
  fields: ABIVariable[];
  /**
   * Fully qualified name of the struct.
   */
  path: string;
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

/**
 * Aztec.nr function types.
 */
export enum FunctionType {
  SECRET = 'secret',
  OPEN = 'open',
  UNCONSTRAINED = 'unconstrained',
}

/**
 * The abi entry of a function.
 */
export interface FunctionAbi {
  /**
   * The name of the function.
   */
  name: string;
  /**
   * Whether the function is secret.
   */
  functionType: FunctionType;
  /**
   * Whether the function is internal.
   */
  isInternal: boolean;
  /**
   * Function parameters.
   */
  parameters: ABIParameter[];
  /**
   * The types of the return values.
   */
  returnTypes: ABIType[];
  /**
   * Whether the function is flagged as an initializer.
   */
  isInitializer: boolean;
}

/**
 * The artifact entry of a function.
 */
export interface FunctionArtifact extends FunctionAbi {
  /**
   * The ACIR bytecode of the function.
   */
  bytecode: Buffer;
  /**
   * The verification key of the function.
   */
  verificationKey?: string;
  /**
   * Maps opcodes to source code pointers
   */
  debugSymbols: string;
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
 * Defines artifact of a contract.
 */
export interface ContractArtifact {
  /**
   * The name of the contract.
   */
  name: string;

  /**
   * The version of compiler used to create this artifact
   */
  aztecNrVersion?: string;

  /**
   * The functions of the contract.
   */
  functions: FunctionArtifact[];
  /**
   * The events of the contract.
   */
  events: EventAbi[];

  /**
   * The map of file ID to the source code and path of the file.
   */
  fileMap: DebugFileMap;
}

/**
 * Debug metadata for a function.
 */
export interface FunctionDebugMetadata {
  /**
   * Maps opcodes to source code pointers
   */
  debugSymbols: DebugInfo;
  /**
   * Maps the file IDs to the file contents to resolve pointers
   */
  files: DebugFileMap;
}

/** A function artifact with optional debug metadata */
export interface FunctionArtifactWithDebugMetadata extends FunctionArtifact {
  /** Debug metadata for the function. */
  debug?: FunctionDebugMetadata;
}

/**
 * Gets a function artifact given its name or selector.
 */
export function getFunctionArtifact(
  artifact: ContractArtifact,
  functionNameOrSelector: string | FunctionSelector,
): FunctionArtifact {
  const functionArtifact = artifact.functions.find(f =>
    typeof functionNameOrSelector === 'string'
      ? f.name === functionNameOrSelector
      : functionNameOrSelector.equals(f.name, f.parameters),
  );
  if (!functionArtifact) {
    throw new Error(`Unknown function ${functionNameOrSelector}`);
  }
  return functionArtifact;
}

/** @deprecated Use getFunctionArtifact instead */
export function getFunctionArtifactWithSelector(artifact: ContractArtifact, selector: FunctionSelector) {
  return getFunctionArtifact(artifact, selector);
}

/**
 * Gets a function artifact including debug metadata given its name or selector.
 */
export function getFunctionArtifactWithDebugMetadata(
  artifact: ContractArtifact,
  functionNameOrSelector: string | FunctionSelector,
): FunctionArtifactWithDebugMetadata {
  const functionArtifact = getFunctionArtifact(artifact, functionNameOrSelector);
  const debugMetadata = getFunctionDebugMetadata(artifact, functionArtifact);
  return { ...functionArtifact, debug: debugMetadata };
}

/**
 * Gets the debug metadata of a given function from the contract artifact
 * @param artifact - The contract build artifact
 * @param functionName - The name of the function
 * @returns The debug metadata of the function
 */
export function getFunctionDebugMetadata(
  contractArtifact: ContractArtifact,
  functionArtifact: FunctionArtifact,
): FunctionDebugMetadata | undefined {
  if (functionArtifact.debugSymbols && contractArtifact.fileMap) {
    const debugSymbols = JSON.parse(
      inflate(Buffer.from(functionArtifact.debugSymbols, 'base64'), { to: 'string', raw: true }),
    );
    return { debugSymbols, files: contractArtifact.fileMap };
  }
  return undefined;
}

/**
 * Returns an initializer from the contract, assuming there is at least one. If there are multiple initializers,
 * it returns the one named "constructor" or "initializer"; if there is none with that name, it returns the first
 * initializer it finds, prioritizing initializers with no arguments and then private ones.
 * @param contractArtifact - The contract artifact.
 * @returns An initializer function, or none if there are no functions flagged as initializers in the contract.
 */
export function getDefaultInitializer(contractArtifact: ContractArtifact): FunctionArtifact | undefined {
  const initializers = contractArtifact.functions.filter(f => f.isInitializer);
  return initializers.length > 1
    ? initializers.find(f => f.name === 'constructor') ??
        initializers.find(f => f.name === 'initializer') ??
        initializers.find(f => f.parameters?.length === 0) ??
        initializers.find(f => f.functionType === FunctionType.SECRET) ??
        initializers[0]
    : initializers[0];
}

/**
 * Returns an initializer from the contract.
 * @param initalizerNameOrArtifact - The name of the constructor, or the artifact of the constructor, or undefined
 * to pick the default initializer.
 */
export function getInitializer(
  contract: ContractArtifact,
  initalizerNameOrArtifact: string | undefined | FunctionArtifact,
): FunctionArtifact | undefined {
  if (typeof initalizerNameOrArtifact === 'string') {
    const found = contract.functions.find(f => f.name === initalizerNameOrArtifact);
    if (!found) {
      throw new Error(`Constructor method ${initalizerNameOrArtifact} not found in contract artifact`);
    } else if (!found.isInitializer) {
      throw new Error(`Method ${initalizerNameOrArtifact} is not an initializer`);
    }
    return found;
  } else if (initalizerNameOrArtifact === undefined) {
    return getDefaultInitializer(contract);
  } else {
    if (!initalizerNameOrArtifact.isInitializer) {
      throw new Error(`Method ${initalizerNameOrArtifact.name} is not an initializer`);
    }
    return initalizerNameOrArtifact;
  }
}
