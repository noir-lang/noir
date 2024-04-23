#!/usr/bin/env node
import { createConsoleLogger } from '@aztec/foundation/log';

import { Command, Option } from 'commander';
import { lookup } from 'dns/promises';
import { dirname } from 'path';

const program = new Command();
const log = createConsoleLogger('aztec:compiler-cli');

/**
 * If we can successfully resolve 'host.docker.internal', then we are running in a container, and we should treat
 * localhost as being host.docker.internal.
 */
const getLocalhost = () =>
  lookup('host.docker.internal')
    .then(() => 'host.docker.internal')
    .catch(() => 'localhost');

const LOCALHOST = await getLocalhost();

const main = async () => {
  const pxeOption = new Option('-u, --rpc-url <string>', 'URL of the PXE')
    .env('PXE_URL')
    .default(`http://${LOCALHOST}:8080`)
    .makeOptionMandatory(true);

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
    .addOption(pxeOption)
    .action(async (projectPath: string, options) => {
      const { update } = await import('./cli/update/update.js');
      const { contract, aztecVersion, rpcUrl } = options;
      await update(projectPath, contract, rpcUrl, aztecVersion, log);
    });

  await program.parseAsync(process.argv);
};

main().catch(err => {
  log(`Error running command`);
  log(err);
  process.exit(1);
});
