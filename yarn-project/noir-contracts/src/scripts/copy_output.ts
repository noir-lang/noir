import { ContractAbi, FunctionAbi, FunctionType } from '@aztec/foundation/abi';
import { createConsoleLogger } from '@aztec/foundation/log';
import { generateNoirContractInterface, generateTSContractInterface } from '@aztec/noir-compiler';

import { readFileSync, writeFileSync } from 'fs';
import camelCase from 'lodash.camelcase';
import omit from 'lodash.omit';
import snakeCase from 'lodash.snakecase';
import upperFirst from 'lodash.upperfirst';
import { join as pathJoin } from 'path';

import mockedKeys from './mockedKeys.json' assert { type: 'json' };

// const STATEMENT_TYPES = ['type', 'params', 'return'] as const;
const log = createConsoleLogger('aztec:noir-contracts');

const PROJECT_CONTRACTS = [
  { name: 'SchnorrSingleKeyAccount', target: '../aztec.js/src/abis/', exclude: [] },
  { name: 'SchnorrAccount', target: '../aztec.js/src/abis/', exclude: [] },
  { name: 'EcdsaAccount', target: '../aztec.js/src/abis/', exclude: [] },
];

const INTERFACE_CONTRACTS = ['test'];

/**
 * Writes the contract to a specific project folder, if needed.
 * @param abi - The Abi to write.
 */
function writeToProject(abi: any) {
  for (const projectContract of PROJECT_CONTRACTS) {
    if (abi.name === projectContract.name) {
      const toWrite = {
        ...abi,
        functions: abi.functions.map((f: any) => omit(f, projectContract.exclude)),
      };
      const targetFilename = pathJoin(projectContract.target, `${snakeCase(abi.name)}_contract.json`);
      writeFileSync(targetFilename, JSON.stringify(toWrite, null, 2) + '\n');
      log(`Written ${targetFilename}`);
    }
  }
}

/**
 * Creates an Aztec function entry.
 * @param type - The type of the function (secret | open | unconstrained).
 * @param params - The parameters of the function ( name, type, visibility ).
 * @param returns - The return types of the function.
 * @param fn - The nargo function entry.
 * @returns The Aztec function entry.
 */
function getFunction(fn: any): FunctionAbi {
  const type = fn.function_type.toLowerCase();
  const returns = fn.abi.return_type;
  const isInternal = fn.is_internal;
  let params = fn.abi.parameters;

  // If the function is not unconstrained, the first item is inputs or CallContext which we should omit
  if (type !== FunctionType.UNCONSTRAINED) params = params.slice(1);

  return {
    name: fn.name,
    functionType: type,
    isInternal,
    parameters: params,
    // If the function is secret, the return is the public inputs, which should be omitted
    returnTypes: type === FunctionType.SECRET ? [] : [returns],
    bytecode: fn.bytecode,
    // verificationKey: Buffer.from(fn.verification_key).toString('hex'),
    verificationKey: mockedKeys.verificationKey,
  };
}

/**
 * Creates the Aztec function entries from the source code and the nargo output.
 * @param sourceCode - The source code of the contract.
 * @param buildJson - The nargo output.
 * @returns The Aztec function entries.
 */
function getFunctions(_sourceCode: string, buildJson: any): FunctionAbi[] {
  /**
   * Sort functions alphabetically, by name.
   * Remove the proving key field of the function.
   *
   */
  return buildJson.functions
    .sort((fnA: any, fnB: any) => fnA.name.localeCompare(fnB.name))
    .map((fn: any) => {
      delete fn.proving_key;
      return getFunction(fn);
    });
}

const main = () => {
  const name = process.argv[2];
  if (!name) throw new Error(`Missing argument contract name`);

  const projectName = `${snakeCase(name)}_contract`;
  const projectDirPath = `src/contracts/${projectName}`;
  const sourceCodeFilePath = `${projectDirPath}/src/main.nr`;
  const sourceCode = readFileSync(sourceCodeFilePath).toString();

  const contractName = upperFirst(camelCase(name));
  const buildJsonFilePath = `${projectDirPath}/target/${projectName}-${contractName}.json`;
  const buildJson = JSON.parse(readFileSync(buildJsonFilePath).toString());

  // Remove extraneous information from the buildJson (which was output by Nargo) to hone in on the function data we actually care about:
  const artifactJson: ContractAbi = {
    name: buildJson.name,
    functions: getFunctions(sourceCode, buildJson),
  };

  // Write the artifact:
  const artifactsDir = 'src/artifacts';
  const artifactDestFilePath = `${artifactsDir}/${snakeCase(name)}_contract.json`;
  writeFileSync(artifactDestFilePath, JSON.stringify(artifactJson, null, 2) + '\n');
  log(`Written ${artifactDestFilePath}`);

  // Write some artifacts to other packages in the monorepo:
  writeToProject(artifactJson);

  // Write a .ts contract interface, for consumption by the typescript code
  const tsInterfaceDestFilePath = `src/types/${name}.ts`;
  writeFileSync(tsInterfaceDestFilePath, generateTSContractInterface(artifactJson, '../artifacts/index.js'));
  log(`Written ${tsInterfaceDestFilePath}`);

  // Write a .nr contract interface, for consumption by other Noir Contracts
  if (INTERFACE_CONTRACTS.includes(name)) {
    const noirInterfaceDestFilePath = `${projectDirPath}/src/${projectName}_interface.nr`;
    try {
      writeFileSync(noirInterfaceDestFilePath, generateNoirContractInterface(artifactJson));
      log(`Written ${noirInterfaceDestFilePath}`);
    } catch (err) {
      log(`Error generating noir interface for ${name}: ${err}`);
    }
  }
};

try {
  main();
} catch (err: unknown) {
  log(err);
  process.exit(1);
}
