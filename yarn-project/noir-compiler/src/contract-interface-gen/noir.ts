import {
  ABIParameter,
  ABIVariable,
  ContractAbi,
  FunctionAbi,
  FunctionType,
  StructType,
  generateFunctionSelector,
} from '@aztec/foundation/abi';

import camelCase from 'lodash.camelcase';
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
function generateCallStatement(selector: string, functionType: FunctionType) {
  const callMethod = isPrivateCall(functionType) ? 'call_private_function' : 'call_public_function';
  return `
    context.${callMethod}(self.address, ${selector}, serialised_args)`;
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
 * Collects all parameters for a given function and flattens them according to how they should be serialised.
 * @param parameters - Paramters for a function.
 * @returns List of parameters flattened to basic data types.
 */
function collectParametersForSerialisation(parameters: ABIVariable[]) {
  const flattened: string[] = [];
  for (const parameter of parameters) {
    const { name } = parameter;
    if (parameter.type.kind === 'array') {
      const nestedType = parameter.type.type;
      const nested = times(parameter.type.length, i =>
        collectParametersForSerialisation([{ name: `${name}[${i}]`, type: nestedType }]),
      );
      flattened.push(...nested.flat());
    } else if (parameter.type.kind === 'struct') {
      const nested = parameter.type.fields.map(field =>
        collectParametersForSerialisation([{ name: `${name}.${field.name}`, type: field.type }]),
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
 * @param parameters - Parameters to serialise.
 * @returns The serialisation code.
 */
function generateSerialisation(parameters: ABIParameter[]) {
  const flattened = collectParametersForSerialisation(parameters);
  const declaration = `    let mut serialised_args = [0; ${flattened.length}];`;
  const lines = flattened.map((param, i) => `    serialised_args[${i}] = ${param};`);
  return [declaration, ...lines].join('\n');
}

/**
 * Generate a function interface for a particular function of the Noir Contract being processed. This function will be a method of the ContractInterface struct being created here.
 * @param functionData - data relating to the function, which can be used to generate a callable Noir Function.
 * @returns a code string.
 */
function generateFunctionInterface(functionData: FunctionAbi) {
  const { name, parameters } = functionData;
  const selector = '0x' + generateFunctionSelector(name, parameters).toString('hex');
  const serialisation = generateSerialisation(parameters);
  const callStatement = generateCallStatement(selector, functionData.functionType);
  const allParams = ['self', 'context: &mut Context', ...parameters.map(p => generateParameter(p, functionData))];
  const retType = isPrivateCall(functionData.functionType) ? `-> [Field; RETURN_VALUES_LENGTH] ` : ``;

  return `
  fn ${name}(
    ${allParams.join(',\n    ')}
  ) ${retType}{
${serialisation}
${callStatement}
  }
  `;
}

/**
 * Generates static impots.
 * @returns A string of code which will be needed in every contract interface, regardless of the contract.
 */
function generateStaticImports() {
  return `use dep::std;
use dep::aztec::context::Context;
use dep::aztec::constants_gen::RETURN_VALUES_LENGTH;`;
}

/**
 * Generate the main focus of this code generator: the contract interface struct.
 * @param contractName - the name of the contract, as matches the original source file.
 * @returns Code.
 */
function generateContractInterfaceStruct(contractName: string) {
  return `struct ${contractName}ContractInterface {
  address: Field,
}
`;
}

/**
 * Generates the implementation of the contract interface struct.
 * @param contractName - The name of the contract, as matches the original source file.
 * @param functions - An array of strings, where each string is valid Noir code describing the function interface of one of the contract's functions (as generated via `generateFunctionInterface` above).
 * @returns Code.
 */
function generateContractInterfaceImpl(contractName: string, functions: string[]) {
  return `impl ${contractName}ContractInterface {
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
 * Generates the Noir code to represent an interface for calling a contract.
 * @param abi - The compiled Noir artifact.
 * @returns The corresponding ts code.
 */
export function generateNoirContractInterface(abi: ContractAbi) {
  // We don't allow calling a constructor, internal fns, or unconstrained fns from other contracts
  const methods = compact(
    abi.functions.filter(
      f => f.name !== 'constructor' && !f.isInternal && f.functionType !== FunctionType.UNCONSTRAINED,
    ),
  );
  const contractStruct: string = generateContractInterfaceStruct(abi.name);
  const paramStructs = methods.flatMap(m => collectStructs(m.parameters, [m.name])).map(generateStruct);
  const functionInterfaces = methods.map(generateFunctionInterface);
  const contractImpl: string = generateContractInterfaceImpl(abi.name, functionInterfaces);

  return `/* Autogenerated file, do not edit! */
  
${generateStaticImports()}
${paramStructs.join('\n')}

${contractStruct}
${contractImpl}
`;
}
