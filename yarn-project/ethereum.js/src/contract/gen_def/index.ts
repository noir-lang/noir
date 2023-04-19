#!/usr/bin/env node
import sourceMapSupport from 'source-map-support';
sourceMapSupport.install();
import fs from 'fs';
import ts, { ClassElement, PropertySignature, TypeNode } from 'typescript';
import { AbiInput, AbiOutput, ContractAbiDefinition, ContractEntryDefinition } from '../abi/index.js';
import { ContractBuildData, loadDataFromConfig } from './sources/index.js';
import { Config } from './sources/config.js';
import { dirname } from 'path';

Error.stackTraceLimit = Infinity;

const printer = ts.createPrinter({
  newLine: ts.NewLineKind.LineFeed,
});

const getImport = (importPath: string, module: string) =>
  importPath[0] === '.' ? `${importPath}/${module}/index.js` : `${importPath}/${module}`;

/**
 * Generate an array of import declarations for the necessary modules and types used in the generated contract file.
 * This function handles relative or absolute import paths, and appends the appropriate module names.
 *
 * @param name - The name of the contract.
 * @param importPath - The base path for importing necessary modules.
 * @returns An array of TypeScript import declarations.
 */
function makeImports(name: string, importPath: string) {
  return [
    ts.factory.createImportDeclaration(
      undefined,
      ts.factory.createImportClause(
        false,
        undefined,
        ts.factory.createNamedImports([
          ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('EthAddress')),
        ]),
      ),
      ts.factory.createStringLiteral('@aztec/foundation'),
    ),
    ts.factory.createImportDeclaration(
      undefined,
      ts.factory.createImportClause(
        false,
        undefined,
        ts.factory.createNamedImports([
          ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('EthereumRpc')),
        ]),
      ),
      ts.factory.createStringLiteral(getImport(importPath, 'eth_rpc')),
    ),
    ts.factory.createImportDeclaration(
      undefined,
      ts.factory.createImportClause(
        false,
        undefined,
        ts.factory.createNamedImports([
          ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('Contract')),
          ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('ContractTxReceipt')),
          ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('EventLog')),
          ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('Options')),
          ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('TxCall')),
          ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('TxSend')),
          // ts.factory.createImportSpecifier(false, undefined, ts.factory.createIdentifier('EventSubscriptionFactory')),
        ]),
      ),
      ts.factory.createStringLiteral(getImport(importPath, 'contract')),
    ),
    ts.factory.createImportDeclaration(
      undefined,
      ts.factory.createImportClause(false, ts.factory.createIdentifier('* as Bytes'), undefined),
      ts.factory.createStringLiteral(`${importPath}/contract/bytes.js`),
    ),
    ts.factory.createImportDeclaration(
      undefined,
      ts.factory.createImportClause(false, ts.factory.createIdentifier('abi'), undefined),
      ts.factory.createStringLiteral(`./${name}Abi.js`),
    ),
  ];
}

/**
 * Generate a TypeScript type alias for an Ethereum contract event.
 * The resulting type alias represents the structure of the event arguments as an object,
 * with each property having the appropriate TypeScript type based on the Solidity type
 * of the corresponding event input.
 *
 * @param definition - The ContractEntryDefinition representing the Ethereum contract event.
 * @returns A TypeScript TypeAliasDeclaration representing the event structure.
 */
