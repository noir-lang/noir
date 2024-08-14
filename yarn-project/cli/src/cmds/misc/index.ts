import { type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

import { prettyPrintJSON } from '../../utils/commands.js';

export * from './deploy_contracts.js';

export function injectCommands(program: Command, log: LogFn) {
  program
    .command('generate-keys')
    .summary('Generates encryption and signing private keys.')
    .description('Generates and encryption and signing private key pair.')
    .option('--json', 'Output the keys in JSON format')
    .action(async ({ json }) => {
      const { generateSecretKey } = await import('./generate_secret_key.js');
      const { secretKey } = generateSecretKey();
      if (json) {
        log(prettyPrintJSON({ secretKey: secretKey.toString() }));
      } else {
        log(`Secret Key: ${secretKey}`);
      }
    });

  program
    .command('generate-p2p-private-key')
    .summary('Generates a LibP2P peer private key.')
    .description('Generates a private key that can be used for running a node on a LibP2P network.')
    .action(async () => {
      const { generateP2PPrivateKey } = await import('./generate_p2p_private_key.js');
      await generateP2PPrivateKey(log);
    });

  program
    .command('example-contracts')
    .description('Lists the example contracts available to deploy from @aztec/noir-contracts.js')
    .action(async () => {
      const { exampleContracts } = await import('./example_contracts.js');
      await exampleContracts(log);
    });

  program
    .command('compute-selector')
    .description('Given a function signature, it computes a selector')
    .argument('<functionSignature>', 'Function signature to compute selector for e.g. foo(Field)')
    .action(async (functionSignature: string) => {
      const { computeSelector } = await import('./compute_selector.js');
      computeSelector(functionSignature, log);
    });

  program
    .command('generate-secret-and-hash')
    .description('Generates an arbitrary secret (Fr), and its hash (using aztec-nr defaults)')
    .action(async () => {
      const { generateSecretAndHash } = await import('./generate_secret_and_hash.js');
      generateSecretAndHash(log);
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
