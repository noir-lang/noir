import { ContractAbi } from '@aztec/foundation/abi';
import { createConsoleLogger } from '@aztec/foundation/log';
import {
  generateAztecAbi,
  generateNoirContractInterface,
  generateTypescriptContractInterface,
} from '@aztec/noir-compiler';

import { readFileSync, writeFileSync } from 'fs';
import camelCase from 'lodash.camelcase';
import omit from 'lodash.omit';
import snakeCase from 'lodash.snakecase';
import upperFirst from 'lodash.upperfirst';
import { join as pathJoin } from 'path';

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
        // If we maintain debug symbols they will get commited to git.
        debug: undefined,
      };
      const targetFilename = pathJoin(projectContract.target, `${snakeCase(abi.name)}_contract.json`);
      writeFileSync(targetFilename, JSON.stringify(toWrite, null, 2) + '\n');
      log(`Written ${targetFilename}`);
    }
  }
}

const main = () => {
  const name = process.argv[2];
  if (!name) throw new Error(`Missing argument contract name`);

  const projectName = `${snakeCase(name)}_contract`;
  const projectDirPath = `src/contracts/${projectName}`;

  const contractName = upperFirst(camelCase(name));
  const artifactFile = `${projectName}-${contractName}.json`;

  const buildJsonFilePath = `${projectDirPath}/target/${artifactFile}`;
  const buildJson = JSON.parse(readFileSync(buildJsonFilePath).toString());

  const debugArtifactFile = `debug_${artifactFile}`;
  let debug = undefined;

  try {
    const debugJsonFilePath = `${projectDirPath}/target/${debugArtifactFile}`;
    const debugJson = JSON.parse(readFileSync(debugJsonFilePath).toString());
    if (debugJson) {
      debug = debugJson;
    }
  } catch (err) {
    // Ignore
  }

  // Remove extraneous information from the buildJson (which was output by Nargo) to hone in on the function data we actually care about:
  const artifactJson: ContractAbi = generateAztecAbi({ contract: buildJson, debug });

  // Write the artifact:
  const artifactsDir = 'src/artifacts';
  const artifactFileName = `${snakeCase(name)}_contract.json`;
  writeFileSync(pathJoin(artifactsDir, artifactFileName), JSON.stringify(artifactJson, null, 2) + '\n');
  log(`Written ${pathJoin(artifactsDir, artifactFileName)}`);

  // Write some artifacts to other packages in the monorepo:
  writeToProject(artifactJson);

  // Write a .ts contract interface, for consumption by the typescript code
  const tsInterfaceDestFilePath = `src/types/${name}.ts`;
  const tsAbiImportPath = `../artifacts/${artifactFileName}`;
  writeFileSync(tsInterfaceDestFilePath, generateTypescriptContractInterface(artifactJson, tsAbiImportPath));
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
