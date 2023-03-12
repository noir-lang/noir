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
      ts.factory.createStringLiteral(getImport(importPath, 'eth_address')),
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

function makeEventTypes(abi: ContractAbiDefinition) {
  return abi.filter(def => def.type === 'event').map(makeEventType);
}

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

function makeEventLogInterfaces(abi: ContractAbiDefinition) {
  return abi.filter(def => def.type === 'event').map(makeEventLogInterface);
}

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

function getTupleType(components: AbiInput[]): ts.TypeLiteralNode {
  return ts.factory.createTypeLiteralNode(
    components!.map(prop =>
      ts.factory.createPropertySignature(undefined, prop.name, undefined, getTsTypeFromSolidityType(prop)),
    ),
  );
}

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

function makeParameter(input: AbiInput, index: number) {
  return ts.factory.createParameterDeclaration(
    undefined,
    undefined,
    input.name || `a${index}`,
    undefined,
    getTsTypeFromSolidityType(input),
  );
}

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

function makeMethodsInterface(name: string, abi: ContractAbiDefinition) {
  const methods = abi.filter(def => def.type === 'function').map(def => makeMethodSignature(name, def));
  return ts.factory.createInterfaceDeclaration(undefined, `${name}Methods`, undefined, undefined, methods);
}

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

function makeAbiExport(name: string) {
  return ts.factory.createVariableStatement(
    [ts.factory.createModifier(ts.SyntaxKind.ExportKeyword)],
    [ts.factory.createVariableDeclaration(`${name}Abi`, undefined, undefined, ts.factory.createIdentifier('abi'))],
  );
}

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

function makeAndWriteAbi(outputPath: string, name: string, abi: ContractAbiDefinition, importPath: string) {
  const abiOutputFile = `${outputPath}/${name}Abi.ts`;
  const output = `import { ContractAbi } from '${getImport(
    importPath,
    'contract',
  )}';\nexport default new ContractAbi(${JSON.stringify(abi, undefined, 2)});`;
  fs.writeFileSync(abiOutputFile, output);
}

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
