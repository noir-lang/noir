import { Fr } from '@aztec/aztec.js';
import { ABIParameter, ABIType, StructType } from '@aztec/foundation/abi';

/**
 * Parses a hex string into an ABI struct type.
 * @param str - The encoded hex string.
 * @param abiType - The ABI Struct type.
 * @returns An object in the ABI struct type's format.
 */
export function parseStructString(str: string, abiType: StructType) {
  // Assing string bytes to struct fields.
  const buf = Buffer.from(str.replace(/^0x/i, ''), 'hex');
  const struct: any = {};
  let byteIndex = 0;
  let argIndex = 0;
  while (byteIndex < buf.length) {
    const { name } = abiType.fields[argIndex];
    struct[name] = Fr.fromBuffer(buf.subarray(byteIndex, byteIndex + 32));
    byteIndex += 32;
    argIndex += 1;
  }

  return struct;
}

/**
 * Helper function to encode CLI string args to an appropriate JS type.
 * @param arg - The CLI argument.
 * @param abiType - The type as described by the contract's ABI.
 * @returns The encoded argument.
 */
function encodeArg(arg: string, abiType: ABIType): any {
  const { kind } = abiType;
  if (kind === 'field' || kind === 'integer') {
    return BigInt(arg);
  } else if (kind === 'boolean') {
    if (arg === 'true') return true;
    if (arg === 'false') return false;
  } else if (kind === 'array') {
    let arr;
    try {
      arr = JSON.parse(arg);
      if (!Array.isArray(arr)) throw Error();
      for (let i = 0; i < abiType.length; i += 1) {
        return encodeArg(arg[i], abiType.type);
      }
    } catch {
      throw new Error(`Unable to parse arg ${arg} as array`);
    }
  } else if (kind === 'struct') {
    // check if input is encoded long string
    if (arg.startsWith('0x')) {
      return parseStructString(arg, abiType);
    }
    let obj;
    try {
      obj = JSON.parse(arg);
      if (Array.isArray(obj)) throw Error();
      const res = [];
      for (const field of abiType.fields) {
        res.push(encodeArg(obj[field.name], field.type));
      }
      return res;
    } catch {
      throw new Error(`Unable to parse arg ${arg} as struct`);
    }
  }
}

/**
 * Tries to encode function args to their equivalent TS type.
 * @param args - An array of function's / constructor's args.
 * @returns The encoded array.
 */
export function encodeArgs(args: any[], params: ABIParameter[]) {
  return args
    .map((arg: any, index) => {
      const paramType = params[index].type;
      return encodeArg(arg, paramType);
    })
    .flat();
}
