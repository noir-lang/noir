import * as errors from './errors.js';
import { EthAddress } from '@aztec/foundation';
import { toBigIntBE, toBufferBE } from '../../../../bigint_buffer/index.js';
import { bufferToHex, hexToBuffer } from '../../../../hex_string/index.js';

const NegativeOne = BigInt(-1);
const Zero = BigInt(0);
const MaxUint256 = BigInt('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff');

/**
 * Type representing a parameter for Ethereum contract events and functions.
 */
type ParamType = {
  /**
   * The name of the parameter or variable.
   */
  name?: string;
  /**
   * The data type for a specific parameter.
   */
  type: string;
  /**
   * Indicates whether the event parameter is indexed or not.
   */
  indexed?: boolean;
  /**
   * An array of component objects representing the structure and types of a tuple.
   */
  components?: Array<any>;
};

// type EventFragment = {
//   type: string;
//   name: string;

//   anonymous: boolean;

//   inputs: Array<ParamType>;
// };

// type FunctionFragment = {
//   type: string;
//   name?: string;

//   constant: boolean;

//   inputs: Array<ParamType>;
//   outputs?: Array<ParamType>;

//   payable: boolean;
//   stateMutability: string | null;

//   gas: bigint | null;
// };

const paramTypeBytes = new RegExp(/^bytes([0-9]*)$/);
const paramTypeNumber = new RegExp(/^(u?int)([0-9]*)$/);
const paramTypeArray = new RegExp(/^(.*)\[([0-9]*)\]$/);

///////////////////////////////////
// Parsing for Solidity Signatures

// const regexParen = new RegExp('^([^)(]*)\\((.*)\\)([^)(]*)$');
// const regexIdentifier = new RegExp('^[A-Za-z_][A-Za-z0-9_]*$');

/**
 * Verifies and transforms the given type string to its full description if necessary.
 * This function handles cases where the input type is a shorthand for specific types
 * like 'uint' or 'int', converting them into their complete representations, such as 'uint256' or 'int256'.
 * In all other cases, the input type is returned as-is.
 *
 * @param type - The data type string to be verified and transformed if necessary.
 * @returns A string representing the full description of the input type.
 */
function verifyType(type: string): string {
  // These need to be transformed to their full description
  if (type.match(/^uint($|[^1-9])/)) {
    type = 'uint256' + type.substring(4);
  } else if (type.match(/^int($|[^1-9])/)) {
    type = 'int256' + type.substring(3);
  }

  return type;
}

/**
 * Type representing the state of a parser for parsing and validating Solidity signatures.
 */
type ParseState = {
  /**
   * Indicates whether an array type is allowed.
   */
  allowArray?: boolean;
  /**
   * Determines if a name is allowed for the parameter.
   */
  allowName?: boolean;
  /**
   * Indicates whether parameters are allowed for the current node.
   */
  allowParams?: boolean;
  /**
   * Determines if type can be accepted or not.
   */
  allowType?: boolean;
  /**
   * Indicates whether the array is being read during parsing.
   */
  readArray?: boolean;
};

/**
 * Represents a node in the Abstract Syntax Tree (AST) during the parsing of Solidity signatures. It contains all necessary information about the current parsing state, such as type, name, parent, and components.
 */
type ParseNode = {
  /**
   * The parent node of the current parse tree.
   */
  parent?: any;
  /**
   * The data type of the parameter.
   */
  type?: string;
  /**
   * The name representing an identifiable entity.
   */
  name?: string;
  /**
   * Represents the current state of parsing in a given node.
   */
  state?: ParseState;
  /**
   * Indicates if the parameter is indexed in event logs.
   */
  indexed?: boolean;
  /**
   * An array of nested parameter types.
   */
  components?: Array<any>;
};

/**
 * Parses a parameter string into a ParamType object with its type, name, components, and indexed information.
 * This function supports parsing complex types, such as tuples and arrays, as well as simple types like uint, int, etc.
 * It also handles optional indexed property for event parameters.
 * Throws an error if there is an unexpected character or mismatched parentheses in the input param string.
 *
 * @param param - The parameter string to be parsed.
 * @param allowIndexed - Optional flag indicating whether to parse indexed property for event parameters.
 * @returns A ParamType object with the parsed information.
 */
function parseParam(param: string, allowIndexed?: boolean): ParamType {
  /**
   * Throws a custom error with the specified reason, code, and additional error information.
   * This function is used to generate standardized error messages for better error handling
   * and debugging throughout the codebase.
   *
   * @param reason - The main reason for the error being thrown.
   * @param code - The error code associated with the particular type of error.
   * @param params - An optional object containing any additional information related to the error.
   * @throws Error - A custom error with the provided details.
   */
  function throwError(i: number) {
    throw new Error('unexpected character "' + param[i] + '" at position ' + i + ' in "' + param + '"');
  }

  const parent: ParseNode = { type: '', name: '', state: { allowType: true } };
  let node: any = parent;

  for (let i = 0; i < param.length; i++) {
    const c = param[i];
    switch (c) {
      case '(':
        if (!node.state.allowParams) {
          throwError(i);
        }
        node.state.allowType = false;
        node.type = verifyType(node.type);
        node.components = [{ type: '', name: '', parent: node, state: { allowType: true } }];
        node = node.components[0];
        break;

      case ')': {
        delete node.state;
        if (allowIndexed && node.name === 'indexed') {
          node.indexed = true;
          node.name = '';
        }
        node.type = verifyType(node.type);

        const child = node;
        node = node.parent;
        if (!node) {
          throwError(i);
        }
        delete child.parent;
        node.state.allowParams = false;
        node.state.allowName = true;
        node.state.allowArray = true;
        break;
      }
      case ',': {
        delete node.state;
        if (allowIndexed && node.name === 'indexed') {
          node.indexed = true;
          node.name = '';
        }
        node.type = verifyType(node.type);

        const sibling: ParseNode = { type: '', name: '', parent: node.parent, state: { allowType: true } };
        node.parent.components.push(sibling);
        delete node.parent;
        node = sibling;
        break;
      }
      // Hit a space...
      case ' ':
        // If reading type, the type is done and may read a param or name
        if (node.state.allowType) {
          if (node.type !== '') {
            node.type = verifyType(node.type);
            delete node.state.allowType;
            node.state.allowName = true;
            node.state.allowParams = true;
          }
        }

        // If reading name, the name is done
        if (node.state.allowName) {
          if (node.name !== '') {
            if (allowIndexed && node.name === 'indexed') {
              node.indexed = true;
              node.name = '';
            } else {
              node.state.allowName = false;
            }
          }
        }

        break;

      case '[':
        if (!node.state.allowArray) {
          throwError(i);
        }

        node.type += c;

        node.state.allowArray = false;
        node.state.allowName = false;
        node.state.readArray = true;
        break;

      case ']':
        if (!node.state.readArray) {
          throwError(i);
        }

        node.type += c;

        node.state.readArray = false;
        node.state.allowArray = true;
        node.state.allowName = true;
        break;

      default:
        if (node.state.allowType) {
          node.type += c;
          node.state.allowParams = true;
          node.state.allowArray = true;
        } else if (node.state.allowName) {
          node.name += c;
          delete node.state.allowArray;
        } else if (node.state.readArray) {
          node.type += c;
        } else {
          throwError(i);
        }
    }
  }

  if (node.parent) {
    throw new Error('unexpected eof');
  }

  delete parent.state;

  if (allowIndexed && node.name === 'indexed') {
    node.indexed = true;
    node.name = '';
  }

  parent.type = verifyType(parent.type!);

  return <ParamType>parent;
}

