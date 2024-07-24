import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

import { ETHEREUM_HOST, l1ChainIdOption, parseEthereumAddress, pxeOption } from '../../utils/commands.js';

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

  program
    .command('drip-faucet')
    .description('Drip the faucet')
    .requiredOption('-u, --faucet-url <string>', 'Url of the faucet', 'http://localhost:8082')
    .requiredOption('-t, --token <string>', 'The asset to drip', 'eth')
    .requiredOption('-a, --address <string>', 'The Ethereum address to drip to', parseEthereumAddress)
    .action(async options => {
      const { dripFaucet } = await import('./faucet.js');
      await dripFaucet(options.faucetUrl, options.token, options.address, log);
    });

  return program;
}
