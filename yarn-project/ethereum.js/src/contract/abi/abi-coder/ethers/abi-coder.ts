import * as errors from './errors.js';
import { EthAddress } from '../../../../eth_address/index.js';
import { toBigIntBE, toBufferBE } from '../../../../bigint_buffer/index.js';
import { bufferToHex, hexToBuffer } from '../../../../hex_string/index.js';

const NegativeOne = BigInt(-1);
const Zero = BigInt(0);
const MaxUint256 = BigInt('0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff');

type ParamType = {
  name?: string;
  type: string;
  indexed?: boolean;
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

function verifyType(type: string): string {
  // These need to be transformed to their full description
  if (type.match(/^uint($|[^1-9])/)) {
    type = 'uint256' + type.substring(4);
  } else if (type.match(/^int($|[^1-9])/)) {
    type = 'int256' + type.substring(3);
  }

  return type;
}

type ParseState = {
  allowArray?: boolean;
  allowName?: boolean;
  allowParams?: boolean;
  allowType?: boolean;
  readArray?: boolean;
};

type ParseNode = {
  parent?: any;
  type?: string;
  name?: string;
  state?: ParseState;
  indexed?: boolean;
  components?: Array<any>;
};

function parseParam(param: string, allowIndexed?: boolean): ParamType {
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

type DecodedResult<T = any> = { consumed: number; value: T };
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

  abstract encode(value: any): Buffer;
  abstract decode(data: Buffer, offset: number): DecodedResult;
}

// Clones the functionality of an existing Coder, but without a localName
class CoderAnonymous extends Coder {
  constructor(private coder: Coder) {
    super(coder.name, coder.type, undefined, coder.dynamic);
  }
  encode(value: any): Buffer {
    return this.coder.encode(value);
  }
  decode(data: Buffer, offset: number): DecodedResult {
    return this.coder.decode(data, offset);
  }
}

class CoderNull extends Coder {
  constructor(localName: string) {
    super('null', '', localName, false);
  }

  encode(): Buffer {
    return Buffer.alloc(0);
  }

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

class CoderNumber extends Coder {
  readonly size: number;
  readonly signed: boolean;
  constructor(size: number, signed: boolean, localName: string) {
    const name = (signed ? 'int' : 'uint') + size * 8;
    super(name, name, localName, false);

    this.size = size;
    this.signed = signed;
  }

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

class CoderBoolean extends Coder {
  constructor(localName: string) {
    super('bool', 'bool', localName, false);
  }

  encode(value: boolean): Buffer {
    return uint256Coder.encode(value ? 1 : 0);
  }

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

class CoderFixedBytes extends Coder {
  readonly length: number;
  constructor(length: number, localName: string) {
    const name = 'bytes' + length;
    super(name, name, localName, false);
    this.length = length;
  }

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

class CoderAddress extends Coder {
  constructor(localName: string) {
    super('address', 'address', localName, false);
  }

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

function _encodeDynamicBytes(value: Buffer): Buffer {
  const dataLength = 32 * Math.ceil(value.length / 32);
  const padding = new Buffer(dataLength - value.length);

  return Buffer.concat([uint256Coder.encode(value.length), value, padding]);
}

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

class CoderDynamicBytes extends Coder {
  constructor(localName: string) {
    super('bytes', 'bytes', localName, true);
  }

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

  decode(data: Buffer, offset: number): DecodedResult {
    const result = _decodeDynamicBytes(data, offset, this.localName);
    result.value = bufferToHex(result.value);
    return result;
  }
}

class CoderString extends Coder {
  constructor(localName: string) {
    super('string', 'string', localName, true);
  }

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

  decode(data: Buffer, offset: number): DecodedResult {
    const result = _decodeDynamicBytes(data, offset, this.localName);
    result.value = new TextDecoder('utf-8').decode(result.value);
    return result;
  }
}

function alignSize(size: number): number {
  return 32 * Math.ceil(size / 32);
}

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

  const parts: Array<{ dynamic: boolean; value: any }> = [];

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

  encode(value: Array<any>): Buffer {
    return pack(this.coders, value);
  }

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

export class AbiCoder {
  constructor() {}

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
