import {
  ABIParameter,
  ABIVariable,
  ContractAbi,
  FunctionAbi,
  FunctionSelector,
  FunctionType,
  StructType,
} from '@aztec/foundation/abi';

import camelCase from 'lodash.camelcase';
import capitalize from 'lodash.capitalize';
import compact from 'lodash.compact';
import times from 'lodash.times';
import upperFirst from 'lodash.upperfirst';

/**
 * Returns whether this function type corresponds to a private call.
 * @param functionType - The function type.
 * @returns Whether this function type corresponds to a private call.
 */
function isPrivateCall(functionType: FunctionType) {
  return functionType === FunctionType.SECRET;
}

/**
 * Generates a call to a private function using the context.
 * @param selector - The selector of a function.
 * @param functionType - Type of the function.
 * @returns A code string.
 */
function generateCallStatement(selector: FunctionSelector, functionType: FunctionType) {
  const callMethod = isPrivateCall(functionType) ? 'call_private_function' : 'call_public_function';
  return `
    context.${callMethod}(self.address, 0x${selector.toString()}, serialized_args)`;
}

/**
 * Formats a string as pascal case.
 * @param str - A string.
 * @returns A capitalised camelcase string.
 */
function toPascalCase(str: string) {
  return upperFirst(camelCase(str));
}

/**
 * Returns a struct name given a list of fragments.
 * @param fragments - Fragments.
 * @returns The concatenation of the capitalised fragments.
 */
function getStructName(...fragments: string[]) {
  return fragments.map(toPascalCase).join('') + 'Struct';
}

/**
 * Returns a Noir type name for the given ABI variable.
 * @param param - ABI variable to translate to a Noir type name.
 * @param parentNames - Function name or parent structs or arrays to use for struct qualified names.
 * @returns A valid Noir basic type name or a name for a struct.
 */
function getTypeName(param: ABIVariable, ...parentNames: string[]): string {
  const type = param.type;
  switch (type.kind) {
    case 'field':
      return 'Field';
    case 'boolean':
      return 'bool';
    case 'integer':
      return `${type.sign === 'signed' ? 'i' : 'u'}${type.width}`;
    case 'string':
      throw new Error(`Strings not supported yet`);
    case 'array':
      return `[${getTypeName({ name: param.name, type: type.type }, ...parentNames)};${type.length}]`;
    case 'struct':
      return getStructName(param.name, ...parentNames);
    default:
      throw new Error(`Unknown type ${type}`);
  }
}

/**
 * Generates a parameter string.
 * @param param - ABI parameter.
 * @param functionData - Parent function.
 * @returns A Noir string with the param name and type to be used in a function call.
 */
function generateParameter(param: ABIParameter, functionData: FunctionAbi) {
  const typename = getTypeName(param, functionData.name);
  return `${param.name}: ${typename}`;
}

/**
 * Collects all parameters for a given function and flattens them according to how they should be serialized.
 * @param parameters - Parameters for a function.
 * @returns List of parameters flattened to basic data types.
 */
function collectParametersForSerialization(parameters: ABIVariable[]) {
  const flattened: string[] = [];
  for (const parameter of parameters) {
    const { name } = parameter;
    if (parameter.type.kind === 'array') {
      const nestedType = parameter.type.type;
      const nested = times(parameter.type.length, i =>
        collectParametersForSerialization([{ name: `${name}[${i}]`, type: nestedType }]),
      );
      flattened.push(...nested.flat());
    } else if (parameter.type.kind === 'struct') {
      const nested = parameter.type.fields.map(field =>
        collectParametersForSerialization([{ name: `${name}.${field.name}`, type: field.type }]),
      );
      flattened.push(...nested.flat());
    } else if (parameter.type.kind === 'string') {
      throw new Error(`String not yet supported`);
    } else if (parameter.type.kind === 'field') {
      flattened.push(name);
    } else {
      flattened.push(`${name} as Field`);
    }
  }
  return flattened;
}

/**
 * Generates Noir code for serialising the parameters into an array of fields.
 * @param parameters - Parameters to serialize.
 * @returns The serialization code.
 */
function generateSerialization(parameters: ABIParameter[]) {
  const flattened = collectParametersForSerialization(parameters);
  const declaration = `    let mut serialized_args = [0; ${flattened.length}];`;
  const lines = flattened.map((param, i) => `    serialized_args[${i}] = ${param};`);
  return [declaration, ...lines].join('\n');
}

/**
 * Generate a function interface for a particular function of the Aztec.nr Contract being processed. This function will be a method of the ContractInterface struct being created here.
 * @param functionData - Data relating to the function, which can be used to generate a callable Aztec.nr Function.
 * @param kind - Whether this interface will be used from private or public functions.
 * @returns A code string.
 */
function generateFunctionInterface(functionData: FunctionAbi, kind: 'private' | 'public') {
  const { name, parameters } = functionData;
  const selector = FunctionSelector.fromNameAndParameters(name, parameters);
  const serialization = generateSerialization(parameters);
  const contextType = kind === 'private' ? '&mut PrivateContext' : 'PublicContext';
  const callStatement = generateCallStatement(selector, functionData.functionType);
  const allParams = ['self', `context: ${contextType}`, ...parameters.map(p => generateParameter(p, functionData))];
  const isPrivate = isPrivateCall(functionData.functionType);
  const isSync = (isPrivate && kind === 'private') || (!isPrivate && kind === 'public');
  const retType = isSync ? `-> [Field; RETURN_VALUES_LENGTH] ` : ``;

  return `
  fn ${name}(
    ${allParams.join(',\n    ')}
  ) ${retType}{
${serialization}
${callStatement}
  }
  `;
}

