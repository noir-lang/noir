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
export function mapAbiTypeToAbiTypeWithGenerics(abiType: AbiType): AbiTypeWithGenerics {
  switch (abiType.kind) {
    case 'field':
    case 'boolean':
    case 'string':
    case 'integer':
      return abiType;
    case 'array':
      return {
        kind: 'array',
        length: abiType.length,
        type: mapAbiTypeToAbiTypeWithGenerics(abiType.type),
      };
    case 'struct': {
      const structType = {
        path: abiType.path,
        fields: abiType.fields.map((field) => ({
          name: field.name,
          type: mapAbiTypeToAbiTypeWithGenerics(field.type),
        })),
        generics: [],
      };
      return {
        kind: 'struct',
        structType,
        args: [],
      };
    }
    case 'tuple':
      return {
        kind: 'tuple',
        fields: abiType.fields.map(mapAbiTypeToAbiTypeWithGenerics),
      };
    default: {
      const exhaustiveCheck: never = abiType;
      throw new Error(`Unhandled abi type: ${exhaustiveCheck}`);
    }
  }
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
    lastStructs = lastStructs.flatMap(function (struct) {
      const fieldStructTypes = struct.structType.fields.flatMap((field) => findStructsInType(field.type));
      const argsStructTypes = struct.args.flatMap(findAllStructsInType);
      const structTypes = fieldStructTypes.concat(argsStructTypes);
      return structTypes;
    });
  }
  return allStructs;
}
