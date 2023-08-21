import { LogFn } from '@aztec/foundation/log';

import { Command } from 'commander';
import { writeFileSync } from 'fs';
import { mkdirpSync } from 'fs-extra';
import path, { resolve } from 'path';

import { compileUsingNargo, generateNoirContractInterface, generateTypescriptContractInterface } from '../index.js';

/**
 * Registers a 'contract' command on the given commander program that compiles a Noir contract project.
 * @param program - Commander program.
 * @param log - Optional logging function.
 * @returns The program with the command registered.
 */
export function compileContract(program: Command, name = 'contract', log: LogFn = () => {}): Command {
  return program
    .command(name)
    .argument('<project-path>', 'Path to the noir project to compile')
    .option('-o, --outdir <path>', 'Output folder for the binary artifacts, relative to the project path', 'target')
    .option('-ts, --typescript <path>', 'Optional output folder for generating typescript wrappers', undefined)
    .option('-i, --interface <path>', 'Optional output folder for generating noir contract interface', undefined)
    .description('Compiles the contracts in the target project')

    .action(
      async (
        projectPath: string,
        /* eslint-disable jsdoc/require-jsdoc */
        options: {
          outdir: string;
          typescript: string | undefined;
          interface: string | undefined;
        },
        /* eslint-enable jsdoc/require-jsdoc */
      ) => {
        const { outdir, typescript, interface: noirInterface } = options;
        if (typeof projectPath !== 'string') throw new Error(`Missing project path argument`);
        const currentDir = process.cwd();

        const compile = compileUsingNargo;
        log(`Compiling contracts...`);
        const result = await compile(projectPath);

        for (const contract of result) {
          const artifactPath = resolve(projectPath, outdir, `${contract.name}.json`);
          log(`Writing ${contract.name} artifact to ${path.relative(currentDir, artifactPath)}`);
          writeFileSync(artifactPath, JSON.stringify(contract, null, 2));

          if (noirInterface) {
            const noirInterfacePath = resolve(projectPath, noirInterface, `${contract.name}_interface.nr`);
            log(`Writing ${contract.name} Noir external interface to ${path.relative(currentDir, noirInterfacePath)}`);
            const noirWrapper = generateNoirContractInterface(contract);
            mkdirpSync(path.dirname(noirInterfacePath));
            writeFileSync(noirInterfacePath, noirWrapper);
          }

          if (typescript) {
            const tsPath = resolve(projectPath, typescript, `${contract.name}.ts`);
            log(`Writing ${contract.name} typescript interface to ${path.relative(currentDir, tsPath)}`);
            const relativeArtifactPath = path.relative(path.dirname(tsPath), artifactPath);
            const tsWrapper = generateTypescriptContractInterface(contract, relativeArtifactPath);
            mkdirpSync(path.dirname(tsPath));
            writeFileSync(tsPath, tsWrapper);
          }
        }
      },
    );
}
