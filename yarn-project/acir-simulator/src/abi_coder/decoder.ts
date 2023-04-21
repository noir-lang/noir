import { Fr } from '@aztec/foundation';
import { ABIType, FunctionAbi } from '@aztec/noir-contracts';

// Simple decoder. It's missing support for integer and string
class ReturnValuesDecoder {
  constructor(private abi: FunctionAbi, private flattened: Fr[]) {}

  private decodeReturn(abiType: ABIType): any {
    switch (abiType.kind) {
      case 'field':
        return this.getNextField().value;
      case 'boolean':
        return !this.getNextField().isZero();
      case 'array': {
        const array = [];
        for (let i = 0; i < abiType.length; i += 1) {
          array.push(this.decodeReturn(abiType.type));
        }
        break;
      }
      case 'struct': {
        const struct: any = {};
        for (const field of abiType.fields) {
          struct[field.name] = this.decodeReturn(field.type);
        }
        break;
      }
      default:
        throw new Error(`Unsupported type: ${abiType.kind}`);
    }
  }

  private getNextField(): Fr {
    const field = this.flattened.shift();
    if (!field) {
      throw new Error('Not enough return values');
    }
    return field;
  }

  public decode() {
    const returnValues = [];
    for (let i = 0; i < this.abi.returnTypes.length; i += 1) {
      returnValues.push(this.decodeReturn(this.abi.returnTypes[i]));
    }
    return returnValues;
  }
}

export function decodeReturnValues(abi: FunctionAbi, returnValues: Fr[]) {
  return new ReturnValuesDecoder(abi, returnValues.slice()).decode();
}
