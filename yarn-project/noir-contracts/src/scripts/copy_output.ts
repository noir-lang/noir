import { readFileSync, writeFileSync } from 'fs';
import camelCase from 'lodash.camelcase';
import snakeCase from 'lodash.snakecase';
import upperFirst from 'lodash.upperfirst';
import mockedKeys from './mockedKeys.json' assert { type: 'json' };
import { ABIParameter, ABIType, FunctionType } from '@aztec/foundation/abi';
import { createLogger } from '@aztec/foundation/log';

const STATEMENT_TYPES = ['type', 'params', 'return'] as const;
const log = createLogger('aztec:noir-contracts');

/**
 * Creates an Aztec function entry.
 * @param type - The type of the function.
 * @param params - The parameters of the function.
 * @param returns - The return types of the function.
 * @param fn - The nargo function entry.
 * @returns The Aztec function entry.
 */
function getFunction(type: FunctionType, params: ABIParameter[], returns: ABIType[], fn: any) {
  if (!params) throw new Error(`ABI comment not found for function ${fn.name}`);
  // If the function is not unconstrained, the first item is inputs or CallContext which we should omit
  if (type !== FunctionType.UNCONSTRAINED) params = params.slice(1);
  // If the function is not secret, drop any padding from the end
  if (type !== FunctionType.SECRET && params[params.length - 1].name.endsWith('padding'))
    params = params.slice(0, params.length - 1);

  return {
    name: fn.name,
    functionType: type,
    parameters: params,
    // If the function is secret, the return is the public inputs, which should be omitted
    returnTypes: type === FunctionType.SECRET ? [] : returns,
    bytecode: Buffer.from(fn.bytecode).toString('hex'),
    // verificationKey: Buffer.from(fn.verification_key).toString('hex'),
    verificationKey: mockedKeys.verificationKey,
  };
}

/**
 * Creates the Aztec function entries from the source code and the nargo output.
 * @param source - The source code of the contract.
 * @param output - The nargo output.
 * @returns The Aztec function entries.
 */
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
        thisFunctionAbisComments.type || (fn.function_type.toLowerCase() as FunctionType),
        thisFunctionAbisComments.params || fn.abi.parameters,
        thisFunctionAbisComments.return || [fn.abi.return_type],
        fn,
      );
    });
}

const main = () => {
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
  log(`Written ${exampleFile}`);
};

try {
  main();
} catch (err: unknown) {
  log(err);
  process.exit(1);
}
