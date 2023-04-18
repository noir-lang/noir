import { readFileSync, writeFileSync } from 'fs';
import camelCase from 'lodash.camelcase';
import snakeCase from 'lodash.snakecase';
import upperFirst from 'lodash.upperfirst';

function getFunction(params: any, fn: any) {
  if (!params) throw new Error(`ABI comment not found for function ${fn.name}`);
  return {
    name: fn.name,
    functionType: fn.func_type,
    parameters: params,
    returnTypes: [],
    bytecode: Buffer.from(fn.function.circuit).toString('hex'),
  };
}

function getFunctions(source: string, output: any) {
  const abis = Array.from(source.matchAll(/\/\/\/ ABI (\w+) (.+)/g)).map(match => ({
    name: match[1],
    params: JSON.parse(match[2]),
  }));

  return Object.keys(output.functions).map((name: string) =>
    getFunction(abis.find(abi => abi.name === name)?.params, { ...output.functions[name], name: name }),
  );
}

function getVerificationKey(folder: string, contractName: string, functionName: string) {
  return readFileSync(`${folder}/target/main-${contractName}-${functionName}.vk`).toString();
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
    functions: getFunctions(source, build).map(fn => ({
      ...fn,
      verificationKey: getVerificationKey(folder, contractName, fn.name),
    })),
  };

  const exampleFile = `${examples}/${snakeCase(name)}_contract.json`;
  writeFileSync(exampleFile, JSON.stringify(abi, null, 2));
  console.log(`Written ${exampleFile}`);
}

try {
  main();
} catch (err: unknown) {
  console.error(err);
  process.exit(1);
}