// @TODO: Better return type
// function parseSignatureEvent(fragment: string): EventFragment {
//   var abi: EventFragment = {
//     anonymous: false,
//     inputs: [],
//     name: '',
//     type: 'event',
//   };

//   var match = fragment.match(regexParen);
//   if (!match) {
//     throw new Error('invalid event: ' + fragment);
//   }

//   abi.name = match[1].trim();

//   splitNesting(match[2]).forEach(function (param) {
//     param = parseParam(param, true);
//     param.indexed = !!param.indexed;
//     abi.inputs.push(param);
//   });

//   match[3].split(' ').forEach(function (modifier) {
//     switch (modifier) {
//       case 'anonymous':
//         abi.anonymous = true;
//         break;
//       case '':
//         break;
//       default:
//         console.log('unknown modifier: ' + modifier);
//     }
//   });

//   if (abi.name && !abi.name.match(regexIdentifier)) {
//     throw new Error('invalid identifier: "' + abi.name + '"');
//   }

//   return abi;
// }

// function parseSignatureFunction(fragment: string): FunctionFragment {
//   var abi: FunctionFragment = {
//     constant: false,
//     gas: null,
//     inputs: [],
//     name: '',
//     outputs: [],
//     payable: false,
//     stateMutability: null, // @TODO: Should this be initialized to 'nonpayable'?
//     type: 'function',
//   };

//   let comps = fragment.split('@');
//   if (comps.length !== 1) {
//     if (comps.length > 2) {
//       throw new Error('invalid signature');
//     }
//     if (!comps[1].match(/^[0-9]+$/)) {
//       throw new Error('invalid signature gas');
//     }
//     abi.gas = BigInt(comps[1]);
//     fragment = comps[0];
//   }

//   comps = fragment.split(' returns ');
//   var left = comps[0].match(regexParen);
//   if (!left) {
//     throw new Error('invalid signature');
//   }

//   abi.name = left[1].trim();
//   if (!abi.name.match(regexIdentifier)) {
//     throw new Error('invalid identifier: "' + left[1] + '"');
//   }

//   splitNesting(left[2]).forEach(function (param) {
//     abi.inputs.push(parseParam(param));
//   });

//   left[3].split(' ').forEach(function (modifier) {
//     switch (modifier) {
//       case 'constant':
//         abi.constant = true;
//         break;
//       case 'payable':
//         abi.payable = true;
//         abi.stateMutability = 'payable';
//         break;
//       case 'pure':
//         abi.constant = true;
//         abi.stateMutability = 'pure';
//         break;
//       case 'view':
//         abi.constant = true;
//         abi.stateMutability = 'view';
//         break;
//       case 'external':
//       case 'public':
//       case '':
//         break;
//       default:
//         console.log('unknown modifier: ' + modifier);
//     }
//   });

//   // We have outputs
//   if (comps.length > 1) {
//     var right: any = comps[1].match(regexParen);
//     if (right[1].trim() != '' || right[3].trim() != '') {
//       throw new Error('unexpected tokens');
//     }

//     splitNesting(right[2]).forEach(function (param) {
//       abi.outputs!.push(parseParam(param));
//     });
//   }

//   if (abi.name === 'constructor') {
//     abi.type = 'constructor';

//     if (abi.outputs!.length) {
//       throw new Error('constructor may not have outputs');
//     }

//     delete abi.name;
//     delete abi.outputs;
//   }

//   return abi;
// }

// export function parseParamType(type: string): ParamType {
//   return parseParam(type, true);
// }

// // @TODO: Allow a second boolean to expose names
// export function formatParamType(paramType: ParamType): string {
//   return getParamCoder(paramType).type;
// }

// // @TODO: Allow a second boolean to expose names and modifiers
// export function formatSignature(fragment: EventFragment | FunctionFragment): string {
//   return fragment.name + '(' + fragment.inputs.map(i => formatParamType(i)).join(',') + ')';
// }

// export function parseSignature(fragment: string): EventFragment | FunctionFragment {
//   if (typeof fragment === 'string') {
//     // Make sure the "returns" is surrounded by a space and all whitespace is exactly one space
//     fragment = fragment.replace(/\(/g, ' (').replace(/\)/g, ') ').replace(/\s+/g, ' ');
//     fragment = fragment.trim();

//     if (fragment.substring(0, 6) === 'event ') {
//       return parseSignatureEvent(fragment.substring(6).trim());
//     } else {
//       if (fragment.substring(0, 9) === 'function ') {
//         fragment = fragment.substring(9);
//       }
//       return parseSignatureFunction(fragment.trim());
//     }
//   }

//   throw new Error('unknown signature');
// }

