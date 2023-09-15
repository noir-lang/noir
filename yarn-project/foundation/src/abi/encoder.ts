import { ABIType, FunctionAbiHeader } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';

/**
 * Encodes arguments for a function call.
 * Missing support for integer and string.
 */
class ArgumentEncoder {
  private flattened: Fr[] = [];

  constructor(private abi: FunctionAbiHeader, private args: any[]) {}

  /**
   * Encodes a single argument from the given type to field.
   * @param abiType - The abi type of the argument.
   * @param arg - The value to encode.
   * @param name - Name.
   */
  private encodeArgument(abiType: ABIType, arg: any, name?: string) {
    if (arg === undefined || arg == null) {
      throw new Error(`Undefined argument ${name ?? 'unnamed'} of type ${abiType.kind}`);
    }
    switch (abiType.kind) {
      case 'field':
        if (typeof arg === 'number') {
          this.flattened.push(new Fr(BigInt(arg)));
        } else if (typeof arg === 'bigint') {
          this.flattened.push(new Fr(arg));
        } else if (typeof arg === 'boolean') {
          this.flattened.push(new Fr(arg ? 1 : 0));
        } else if (typeof arg === 'object') {
          if (Buffer.isBuffer(arg)) {
            this.flattened.push(Fr.fromBuffer(arg));
          } else if (typeof arg.toField === 'function') {
            this.flattened.push(arg.toField());
          } else if (arg instanceof Fr) {
            this.flattened.push(arg);
          } else {
            throw new Error(`Argument for ${name} cannot be serialised to a field`);
          }
        } else {
          throw new Error(`Invalid argument "${arg}" of type ${abiType.kind}`);
        }
        break;
      case 'boolean':
        this.flattened.push(new Fr(arg ? 1n : 0n));
        break;
      case 'array':
        for (let i = 0; i < abiType.length; i += 1) {
          this.encodeArgument(abiType.type, arg[i], `${name}[${i}]`);
        }
        break;
      case 'struct':
        for (const field of abiType.fields) {
          this.encodeArgument(field.type, arg[field.name], `${name}.${field.name}`);
        }
        break;
      case 'integer':
        this.flattened.push(new Fr(arg));
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
      this.encodeArgument(parameterAbi.type, this.args[i], parameterAbi.name);
    }
    return this.flattened;
  }
}

/**
 * Encodes all the arguments for a function call.
 * @param abi - The function ABI entry.
 * @param args - The arguments to encode.
 * @returns The encoded arguments.
 */
export function encodeArguments(abi: FunctionAbiHeader, args: any[]) {
  return new ArgumentEncoder(abi, args).encode();
}
