import { ABIType } from '@aztec/foundation/abi';

/**
 * Get the size of an ABI type in field elements.
 * @param type - The ABI type.
 * @returns The size of the type in field elements.
 */
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
