import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

import {
  ETHEREUM_HOST,
  PRIVATE_KEY,
  chainIdOption,
  parseAztecAddress,
  parseBigint,
  parseEthereumAddress,
  pxeOption,
} from '../../utils/commands.js';

export function injectCommands(program: Command, log: LogFn, debugLogger: DebugLogger) {
  program
    .command('deploy-l1-contracts')
    .description('Deploys all necessary Ethereum contracts for Aztec.')
    .requiredOption(
      '-u, --rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .requiredOption('-p, --private-key <string>', 'The private key to use for deployment', PRIVATE_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use in deployment',
      'test test test test test test test test test test test junk',
    )
    .addOption(chainIdOption)
    .action(async options => {
      const { deployL1Contracts } = await import('./deploy_l1_contracts.js');
      await deployL1Contracts(options.rpcUrl, options.chainId, options.privateKey, options.mnemonic, log, debugLogger);
    });

  program
    .command('deploy-l1-verifier')
    .description('Deploys the rollup verifier contract')
    .requiredOption(
      '--eth-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .addOption(pxeOption)
    .requiredOption('-p, --private-key <string>', 'The private key to use for deployment', PRIVATE_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use in deployment',
      'test test test test test test test test test test test junk',
    )
    .requiredOption('--verifier <verifier>', 'Either mock or real', 'real')
    .option('--bb <path>', 'Path to bb binary')
    .option('--bb-working-dir <path>', 'Path to bb working directory')
    .action(async options => {
      const { deployMockVerifier, deployUltraVerifier } = await import('./deploy_l1_verifier.js');
      if (options.verifier === 'mock') {
        await deployMockVerifier(
          options.ethRpcUrl,
          options.privateKey,
          options.mnemonic,
          options.rpcUrl,
          log,
          debugLogger,
        );
      } else {
        await deployUltraVerifier(
          options.ethRpcUrl,
          options.privateKey,
          options.mnemonic,
          options.rpcUrl,
          options.bb,
          options.bbWorkingDir,
          log,
          debugLogger,
        );
      }
    });

  program
    .command('bridge-l1-gas')
    .description('Mints L1 gas tokens and pushes them to L2.')
    .argument('<amount>', 'The amount of gas tokens to mint and bridge.', parseBigint)
    .argument('<recipient>', 'Aztec address of the recipient.', parseAztecAddress)
    .requiredOption(
      '--l1-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use for deriving the Ethereum address that will mint and bridge',
      'test test test test test test test test test test test junk',
    )
    .addOption(pxeOption)
    .addOption(chainIdOption)
    .action(async (amount, recipient, options) => {
      const { bridgeL1Gas } = await import('./bridge_l1_gas.js');
      await bridgeL1Gas(
        amount,
        recipient,
        options.rpcUrl,
        options.l1RpcUrl,
        options.chainId,
        options.mnemonic,
        log,
        debugLogger,
      );
    });

  program
    .command('get-l1-balance')
    .description('Gets the balance of gas tokens in L1 for the given Ethereum address.')
    .argument('<who>', 'Ethereum address to check.', parseEthereumAddress)
    .requiredOption(
      '--l1-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .addOption(pxeOption)
    .addOption(chainIdOption)
    .action(async (who, options) => {
      const { getL1Balance } = await import('./get_l1_balance.js');
      await getL1Balance(who, options.rpcUrl, options.l1RpcUrl, options.chainId, log, debugLogger);
    });

  return program;
}
