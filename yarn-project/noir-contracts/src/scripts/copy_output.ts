import { readFileSync, writeFileSync } from 'fs';
import camelCase from 'lodash.camelcase';
import snakeCase from 'lodash.snakecase';
import upperFirst from 'lodash.upperfirst';
import mockedKeys from './mockedKeys.json' assert { type: 'json' };
import { ABIParameter, ABIType, FunctionType } from '../abi.js';

const STATEMENT_TYPES = ['type', 'params', 'return'] as const;

function getFunction(type: FunctionType, params: ABIParameter[], returns: ABIType[], fn: any) {
  if (!params) throw new Error(`ABI comment not found for function ${fn.name}`);
  return {
    name: fn.name,
    functionType: type,
    parameters: params,
    returnTypes: returns,
    bytecode: Buffer.from(fn.bytecode).toString('hex'),
    // verificationKey: Buffer.from(fn.verification_key).toString('hex'),
    verificationKey: mockedKeys.verificationKey,
  };
}

function getFunctions(source: string, output: any) {
  const abiComments = Array.from(source.matchAll(/\/\/\/ ABI (\w+) (params|return|type) (.+)/g)).map(match => ({
    functionName: match[1],
    statementType: match[2],
    value: JSON.parse(match[3]),
  }));

  return output.functions
    .sort((fnA: any, fnB: any) => fnA.name.localeCompare(fnB.name))
    .map((fn: any) => {
      delete fn.proving_key;
      const thisFunctionAbisComments = abiComments
        .filter(abi => abi.functionName === fn.name)
        .reduce(
          (acc, comment) => ({
            ...acc,
            [comment.statementType]: comment.value,
          }),
          {} as Record<(typeof STATEMENT_TYPES)[number], any>,
        );
      return getFunction(
        thisFunctionAbisComments.type || 'secret',
        thisFunctionAbisComments.params || fn.abi.parameters,
        thisFunctionAbisComments.return || [fn.abi.return_type],
        fn,
      );
    });
}

function main() {
  const name = process.argv[2];
  if (!name) throw new Error(`Missing argument contract name`);

  const folder = `src/contracts/${snakeCase(name)}_contract`;
  const source = readFileSync(`${folder}/src/main.nr`).toString();
  const contractName = process.argv[3] ?? upperFirst(camelCase(name));
  const build = JSON.parse(readFileSync(`${folder}/target/main-${contractName}.json`).toString());
  const examples = `src/examples`;

  const abi = {
    name: build.name,
    functions: getFunctions(source, build),
  };

  const exampleFile = `${examples}/${snakeCase(name)}_contract.json`;
  writeFileSync(exampleFile, JSON.stringify(abi, null, 2) + '\n');
  console.log(`Written ${exampleFile}`);
}

try {
  main();
} catch (err: unknown) {
  console.error(err);
  process.exit(1);
}
