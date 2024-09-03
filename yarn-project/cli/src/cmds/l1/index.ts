import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command } from 'commander';

import {
  ETHEREUM_HOST,
  PRIVATE_KEY,
  l1ChainIdOption,
  parseAztecAddress,
  parseBigint,
  parseEthereumAddress,
  pxeOption,
} from '../../utils/commands.js';

export function injectCommands(program: Command, log: LogFn, debugLogger: DebugLogger) {
  const { BB_BINARY_PATH, BB_WORKING_DIRECTORY } = process.env;

  program
    .command('deploy-l1-contracts')
    .description('Deploys all necessary Ethereum contracts for Aztec.')
    .requiredOption(
      '-u, --rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .option('-pk, --private-key <string>', 'The private key to use for deployment', PRIVATE_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use in deployment',
      'test test test test test test test test test test test junk',
    )
    .addOption(l1ChainIdOption)
    .option('--salt <number>', 'The optional salt to use in deployment', arg => parseInt(arg))
    .option('--json', 'Output the contract addresses in JSON format')
    .action(async options => {
      const { deployL1Contracts } = await import('./deploy_l1_contracts.js');
      await deployL1Contracts(
        options.rpcUrl,
        options.l1ChainId,
        options.privateKey,
        options.mnemonic,
        options.salt,
        options.json,
        log,
        debugLogger,
      );
    });

  program
    .command('generate-l1-account')
    .description('Generates a new private key for an account on L1.')
    .option('--json', 'Output the private key in JSON format')
    .action(async () => {
      const { generateL1Account } = await import('./update_l1_validators.js');
      const account = generateL1Account();
      log(JSON.stringify(account, null, 2));
    });

  program
    .command('add-l1-validator')
    .description('Adds a validator to the L1 rollup contract.')
    .requiredOption(
      '-u, --rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .option('-pk, --private-key <string>', 'The private key to use for deployment', PRIVATE_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use in deployment',
      'test test test test test test test test test test test junk',
    )
    .addOption(l1ChainIdOption)
    .option('--validator <addresse>', 'ethereum address of the validator', parseEthereumAddress)
    .option('--rollup <address>', 'ethereum address of the rollup contract', parseEthereumAddress)
    .action(async options => {
      const { addL1Validator } = await import('./update_l1_validators.js');
      await addL1Validator({
        rpcUrl: options.rpcUrl,
        chainId: options.l1ChainId,
        privateKey: options.privateKey,
        mnemonic: options.mnemonic,
        validatorAddress: options.validator,
        rollupAddress: options.rollup,
        log,
        debugLogger,
      });
    });

  program
    .command('fast-forward-epochs')
    .description('Fast forwards the epoch of the L1 rollup contract.')
    .requiredOption(
      '-u, --rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .addOption(l1ChainIdOption)
    .option('--rollup <address>', 'ethereum address of the rollup contract', parseEthereumAddress)
    .option('--count <number>', 'The number of epochs to fast forward', arg => BigInt(parseInt(arg)), 1n)
    .action(async options => {
      const { fastForwardEpochs } = await import('./update_l1_validators.js');
      await fastForwardEpochs({
        rpcUrl: options.rpcUrl,
        chainId: options.l1ChainId,
        rollupAddress: options.rollup,
        numEpochs: options.count,
        log,
        debugLogger,
      });
    });

  program
    .command('debug-rollup')
    .description('Debugs the rollup contract.')
    .requiredOption(
      '-u, --rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .addOption(l1ChainIdOption)
    .option('--rollup <address>', 'ethereum address of the rollup contract', parseEthereumAddress)
    .action(async options => {
      const { debugRollup } = await import('./update_l1_validators.js');
      await debugRollup({
        rpcUrl: options.rpcUrl,
        chainId: options.l1ChainId,
        privateKey: options.privateKey,
        mnemonic: options.mnemonic,
        rollupAddress: options.rollup,
        log,
        debugLogger,
      });
    });

  program
    .command('deploy-l1-verifier')
    .description('Deploys the rollup verifier contract')
    .requiredOption(
      '--l1-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .requiredOption('--l1-chain-id <string>', 'The chain id of the L1 network', '31337')
    .addOption(pxeOption)
    .option('--l1-private-key <string>', 'The L1 private key to use for deployment', PRIVATE_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use in deployment',
      'test test test test test test test test test test test junk',
    )
    .requiredOption('--verifier <verifier>', 'Either mock or real', 'real')
    .option('--bb <path>', 'Path to bb binary', BB_BINARY_PATH)
    .option('--bb-working-dir <path>', 'Path to bb working directory', BB_WORKING_DIRECTORY)
    .action(async options => {
      const { deployMockVerifier, deployUltraHonkVerifier } = await import('./deploy_l1_verifier.js');
      if (options.verifier === 'mock') {
        await deployMockVerifier(
          options.l1RpcUrl,
          options.l1ChainId,
          options.l1PrivateKey,
          options.mnemonic,
          options.rpcUrl,
          log,
          debugLogger,
        );
      } else {
        await deployUltraHonkVerifier(
          options.l1RpcUrl,
          options.l1ChainId,
          options.l1PrivateKey,
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
    .command('bridge-erc20')
    .description('Bridges ERC20 tokens to L2.')
    .argument('<amount>', 'The amount of Fee Juice to mint and bridge.', parseBigint)
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
    .option('--mint', 'Mint the tokens on L1', false)
    .option('--private', 'If the bridge should use the private flow', false)
    .addOption(l1ChainIdOption)
    .requiredOption('-t, --token <string>', 'The address of the token to bridge', parseEthereumAddress)
    .requiredOption('-p, --portal <string>', 'The address of the portal contract', parseEthereumAddress)
    .option('--l1-private-key <string>', 'The private key to use for deployment', PRIVATE_KEY)
    .option('--json', 'Output the claim in JSON format')
    .action(async (amount, recipient, options) => {
      const { bridgeERC20 } = await import('./bridge_erc20.js');
      await bridgeERC20(
        amount,
        recipient,
        options.l1RpcUrl,
        options.l1ChainId,
        options.l1PrivateKey,
        options.mnemonic,
        options.token,
        options.portal,
        options.private,
        options.mint,
        options.json,
        log,
        debugLogger,
      );
    });

  program
    .command('create-l1-account')
    .option('--json', 'Output the account in JSON format')
    .action(async options => {
      const { createL1Account } = await import('./create_l1_account.js');
      createL1Account(options.json, log);
    });

  program
    .command('get-l1-balance')
    .description('Gets the balance of an ERC token in L1 for the given Ethereum address.')
    .argument('<who>', 'Ethereum address to check.', parseEthereumAddress)
    .requiredOption(
      '--l1-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .option('-t, --token <string>', 'The address of the token to check the balance of', parseEthereumAddress)
    .addOption(l1ChainIdOption)
    .option('--json', 'Output the balance in JSON format')
    .action(async (who, options) => {
      const { getL1Balance } = await import('./get_l1_balance.js');
      await getL1Balance(who, options.token, options.l1RpcUrl, options.l1ChainId, options.json, log);
    });

  program
    .command('set-proven-until', { hidden: true })
    .description(
      'Instructs the L1 rollup contract to assume all blocks until the given number are automatically proven.',
    )
    .argument('[blockNumber]', 'The target block number, defaults to the latest pending block number.', parseBigint)
    .requiredOption(
      '--l1-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .addOption(pxeOption)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use for deriving the Ethereum address that will mint and bridge',
      'test test test test test test test test test test test junk',
    )
    .addOption(l1ChainIdOption)
    .option('--l1-private-key <string>', 'The private key to use for deployment', PRIVATE_KEY)
    .action(async (blockNumber, options) => {
      const { assumeProvenUntil } = await import('./assume_proven_until.js');
      await assumeProvenUntil(
        blockNumber,
        options.l1RpcUrl,
        options.rpcUrl,
        options.l1ChainId,
        options.l1PrivateKey,
        options.mnemonic,
        log,
      );
    });

  program
    .command('prover-stats', { hidden: true })
    .requiredOption(
      '--l1-rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .addOption(l1ChainIdOption)
    .option('--start-block <number>', 'The block number to start from', parseBigint, 1n)
    .option('--batch-size <number>', 'The number of blocks to query in each batch', parseBigint, 100n)
    .option('--l1-rollup-address <string>', 'Address of the rollup contract (required if node URL is not set)')
    .option(
      '--node-url <string>',
      'JSON RPC URL of an Aztec node to retrieve the rollup contract address (required if L1 rollup address is not set)',
    )
    .action(async options => {
      const { proverStats } = await import('./prover_stats.js');
      const { l1RpcUrl, chainId, l1RollupAddress, startBlock, batchSize, nodeUrl } = options;
      await proverStats({ l1RpcUrl, chainId, l1RollupAddress, startBlock, batchSize, nodeUrl, log });
    });

  return program;
}
