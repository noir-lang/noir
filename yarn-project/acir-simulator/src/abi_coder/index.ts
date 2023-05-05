import { ABIType } from '@aztec/noir-contracts';

export * from './encoder.js';
export * from './decoder.js';

export function sizeOfType(type: ABIType): number {
  switch (type.kind) {
    case 'field':
    case 'boolean':
    case 'integer':
      return 1;
    case 'string':
      return type.length;
    case 'array':
      return type.length * sizeOfType(type.type);
    case 'struct':
      return type.fields.reduce((sum, field) => sum + sizeOfType(field.type), 0);
    default: {
      const exhaustiveCheck: never = type;
      throw new Error(`Unhandled abi type: ${exhaustiveCheck}`);
    }
  }
}
