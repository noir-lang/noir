import { readFileSync, writeFileSync } from 'fs';
import camelCase from 'lodash.camelcase';
import snakeCase from 'lodash.snakecase';
import upperFirst from 'lodash.upperfirst';
import mockedKeys from './mockedKeys.json' assert { type: 'json' };

function getFunction(params: any, returns: any, fn: any) {
  if (!params) throw new Error(`ABI comment not found for function ${fn.name}`);
  return {
    name: fn.name,
    functionType: fn.function_type,
    parameters: params,
    returnTypes: returns,
    bytecode: Buffer.from(fn.bytecode).toString('hex'),
    // verificationKey: Buffer.from(fn.verification_key).toString('hex'),
    verificationKey: mockedKeys.verificationKey,
  };
}

function getFunctions(source: string, output: any) {
  const abis = Array.from(source.matchAll(/\/\/\/ ABI (\w+) (params|return) (.+)/g)).map(match => ({
    functionName: match[1],
    abiType: match[2],
    interface: JSON.parse(match[3]),
  }));

  return output.functions
    .sort((fnA: any, fnB: any) => fnA.name.localeCompare(fnB.name))
    .map((fn: any) => {
      delete fn.proving_key;
      return getFunction(
        abis.find(abi => abi.functionName === fn.name && abi.abiType === 'params')?.interface || [],
        abis.find(abi => abi.functionName === fn.name && abi.abiType === 'return')?.interface || [],
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
