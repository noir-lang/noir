import { AztecAddress } from '../aztec-address/index.js';
import { Fr } from '../fields/index.js';
import { ABIParameter, type ABIType, ABIVariable, FunctionArtifact } from './abi.js';
import { isAztecAddressStruct } from './utils.js';

/**
 * The type of our decoded ABI.
 */
export type DecodedReturn = bigint | boolean | AztecAddress | DecodedReturn[] | { [key: string]: DecodedReturn };

/**
 * Decodes return values from a function call.
 * Missing support for integer and string.
 */
class ReturnValuesDecoder {
  constructor(private artifact: FunctionArtifact, private flattened: Fr[]) {}

  /**
   * Decodes a single return value from field to the given type.
   * @param abiType - The type of the return value.
   * @returns The decoded return value.
   */
  private decodeReturn(abiType: ABIType): DecodedReturn {
    switch (abiType.kind) {
      case 'field':
        return this.getNextField().toBigInt();
      case 'integer':
        if (abiType.sign === 'signed') {
          throw new Error('Unsupported type: signed integer');
        }
        return this.getNextField().toBigInt();
      case 'boolean':
        return !this.getNextField().isZero();
      case 'array': {
        const array = [];
        for (let i = 0; i < abiType.length; i += 1) {
          array.push(this.decodeReturn(abiType.type));
        }
        return array;
      }
      case 'struct': {
        const struct: { [key: string]: DecodedReturn } = {};
        if (isAztecAddressStruct(abiType)) {
          return new AztecAddress(this.getNextField().toBuffer());
        }

        for (const field of abiType.fields) {
          struct[field.name] = this.decodeReturn(field.type);
        }
        return struct;
      }
      case 'string': {
        const array = [];
        for (let i = 0; i < abiType.length; i += 1) {
          array.push(this.getNextField().toBigInt());
        }
        return array;
      }
      default:
        throw new Error(`Unsupported type: ${abiType}`);
    }
  }

  /**
   * Gets the next field in the flattened return values.
   * @returns The next field in the flattened return values.
   */
  private getNextField(): Fr {
    const field = this.flattened.shift();
    if (!field) {
      throw new Error('Not enough return values');
    }
    return field;
  }

  /**
   * Decodes all the return values for the given function ABI.
   * Aztec.nr support only single return value
   * The return value can however be simple types, structs or arrays
   * @returns The decoded return values.
   */
  public decode(): DecodedReturn {
    if (this.artifact.returnTypes.length > 1) {
      throw new Error('Multiple return values not supported');
    }
    if (this.artifact.returnTypes.length === 0) {
      return [];
    }
    return this.decodeReturn(this.artifact.returnTypes[0]);
  }
}

/**
 * Decodes return values from a function call.
 * @param abi - The ABI entry of the function.
 * @param returnValues - The decoded return values.
 * @returns
 */
export function decodeReturnValues(abi: FunctionArtifact, returnValues: Fr[]) {
  return new ReturnValuesDecoder(abi, returnValues.slice()).decode();
}

/**
 * Decodes the signature of a function from the name and parameters.
 */
export class FunctionSignatureDecoder {
  private separator: string;
  constructor(private name: string, private parameters: ABIParameter[], private includeNames = false) {
    this.separator = includeNames ? ', ' : ',';
  }

  /**
   * Decodes a single function parameter type for the function signature.
   * @param param - The parameter type to decode.
   * @returns A string representing the parameter type.
   */
  private getParameterType(param: ABIType): string {
    switch (param.kind) {
      case 'field':
        return 'Field';
      case 'integer':
        if (param.sign === 'signed') {
          throw new Error('Unsupported type: signed integer');
        }
        return `u${param.width}`;
      case 'boolean':
        return 'bool';
      case 'array':
        return `[${this.getParameterType(param.type)};${param.length}]`;
      case 'string':
        return `str<${param.length}>`;
      case 'struct':
        return `(${param.fields.map(field => `${this.decodeParameter(field)}`).join(this.separator)})`;
      default:
        throw new Error(`Unsupported type: ${param}`);
    }
  }

  /**
   * Decodes a single function parameter for the function signature.
   * @param param - The parameter to decode.
   * @returns A string representing the parameter type and optionally its name.
   */
  private decodeParameter(param: ABIVariable): string {
    const type = this.getParameterType(param.type);
    return this.includeNames ? `${param.name}: ${type}` : type;
  }

  /**
   * Decodes all the parameters and build the function signature
   * @returns The function signature.
   */
  public decode(): string {
    return `${this.name}(${this.parameters.map(param => this.decodeParameter(param)).join(this.separator)})`;
  }
}

/**
 * Decodes a function signature from the name and parameters.
 * @param name - The name of the function.
 * @param parameters - The parameters of the function.
 * @returns - The function signature.
 */
export function decodeFunctionSignature(name: string, parameters: ABIParameter[]) {
  return new FunctionSignatureDecoder(name, parameters).decode();
}

/**
 * Decodes a function signature from the name and parameters including parameter names.
 * @param name - The name of the function.
 * @param parameters - The parameters of the function.
 * @returns - The user-friendly function signature.
 */
export function decodeFunctionSignatureWithParameterNames(name: string, parameters: ABIParameter[]) {
  return new FunctionSignatureDecoder(name, parameters, true).decode();
}