///////////////////////////////////
// Coders
/**
 * Represents the result of a decoding operation, containing both the decoded value and the number of bytes consumed during the process.
 * This type is used to return information from functions that decode binary data according to specific data types or encoding schemes.
 */
type DecodedResult<T = any> = {
  /**
   * The number of bytes consumed during decoding.
   */
  consumed: number;
  /**
   * The actual data value for the corresponding coder type.
   */
  value: T;
};
/**
 * The Coder class is an abstract base class that provides encoding and decoding functionality
 * for specific types in the Ethereum ABI (Application Binary Interface) format. It handles the
 * conversion of Solidity types to JavaScript types and vice versa, allowing for easy interaction
 * with Ethereum smart contracts. Each derived Coder class corresponds to a specific Solidity type,
 * implementing the necessary logic for encoding and decoding values of that type.
 */
abstract class Coder {
  readonly name: string;
  readonly type: string;
  readonly localName: string;
  readonly dynamic: boolean;
  constructor(name: string, type: string, localName = '', dynamic: boolean) {
    this.name = name;
    this.type = type;
    this.localName = localName;
    this.dynamic = dynamic;
  }

  /**
   * Encode the given value using the coder's type and rules.
   * The function takes a value as input, processes it according to the specific
   * coder implementation and returns a buffer containing the encoded value.
   * Throws an error if the input value is not valid for the coder's type.
   *
   * @param value - The value to be encoded.
   * @returns A Buffer containing the encoded value.
   */
  abstract encode(value: any): Buffer;
  /**
   * Decodes the given data buffer at the specified offset using the coder's type and properties.
   * Returns an object containing the number of bytes consumed during decoding and the decoded value.
   * Throws an error if there is insufficient data or any issues encountered during the decoding process.
   *
   * @param data - The input data buffer to be decoded.
   * @param offset - The starting position in the data buffer where decoding should begin.
   * @returns A DecodedResult object with the 'consumed' and 'value' properties.
   */
  abstract decode(data: Buffer, offset: number): DecodedResult;
}

/**
 * Clones the functionality of an existing Coder, but without a localName.
 */
class CoderAnonymous extends Coder {
  constructor(private coder: Coder) {
    super(coder.name, coder.type, undefined, coder.dynamic);
  }
  /**
   * Encode the given value into a Buffer based on the coder type.
   * This function handles various data types such as numbers, booleans, fixed bytes, and strings.
   * Throws an error if the input value is invalid or not compatible with the coder type.
   *
   * @param value - The value to be encoded according to the coder type.
   * @returns A Buffer containing the encoded value.
   */
  encode(value: any): Buffer {
    return this.coder.encode(value);
  }
  /**
   * Decodes the given data starting from the specified offset using the associated coder.
   * Returns an object containing the consumed bytes and the decoded value.
   * Throws an error if there is insufficient data for decoding or any other issue occurs during decoding.
   *
   * @param data - The buffer containing the encoded data to be decoded.
   * @param offset - The position in the buffer where the decoding should start.
   * @returns An object with 'consumed' property indicating the number of bytes consumed during decoding,
   *          and 'value' property holding the decoded value.
   */
  decode(data: Buffer, offset: number): DecodedResult {
    return this.coder.decode(data, offset);
  }
}

/**
 * CoderNull is a specific coder class for handling null values in encoding and decoding operations.
 * It extends the base Coder class and provides custom implementations for encoding and decoding null values
 * while complying with the Ethereum ABI specification. The encoded output for a null value is an empty buffer
 * and consumes no data during the decoding process, returning a null value as the result.
 */
class CoderNull extends Coder {
  constructor(localName: string) {
    super('null', '', localName, false);
  }

  /**
   * Encode the given value using the Coder's type and rules.
   * Converts various data types (boolean, number, string, etc.) into a Buffer representation
   * based on the ABI encoding specifications. Throws an error if the input value is invalid
   * or cannot be encoded according to the Coder's rules.
   *
   * @param value - The value to be encoded according to the Coder's type and rules.
   * @returns A Buffer containing the encoded representation of the input value.
   */
  encode(): Buffer {
    return Buffer.alloc(0);
  }

  /**
   * Decodes the provided data buffer starting from the given offset and returns an object with
   * the decoded value and the number of bytes consumed during the decoding process.
   * This function is used to decode ABI-encoded data for the specific coder type.
   *
   * @param data - The buffer containing the ABI-encoded data to be decoded.
   * @param offset - The index at which to start decoding in the data buffer.
   * @returns An object with the following properties:
   *   - `value`: The decoded value according to the coder type.
   *   - `consumed`: The number of bytes consumed during the decoding process.
   * @throws An error if there is insufficient data or the data is invalid for the coder type.
   */
  decode(data: Buffer, offset: number): DecodedResult {
    if (offset > data.length) {
      throw new Error('invalid null');
    }
    return {
      consumed: 0,
      value: null,
    };
  }
}

/**
 * CoderNumber is a class that represents numeric values in the Ethereum ABI encoding/decoding process.
 * It handles encoding and decoding of signed and unsigned integers of various sizes (number of bits).
 * This class supports fixed-size integer types like int8, uint16, int256, etc. The encoded output
 * is a Buffer of 32 bytes containing the value in big-endian format. When decoding, it returns the decoded
 * result as a JavaScript BigInt or number (based on the size) along with the number of bytes consumed.
 */
class CoderNumber extends Coder {
  readonly size: number;
  readonly signed: boolean;
  constructor(size: number, signed: boolean, localName: string) {
    const name = (signed ? 'int' : 'uint') + size * 8;
    super(name, name, localName, false);

    this.size = size;
    this.signed = signed;
  }

