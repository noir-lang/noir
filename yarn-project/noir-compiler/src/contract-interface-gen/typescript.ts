import {
  ABIParameter,
  ContractAbi,
  FunctionAbi,
  isAztecAddressStruct,
  isEthereumAddressStruct,
} from '@aztec/foundation/abi';

import compact from 'lodash.compact';

/**
 * Returns the corresponding typescript type for a given Noir type.
 * @param type - The input Noir type.
 * @returns An equivalent typescript type.
 */
function abiTypeToTypescript(type: ABIParameter['type']): string {
  switch (type.kind) {
    case 'field':
      return 'FieldLike';
    case 'boolean':
      return 'boolean';
    case 'integer':
      return '(bigint | number)';
    case 'string':
      return 'string';
    case 'array':
      return `${abiTypeToTypescript(type.type)}[]`;
    case 'struct':
      if (isEthereumAddressStruct(type)) return 'EthAddressLike';
      if (isAztecAddressStruct(type)) return 'AztecAddressLike';
      return `{ ${type.fields.map(f => `${f.name}: ${abiTypeToTypescript(f.type)}`).join(', ')} }`;
    default:
      throw new Error(`Unknown type ${type}`);
  }
}

/**
 * Generates the typescript code to represent a Noir parameter.
 * @param param - A Noir parameter with name and type.
 * @returns The corresponding ts code.
 */
function generateParameter(param: ABIParameter) {
  return `${param.name}: ${abiTypeToTypescript(param.type)}`;
}

/**
 * Generates the typescript code to represent a Noir function as a type.
 * @param param - A Noir function.
 * @returns The corresponding ts code.
 */
function generateMethod(entry: FunctionAbi) {
  const args = entry.parameters.map(generateParameter).join(', ');
  return `
    /** ${entry.name}(${entry.parameters.map(p => `${p.name}: ${p.type.kind}`).join(', ')}) */
    ${entry.name}: ((${args}) => ContractFunctionInteraction) & Pick<ContractMethod, 'selector'>;`;
}

/**
 * Generates a deploy method for this contract.
 * @param input - ABI of the contract.
 * @returns A type-safe deploy method in ts.
 */
function generateDeploy(input: ContractAbi) {
  const ctor = input.functions.find(f => f.name === 'constructor');
  const args = (ctor?.parameters ?? []).map(generateParameter).join(', ');
  const abiName = `${input.name}ContractAbi`;

  return `
  /**
   * Creates a tx to deploy a new instance of this contract.
   */
  public static deploy(pxe: PXE, ${args}) {
    return new DeployMethod<${input.name}Contract>(Point.ZERO, pxe, ${abiName}, Array.from(arguments).slice(1));
  }

  /**
   * Creates a tx to deploy a new instance of this contract using the specified public key to derive the address.
   */
  public static deployWithPublicKey(pxe: PXE, publicKey: PublicKey, ${args}) {
    return new DeployMethod<${input.name}Contract>(publicKey, pxe, ${abiName}, Array.from(arguments).slice(2));
  }
  `;
}

/**
 * Generates the constructor by supplying the ABI to the parent class so the user doesn't have to.
 * @param name - Name of the contract to derive the ABI name from.
 * @returns A constructor method.
 * @remarks The constructor is private because we want to force the user to use the create method.
 */
function generateConstructor(name: string) {
  return `
  private constructor(
    /** The deployed contract's complete address. */
    completeAddress: CompleteAddress,
    /** The wallet. */
    wallet: Wallet,
  ) {
    super(completeAddress, ${name}ContractAbi, wallet);
  }
  `;
}

/**
 * Generates the at method for this contract.
 * @param name - Name of the contract to derive the ABI name from.
 * @returns An at method.
 * @remarks We don't use constructor directly because of the async `wallet.getContractData` call.
 */
function generateAt(name: string) {
  return `
  /**
   * Creates a contract instance.
   * @param address - The deployed contract's address.
   * @param wallet - The wallet to use when interacting with the contract.
   * @returns A promise that resolves to a new Contract instance.
   */
  public static async at(
    /** The deployed contract's address. */
    address: AztecAddress,
    /** The wallet. */
    wallet: Wallet,
  ) {
    const extendedContractData = await wallet.getExtendedContractData(address);
    if (extendedContractData === undefined) {
      throw new Error('Contract ' + address.toString() + ' is not deployed');
    }
    return new ${name}Contract(extendedContractData.getCompleteAddress(), wallet);
  }`;
}

/**
 * Generates a static getter for the contract's ABI.
 * @param name - Name of the contract used to derive name of the ABI import.
 */
function generateAbiGetter(name: string) {
  const abiName = `${name}ContractAbi`;
  return `
  /**
   * Returns this contract's ABI.
   */
  public static get abi(): ContractAbi {
    return ${abiName};
  }
  `;
}

/**
 * Generates statements for importing the abi from a json artifact and re-exporting it.
 * @param name - Name of the contract.
 * @param abiImportPath - Path to load the ABI from.
 * @returns Code.
 */
function generateAbiStatement(name: string, abiImportPath: string) {
  const stmts = [
    `import ${name}ContractAbiJson from '${abiImportPath}' assert { type: 'json' };`,
    `export const ${name}ContractAbi = ${name}ContractAbiJson as ContractAbi;`,
  ];
  return stmts.join('\n');
}

/**
 * Generates the typescript code to represent a contract.
 * @param input - The compiled Noir artifact.
 * @param abiImportPath - Optional path to import the ABI (if not set, will be required in the constructor).
 * @returns The corresponding ts code.
 */
export function generateTypescriptContractInterface(input: ContractAbi, abiImportPath?: string) {
  // `compact` removes all falsey values from an array
  const methods = compact(input.functions.filter(f => f.name !== 'constructor').map(generateMethod));
  const deploy = abiImportPath && generateDeploy(input);
  const ctor = abiImportPath && generateConstructor(input.name);
  const at = abiImportPath && generateAt(input.name);
  const abiStatement = abiImportPath && generateAbiStatement(input.name, abiImportPath);
  const abiGetter = abiImportPath && generateAbiGetter(input.name);

  return `
/* Autogenerated file, do not edit! */

/* eslint-disable */
import { AztecAddress, CompleteAddress, ContractBase, ContractFunctionInteraction, ContractMethod, DeployMethod, FieldLike, AztecAddressLike, EthAddressLike, Wallet } from '@aztec/aztec.js';
import { Fr, Point } from '@aztec/foundation/fields';
import { PXE, PublicKey } from '@aztec/types';
import { ContractAbi } from '@aztec/foundation/abi';
${abiStatement}

/**
 * Type-safe interface for contract ${input.name};
 */
export class ${input.name}Contract extends ContractBase {
  ${ctor}

  ${at}

  ${deploy}

  ${abiGetter}

  /** Type-safe wrappers for the public methods exposed by the contract. */
  public methods!: {
    ${methods.join('\n')}
  };
}
`;
}
