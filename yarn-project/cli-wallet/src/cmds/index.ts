import { getIdentities } from '@aztec/accounts/utils';
import { createCompatibleClient } from '@aztec/aztec.js';
import { Fr, PublicKeys } from '@aztec/circuits.js';
import {
  ETHEREUM_HOST,
  PRIVATE_KEY,
  addOptions,
  createSecretKeyOption,
  l1ChainIdOption,
  logJson,
  parseBigint,
  parseFieldFromHexString,
  parsePublicKey,
  pxeOption,
} from '@aztec/cli/utils';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';

import { type Command, Option } from 'commander';
import inquirer from 'inquirer';

import { type WalletDB } from '../storage/wallet_db.js';
import { type AccountType, createOrRetrieveAccount } from '../utils/accounts.js';
import { FeeOpts } from '../utils/options/fees.js';
import {
  ARTIFACT_DESCRIPTION,
  aliasedAddressParser,
  aliasedSecretKeyParser,
  aliasedTxHashParser,
  artifactPathFromPromiseOrAlias,
  artifactPathParser,
  createAccountOption,
  createAliasOption,
  createArgsOption,
  createArtifactOption,
  createContractAddressOption,
  createTypeOption,
} from '../utils/options/index.js';

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
    .option(
      '-p, --public-key <string>',
      'Public key that identifies a private signing key stored outside of the wallet. Used for ECDSA SSH accounts over the secp256r1 curve.',
    )
    .addOption(pxeOption)
    .addOption(
      createSecretKeyOption('Secret key for account. Uses random by default.', false, sk =>
        aliasedSecretKeyParser(sk, db),
      ).conflicts('public-key'),
    )
    .addOption(createAliasOption('Alias for the account. Used for easy reference in subsequent commands.', !db))
    .addOption(createTypeOption(true))
    .option(
      '--register-only',
      'Just register the account on the PXE. Do not deploy or initialize the account contract.',
    )
    .option('--json', 'Emit output as json')
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction');

  addOptions(createAccountCommand, FeeOpts.getOptions()).action(async (_options, command) => {
    const { createAccount } = await import('../cmds/create_account.js');
    const options = command.optsWithGlobals();
    const { type, secretKey, wait, registerOnly, skipInitialization, publicDeploy, rpcUrl, alias, json } = options;
    let { publicKey } = options;
    if ((type as AccountType) === 'ecdsasecp256r1ssh' && !publicKey) {
      const identities = await getIdentities();
      const answers = await inquirer.prompt([
        {
          type: 'list',
          name: 'identity',
          message: 'What public key to use?',
          choices: identities.map(key => `${key.type} ${key.publicKey} ${key.comment}`),
          // Any required until https://github.com/SBoudrias/Inquirer.js/issues/1495 is fixed
        } as any,
      ]);
      publicKey = answers.identity.split(' ')[1];
    }
    const client = await createCompatibleClient(rpcUrl, debugLogger);
    const accountCreationResult = await createAccount(
      client,
      type,
      secretKey,
      publicKey,
      alias,
      registerOnly,
      skipInitialization,
      publicDeploy,
      wait,
      FeeOpts.fromCli(options, log, db),
      json,
      debugLogger,
      log,
    );
    if (db) {
      const { address, alias, secretKey, salt } = accountCreationResult;
      await db.storeAccount(address, { type, secretKey, salt, alias, publicKey }, log);
    }
  });

  const deployAccountCommand = program
    .command('deploy-account')
    .description('Deploys an already registered aztec account that can be used for sending transactions.')
    .addOption(createAccountOption('Alias or address of the account to deploy', !db, db))
    .addOption(pxeOption)
    .option('--json', 'Emit output as json')
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction');

  addOptions(deployAccountCommand, FeeOpts.getOptions()).action(async (_options, command) => {
    const { deployAccount } = await import('../cmds/deploy_account.js');
    const options = command.optsWithGlobals();
    const { rpcUrl, wait, from: parsedFromAddress, json } = options;

    const client = await createCompatibleClient(rpcUrl, debugLogger);
    const account = await createOrRetrieveAccount(client, parsedFromAddress, db);

    await deployAccount(account, wait, FeeOpts.fromCli(options, log, db), json, debugLogger, log);
  });

  const deployCommand = program
    .command('deploy')
    .description('Deploys a compiled Aztec.nr contract to Aztec.')
    .argument('[artifact]', ARTIFACT_DESCRIPTION, artifactPathParser)
    .option('--init <string>', 'The contract initializer function to call', 'constructor')
    .option('--no-init', 'Leave the contract uninitialized')
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
    .addOption(createArgsOption(true, db))
    .addOption(
      createSecretKeyOption("The sender's secret key", !db, sk => aliasedSecretKeyParser(sk, db)).conflicts('account'),
    )
    .addOption(createAccountOption('Alias or address of the account to deploy from', !db, db))
    .addOption(createAliasOption('Alias for the contract. Used for easy reference subsequent commands.', !db))
    .addOption(createTypeOption(false))
    .option('--json', 'Emit output as json')
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction')
    .option('--no-class-registration', "Don't register this contract class")
    .option('--no-public-deployment', "Don't emit this contract's public bytecode");

  addOptions(deployCommand, FeeOpts.getOptions()).action(async (artifactPathPromise, _options, command) => {
    const { deploy } = await import('../cmds/deploy.js');
    const options = command.optsWithGlobals();
    const {
      json,
      publicKey,
      args,
      salt,
      wait,
      secretKey,
      classRegistration,
      init,
      publicDeployment,
      universal,
      rpcUrl,
      from: parsedFromAddress,
      alias,
      type,
    } = options;
    const client = await createCompatibleClient(rpcUrl, debugLogger);
    const account = await createOrRetrieveAccount(client, parsedFromAddress, db, type, secretKey, Fr.ZERO, publicKey);
    const wallet = await account.getWallet();
    const artifactPath = await artifactPathPromise;

    debugLogger.info(`Using wallet with address ${wallet.getCompleteAddress().address.toString()}`);

    const address = await deploy(
      client,
      wallet,
      artifactPath,
      json,
      publicKey ? PublicKeys.fromString(publicKey) : undefined,
      args,
      salt,
      typeof init === 'string' ? init : undefined,
      !publicDeployment,
      !classRegistration,
      typeof init === 'string' ? false : init,
      universal,
      wait,
      FeeOpts.fromCli(options, log, db),
      debugLogger,
      log,
      logJson(log),
    );
    if (db && address) {
      await db.storeContract(address, artifactPath, log, alias);
    }
  });

  const sendCommand = program
    .command('send')
    .description('Calls a function on an Aztec contract.')
    .argument('<functionName>', 'Name of function to execute')
    .addOption(pxeOption)
    .addOption(createArgsOption(false, db))
    .addOption(createArtifactOption(db))
    .addOption(createContractAddressOption(db))
    .addOption(
      createAliasOption('Alias for the transaction hash. Used for easy reference in subsequent commands.', !db),
    )
    .addOption(
      createSecretKeyOption("The sender's secret key", !db, sk => aliasedSecretKeyParser(sk, db)).conflicts('account'),
    )
    .addOption(createAccountOption('Alias or address of the account to send the transaction from', !db, db))
    .addOption(createTypeOption(false))
    .option('--no-wait', 'Print transaction hash without waiting for it to be mined');

  addOptions(sendCommand, FeeOpts.getOptions()).action(async (functionName, _options, command) => {
    const { send } = await import('../cmds/send.js');
    const options = command.optsWithGlobals();
    const {
      args,
      contractArtifact: artifactPathPromise,
      contractAddress,
      from: parsedFromAddress,
      noWait,
      rpcUrl,
      type,
      secretKey,
      publicKey,
      alias,
    } = options;
    const client = await createCompatibleClient(rpcUrl, debugLogger);
    const account = await createOrRetrieveAccount(client, parsedFromAddress, db, type, secretKey, Fr.ZERO, publicKey);
    const wallet = await account.getWallet();
    const artifactPath = await artifactPathFromPromiseOrAlias(artifactPathPromise, contractAddress, db);

    debugLogger.info(`Using wallet with address ${wallet.getCompleteAddress().address.toString()}`);

    const txHash = await send(
      wallet,
      functionName,
      args,
      artifactPath,
      contractAddress,
      !noWait,
      FeeOpts.fromCli(options, log, db),
      log,
    );
    if (db && txHash) {
      await db.storeTxHash(txHash, log, alias);
    }
  });

  program
    .command('simulate')
    .description('Simulates the execution of a function on an Aztec contract.')
    .argument('<functionName>', 'Name of function to simulate')
    .addOption(pxeOption)
    .addOption(createArgsOption(false, db))
    .addOption(createContractAddressOption(db))
    .addOption(createArtifactOption(db))
    .addOption(
      createSecretKeyOption("The sender's secret key", !db, sk => aliasedSecretKeyParser(sk, db)).conflicts('account'),
    )
    .addOption(createAccountOption('Alias or address of the account to simulate from', !db, db))
    .addOption(createTypeOption(false))
    .action(async (functionName, _options, command) => {
      const { simulate } = await import('../cmds/simulate.js');
      const options = command.optsWithGlobals();
      const {
        args,
        contractArtifact: artifactPathPromise,
        contractAddress,
        from: parsedFromAddress,
        rpcUrl,
        type,
        secretKey,
        publicKey,
      } = options;

      const client = await createCompatibleClient(rpcUrl, debugLogger);
      const account = await createOrRetrieveAccount(client, parsedFromAddress, db, type, secretKey, Fr.ZERO, publicKey);
      const wallet = await account.getWallet();
      const artifactPath = await artifactPathFromPromiseOrAlias(artifactPathPromise, contractAddress, db);
      await simulate(wallet, functionName, args, artifactPath, contractAddress, log);
    });

  program
    .command('bridge-fee-juice')
    .description('Mints L1 Fee Juice and pushes them to L2.')
    .argument('<amount>', 'The amount of Fee Juice to mint and bridge.', parseBigint)
    .argument('<recipient>', 'Aztec address of the recipient.', address =>
      aliasedAddressParser('accounts', address, db),
    )
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
    .option('--l1-private-key <string>', 'The private key to the eth account bridging', PRIVATE_KEY)
    .addOption(pxeOption)
    .addOption(l1ChainIdOption)
    .option('--json', 'Output the claim in JSON format')
    .action(async (amount, recipient, options) => {
      const { bridgeL1FeeJuice } = await import('./bridge_fee_juice.js');
      const secret = await bridgeL1FeeJuice(
        amount,
        recipient,
        options.rpcUrl,
        options.l1RpcUrl,
        options.l1ChainId,
        options.l1PrivateKey,
        options.mnemonic,
        options.mint,
        options.json,
        log,
        debugLogger,
      );
      if (db) {
        await db.pushBridgedFeeJuice(recipient, secret, amount, log);
      }
    });

  program
    .command('add-note')
    .description('Adds a note to the database in the PXE.')
    .argument('[name]', 'The Note name')
    .argument(
      '[storageFieldName]',
      'The name of the variable in the storage field that contains the note. WARNING: Maps are not supported',
    )
    .requiredOption('-a, --address <string>', 'The Aztec address of the note owner.', address =>
      aliasedAddressParser('accounts', address, db),
    )
    .requiredOption('-h, --hash <string>', 'The tx hash of the tx containing the note.', txHash =>
      aliasedTxHashParser(txHash, db),
    )
    .addOption(createContractAddressOption(db))
    .addOption(createArtifactOption(db))
    .addOption(
      new Option('-b, --body [noteFields...]', 'The members of a Note')
        .argParser((arg, prev: string[]) => {
          const next = db?.tryRetrieveAlias(arg) || arg;
          prev.push(next);
          return prev;
        })
        .default([]),
    )
    .addOption(pxeOption)
    .action(async (noteName, storageFieldName, _options, command) => {
      const { addNote } = await import('./add_note.js');
      const options = command.optsWithGlobals();
      const { contractArtifact: artifactPathPromise, contractAddress, address, rpcUrl, fields, hash } = options;
      const artifactPath = await artifactPathFromPromiseOrAlias(artifactPathPromise, contractAddress, db);
      const client = await createCompatibleClient(rpcUrl, debugLogger);
      const account = await createOrRetrieveAccount(client, address, db);
      const wallet = await account.getWallet();

      await addNote(wallet, address, contractAddress, noteName, storageFieldName, artifactPath, hash, fields, log);
    });

  return program;
}