function makeEventType(definition: ContractEntryDefinition) {
  const props = ts.factory.createTypeLiteralNode(
    definition.inputs!.map(input =>
      ts.factory.createPropertySignature(undefined, input.name, undefined, getTsTypeFromSolidityType(input)),
    ),
  );

  return ts.factory.createTypeAliasDeclaration(
    [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
    `${definition.name}Event`,
    undefined,
    props,
  );
}

/**
 * Generate TypeScript type aliases for the event types specified in the given Contract ABI definition.
 * Each event type alias is created by mapping the corresponding inputs of the event to their appropriate
 * TypeScript types based on their Solidity data types.
 *
 * @param abi - The ContractAbiDefinition containing the events for which type aliases will be generated.
 * @returns An array of TypeScript type alias declarations for the event types.
 */
function makeEventTypes(abi: ContractAbiDefinition) {
  return abi.filter(def => def.type === 'event').map(makeEventType);
}

/**
 * Create an interface for event logs of a given ContractEntryDefinition.
 * The generated interface extends 'EventLog' with the name and structure of the specific event log in the given definition.
 * This helps in creating type-safe event logs for Ethereum smart contracts using the ABI.
 *
 * @param definition - The ContractEntryDefinition representing an event from the ABI.
 * @returns A TypeScript interface declaration for the event log based on the given definition.
 */
function makeEventLogInterface(definition: ContractEntryDefinition) {
  const eventName = `${definition.name!}Event`;
  return ts.factory.createInterfaceDeclaration(
    [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
    `${eventName}Log`,
    undefined,
    [
      ts.factory.createHeritageClause(ts.SyntaxKind.ExtendsKeyword, [
        ts.factory.createExpressionWithTypeArguments(ts.factory.createRegularExpressionLiteral('EventLog'), [
          ts.factory.createTypeReferenceNode(eventName, undefined),
          ts.factory.createLiteralTypeNode(ts.factory.createStringLiteral(definition.name!)),
        ]),
      ]),
    ],
    [],
  );
}

/**
 * Generate TypeScript interface declarations for event logs of a contract.
 * For each event in the ABI, it creates an exported interface extending 'EventLog' with event-specific properties.
 *
 * @param abi - The ContractAbiDefinition object representing the contract's ABI.
 * @returns An array of TypeScript InterfaceDeclaration nodes representing the event log interfaces.
 */
function makeEventLogInterfaces(abi: ContractAbiDefinition) {
  return abi.filter(def => def.type === 'event').map(makeEventLogInterface);
}

/**
 * Generate TypeScript interface for events in a given contract ABI.
 * The interface contains typed definitions of each event present in the ABI,
 * allowing them to be accessed and used effectively when working with the contract.
 *
 * @param name - The name of the contract.
 * @param abi - The ContractAbiDefinition array from the contract's ABI JSON.
 * @returns A TypeScript InterfaceDeclaration representing the events in the contract.
 */
function makeEventsInterface(name: string, abi: ContractAbiDefinition) {
  const events = abi.filter(def => def.type === 'event').map(event => event.name!);
  return ts.factory.createInterfaceDeclaration(
    undefined,
    `${name}Events`,
    undefined,
    undefined,
    events.map(eventName =>
      ts.factory.createPropertySignature(
        undefined,
        eventName,
        undefined,
        ts.factory.createTypeReferenceNode(`${eventName}Event`, undefined),
        // ts.factory.createTypeReferenceNode(`EventSubscriptionFactory`, [
        //   ts.factory.createTypeReferenceNode(`${eventName}EventLog`, undefined),
        // ]),
      ),
    ),
  );
}

/**
 * Generates an interface for Event Logs of the given name and Contract ABI definition.
 * The generated interface consists of property signatures with each property representing
 * an event log type for a specific event in the contract. It provides a way to access event logs
 * based on the event names.
 *
 * @param name - The name of the contract.
 * @param abi - The Contract ABI definition object containing event definitions.
 * @returns A TypeScript InterfaceDeclaration for the contract's event logs.
 */
function makeEventLogsInterface(name: string, abi: ContractAbiDefinition) {
  const events = abi.filter(def => def.type === 'event').map(event => event.name!);
  return ts.factory.createInterfaceDeclaration(
    undefined,
    `${name}EventLogs`,
    undefined,
    undefined,
    events.map(eventName =>
      ts.factory.createPropertySignature(
        undefined,
        eventName,
        undefined,
        ts.factory.createTypeReferenceNode(`${eventName}EventLog`, undefined),
      ),
    ),
  );
}

/**
 * Generates a TypeScript interface for the given contract's transaction event logs.
 * The generated interface includes properties for each event in the ABI, with their respective
 * log types as array values. This allows easier interaction and validation with contract events
 * during transaction execution.
 *
 * @param name - The name of the contract.
 * @param abi - The Contract ABI definition.
 * @returns A TypeScript InterfaceDeclaration for the contract's transaction event logs.
 */
function makeTxEventLogsInterface(name: string, abi: ContractAbiDefinition) {
  const events = abi.filter(def => def.type === 'event').map(event => event.name!);
  return ts.factory.createInterfaceDeclaration(
    undefined,
    `${name}TxEventLogs`,
    undefined,
    undefined,
    events.map(eventName =>
      ts.factory.createPropertySignature(
        undefined,
        eventName,
        undefined,
        ts.factory.createArrayTypeNode(ts.factory.createTypeReferenceNode(`${eventName}EventLog`, undefined)),
      ),
    ),
  );
}

/**
 * Generate a TypeScript interface for the transaction receipt of the given contract name.
 * The generated interface extends the 'ContractTxReceipt' type with the contract's specific event log types.
 *
 * @param name - The name of the contract.
 * @returns A TypeScript interface node representing the contract's transaction receipt.
 */
function makeTransactionReceiptInterface(name: string) {
  return ts.factory.createInterfaceDeclaration(
    [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
    `${name}TransactionReceipt`,
    undefined,
    [
      ts.factory.createHeritageClause(ts.SyntaxKind.ExtendsKeyword, [
        ts.factory.createExpressionWithTypeArguments(ts.factory.createRegularExpressionLiteral('ContractTxReceipt'), [
          ts.factory.createTypeReferenceNode(`${name}TxEventLogs`, undefined),
        ]),
      ]),
    ],
    [],
  );
}

/**
 * Get the TypeScript base type from a given Solidity type string.
 * Handles cases for unsigned and signed integer types, fixed types, byte arrays,
 * boolean, and Ethereum address types. For other types, it defaults to string.
 *
 * @param type - The Solidity type string to be converted.
 * @returns A TypeScript TypeNode representing the corresponding TypeScript type.
 */
function getBaseType(type: string /*, returnValue: boolean*/) {
  let m: RegExpMatchArray | null;
  if ((m = type.match(/u?int(\d*)/) || type.match(/u?fixed([0-9x]*)/))) {
    const width = m[1] ? +m[1] : 256;
    if (width <= 32) {
      return ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword);
      // return !returnValue
      //   ? ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword)
      //   : ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword);
    } else {
      return ts.factory.createKeywordTypeNode(ts.SyntaxKind.BigIntKeyword);
      // return !returnValue
      //   ? ts.factory.createKeywordTypeNode(ts.SyntaxKind.BigIntKeyword)
      //   : ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword);
    }
  }

  if ((m = type.match(/bytes(\d*)/))) {
    return ts.factory.createTypeReferenceNode(`Bytes.Bytes${m[1]}`, undefined);
  }

  if (type === 'bool') {
    return ts.factory.createKeywordTypeNode(ts.SyntaxKind.BooleanKeyword);
  }

  if (type === 'address') {
    return ts.factory.createTypeReferenceNode('EthAddress', undefined);
  }

  return ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword);
}

/**
 * Generate a TypeScript TypeLiteralNode representing a tuple type from an array of ABI inputs.
 * The resulting tuple type will have the corresponding TypeScript types based on the Solidity types
 * in the input components.
 *
 * @param components - An array of AbiInput objects that make up the tuple type.
 * @returns A TypeLiteralNode representing the generated tuple type in TypeScript.
 */
function getTupleType(components: AbiInput[]): ts.TypeLiteralNode {
  return ts.factory.createTypeLiteralNode(
    components!.map(prop =>
      ts.factory.createPropertySignature(undefined, prop.name, undefined, getTsTypeFromSolidityType(prop)),
    ),
  );
}

/**
 * Generates the TypeScript type corresponding to a given Solidity type.
 * Handles base types, tuples and arrays, including nested arrays.
 * For tuple types, generates a TypeLiteralNode with the components as properties.
 *
 * @param input - The AbiInput object containing information about the Solidity type.
 * @param type - An optional string representing the Solidity type (defaults to input.type).
 * @returns A TypeScript TypeNode representing the corresponding TypeScript type.
 */
function getTsTypeFromSolidityType(input: AbiInput, type?: string) {
  type = type || input.type;
  const arrayMatched = type.match(/(.+)\[\d*\]$/);
  if (arrayMatched) {
    const tsType = getTsTypeFromSolidityType(input, arrayMatched[1]);
    return ts.factory.createArrayTypeNode(tsType);
  } else {
    const isTuple = type === 'tuple';
    return isTuple ? getTupleType(input.components) : getBaseType(type);
  }
}

/**
 * Create a TypeScript parameter declaration from an ABI input.
 * This function is used to generate TypeScript method signatures for smart contract methods based on their inputs.
 * It takes an AbiInput object, which contains information about the name and type of the input,
 * and its index in the inputs array, to generate a matching TypeScript parameter with the appropriate type.
 *
 * @param input - The AbiInput object containing the name and type of the input parameter.
 * @param index - The index of the input parameter in the inputs array.
 * @returns A TypeScript ParameterDeclaration for the given input.
 */
function makeParameter(input: AbiInput, index: number) {
  return ts.factory.createParameterDeclaration(
    undefined,
    undefined,
    input.name || `a${index}`,
    undefined,
    getTsTypeFromSolidityType(input),
  );
}

/**
 * Generate TypeScript return type nodes for a given array of ABI outputs.
 * Handles multiple return values by creating an object with properties corresponding to the output names and indices.
 * Supports base types, tuple types, and array types based on the provided ABI outputs.
 *
 * @param outputs - Array of ABI outputs from a contract function.
 * @returns An array of TypeScript TypeNodes representing the return types.
 */
function generateReturnTypes(outputs: AbiOutput[]): ReadonlyArray<TypeNode> {
  if (outputs.length === 0) {
    return [];
  } else if (outputs.length === 1) {
    // original return value.
    return [getTsTypeFromSolidityType(outputs[0])];
  } else {
    // multiple return values: return an object.
    const propSigs: PropertySignature[] = [];
    for (let index = 0; index < outputs.length; index++) {
      const output = outputs[index];
      const type = getTsTypeFromSolidityType(output as AbiInput);
      if (output.name) {
        // name exists for the output: create a key for that
        const nameSig = ts.factory.createPropertySignature(
          undefined,
          ts.factory.createStringLiteral(output.name),
          undefined,
          type,
        );
        propSigs.push(nameSig);
      }
      // always create a key for the index.
      const indexSig = ts.factory.createPropertySignature(
        undefined,
        ts.factory.createNumericLiteral(index.toString()),
        undefined,
        type,
      );
      propSigs.push(indexSig);
    }
    return [ts.factory.createTypeLiteralNode(propSigs)];
  }
}

/**
 * Determine the TypeScript type representing the output of a given contract function based on its definition.
 * The output type is either 'TxCall' for view or pure functions, or 'TxSend' for functions that mutate state.
 * For multiple return values, the output type will be an object containing each of them.
 *
 * @param name - The name of the smart contract.
 * @param definition - The ContractEntryDefinition object representing the contract function.
 * @returns A TypeScript TypeNode representing the output type of the contract function.
 */
function getOutputType(name: string, definition: ContractEntryDefinition) {
  if (!definition.stateMutability) {
    if (definition.constant && definition.constant === true) {
      return ts.factory.createTypeReferenceNode('TxCall', generateReturnTypes(definition.outputs || []));
    } else {
      return ts.factory.createTypeReferenceNode('TxSend', [
        ts.factory.createTypeReferenceNode(`${name}TransactionReceipt`, undefined),
        ...generateReturnTypes(definition.outputs || []),
      ]);
    }
  }
  if (definition.stateMutability === 'view' || definition.stateMutability === 'pure') {
    if (definition.outputs && definition.outputs.length) {
      return ts.factory.createTypeReferenceNode('TxCall', generateReturnTypes(definition.outputs));
    } else {
      return ts.factory.createTypeReferenceNode('TxCall', [
        ts.factory.createKeywordTypeNode(ts.SyntaxKind.VoidKeyword),
      ]);
    }
  } else {
    return ts.factory.createTypeReferenceNode('TxSend', [
      ts.factory.createTypeReferenceNode(`${name}TransactionReceipt`, undefined),
      ...generateReturnTypes(definition.outputs || []),
    ]);
  }
}

/**
 * Generates a method signature for a contract function based on its ABI definition.
 * This method takes the name of the contract and the entry definition from the ABI,
 * creates TypeScript parameter declarations for the inputs of the function, and
 * sets the appropriate return type based on the stateMutability and outputs of the function.
 *
 * @param name - The name of the contract.
 * @param definition - The ABI entry definition for the contract function.
 * @returns A TypeScript methodSignature with the proper input parameters and output type.
 */
function makeMethodSignature(name: string, definition: ContractEntryDefinition) {
  return ts.factory.createMethodSignature(
    undefined,
    definition.name!,
    undefined,
    undefined,
    definition.inputs!.map(makeParameter),
    getOutputType(name, definition),
  );
}

/**
 * Generate a TypeScript interface for the contract methods based on the ABI definition.
 * The generated interface includes method signatures with input parameters and return types
 * according to the contract's ABI, which can be used for type checking and code generation.
 *
 * @param name - The name of the contract.
 * @param abi - The ContractAbiDefinition object containing the contract's ABI information.
 * @returns A TypeScript InterfaceDeclaration for the contract methods.
 */
function makeMethodsInterface(name: string, abi: ContractAbiDefinition) {
  const methods = abi.filter(def => def.type === 'function').map(def => makeMethodSignature(name, def));
  return ts.factory.createInterfaceDeclaration(undefined, `${name}Methods`, undefined, undefined, methods);
}

/**
 * Generate TypeScript code for a contract class which extends the Contract class.
 * The generated contract class includes methods, events, and event logs from the ABI.
 * If initData is provided, it also creates a deployment method.
 *
 * @param name - The name of the generated contract.
 * @param initData - The initialization data (constructor bytecode) for deploying the contract. Optional.
 * @param abi - The contract's ABI definition.
 * @returns A TypeScript ClassDeclaration node for the contract class.
 */
function makeContract(name: string, initData: string | undefined, abi: ContractAbiDefinition) {
  const members: ClassElement[] = [];

  const ctor = ts.factory.createConstructorDeclaration(
    undefined,
    [
      ts.factory.createParameterDeclaration(
        undefined,
        undefined,
        'eth',
        undefined,
        ts.factory.createTypeReferenceNode('EthereumRpc', undefined),
      ),
      ts.factory.createParameterDeclaration(
        undefined,
        undefined,
        'address',
        ts.factory.createToken(ts.SyntaxKind.QuestionToken),
        ts.factory.createTypeReferenceNode('EthAddress', undefined),
      ),
      ts.factory.createParameterDeclaration(
        undefined,
        undefined,
        'options',
        ts.factory.createToken(ts.SyntaxKind.QuestionToken),
        ts.factory.createTypeReferenceNode('Options', undefined),
      ),
    ],
    ts.factory.createBlock(
      [
        ts.factory.createExpressionStatement(
          ts.factory.createCallExpression(ts.factory.createSuper(), undefined, [
            ts.factory.createIdentifier('eth'),
            ts.factory.createIdentifier('abi'),
            ts.factory.createIdentifier('address'),
            ts.factory.createIdentifier('options'),
          ]),
        ),
      ],
      true,
    ),
  );

  members.push(ctor);

  if (initData) {
    const ctorDef = abi.find(def => def.type === 'constructor')!;
    const inputs = ctorDef && ctorDef.inputs ? ctorDef.inputs : [];
    const deployMethod = ts.factory.createMethodDeclaration(
      undefined,
      undefined,
      'deploy',
      undefined,
      undefined,
      inputs.map(makeParameter),
      ts.factory.createTypeReferenceNode('TxSend', [
        ts.factory.createTypeReferenceNode(`${name}TransactionReceipt`, undefined),
      ]),
      ts.factory.createBlock(
        [
          ts.factory.createReturnStatement(
            ts.factory.createAsExpression(
              ts.factory.createCallExpression(
                ts.factory.createPropertyAccessExpression(ts.factory.createSuper(), 'deployBytecode'),
                undefined,
                [
                  ts.factory.createStringLiteral(initData),
                  ...inputs.map(input => ts.factory.createIdentifier(input.name)),
                ],
              ),
              ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword),
            ),
          ),
        ],
        true,
      ),
    );
    members.push(deployMethod);
  }

  return ts.factory.createClassDeclaration(
    [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
    name,
    undefined,
    [
      ts.factory.createHeritageClause(ts.SyntaxKind.ExtendsKeyword, [
        ts.factory.createExpressionWithTypeArguments(ts.factory.createRegularExpressionLiteral('Contract'), [
          ts.factory.createTypeReferenceNode(`${name}Definition`, undefined),
        ]),
      ]),
    ],
    members,
  );
}

/**
 * Creates and returns a TypeScript interface declaration for the contract definition.
 * The generated interface contains property signatures for 'methods', 'events', and 'eventLogs'.
 * This interface serves as a type definition for the contract instance, providing type information
 * for its methods, events, and event logs, making it easier to interact with the contract in a type-safe manner.
 *
 * @param name - The name of the contract.
 * @returns A TypeScript InterfaceDeclaration representing the contract definition.
 */
function makeDefinitionInterface(name: string) {
  const props = [
    ts.factory.createPropertySignature(
      undefined,
      'methods',
      undefined,
      ts.factory.createTypeReferenceNode(`${name}Methods`, undefined),
    ),
    ts.factory.createPropertySignature(
      undefined,
      'events',
      undefined,
      ts.factory.createTypeReferenceNode(`${name}Events`, undefined),
    ),
    ts.factory.createPropertySignature(
      undefined,
      'eventLogs',
      undefined,
      ts.factory.createTypeReferenceNode(`${name}EventLogs`, undefined),
    ),
  ];

  return ts.factory.createInterfaceDeclaration(
    [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
    `${name}Definition`,
    undefined,
    undefined,
    props,
  );
}

/**
 * Generate a TypeScript export statement for the given ABI name.
 * The exported variable is named '$(name)Abi' and its value is the 'abi' identifier.
 *
 * @param name - The name of the contract ABI to be exported.
 * @returns A TypeScript export statement node.
 */
function makeAbiExport(name: string) {
  return ts.factory.createVariableStatement(
    [ts.factory.createModifier(ts.SyntaxKind.ExportKeyword)],
    [ts.factory.createVariableDeclaration(`${name}Abi`, undefined, undefined, ts.factory.createIdentifier('abi'))],
  );
}

/**
 * Generate TypeScript source code as AST nodes for a Contract based on the provided ABI and initialization data.
 * The generated code includes imports, event types, event log interfaces, events interface, transaction receipt interface,
 * methods interface, definition interface, contract class, and ABI export.
 *
 * @param name - The name of the Contract.
 * @param abi - The Contract ABI Definition.
 * @param initData - The initialization data (bytecode) of the Contract.
 * @param importPath - Path to '\@aztec/ethereum.js' package used for generating import statements.
 * @returns An array of TypeScript Nodes representing the generated source code.
 */
function makeFile(name: string, abi: ContractAbiDefinition, initData: string | undefined, importPath: string) {
  const imports = makeImports(name, importPath);
  const eventTypes = makeEventTypes(abi);
  const eventLogTypes = makeEventLogInterfaces(abi);
  const eventsInterface = makeEventsInterface(name, abi);
  const eventLogsInterface = makeEventLogsInterface(name, abi);
  const eventTxLogsInterface = makeTxEventLogsInterface(name, abi);
  const txReceiptInterface = makeTransactionReceiptInterface(name);
  const methodsInterface = makeMethodsInterface(name, abi);
  const definitionInterface = makeDefinitionInterface(name);
  const contract = makeContract(name, initData, abi);
  const abiExport = makeAbiExport(name);

  return ts.factory.createNodeArray([
    ...imports,
    ...eventTypes,
    ...eventLogTypes,
    eventsInterface,
    eventLogsInterface,
    eventTxLogsInterface,
    txReceiptInterface,
    methodsInterface,
    definitionInterface,
    contract,
    abiExport,
  ]);
}

/**
 * Generate the ABI (Application Binary Interface) for a contract and write it to a file.
 * The output file will be named as '(name)Abi.ts' in the specified 'outputPath'.
 * The generated file contains an exported default instance of ContractAbi class created
 * with the provided ABI definition. The import path for the required modules is also generated.
 *
 * @param outputPath - The path where the generated ABI file will be saved.
 * @param name - The name of the contract for which the ABI file is being generated.
 * @param abi - The ContractAbiDefinition object containing the ABI details of the contract.
 * @param importPath - The import path for the required modules in the generated file.
 */
function makeAndWriteAbi(outputPath: string, name: string, abi: ContractAbiDefinition, importPath: string) {
  const abiOutputFile = `${outputPath}/${name}Abi.ts`;
  const output = `import { ContractAbi } from '${getImport(
    importPath,
    'contract',
  )}';\nexport default new ContractAbi(${JSON.stringify(abi, undefined, 2)});`;
  fs.writeFileSync(abiOutputFile, output);
}

/**
 * Generate and write TypeScript interface and ABI files for a contract, based on the ABI
 * and other data from the build artifact. The generated files will be written to the specified
 * outputPath with the provided name.
 *
 * @param outputPath - The path where the output files will be created.
 * @param name - The name of the contract used as a prefix for the generated files.
 * @param buildData - An object containing the ABI and initData of the contract.
 * @param importPath - The import path for the '\@aztec/ethereum.js' module.
 * @returns A Promise that resolves when all files have been written.
 */
export async function makeAndWriteFiles(
  outputPath: string,
  name: string,
  { abi, initData }: ContractBuildData,
  importPath: string,
) {
  const interfaceOutputFile = `${outputPath}/${name}.ts`;

  const resultFile = ts.createSourceFile('', '', ts.ScriptTarget.Latest, false, ts.ScriptKind.TS);
  const nodes = makeFile(name, abi, initData, importPath);

  // Not sure how to make a single Node out of the NodeArray. Otherwise this would be a clean one liner.
  await fs.promises.unlink(interfaceOutputFile).catch(() => {});
  fs.appendFileSync(interfaceOutputFile, '// THIS IS GENERATED CODE, DO NOT EDIT!\n');
  fs.appendFileSync(interfaceOutputFile, '/* eslint-disable */\n');
  for (const node of nodes) {
    fs.appendFileSync(interfaceOutputFile, printer.printNode(ts.EmitHint.Unspecified, node, resultFile) + '\n');
  }

  makeAndWriteAbi(outputPath, name, abi, importPath);
}

/**
 * Execute the main function to generate and write contract files.
 * It reads the configuration from the provided JSON file or a default 'contracts.json'.
 * The function will output generated contract files into the specified outputPath,
 * and use the importPath for resolving imports in the generated code.
 *
 * @throws If an error occurs while reading the config file, creating directories, or generating output files.
 */
async function main() {
  const configFile = process.argv[2] || 'contracts.json';
  const config = JSON.parse(fs.readFileSync(configFile).toString()) as Config;
  const { outputPath = './contracts', importPath = '@aztec/ethereum.js' } = config;

  process.chdir(dirname(configFile));
  fs.mkdirSync(outputPath, { recursive: true });

  await Promise.all(
    Object.entries(config.contracts).map(async entry => {
      const buildData = await loadDataFromConfig(entry[1]);
      return makeAndWriteFiles(outputPath, entry[0], buildData, importPath);
    }),
  );
}

main().catch(console.error);
