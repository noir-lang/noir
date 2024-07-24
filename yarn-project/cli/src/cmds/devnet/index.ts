import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

import { ETHEREUM_HOST, l1ChainIdOption, pxeOption } from '../../utils/commands.js';

export function injectCommands(program: Command, log: LogFn, debugLogger: DebugLogger) {
  program
    .command('bootstrap-devnet')
    .description('Bootstrap the devnet')
    .addOption(pxeOption)
    .addOption(l1ChainIdOption)
    .requiredOption(
      '--l1-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .option('--l1-private-key <string>', 'The private key to use for deployment', process.env.PRIVATE_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use in deployment',
      'test test test test test test test test test test test junk',
    )
    .option('--json', 'Output the result as JSON')
    .action(async options => {
      const { bootstrapDevnet } = await import('./bootstrap_devnet.js');
      await bootstrapDevnet(
        options[pxeOption.attributeName()],
        options.l1RpcUrl,
        options[l1ChainIdOption.attributeName()],
        options.l1PrivateKey,
        options.mnemonic,
        options.json,
        log,
        debugLogger,
      );
    });

  return program;
}