  /**
   * Encodes the given array of values according to the CoderArray rules.
   * The input value must be an array, and its length should match
   * the length specified in the CoderArray instance. If the length is dynamic,
   * any number of elements are allowed. Throws an error if the input is not an array
   * or its length does not match the expected length.
   *
   * @param value - The array of values to be encoded.
   * @returns A Buffer containing the encoded data.
   */
  encode(value: any): Buffer {
    try {
      let v = BigInt(value);
      if (this.signed) {
        let bounds = BigInt(1) << BigInt(this.size * 8 - 1);
        if (v > bounds) {
          throw new Error(`out-of-bounds: ${v} > ${bounds}`);
        }
        bounds = (bounds + 1n) * NegativeOne;
        if (v < bounds) {
          throw new Error(`out-of-bounds: ${v} < ${bounds}`);
        }
      } else if (v < Zero || v >= BigInt(1) << BigInt(this.size * 8)) {
        throw new Error(`out-of-bounds: 0 <= ${v} <= ${BigInt(1) << BigInt(this.size * 8)} `);
      }

      if (this.signed && v < 0) {
        // Compute twos complement.
        const absV = v * -1n;
        v = MaxUint256 - absV + 1n;
      }

      return toBufferBE(v, 32);
    } catch (error: any) {
      return errors.throwError('invalid number value', errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: this.name,
        value: value,
        msg: error.message,
        size: this.size,
      });
    }
  }

  /**
   * Decodes the provided data buffer at the specified offset using the current coder instance.
   * Consumes a certain number of bytes from the data buffer and returns the decoded value along with the consumed byte count.
   * Throws an error if there is insufficient data or any issues while decoding the given data.
   *
   * @param data - The data buffer to decode.
   * @param offset - The starting offset in the data buffer for decoding.
   * @returns An object containing the decoded value and the number of bytes consumed from the data buffer.
   */
  decode(data: Buffer, offset: number): DecodedResult<bigint | number> {
    if (data.length < offset + 32) {
      errors.throwError('insufficient data for ' + this.name + ' type', errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: this.name,
        value: bufferToHex(data.subarray(offset, offset + 32)),
      });
    }
    const junkLength = 32 - this.size;
    let value = toBigIntBE(Buffer.from(data.subarray(offset + junkLength, offset + 32)));
    if (this.signed && value > BigInt(1) << BigInt(this.size * 8 - 1)) {
      value = value - MaxUint256 - 1n;
    }

    return {
      consumed: 32,
      value: this.size <= 6 ? Number(value) : value,
    };
  }
}
const uint256Coder = new CoderNumber(32, false, 'none');

/**
 * CoderBoolean is a class that represents the 'bool' data type in Ethereum ABI encoding.
 * It provides methods to encode and decode boolean values into their binary representation
 * for use in Ethereum function calls and event logs. The class extends the abstract Coder class,
 * inheriting its properties and methods while also implementing custom logic for handling boolean types.
 * Instances of this class can be used to encode and decode boolean data, ensuring proper
 * formatting and compatibility with the Ethereum blockchain.
 */
class CoderBoolean extends Coder {
  constructor(localName: string) {
    super('bool', 'bool', localName, false);
  }

  /**
   * Encodes the given value using the appropriate Coder, resulting in a Buffer.
   * The encoded data can be later decoded using the corresponding 'decode' function.
   * Throws an error if the input value is invalid or not compatible with the Coder type.
   *
   * @param value - The value to be encoded according to the Coder's type.
   * @returns A Buffer containing the encoded data.
   */
  encode(value: boolean): Buffer {
    return uint256Coder.encode(value ? 1 : 0);
  }

  /**
   * Decodes the binary data from the provided buffer using the coder's associated type and offset.
   * Throws an error if there is insufficient data, or if the decoded value does not match the expected format.
   *
   * @param data - The buffer containing the binary data to decode.
   * @param offset - The starting position within the buffer to begin decoding.
   * @returns An object containing the number of bytes consumed and the decoded value of the specified type.
   */
  decode(data: Buffer, offset: number): DecodedResult<boolean> {
    try {
      const result = uint256Coder.decode(data, offset);
      return {
        consumed: result.consumed,
        value: !!Number(result.value),
      };
    } catch (error: any) {
      if (error.reason === 'insufficient data for uint256 type') {
        errors.throwError('insufficient data for boolean type', errors.INVALID_ARGUMENT, {
          arg: this.localName,
          coderType: 'boolean',
          value: error.value,
        });
      }
      throw error;
    }
  }
}

/**
 * The CoderFixedBytes class is responsible for encoding and decoding fixed-length byte arrays in ABI format.
 * It inherits from the Coder base class and provides methods to encode and decode values of 'bytes' type with a specified length.
 * The encoded data is compatible with Ethereum smart contracts, and this class plays a vital role in handling contract interactions.
 */
class CoderFixedBytes extends Coder {
  readonly length: number;
  constructor(length: number, localName: string) {
    const name = 'bytes' + length;
    super(name, name, localName, false);
    this.length = length;
  }

  /**
   * Encodes the given value using the coder and returns a Buffer.
   * This function handles various data types such as numbers, booleans, fixed bytes,
   * addresses, dynamic bytes, strings, arrays and tuples. It validates the input value
   * based on the coder properties and converts them into a suitable binary format
   * compatible with Ethereum encoding standards.
   *
   * @param value - The value to be encoded.
   * @returns A Buffer containing the encoded value.
   */
  encode(value: Buffer | string): Buffer {
    if (typeof value === 'string') {
      value = hexToBuffer(value);
    }

    try {
      if (value.length > this.length) {
        throw new Error(`incorrect data length`);
      }
    } catch (error: any) {
      errors.throwError('invalid ' + this.name + ' value', errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: this.name,
        value: error.value || value,
      });
    }

    return value;
  }

  /**
   * Decode the given data buffer starting from the specified offset using the implemented coder.
   * Returns an object containing the decoded value and the number of bytes consumed during decoding.
   * Throws an error if the input data is insufficient or invalid for the implemented coder type.
   *
   * @param data - The data buffer to be decoded.
   * @param offset - The starting index for decoding in the data buffer.
   * @returns DecodedResult object containing the decoded value and the consumed bytes count.
   */
  decode(data: Buffer, offset: number): DecodedResult {
    if (data.length < offset + 32) {
      errors.throwError('insufficient data for ' + name + ' type', errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: this.name,
        value: bufferToHex(data.subarray(offset, offset + 32)),
      });
    }

    return {
      consumed: 32,
      // TODO: Just return buffer.
      value: bufferToHex(Buffer.from(data.subarray(offset, offset + this.length))),
    };
  }
}

