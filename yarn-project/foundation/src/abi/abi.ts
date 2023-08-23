import { inflate } from 'pako';

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
}

/**
 * Noir function types.
 */
export enum FunctionType {
  SECRET = 'secret',
  OPEN = 'open',
  UNCONSTRAINED = 'unconstrained',
}

/**
 * The ABI entry of a function.
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
   * The ACIR bytecode of the function.
   */
  bytecode: string;
  /**
   * The verification key of the function.
   */
  verificationKey?: string;
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
 * The debug metadata of an ABI.
 */
export interface DebugMetadata {
  /**
   * The DebugInfo object, deflated as JSON, compressed using gzip and serialized with base64.
   */
  debugSymbols: string[];
  /**
   * The map of file ID to the source code and path of the file.
   */
  fileMap: DebugFileMap;
}

/**
 * Defines ABI of a contract.
 */
export interface ContractAbi {
  /**
   * The name of the contract.
   */
  name: string;
  /**
   * The functions of the contract.
   */
  functions: FunctionAbi[];

  /**
   * The debug metadata of the contract.
   * It's used to include the relevant source code section when a constraint is not met during simulation.
   */
  debug?: DebugMetadata;
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

/**
 * Gets the debug metadata of a given function from the contract abi
 * @param abi - The contract abi
 * @param functionName - The name of the function
 * @returns The debug metadata of the function
 */
export function getFunctionDebugMetadata(abi: ContractAbi, functionName: string): FunctionDebugMetadata | undefined {
  const functionIndex = abi.functions.findIndex(f => f.name === functionName);
  if (abi.debug && functionIndex !== -1) {
    const debugSymbols = JSON.parse(
      inflate(Buffer.from(abi.debug.debugSymbols[functionIndex], 'base64'), { to: 'string' }),
    );
    const files = abi.debug.fileMap;
    return {
      debugSymbols,
      files,
    };
  }
  return undefined;
}
