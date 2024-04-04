import { type AbiType, type BasicType, type ContractArtifact, type StructType } from '@aztec/foundation/abi';

/**
 * Represents a type derived from input type T with the 'kind' property removed.
 * Useful when checking attributes of a specific kind and validating their types.
 */
type TypeWithoutKind<T> = Omit<{ [key in keyof T]: any }, 'kind'>;

/**
 * Validates the given ContractArtifact object by checking its functions and their parameters.
 * Ensures that the ABI has at least one function, a constructor, valid bytecode, and correct parameter types.
 * Throws an error if any inconsistency is detected during the validation process.
 *
 * @param artifact - The ContractArtifact object to be validated.
 * @returns A boolean value indicating whether the artifact is valid or not.
 */
export function abiChecker(artifact: ContractArtifact) {
  if (!artifact.functions || artifact.functions.length === 0) {
    throw new Error('artifact has no functions');
  }

  artifact.functions.forEach(func => {
    if (!('name' in func && typeof func.name === 'string' && func.name.length > 0)) {
      throw new Error('ABI function has no name');
    }

    // TODO: implement a better check for bytecode (right now only checks if it's > 0)
    if (!('bytecode' in func && func.bytecode.length > 0)) {
      throw new Error('ABI function parameter has incorrect bytecode');
    }

    func.parameters.forEach(param => {
      if (!param.type) {
        throw new Error('ABI function parameter has no type');
      }

      abiParameterTypeChecker(param.type);
    });
  });

  // TODO: implement a better check for constructor (right now only checks if it has it or not)
  if (!artifact.functions.find(func => func.name === 'constructor')) {
    throw new Error('ABI has no constructor');
  }

  return true;
}

/**
 * Validates the ABI function parameter's type by checking its kind and attributes.
 * Throws an error if the type has an unrecognized kind or incorrectly formed attributes.
 * Additionally, checks nested types for array and struct kinds.
 *
 * @param type - The AbiType object representing the type of the ABI function parameter.
 * @returns A boolean value indicating whether the type is valid or not.
 */
function abiParameterTypeChecker(type: AbiType): boolean {
  switch (type.kind) {
    case 'field':
    case 'boolean':
      return checkAttributes(type, {});
    case 'integer':
      return checkAttributes(type, { sign: 'string', width: 'number' });
    case 'string':
      return checkAttributes(type, { length: 'number' });
    case 'array':
      return checkAttributes(type, { length: 'number', type: 'object' }) && abiParameterTypeChecker(type.type);
    case 'struct':
      return checkAttributes(type, { fields: 'object', path: 'string' }) && checkStruct(type);
    default:
      throw new Error('ABI function parameter has an unrecognized type');
  }
}

/**
 * Check if the structure of the AbiType 'struct' is valid by ensuring field names are strings
 * and their type attribute passes the abiParameterTypeChecker. Returns true on successful validation,
 * otherwise throws an error providing insight into the incorrect formation in the struct.
 *
 * @param type - The StructType object containing an array of fields to validate.
 * @returns A boolean value indicating successful validation of the struct's fields.
 */
function checkStruct(type: StructType) {
  return type.fields.reduce((acc, field) => {
    if (!('name' in field && typeof field.name === 'string')) {
      throw new Error('ABI function parameter has an incorrectly formed struct');
    }
    return acc && abiParameterTypeChecker(field.type);
  }, true);
}

/**
 * Check if a provided ABI type has the correct attributes and their associated types.
 * This function compares the given 'type' object's keys with the expected attribute types
 * specified in 'incompleteAttributes', as well as the required 'kind' property.
 * Throws an error if there are any unrecognized attributes or incorrect attribute types.
 *
 * @param type - The ABI type object to be checked for correct attributes.
 * @param incompleteAttributes - An object representing the expected attribute types without the 'kind' property.
 * @returns Returns true if the provided ABI type has the correct attributes and their associated types, otherwise throws an error.
 */
function checkAttributes<T extends BasicType<string>>(type: T, incompleteAttributes: TypeWithoutKind<T>) {
  const typeKeys = Object.keys(type);
  const attributes = { ...incompleteAttributes, kind: 'string' };

  if (typeKeys.length !== Object.keys(attributes).length) {
    throw new Error(`Unrecognized attribute on type ${type.kind}`);
  }

  typeKeys.forEach(element => {
    if (!(element in type && typeof (type as any)[element] === (attributes as any)[element])) {
      throw new Error(`ABI function parameter has an incorrectly formed ${type.kind}`);
    }
  });

  return true;
}