/**
 * The CoderAddress class extends the Coder base class, providing specific encoding and decoding
 * functionality for Ethereum addresses. It ensures that address values are properly formatted
 * and converted between different representations such as strings, BigNumber, and hexadecimal.
 * This class facilitates ABI encoding and decoding of contract function parameters and event logs
 * that involve Ethereum addresses.
 */
class CoderAddress extends Coder {
  constructor(localName: string) {
    super('address', 'address', localName, false);
  }

  /**
   * Encode the provided value according to the Coder type rules.
   * This function converts any given value into a Buffer format based on the specific
   * encoding rules defined for each Coder type, such as address, boolean, number, etc.
   * Throws an error if the input value is not compatible with the Coder type or if
   * any internal encoding operation fails.
   *
   * @param value - The value to be encoded according to the Coder rules.
   * @returns A Buffer instance containing the encoded value.
   */
  encode(value: EthAddress | string): Buffer {
    if (typeof value === 'string') {
      value = EthAddress.fromString(value);
    }
    try {
      return value.toBuffer32();
    } catch (error: any) {
      errors.throwError(`invalid address (${error.message})`, errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: 'address',
        value: value.toString(),
      });
    }
  }

  /**
   * Decode the data buffer at the given offset according to the coder's type.
   * This function extracts and interprets the relevant data from the buffer based on the coder specification,
   * consuming a specific number of bytes in the process. It returns an object containing the decoded value
   * and the number of bytes consumed during decoding.
   *
   * @param data - The data buffer to decode.
   * @param offset - The starting offset within the data buffer to begin decoding.
   * @returns An object containing the decoded value and the number of bytes consumed during decoding.
   */
  decode(data: Buffer, offset: number): DecodedResult {
    if (data.length < offset + 32) {
      errors.throwError('insufficuent data for address type', errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: 'address',
        value: bufferToHex(data.subarray(offset, offset + 32)),
      });
    }
    return {
      consumed: 32,
      value: new EthAddress(Buffer.from(data.subarray(offset + 12, offset + 32))),
    };
  }
}

/**
 * Encodes the given dynamic bytes value into a buffer with its length as a prefix.
 * The function first encodes the length of the byte array as a uint256 and then concatenates
 * the actual byte array followed by padding to align it to 32-byte boundary.
 *
 * @param value - The buffer or hex string representing the dynamic bytes value to be encoded.
 * @returns A buffer containing the encoded dynamic bytes value, including length prefix and proper padding.
 */
function _encodeDynamicBytes(value: Buffer): Buffer {
  const dataLength = 32 * Math.ceil(value.length / 32);
  const padding = new Buffer(dataLength - value.length);

  return Buffer.concat([uint256Coder.encode(value.length), value, padding]);
}

/**
 * Decodes dynamic bytes from a given data buffer at the specified offset.
 * Handles errors such as insufficient data, and returns an object containing
 * the consumed size (number of bytes used) and the resulting value (the decoded bytes).
 *
 * @param data - The data buffer to decode from.
 * @param offset - The starting position in the data buffer to begin decoding.
 * @param localName - The name of the argument being processed, used for error reporting.
 * @returns An object containing the number of bytes consumed and the decoded bytes as a Buffer.
 */
function _decodeDynamicBytes(data: Buffer, offset: number, localName: string): DecodedResult {
  if (data.length < offset + 32) {
    errors.throwError('insufficient data for dynamicBytes length', errors.INVALID_ARGUMENT, {
      arg: localName,
      coderType: 'dynamicBytes',
      value: bufferToHex(data.subarray(offset, offset + 32)),
    });
  }

  const lengthBI = uint256Coder.decode(data, offset).value;
  if (lengthBI > Number.MAX_SAFE_INTEGER) {
    errors.throwError('dynamic bytes count too large', errors.INVALID_ARGUMENT, {
      arg: localName,
      coderType: 'dynamicBytes',
      value: lengthBI.toString(),
    });
  }
  const length = Number(lengthBI);

  if (data.length < offset + 32 + length) {
    errors.throwError('insufficient data for dynamicBytes type', errors.INVALID_ARGUMENT, {
      arg: localName,
      coderType: 'dynamicBytes',
      value: bufferToHex(data.subarray(offset, offset + 32 + length)),
    });
  }

  return {
    consumed: 32 + 32 * Math.ceil(length / 32),
    value: data.subarray(offset + 32, offset + 32 + length),
  };
}

/**
 * The CoderDynamicBytes class is a coder for encoding and decoding dynamic bytes data types in ABI.
 * It handles the variable-length byte arrays, allowing efficient serialization and deserialization of
 * such data while interacting with the Ethereum blockchain through smart contracts. The class extends the
 * base Coder class and overrides its methods to provide specific implementation for dynamic bytes.
 */
class CoderDynamicBytes extends Coder {
  constructor(localName: string) {
    super('bytes', 'bytes', localName, true);
  }

  /**
   * Encodes the input values according to the specified ABI types, returning a hex-encoded string of the packed data.
   * This function takes an array of types and an array of corresponding values as input, and generates a representation
   * that can be used in Ethereum smart contracts for function calls or events.
   *
   * @param types - An array of strings or ParamType objects describing the types of the input values.
   * @param values - An array of input values matching the types specified in the "types" parameter.
   * @returns A hex-encoded string representing the packed data according to the ABI types.
   */
  encode(value: Buffer | string): Buffer {
    try {
      if (typeof value === 'string') {
        value = hexToBuffer(value);
      }
      return _encodeDynamicBytes(value);
    } catch (error: any) {
      return errors.throwError('invalid bytes value', errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: 'bytes',
        value: error.value,
      });
    }
  }

  /**
   * Decodes the given data according to the specified parameter types.
   * The types array represents the data types of the expected decoding results,
   * where each type is either a string or a ParamType object.
   * The data argument should be a buffer containing the encoded data.
   * Returns an array of decoded values, with each value corresponding to the provided types.
   *
   * @param types - An array of strings or ParamType objects representing the data types to decode.
   * @param data - A Buffer containing the encoded data to be decoded.
   * @returns An array of decoded values corresponding to the specified types.
   */
  decode(data: Buffer, offset: number): DecodedResult {
    const result = _decodeDynamicBytes(data, offset, this.localName);
    result.value = bufferToHex(result.value);
    return result;
  }
}

