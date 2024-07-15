import { type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';
import { dirname } from 'path';

export function injectCommands(program: Command, log: LogFn) {
  program
    .command('codegen')
    .argument('<noir-abi-path>', 'Path to the Noir ABI or project dir.')
    .option('-o, --outdir <path>', 'Output folder for the generated code.')
    .option('--force', 'Force code generation even when the contract has not changed.')
    .description('Validates and generates an Aztec Contract ABI from Noir ABI.')
    .action(async (noirAbiPath: string, { outdir, force }) => {
      const { codegen } = await import('./codegen.js');
      codegen(outdir || dirname(noirAbiPath), noirAbiPath, { force });
    });

  program
    .command('update')
    .description('Updates Nodejs and Noir dependencies')
    .argument('[projectPath]', 'Path to the project directory', process.cwd())
    .option('--contract [paths...]', 'Paths to contracts to update dependencies', [])
    .option('--aztec-version <semver>', 'The version to update Aztec packages to. Defaults to latest', 'latest')
    .action(async (projectPath: string, options) => {
      const { updateProject } = await import('./update.js');
      const { contract, aztecVersion } = options;
      await updateProject(projectPath, contract, aztecVersion, log);
    });
  return program;
}
