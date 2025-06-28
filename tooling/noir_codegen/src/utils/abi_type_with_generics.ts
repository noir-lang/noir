import { AbiType } from '@noir-lang/noirc_abi';

/**
 * Represents a binding to a generic.
 */
export class BindingId {
  constructor(
    public id: number,
    public isNumeric: boolean,
  ) {}
}

export type StructType = {
  path: string;
  fields: { name: string; type: AbiTypeWithGenerics }[];
  /** The generics of the struct, bound to the fields */
  generics: BindingId[];
};

export type StringType = {
  kind: 'string';
  length: number | BindingId | null;
};

export type Constant = {
  kind: 'constant';
  value: number;
};

export type ArrayType = {
  kind: 'array';
  length: number | BindingId | null;
  type: AbiTypeWithGenerics;
};

export type Tuple = {
  kind: 'tuple';
  fields: AbiTypeWithGenerics[];
};

export type Struct = {
  kind: 'struct';
  structType: StructType;
  /** The arguments are the concrete instantiation of the generics in the struct type. */
  args: AbiTypeWithGenerics[];
};

export type AbiTypeWithGenerics =
  | { kind: 'field' }
  | { kind: 'boolean' }
  | { kind: 'integer'; sign: string; width: number }
  | { kind: 'binding'; id: BindingId }
  | { kind: 'constant'; value: number }
  | StringType
  | ArrayType
  | Tuple
  | Struct;

/**
 * Maps an ABI type to an ABI type with generics.
 * This performs pure type conversion, and does not generate any bindings.
 */
export function mapAbiTypeToAbiTypeWithGenerics(
  abiType: AbiType,
  allTypes: AbiTypeWithGenerics[],
): AbiTypeWithGenerics {
  let returnedType: AbiTypeWithGenerics;
  switch (abiType.kind) {
    case 'field':
    case 'boolean':
    case 'string':
    case 'integer':
      returnedType = abiType;
      break;
    case 'array': {
      const type = mapAbiTypeToAbiTypeWithGenerics(abiType.type, allTypes);
      returnedType = {
        kind: 'array',
        length: abiType.length,
        type,
      };
      break;
    }
    case 'struct': {
      const structType = {
        path: abiType.path,
        fields: abiType.fields.map(function (field) {
          const type = mapAbiTypeToAbiTypeWithGenerics(field.type, allTypes);
          return {
            name: field.name,
            type,
          };
        }),
        generics: [],
      };
      returnedType = {
        kind: 'struct',
        structType,
        args: [],
      };
      break;
    }
    case 'tuple':
      returnedType = {
        kind: 'tuple',
        fields: abiType.fields.map(function (field) {
          const type = mapAbiTypeToAbiTypeWithGenerics(field, allTypes);
          allTypes.push(type);
          return type;
        }),
      };
      break;
    default: {
      const exhaustiveCheck: never = abiType;
      throw new Error(`Unhandled abi type: ${exhaustiveCheck}`);
    }
  }
  allTypes.push(returnedType);
  return returnedType;
}

/**
 * Finds the structs in an ABI type.
 * This won't explore nested structs.
 */
export function findStructsInType(abiType: AbiTypeWithGenerics): Struct[] {
  switch (abiType.kind) {
    case 'field':
    case 'boolean':
    case 'string':
    case 'integer':
      return [];
    case 'array':
      return findStructsInType(abiType.type);
    case 'tuple':
      return abiType.fields.flatMap(findStructsInType);
    case 'struct':
      return [abiType];
    default: {
      return [];
    }
  }
}

/**
 * Finds all the structs in an ABI type, including nested structs.
 */
export function findAllStructsInType(abiType: AbiTypeWithGenerics): Struct[] {
  let allStructs: Struct[] = [];
  let lastStructs = findStructsInType(abiType);
  while (lastStructs.length > 0) {
    allStructs = allStructs.concat(lastStructs);
    lastStructs = lastStructs.flatMap((struct) =>
      struct.structType.fields.flatMap((field) => findStructsInType(field.type)),
    );
  }
  return allStructs;
}