/**
 * The CoderString class is responsible for encoding and decoding string values in the ABI format.
 * It inherits from the Coder class and overrides the encode and decode methods to specifically handle
 * string data types. This class enables efficient and accurate serialization and deserialization
 * of string values within the context of Ethereum contract function calls and events.
 */
class CoderString extends Coder {
  constructor(localName: string) {
    super('string', 'string', localName, true);
  }

  /**
   * Encodes the given types and values into a single ABI-formatted hex string.
   * The types array should contain a list of type strings or ParamType objects that describe each value's type.
   * The values array should have the same length as the types array and contain the data to be encoded.
   * Throws an error if the types/values length mismatch or if any invalid argument is encountered during encoding.
   *
   * @param types - An array of type strings or ParamType objects describing each value's type.
   * @param values - An array of values corresponding to the types provided.
   * @returns A hex-encoded ABI-formatted string representing the encoded values.
   */
  encode(value: string): Buffer {
    if (typeof value !== 'string') {
      errors.throwError('invalid string value', errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: 'string',
        value: value,
      });
    }
    return _encodeDynamicBytes(Buffer.from(new TextEncoder().encode(value)));
  }

  /**
   * Decodes the ABI-encoded data based on the specified input types.
   * Takes an array of input types (strings or ParamType objects) and a buffer containing
   * the ABI-encoded data. Returns an array or an object containing the decoded values, depending
   * on whether the local names are available in the input types. If any error occurs during decoding,
   * it throws an exception with a detailed message about the issue.
   *
   * @param types - An array of input types, either as strings or ParamType objects.
   * @param data - A Buffer containing the ABI-encoded data to decode.
   * @returns An array or an object containing the decoded values based on the input types.
   */
  decode(data: Buffer, offset: number): DecodedResult {
    const result = _decodeDynamicBytes(data, offset, this.localName);
    result.value = new TextDecoder('utf-8').decode(result.value);
    return result;
  }
}

/**
 * Calculate the aligned size of a value, rounding up to the nearest multiple of 32.
 * This function is commonly used when dealing with tightly packed data structures in
 * ABI encoding and decoding where data needs to be aligned to 32-byte boundaries.
 *
 * @param size - The original size of a value in bytes.
 * @returns The aligned size, rounded up to the nearest multiple of 32 bytes.
 */
function alignSize(size: number): number {
  return 32 * Math.ceil(size / 32);
}

/**
 * Packs an array of values according to their respective coders into a single Buffer.
 * The 'coders' and 'values' arrays must have the same length. Each value in the 'values' array
 * will be encoded using its corresponding coder in the 'coders' array, then combined into
 * a single Buffer with proper padding and dynamic content offsets.
 *
 * @param coders - An array of Coder instances used to encode each value.
 * @param values - An array of values to be packed together into a single Buffer.
 * @returns A Buffer containing the packed values according to their coders.
 */
function pack(coders: Array<Coder>, values: Array<any>): Buffer {
  if (Array.isArray(values)) {
    // do nothing
  } else if (values && typeof values === 'object') {
    const arrayValues: Array<any> = [];
    coders.forEach(function (coder) {
      arrayValues.push((<any>values)[coder.localName]);
    });
    values = arrayValues;
  } else {
    errors.throwError('invalid tuple value', errors.INVALID_ARGUMENT, {
      coderType: 'tuple',
      value: values,
    });
  }

  if (coders.length !== values.length) {
    errors.throwError('types/value length mismatch', errors.INVALID_ARGUMENT, {
      coderType: 'tuple',
      value: values,
    });
  }

  const parts: Array<{
    /**
     * Indicates if the coder has a dynamic size.
     */
    /**
     * Indicates if the coder has a dynamic size.
     */
    dynamic: boolean;
    /**
     * The encoded or decoded value based on the ABI data type.
     */
    /**
     * The encoded or decoded value based on the ABI data type.
     */
    value: any;
  }> = [];

  coders.forEach(function (coder, index) {
    parts.push({ dynamic: coder.dynamic, value: coder.encode(values[index]) });
  });

  let staticSize = 0,
    dynamicSize = 0;
  parts.forEach(function (part) {
    if (part.dynamic) {
      staticSize += 32;
      dynamicSize += alignSize(part.value.length);
    } else {
      staticSize += alignSize(part.value.length);
    }
  });

  let offset = 0,
    dynamicOffset = staticSize;
  const data = new Buffer(staticSize + dynamicSize);

  parts.forEach(function (part) {
    if (part.dynamic) {
      data.set(uint256Coder.encode(dynamicOffset), offset);
      offset += 32;

      data.set(part.value, dynamicOffset);
      dynamicOffset += alignSize(part.value.length);
    } else {
      data.set(part.value, offset);
      offset += alignSize(part.value.length);
    }
  });

  return data;
}

/**
 * Unpack the values from the provided coders and data buffer at the specified offset.
 * The function iterates through each coder, decodes its corresponding value in the data buffer,
 * and appends it to an array of decoded values. If the coder has a localName, the decoded value
 * is also assigned to the resulting object using the localName as the key.
 *
 * @param coders - Array of Coder instances to decode the data buffer.
 * @param data - Buffer containing the encoded data to be unpacked.
 * @param offset - The starting position of the data buffer to begin decoding from.
 * @returns An object with two properties: 'value', which is an array of decoded values, and 'consumed', which is the number of bytes consumed during decoding.
 */
function unpack(coders: Array<Coder>, data: Buffer, offset: number): DecodedResult {
  const baseOffset = offset;
  let consumed = 0;
  const value: any = [];
  coders.forEach(function (coder) {
    let result;
    if (coder.dynamic) {
      const dynamicOffset = uint256Coder.decode(data, offset);
      result = coder.decode(data, baseOffset + Number(dynamicOffset.value));
      // The dynamic part is leap-frogged somewhere else; doesn't count towards size
      result.consumed = dynamicOffset.consumed;
    } else {
      result = coder.decode(data, offset);
    }

    if (result.value != undefined) {
      value.push(result.value);
    }

    offset += result.consumed;
    consumed += result.consumed;
  });

  coders.forEach(function (coder: Coder, index: number) {
    let name: string = coder.localName;
    if (!name) {
      return;
    }

    if (name === 'length') {
      name = '_length';
    }

    if (value[name] != null) {
      return;
    }

    value[name] = value[index];
  });

  return {
    value: value,
    consumed: consumed,
  };
}