/**
 * Generates static imports.
 * @returns A string of code which will be needed in every contract interface, regardless of the contract.
 */
function generateStaticImports() {
  return `use dep::std;
use dep::aztec::context::{ PrivateContext, PublicContext };
use dep::aztec::constants_gen::RETURN_VALUES_LENGTH;`;
}

/**
 * Generates the name of the contract struct, based on whether it's for private or public usage.
 * @param contractName - Name of the contract.
 * @param kind - Whether this interface will be used from private or public functions.
 * @returns A name.
 */
function generateContractStructName(contractName: string, kind: 'private' | 'public') {
  return `${contractName}${capitalize(kind)}ContextInterface`;
}

/**
 * Generate the main focus of this code generator: the contract interface struct.
 * @param contractName - the name of the contract, as matches the original source file.
 * @param kind - Whether this interface will be used from private or public functions.
 * @returns Code.
 */
function generateContractInterfaceStruct(contractName: string, kind: 'private' | 'public') {
  return `// Interface for calling ${contractName} functions from a ${kind} context
struct ${generateContractStructName(contractName, kind)} {
  address: Field,
}
`;
}

/**
 * Generates the implementation of the contract interface struct.
 * @param contractName - The name of the contract, as matches the original source file.
 * @param kind - Whether this interface will be used from private or public functions.
 * @param functions - An array of strings, where each string is valid Noir code describing the function interface of one of the contract's functions (as generated via `generateFunctionInterface` above).
 * @returns Code.
 */
function generateContractInterfaceImpl(contractName: string, kind: 'private' | 'public', functions: string[]) {
  return `impl ${generateContractStructName(contractName, kind)} {
  fn at(address: Field) -> Self {
      Self {
          address,
      }
  }
  ${functions.join('\n')}
}
`;
}

/** Represents a struct along its parent names to derive a fully qualified name. */
type StructInfo = ABIVariable & { /** Parent name */ parentNames: string[] };

/**
 * Generates a Noir struct.
 * @param struct - Struct info.
 * @returns Code representing the struct.
 */
function generateStruct(struct: StructInfo) {
  const fields = (struct.type as StructType).fields.map(
    field => `  ${field.name}: ${getTypeName(field, struct.name, ...struct.parentNames)},`,
  );

  return `
struct ${getStructName(struct.name, ...struct.parentNames)} {
${fields.join('\n')}
}`;
}

/**
 * Collects all structs across all parameters.
 * @param params - Parameters to look for structs, either structs themselves or nested.
 * @param parentNames - Parent names to derive fully qualified names when needed.
 * @returns A list of struct infos.
 */
function collectStructs(params: ABIVariable[], parentNames: string[]): StructInfo[] {
  const structs: StructInfo[] = [];
  for (const param of params) {
    if (param.type.kind === 'struct') {
      const struct = { ...param, parentNames };
      structs.push(struct, ...collectStructs(param.type.fields, [param.name, ...parentNames]));
    } else if (param.type.kind === 'array') {
      structs.push(...collectStructs([{ name: param.name, type: param.type.type }], [...parentNames]));
    }
  }
  return structs;
}

/**
 * Generates the struct definition and implementation for a contract interface.
 * @param abiName - Name of the contract.
 * @param kind - Whether this interface will be used from private or public functions.
 * @param methods - Contract methods to generate (private ones will be excluded if kind is public)
 * @returns Code.
 */
function generateContractStruct(abiName: string, kind: 'private' | 'public', methods: FunctionAbi[]) {
  const contractStruct: string = generateContractInterfaceStruct(abiName, kind);
  const applicableMethods = methods.filter(m => kind === 'private' || !isPrivateCall(m.functionType));
  const functionInterfaces = applicableMethods.map(m => generateFunctionInterface(m, kind));
  const contractImpl: string = generateContractInterfaceImpl(abiName, kind, functionInterfaces);

  return `
${contractStruct}
${contractImpl}  
  `;
}

/**
 * Generates the Noir code to represent an interface for calling a contract.
 * @param abi - The compiled Aztec.nr artifact.
 * @returns The corresponding ts code.
 */
export function generateNoirContractInterface(abi: ContractAbi) {
  // We don't allow calling a constructor, internal fns, or unconstrained fns from other contracts
  const methods = compact(
    abi.functions.filter(
      f => f.name !== 'constructor' && !f.isInternal && f.functionType !== FunctionType.UNCONSTRAINED,
    ),
  );
  const paramStructs = methods.flatMap(m => collectStructs(m.parameters, [m.name])).map(generateStruct);
  const privateContractStruct = generateContractStruct(abi.name, 'private', methods);
  const publicContractStruct = generateContractStruct(abi.name, 'public', methods);

  return `/* Autogenerated file, do not edit! */
  
${generateStaticImports()}
${paramStructs.join('\n')}

${privateContractStruct}

${publicContractStruct}
`;
}
