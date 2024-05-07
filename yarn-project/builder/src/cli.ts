#!/usr/bin/env node
import { createConsoleLogger } from '@aztec/foundation/log';

import { Command } from 'commander';
import { dirname } from 'path';

const program = new Command();
const log = createConsoleLogger('aztec:builder');

const main = async () => {
  program.name('aztec-builder');
  program
    .command('codegen')
    .argument('<noir-abi-path>', 'Path to the Noir ABI or project dir.')
    .option('-o, --outdir <path>', 'Output folder for the generated code.')
    .option('--force', 'Force code generation even when the contract has not changed.')
    .description('Validates and generates an Aztec Contract ABI from Noir ABI.')
    .action(async (noirAbiPath: string, { outdir, force }) => {
      const { generateCode } = await import('./cli/codegen.js');
      generateCode(outdir || dirname(noirAbiPath), noirAbiPath, { force });
    });

  program
    .command('update')
    .description('Updates Nodejs and Noir dependencies')
    .argument('[projectPath]', 'Path to the project directory', process.cwd())
    .option('--contract [paths...]', 'Paths to contracts to update dependencies', [])
    .option('--aztec-version <semver>', 'The version to update Aztec packages to. Defaults to latest', 'latest')
    .action(async (projectPath: string, options) => {
      const { update } = await import('./cli/update/update.js');
      const { contract, aztecVersion } = options;
      await update(projectPath, contract, aztecVersion, log);
    });

  await program.parseAsync(process.argv);
  // I force exit here because spawnSync in npm.ts just blocks the process from exiting. Spent a bit of time debugging
  // it without success and I think it doesn't make sense to invest more time in this.
  process.exit(0);
};

main().catch(err => {
  log(`Error running command`);
  log(err);
  process.exit(1);
});