/**
 * The CoderArray class extends the Coder class for encoding and decoding array data types in Ethereum ABI.
 * It handles fixed-size arrays (e.g., uint256[5]) and dynamic-size arrays (e.g., address[]), providing
 * methods to encode and decode values according to the specified element type and length. By leveraging
 * the base Coder implementation and an additional coder for nested elements, CoderArray ensures proper
 * handling of both simple and complex arrays within contract function signatures and event topics.
 */
class CoderArray extends Coder {
  readonly coder: Coder;
  readonly length: number;
  constructor(coder: Coder, length: number, localName: string) {
    const type = coder.type + '[' + (length >= 0 ? length : '') + ']';
    const dynamic = length === -1 || coder.dynamic;
    super('array', type, localName, dynamic);

    this.coder = coder;
    this.length = length;
  }

  /**
   * Encode the given input types and values into a hexadecimal string according to the ABI specification.
   * The function takes an array of types and values, and encodes them into a single data string,
   * which can be used for contract function calls or event encoding. The types array should contain
   * strings representing Ethereum Solidity types (e.g. 'uint256', 'address', 'bytes32'),
   * and the values array should contain corresponding JavaScript values to be encoded.
   * Throws an error if the types and values length mismatch or if there's an issue during encoding.
   *
   * @param types - An array of strings or ParamType objects representing the Ethereum Solidity types.
   * @param values - An array of JavaScript values corresponding to the input types.
   * @returns A hex-encoded string of the encoded input types and values.
   */
  encode(value: Array<any>): Buffer {
    if (!Array.isArray(value)) {
      errors.throwError('expected array value', errors.INVALID_ARGUMENT, {
        arg: this.localName,
        coderType: 'array',
        value: value,
      });
    }

    let count = this.length;

    let result = new Buffer(0);
    if (count === -1) {
      count = value.length;
      result = uint256Coder.encode(count);
    }

    errors.checkArgumentCount(count, value.length, 'in coder array' + (this.localName ? ' ' + this.localName : ''));

    const coders: any[] = [];
    for (let i = 0; i < value.length; i++) {
      coders.push(this.coder);
    }

    return Buffer.concat([result, pack(coders, value)]);
  }

  /**
   * Decodes the ABI (Application Binary Interface) encoded data based on the specified types.
   * The function takes an array of type descriptors and a buffer containing the ABI encoded data,
   * and returns an object with decoded values.
   *
   * @param types - An array of type descriptors, either as strings or ParamType objects.
   * @param  data - A Buffer containing the ABI encoded data to be decoded.
   * @returns - An object with the decoded values based on the provided types.
   */
  decode(data: Buffer, offset: number) {
    // @TODO:
    //if (data.length < offset + length * 32) { throw new Error('invalid array'); }

    let consumed = 0;

    let count = this.length;

    if (count === -1) {
      let decodedLength: any;
      try {
        decodedLength = uint256Coder.decode(data, offset);
      } catch (error: any) {
        return errors.throwError('insufficient data for dynamic array length', errors.INVALID_ARGUMENT, {
          arg: this.localName,
          coderType: 'array',
          value: error.value,
        });
      }
      if (decodedLength.value > Number.MAX_SAFE_INTEGER) {
        errors.throwError('array count too large', errors.INVALID_ARGUMENT, {
          arg: this.localName,
          coderType: 'array',
          value: decodedLength.value.toString(),
        });
      }
      count = Number(decodedLength.value);
      consumed += decodedLength.consumed;
      offset += decodedLength.consumed;
    }

    const coders: any[] = [];
    for (let i = 0; i < count; i++) {
      coders.push(new CoderAnonymous(this.coder));
    }

    const result = unpack(coders, data, offset);
    result.consumed += consumed;
    return result;
  }
}

/**
 * The CoderTuple class is responsible for encoding and decoding tuple data types in the ABI encoding format.
 * It extends the Coder class and takes an array of coders representing each component.
 * When encoding, it processes the components using the appropriate coder instances and returns the encoded data.
 * When decoding, it parses the encoded data and constructs the tuple by applying each coder's decode method to their respective components.
 */
class CoderTuple extends Coder {
  constructor(private coders: Array<Coder>, localName: string) {
    let dynamic = false;
    const types: Array<string> = [];
    coders.forEach(function (coder) {
      if (coder.dynamic) {
        dynamic = true;
      }
      types.push(coder.type);
    });
    const type = 'tuple(' + types.join(',') + ')';

    super('tuple', type, localName, dynamic);
    this.coders = coders;
  }

  /**
   * Encodes the given function signature with the corresponding parameter types and values.
   * This function takes an array of parameter types, such as strings or ParamType objects,
   * and an array of corresponding parameter values to generate the ABI-encoded data.
   * The generated encoded data is useful for interacting with smart contracts.
   * Throws an error if the length of the input types and values mismatch.
   *
   * @param types - An array of parameter types represented as strings or ParamType objects.
   * @param values - An array of corresponding values to be encoded with the parameter types.
   * @returns A hex-encoded string representing the ABI-encoded data.
   */
  encode(value: Array<any>): Buffer {
    return pack(this.coders, value);
  }

  /**
   * Decodes the provided data using the specified input types and returns an array of decoded values.
   * The input 'types' is an array of either strings or ParamType objects representing the expected data types.
   * The input 'data' should be a Buffer containing the encoded data to decode.
   * Throws an error if the number of input types does not match the number of values in the data or if decoding fails.
   *
   * @param types - Array of strings or ParamType objects representing the expected data types.
   * @param data - Buffer containing the encoded data to decode.
   * @returns An array of decoded values.
   */
  decode(data: Buffer, offset: number): DecodedResult {
    const result = unpack(this.coders, data, offset);
    return result;
  }
}

