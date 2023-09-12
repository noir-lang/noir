import { LogFn } from '@aztec/foundation/log';

import { Command } from 'commander';
import { readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import { mkdirpSync } from 'fs-extra';
import path, { resolve } from 'path';

import { generateNoirContractInterface } from '../index.js';
import { isContractAbi } from '../utils.js';

/**
 * Registers a 'interface' command on the given commander program that generates a Noir interface out of an ABI.
 * @param program - Commander program.
 * @param log - Optional logging function.
 * @returns The program with the command registered.
 */
export function generateNoirInterface(program: Command, name = 'interface', log: LogFn = () => {}): Command {
  return program
    .command(name)
    .argument('<project-path>', 'Path to the noir project')
    .option('--artifacts <path>', 'Folder containing the compiled artifacts, relative to the project path', 'target')
    .option(
      '-o, --outdir <path>',
      'Output folder for the generated noir interfaces, relative to the project path',
      'interfaces',
    )
    .description('Generates Noir interfaces from the artifacts in the given project')

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
        if (typeof projectPath !== 'string') throw new Error(`Missing project path argument`);
        const currentDir = process.cwd();

        const artifactsDir = resolve(projectPath, artifacts);
        for (const artifactsDirItem of readdirSync(artifactsDir)) {
          const artifactPath = resolve(artifactsDir, artifactsDirItem);
          if (statSync(artifactPath).isFile() && artifactPath.endsWith('.json')) {
            const contract = JSON.parse(readFileSync(artifactPath).toString());
            if (!isContractAbi(contract)) continue;
            const interfacePath = resolve(projectPath, outdir, `${contract.name}_interface.nr`);
            log(`Writing ${contract.name} Noir external interface to ${path.relative(currentDir, interfacePath)}`);
            try {
              const noirInterface = generateNoirContractInterface(contract);
              mkdirpSync(path.dirname(interfacePath));
              writeFileSync(interfacePath, noirInterface);
            } catch (err) {
              log(`Error generating interface for ${artifactPath}: ${err}`);
            }
          }
        }
      },
    );
}
