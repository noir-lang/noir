import { type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

export function injectCommands(program: Command, log: LogFn) {
  program
    .command('generate-keys')
    .summary('Generates encryption and signing private keys.')
    .description('Generates and encryption and signing private key pair.')
    .option(
      '-m, --mnemonic',
      'An optional mnemonic string used for the private key generation. If not provided, random private key will be generated.',
    )
    .action(async _options => {
      const { generateKeys } = await import('./generate_private_key.js');
      const { privateEncryptionKey, privateSigningKey } = generateKeys();
      log(`Encryption Private Key: ${privateEncryptionKey}\nSigning Private key: ${privateSigningKey}\n`);
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

  return program;
}