// function splitNesting(value: string): Array<any> {
//   value = value.trim();

//   var result: string[] = [];
//   var accum = '';
//   var depth = 0;
//   for (var offset = 0; offset < value.length; offset++) {
//     var c = value[offset];
//     if (c === ',' && depth === 0) {
//       result.push(accum);
//       accum = '';
//     } else {
//       accum += c;
//       if (c === '(') {
//         depth++;
//       } else if (c === ')') {
//         depth--;
//         if (depth === -1) {
//           throw new Error('unbalanced parenthsis');
//         }
//       }
//     }
//   }
//   if (accum) {
//     result.push(accum);
//   }

//   return result;
// }

const paramTypeSimple: { [key: string]: any } = {
  address: CoderAddress,
  bool: CoderBoolean,
  string: CoderString,
  bytes: CoderDynamicBytes,
};

/**
 * Creates a CoderTuple instance from an array of components with their corresponding local names.
 * The 'components' should be an array of ParamType objects, each containing the type and name of each component.
 * Throws an error if the input components are invalid or any ParamType is not supported.
 *
 * @param components - An array of ParamType objects representing the components of the tuple.
 * @param localName - The string representing the local name of the tuple.
 * @returns A CoderTuple instance for encoding and decoding the tuple values.
 */
function getTupleParamCoder(components: Array<any>, localName: string): CoderTuple {
  if (!components) {
    components = [];
  }
  const coders: Array<Coder> = [];
  components.forEach(function (component) {
    coders.push(getParamCoder(component));
  });

  return new CoderTuple(coders, localName);
}

/**
 * Returns an instance of the appropriate Coder class based on the given ParamType.
 * This function is responsible for selecting the correct coder to handle encoding and
 * decoding of various data types specified in the ABI. It supports basic types like 'address',
 * 'bool', 'string', and 'bytes', as well as more complex types like fixed-size arrays, dynamic arrays,
 * and tuples with nested components.
 *
 * @param param - The ParamType object containing the type and name of the parameter.
 * @returns An instance of a Coder subclass corresponding to the given ParamType.
 */
function getParamCoder(param: ParamType): Coder {
  const coder = paramTypeSimple[param.type];
  if (coder) {
    return new coder(param.name);
  }
  let match = param.type.match(paramTypeNumber);
  if (match) {
    const size = parseInt(match[2] || '256');
    if (size === 0 || size > 256 || size % 8 !== 0) {
      return errors.throwError('invalid ' + match[1] + ' bit length', errors.INVALID_ARGUMENT, {
        arg: 'param',
        value: param,
      });
    }
    return new CoderNumber(size / 8, match[1] === 'int', param.name!);
  }

  match = param.type.match(paramTypeBytes);
  if (match) {
    const size = parseInt(match[1]);
    if (size === 0 || size > 32) {
      errors.throwError('invalid bytes length', errors.INVALID_ARGUMENT, {
        arg: 'param',
        value: param,
      });
    }
    return new CoderFixedBytes(size, param.name!);
  }

  match = param.type.match(paramTypeArray);
  if (match) {
    const size = parseInt(match[2] || '-1');
    param = {
      ...param,
      type: match[1],
    };
    return new CoderArray(getParamCoder(param), size, param.name!);
  }

  if (param.type.substring(0, 5) === 'tuple') {
    return getTupleParamCoder(param.components!, param.name!);
  }

  if (param.type === '') {
    return new CoderNull(param.name!);
  }

  return errors.throwError('invalid type', errors.INVALID_ARGUMENT, {
    arg: 'type',
    value: param.type,
  });
}

/**
 * The AbiCoder class provides an interface for encoding and decoding contract function calls and events
 * using Ethereum's Application Binary Interface (ABI). It supports the conversion of Solidity data types
 * to JavaScript and vice versa. This class enables encoding of function arguments for contract method calls,
 * as well as decoding of event logs and return values from transactions and contract calls.
 */
export class AbiCoder {
  constructor() {}

  /**
   * Encodes the given types and values into a hex-encoded ABI string.
   * Takes an array of types (strings or ParamType objects) and an array of corresponding values.
   * Each type in the 'types' array should have a corresponding value in the 'values' array.
   * Throws an error if the length of types and values arrays do not match, or if there are any issues during encoding.
   *
   * @param types - An array of strings or ParamType objects representing the data types.
   * @param values - An array of values corresponding to the types.
   * @returns A hex-encoded string representing the encoded ABI data.
   */
  encode(types: Array<string | ParamType>, values: Array<any>): string {
    if (types.length !== values.length) {
      errors.throwError('types/values length mismatch', errors.INVALID_ARGUMENT, {
        count: { types: types.length, values: values.length },
        value: { types: types, values: values },
      });
    }

    const coders: Array<Coder> = [];
    types.forEach(type => {
      // Convert types to type objects
      //   - "uint foo" => { type: "uint", name: "foo" }
      //   - "tuple(uint, uint)" => { type: "tuple", components: [ { type: "uint" }, { type: "uint" }, ] }

      let typeObject: ParamType;
      if (typeof type === 'string') {
        typeObject = parseParam(type);
      } else {
        typeObject = type;
      }

      coders.push(getParamCoder(typeObject));
    });

    return bufferToHex(new CoderTuple(coders, '_').encode(values));
  }

  /**
   * Decodes the ABI-encoded data using the provided array of types and returns the corresponding values.
   * Each type can be a string or a ParamType object, which includes type information and an optional name.
   * The input 'data' should be a valid ABI-encoded Buffer.
   * Throws an error if the types and data do not match, or if any decoding issues occur.
   *
   * @param types - An array of strings or ParamType objects representing the expected types of the decoded data.
   * @param data - A Buffer containing the ABI-encoded data to be decoded.
   * @returns An array or an object containing the decoded values, with optional keys if names are provided in the types.
   */
  decode(types: Array<string | ParamType>, data: Buffer): any {
    const coders = types.map(type => {
      if (typeof type === 'string') {
        type = parseParam(type);
      }
      return getParamCoder(type);
    });

    return new CoderTuple(coders, '_').decode(data, 0).value;
  }
}
