import { createCompatibleClient } from '@aztec/aztec.js';
import { PublicKeys } from '@aztec/circuits.js';
import {
  addOptions,
  createPrivateKeyOption,
  logJson,
  parseAztecAddress,
  parseFieldFromHexString,
  parsePublicKey,
  pxeOption,
} from '@aztec/cli/utils';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command, Option } from 'commander';

import { type WalletDB } from '../storage/wallet_db.js';
import { AccountType, createOrRetrieveWallet } from '../utils/accounts.js';
import { FeeOpts } from '../utils/fees.js';

function createAliasOption(allowAddress: boolean, description: string, hide: boolean) {
  return new Option(`-a, --alias${allowAddress ? '-or-address' : ''} <string>`, description).hideHelp(hide);
}

export function injectCommands(program: Command, log: LogFn, debugLogger: DebugLogger, db?: WalletDB) {
  const createAccountCommand = program
    .command('create-account')
    .description(
      'Creates an aztec account that can be used for sending transactions. Registers the account on the PXE and deploys an account contract. Uses a Schnorr single-key account which uses the same key for encryption and authentication (not secure for production usage).',
    )
    .summary('Creates an aztec account that can be used for sending transactions.')
    .option(
      '--skip-initialization',
      'Skip initializing the account contract. Useful for publicly deploying an existing account.',
    )
    .option('--public-deploy', 'Publicly deploys the account and registers the class if needed.')
    .addOption(pxeOption)
    .addOption(createPrivateKeyOption('Private key for account. Uses random by default.', false))
    .addOption(createAliasOption(false, 'Alias for the account. Used for easy reference in the PXE.', !db));

  addOptions(createAccountCommand, FeeOpts.getOptions())
    .option(
      '--register-only',
      'Just register the account on the PXE. Do not deploy or initialize the account contract.',
    )
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction')
    .action(async (_options, command) => {
      const { createAccount } = await import('../cmds/create_account.js');
      const options = command.optsWithGlobals();
      const { privateKey, wait, registerOnly, skipInitialization, publicDeploy, rpcUrl, alias } = options;
      const accountCreationResult = await createAccount(
        rpcUrl,
        privateKey,
        alias,
        registerOnly,
        skipInitialization,
        publicDeploy,
        wait,
        FeeOpts.fromCli(options, log),
        debugLogger,
        log,
      );
      if (db) {
        const { alias, address, privateKey, salt } = accountCreationResult;
        await db.storeAccount(address, { alias, privateKey, salt });
        log(`Account stored in database with alias ${alias}`);
      }
    });

  const deployCommand = program
    .command('deploy')
    .description('Deploys a compiled Aztec.nr contract to Aztec.')
    .argument(
      '<artifact>',
      "A compiled Aztec.nr contract's artifact in JSON format or name of a contract artifact exported by @aztec/noir-contracts.js",
    )
    .option('--init <string>', 'The contract initializer function to call', 'constructor')
    .option('--no-init', 'Leave the contract uninitialized')
    .option('--args <constructorArgs...>', 'Contract constructor arguments', [])
    .option(
      '-k, --public-key <string>',
      'Optional encryption public key for this address. Set this value only if this contract is expected to receive private notes, which will be encrypted using this public key.',
      parsePublicKey,
    )
    .option(
      '-s, --salt <hex string>',
      'Optional deployment salt as a hex string for generating the deployment address.',
      parseFieldFromHexString,
    )
    .option('--universal', 'Do not mix the sender address into the deployment.')
    .addOption(pxeOption)
    .addOption(createPrivateKeyOption("The sender's private key", !db).conflicts('alias'))
    .addOption(createAliasOption(true, 'Alias or address of the account to deploy from', !db))
    .option('--json', 'Emit output as json')
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction')
    .option('--no-class-registration', "Don't register this contract class")
    .option('--no-public-deployment', "Don't emit this contract's public bytecode");

  addOptions(deployCommand, FeeOpts.getOptions()).action(async (artifactPath, _options, command) => {
    const { deploy } = await import('../cmds/deploy.js');
    const options = command.optsWithGlobals();
    const {
      json,
      publicKey,
      args: rawArgs,
      salt,
      wait,
      privateKey,
      classRegistration,
      init,
      publicDeployment,
      universal,
      rpcUrl,
      aliasOrAddress,
    } = options;
    const client = await createCompatibleClient(rpcUrl, debugLogger);
    const wallet = await createOrRetrieveWallet(AccountType.SCHNORR, client, privateKey, aliasOrAddress, db);

    await deploy(
      client,
      wallet,
      artifactPath,
      json,
      publicKey ? PublicKeys.fromString(publicKey) : undefined,
      rawArgs,
      salt,
      typeof init === 'string' ? init : undefined,
      !publicDeployment,
      !classRegistration,
      typeof init === 'string' ? false : init,
      universal,
      wait,
      FeeOpts.fromCli(options, log),
      debugLogger,
      log,
      logJson(log),
    );
  });

  const sendCommand = program
    .command('send')
    .description('Calls a function on an Aztec contract.')
    .argument('<functionName>', 'Name of function to execute')
    .addOption(pxeOption)
    .option('--args [functionArgs...]', 'Function arguments', [])
    .requiredOption('-c, --contract-artifact <fileLocation>', "A compiled Aztec.nr contract's ABI in JSON format")
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.', parseAztecAddress)
    .addOption(createPrivateKeyOption("The sender's private key.", !db).conflicts('alias'))
    .addOption(createAliasOption(true, 'Alias or address of the account to deploy from', !db))
    .option('--no-wait', 'Print transaction hash without waiting for it to be mined');

  addOptions(sendCommand, FeeOpts.getOptions()).action(async (functionName, _options, command) => {
    const { send } = await import('../cmds/send.js');
    const options = command.optsWithGlobals();
    const { args, contractArtifact, contractAddress, privateKey, aliasOrAddress, noWait, rpcUrl } = options;
    const client = await createCompatibleClient(rpcUrl, debugLogger);
    const wallet = await createOrRetrieveWallet(AccountType.SCHNORR, client, privateKey, aliasOrAddress, db);
    await send(
      wallet,
      functionName,
      args,
      contractArtifact,
      contractAddress,
      !noWait,
      FeeOpts.fromCli(options, log),
      log,
    );
  });

  return program;
}
