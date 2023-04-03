import { ABIType, BasicType, ContractAbi, StructType } from '@aztec/noir-contracts';

type TypeWithoutKind<T> = Omit<{ [key in keyof T]: any }, 'kind'>;

export function abiChecker(abi: ContractAbi) {
  if (!abi.functions || abi.functions.length === 0) {
    throw new Error('ABI has no functions');
  }

  abi.functions.forEach(func => {
    if (!('name' in func && typeof func.name === 'string' && func.name.length > 0)) {
      throw new Error('ABI function has no name');
    }

    // TODO: implement a better check for bytecode (right now only checks if it's > 0)
    if (!('bytecode' in func && typeof func.bytecode === 'string' && func.bytecode.length > 0)) {
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
  if (!abi.functions.find(func => func.name === 'constructor')) {
    throw new Error('ABI has no constructor');
  }

  return true;
}

function abiParameterTypeChecker(type: ABIType): boolean {
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
      return checkAttributes(type, { fields: 'object' }) && checkStruct(type);
    default:
      throw new Error('ABI function parameter has an unrecognised type');
  }
}

function checkStruct(type: StructType) {
  return type.fields.reduce((acc, field) => {
    if (!('name' in field && typeof field.name === 'string')) {
      throw new Error('ABI function parameter has an incorrectly formed struct');
    }
    return acc && abiParameterTypeChecker(field.type);
  }, true);
}

function checkAttributes<T extends BasicType<string>>(type: T, incompleteAttributes: TypeWithoutKind<T>) {
  const typeKeys = Object.keys(type);
  const attributes = { ...incompleteAttributes, kind: 'string' };

  if (typeKeys.length !== Object.keys(attributes).length) {
    throw new Error(`Unrecognised attribute on type ${type.kind}`);
  }

  typeKeys.forEach(element => {
    if (!(element in type && typeof (type as any)[element] === (attributes as any)[element])) {
      throw new Error(`ABI function parameter has an incorrectly formed ${type.kind}`);
    }
  });

  return true;
}
