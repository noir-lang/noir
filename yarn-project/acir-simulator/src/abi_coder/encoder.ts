import { ARGS_LENGTH } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';

import { ABIType, FunctionAbi } from '@aztec/noir-contracts';

/**
 * Encodes arguments for a function call.
 * Missing support for integer and string.
 */
class ArgumentEncoder {
  private flattened: Fr[] = [];

  constructor(private abi: FunctionAbi, private args: any[]) {}

  /**
   * Encodes a single argument from the given type to field.
   * @param abiType - The abi type of the argument.
   * @param arg - The value to encode.
   */
  private encodeArgument(abiType: ABIType, arg: any) {
    switch (abiType.kind) {
      case 'field':
        this.flattened.push(new Fr(arg));
        break;
      case 'boolean':
        this.flattened.push(new Fr(arg ? 1n : 0n));
        break;
      case 'array':
        for (let i = 0; i < abiType.length; i += 1) {
          this.encodeArgument(abiType.type, arg[i]);
        }
        break;
      case 'struct':
        for (const field of abiType.fields) {
          this.encodeArgument(field.type, arg[field.name]);
        }
        break;
      default:
        throw new Error(`Unsupported type: ${abiType.kind}`);
    }
  }

  /**
   * Encodes all the arguments for the given function ABI.
   * @returns The encoded arguments.
   */
  public encode() {
    for (let i = 0; i < this.abi.parameters.length; i += 1) {
      const parameterAbi = this.abi.parameters[i];
      this.encodeArgument(parameterAbi.type, this.args[i]);
    }
    return this.flattened;
  }
}

/**
 * Encodes all the arguments for a function call.
 * @param abi - The function ABI entry.
 * @param args - The arguments to encode.
 * @param pad - Whether to pad the arguments to the MAX_ARGS_LENGTH.
 * @returns The encoded arguments.
 */
export function encodeArguments(abi: FunctionAbi, args: any[], pad = true) {
  const flatArgs = new ArgumentEncoder(abi, args).encode();
  if (!pad) return flatArgs;

  if (flatArgs.length > ARGS_LENGTH) {
    throw new Error(`Too many arguments: ${flatArgs.length}`);
  }
  return flatArgs.concat(new Array(ARGS_LENGTH - flatArgs.length).fill(Fr.ZERO));
}
