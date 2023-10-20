/* eslint-disable jsdoc/require-jsdoc */
import { createConsoleLogger } from '@aztec/foundation/log';

import * as fs from 'fs';
import snakeCase from 'lodash.snakecase';
import * as path from 'path';
import { format } from 'util';

// heavily copying yarn-project/noir-contracts/src/scripts/copy_output.ts
const log = createConsoleLogger('aztec:noir-contracts:source_copy');

/**
 * for the typechecker...
 */
interface NoirSourceCopy {
  name: string;
  target: string;
  exclude: string[];
}

const NOIR_SOURCE_COPIES: NoirSourceCopy[] = [
  { name: 'PrivateToken', target: '../boxes/private-token/src/artifacts', exclude: [] },
];

/**
 * Sometimes we want to duplicate the noir source code elsewhere,
 * for example in the boxes we provide as Aztec Quickstarts.
 * @param contractName - UpperCamelCase contract name that we check need copying
 */
function copyNrFilesExceptInterface(contractName: string): void {
  // stored in `noir-contracts` under snake case nameing
  const snakeCaseContractName = `${snakeCase(contractName)}_contract`;
  const projectDirPath = `src/contracts/${snakeCaseContractName}`;

  for (const noirCopy of NOIR_SOURCE_COPIES) {
    if (noirCopy.name === contractName) {
      const target = noirCopy.target;

      try {
        // Ensure target directory exists
        if (!fs.existsSync(target)) {
          throw Error(`target copy path ${target} doesnt exist`);
        }
        // Read the project directory
        const files = fs.readdirSync(projectDirPath);

        // Filter and copy *.nr files except interface.nr
        files
          .filter(
            file =>
              file.endsWith('.nr') &&
              file !== 'interface.nr' &&
              (!noirCopy.exclude || !noirCopy.exclude.includes(file)),
          )
          .forEach(file => {
            const sourcePath = path.join(projectDirPath, file);
            const targetPath = path.join(target, file);
            log(`copying ${sourcePath} to ${targetPath}`);
            fs.copyFileSync(sourcePath, targetPath);
          });

        log(`Copied .nr files from ${contractName} to ${target} successfully!`);
      } catch (err) {
        log(format(`Error copying files from ${contractName} to ${target}:`, err));
      }
    }
  }
}

const main = () => {
  const contractName = process.argv[2];
  if (!contractName) throw new Error(`Missing argument contract name`);

  copyNrFilesExceptInterface(contractName);
};

try {
  main();
} catch (err: unknown) {
  log(format(`Error copying build output`, err));
  process.exit(1);
}
