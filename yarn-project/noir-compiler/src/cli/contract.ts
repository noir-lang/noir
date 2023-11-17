import { LogFn } from '@aztec/foundation/log';

import { Command } from 'commander';
import { mkdirSync, writeFileSync } from 'fs';
import { mkdirpSync } from 'fs-extra';
import path, { resolve } from 'path';

import {
  compileUsingNargo,
  compileUsingNoirWasm,
  generateNoirContractInterface,
  generateTypescriptContractInterface,
} from '../index.js';

/**
 * Registers a 'contract' command on the given commander program that compiles an Aztec.nr contract project.
 * @param program - Commander program.
 * @param log - Optional logging function.
 * @returns The program with the command registered.
 */
export function compileContract(program: Command, name = 'contract', log: LogFn = () => {}): Command {
  return program
    .command(name)
    .argument('<project-path>', 'Path to the Aztec.nr project to compile')
    .option('-o, --outdir <path>', 'Output folder for the binary artifacts, relative to the project path', 'target')
    .option('-ts, --typescript <path>', 'Optional output folder for generating typescript wrappers', undefined)
    .option('-i, --interface <path>', 'Optional output folder for generating an Aztec.nr contract interface', undefined)
    .option('-c --compiler <string>', 'Which compiler to use. Either nargo or wasm. Defaults to nargo', 'wasm')
    .description('Compiles the contracts in the target project')

    .action(
      async (
        projectPath: string,
        /* eslint-disable jsdoc/require-jsdoc */
        options: {
          outdir: string;
          typescript: string | undefined;
          interface: string | undefined;
          compiler: string | undefined;
        },
        /* eslint-enable jsdoc/require-jsdoc */
      ) => {
        const { outdir, typescript, interface: noirInterface, compiler } = options;
        if (typeof projectPath !== 'string') throw new Error(`Missing project path argument`);
        if (compiler !== 'nargo' && compiler !== 'wasm') throw new Error(`Invalid compiler: ${compiler}`);
        const currentDir = process.cwd();

        const compile = compiler === 'wasm' ? compileUsingNoirWasm : compileUsingNargo;
        log(`Compiling contracts...`);
        const result = await compile(projectPath, { log });

        for (const contract of result) {
          const artifactPath = resolve(projectPath, outdir, `${contract.name}.json`);
          log(`Writing ${contract.name} artifact to ${path.relative(currentDir, artifactPath)}`);
          mkdirSync(path.dirname(artifactPath), { recursive: true });
          writeFileSync(artifactPath, JSON.stringify(contract, null, 2));

          if (noirInterface) {
            const noirInterfacePath = resolve(projectPath, noirInterface, `${contract.name}_interface.nr`);
            log(
              `Writing ${contract.name} Aztec.nr external interface to ${path.relative(currentDir, noirInterfacePath)}`,
            );
            const noirWrapper = generateNoirContractInterface(contract);
            mkdirpSync(path.dirname(noirInterfacePath));
            writeFileSync(noirInterfacePath, noirWrapper);
          }

          if (typescript) {
            const tsPath = resolve(projectPath, typescript, `${contract.name}.ts`);
            log(`Writing ${contract.name} typescript interface to ${path.relative(currentDir, tsPath)}`);
            let relativeArtifactPath = path.relative(path.dirname(tsPath), artifactPath);
            if (relativeArtifactPath === `${contract.name}.json`) {
              // relative path edge case, prepending ./ for local import - the above logic just does
              // `${contract.name}.json`, which is not a valid import for a file in the same directory
              relativeArtifactPath = `./${contract.name}.json`;
            }
            const tsWrapper = generateTypescriptContractInterface(contract, relativeArtifactPath);
            mkdirpSync(path.dirname(tsPath));
            writeFileSync(tsPath, tsWrapper);
          }
        }
      },
    );
}
