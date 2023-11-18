import { LogFn } from '@aztec/foundation/log';

import { Command } from 'commander';
import { readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import { mkdirpSync } from 'fs-extra';
import path, { resolve } from 'path';

import { generateTypescriptContractInterface } from '../index.js';
import { isContractArtifact } from '../utils.js';

/**
 * Registers a 'typescript' command on the given commander program that generates typescript interface out of an ABI.
 * @param program - Commander program.
 * @param log - Optional logging function.
 * @returns The program with the command registered.
 */
export function generateTypescriptInterface(program: Command, name = 'typescript', log: LogFn = () => {}): Command {
  return program
    .command(name)
    .argument('<project-path>', 'Path to the noir project')
    .option('--artifacts <path>', 'Folder containing the compiled artifacts, relative to the project path', 'target')
    .option(
      '-o, --outdir <path>',
      'Output folder for the generated typescript wrappers, relative to the project path',
      'types',
    )
    .description('Generates typescript interfaces from the artifacts in the given project')

    .action(
      (
        projectPath: string,
        /* eslint-disable jsdoc/require-jsdoc */
        options: {
          outdir: string;
          artifacts: string;
        },
        /* eslint-enable jsdoc/require-jsdoc */
      ) => {
        const { outdir, artifacts } = options;
        if (typeof projectPath !== 'string') {
          throw new Error(`Missing project path argument`);
        }
        const currentDir = process.cwd();

        const artifactsDir = resolve(projectPath, artifacts);
        for (const artifactsDirItem of readdirSync(artifactsDir)) {
          const artifactPath = resolve(artifactsDir, artifactsDirItem);
          if (statSync(artifactPath).isFile() && artifactPath.endsWith('.json')) {
            const contract = JSON.parse(readFileSync(artifactPath).toString());
            if (!isContractArtifact(contract)) {
              continue;
            }
            const tsPath = resolve(projectPath, outdir, `${contract.name}.ts`);
            log(`Writing ${contract.name} typescript interface to ${path.relative(currentDir, tsPath)}`);
            let relativeArtifactPath = path.relative(path.dirname(tsPath), artifactPath);
            if (relativeArtifactPath === `${contract.name}.json`) {
              // relative path edge case, prepending ./ for local import - the above logic just does
              // `${contract.name}.json`, which is not a valid import for a file in the same directory
              relativeArtifactPath = `./${contract.name}.json`;
            }
            try {
              const tsWrapper = generateTypescriptContractInterface(contract, relativeArtifactPath);
              mkdirpSync(path.dirname(tsPath));
              writeFileSync(tsPath, tsWrapper);
            } catch (err) {
              log(`Error generating interface for ${artifactPath}: ${err}`);
            }
          }
        }
      },
    );
}
