import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

import { ETHEREUM_HOST, l1ChainIdOption, parseOptionalInteger, pxeOption } from '../../utils/commands.js';

export function injectCommands(program: Command, log: LogFn, debugLogger: DebugLogger) {
  program
    .command('deploy-protocol-contracts')
    .description('Bootstrap the blockchain by deploying all the protocol contracts')
    .addOption(pxeOption)
    .addOption(l1ChainIdOption)
    .option('--json', 'Output the contract addresses in JSON format')
    .action(async options => {
      const { deployProtocolContracts } = await import('./deploy_protocol_contract.js');
      await deployProtocolContracts(options.rpcUrl, options.l1ChainId, options.json, log);
    });

  program
    .command('sequencers')
    .argument('<command>', 'Command to run: list, add, remove, who-next')
    .argument('[who]', 'Who to add/remove')
    .description('Manages or queries registered sequencers on the L1 rollup contract.')
    .requiredOption(
      '--l1-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic for the sender of the tx',
      'test test test test test test test test test test test junk',
    )
    .option('--block-number <number>', 'Block number to query next sequencer for', parseOptionalInteger)
    .addOption(pxeOption)
    .addOption(l1ChainIdOption)
    .action(async (command, who, options) => {
      const { sequencers } = await import('./sequencers.js');
      await sequencers({
        command: command,
        who,
        mnemonic: options.mnemonic,
        rpcUrl: options.rpcUrl,
        l1RpcUrl: options.l1RpcUrl,
        chainId: options.l1ChainId,
        blockNumber: options.blockNumber,
        log,
        debugLogger,
      });
    });

  return program;
}
