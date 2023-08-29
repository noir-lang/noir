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
function encodeArg(arg: string, abiType: ABIType, name: string): any {
  const { kind } = abiType;
  if (kind === 'field' || kind === 'integer') {
    let res: bigint;
    try {
      res = BigInt(arg);
    } catch (err) {
      throw new Error(
        `Invalid value passed for ${name}. Could not parse ${arg} as a${kind === 'integer' ? 'n' : ''} ${kind}.`,
      );
    }
    return res;
  } else if (kind === 'boolean') {
    if (arg === 'true') return true;
    if (arg === 'false') return false;
    else throw Error(`Invalid boolean value passed for ${name}: ${arg}.`);
  } else if (kind === 'array') {
    let arr;
    const res = [];
    try {
      arr = JSON.parse(arg);
    } catch {
      throw new Error(`Unable to parse arg ${arg} as array for ${name} parameter`);
    }
    if (!Array.isArray(arr)) throw Error(`Invalid argument ${arg} passed for array parameter ${name}.`);
    if (arr.length !== abiType.length)
      throw Error(`Invalid array length passed for ${name}. Expected ${abiType.length}, received ${arr.length}.`);
    for (let i = 0; i < abiType.length; i += 1) {
      res.push(encodeArg(arr[i], abiType.type, name));
    }
    return res;
  } else if (kind === 'struct') {
    // check if input is encoded long string
    if (arg.startsWith('0x')) {
      return parseStructString(arg, abiType);
    }
    let obj;
    try {
      obj = JSON.parse(arg);
    } catch {
      throw new Error(`Unable to parse arg ${arg} as struct`);
    }
    if (Array.isArray(obj)) throw Error(`Array passed for arg ${name}. Expected a struct.`);
    const res = [];
    for (const field of abiType.fields) {
      // Remove field name from list as it's present
      const arg = obj[field.name];
      if (!arg) throw Error(`Expected field ${field.name} not found in struct ${name}.`);
      res.push(encodeArg(obj[field.name], field.type, field.name));
    }
    return res;
  }
}

/**
 * Tries to encode function args to their equivalent TS type.
 * @param args - An array of function's / constructor's args.
 * @returns The encoded array.
 */
export function encodeArgs(args: any[], params: ABIParameter[]) {
  if (args.length !== params.length) {
    throw new Error(`Invalid number of args provided. Expected: ${params.length}, received: ${args.length}`);
  }
  return args
    .map((arg: any, index) => {
      const { type, name } = params[index];
      return encodeArg(arg, type, name);
    })
    .flat();
}
