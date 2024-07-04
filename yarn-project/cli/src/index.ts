import { Fr, PublicKeys } from '@aztec/circuits.js';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';

import { Command as CommanderCommand, Option } from 'commander';
import { lookup } from 'dns/promises';
import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';

import { FeeOpts } from './fees.js';
import {
  parseAztecAddress,
  parseBigint,
  parseEthereumAddress,
  parseField,
  parseFieldFromHexString,
  parseOptionalAztecAddress,
  parseOptionalInteger,
  parseOptionalLogId,
  parseOptionalTxHash,
  parsePartialAddress,
  parsePrivateKey,
  parsePublicKey,
  parseTxHash,
} from './parse_args.js';

/**
 * If we can successfully resolve 'host.docker.internal', then we are running in a container, and we should treat
 * localhost as being host.docker.internal.
 */
const getLocalhost = () =>
  lookup('host.docker.internal')
    .then(() => 'host.docker.internal')
    .catch(() => 'localhost');

const LOCALHOST = await getLocalhost();
const { ETHEREUM_HOST = `http://${LOCALHOST}:8545`, PRIVATE_KEY, API_KEY, CLI_VERSION } = process.env;

class Command extends CommanderCommand {
  addOptions(options: Option[]) {
    options.forEach(option => this.addOption(option));
    return this;
  }

  override createCommand(name?: string): Command {
    return new Command(name);
  }
}

/**
 * Returns commander program that defines the CLI.
 * @param log - Console logger.
 * @param debugLogger - Debug logger.
 * @returns The CLI.
 */
export function getProgram(log: LogFn, debugLogger: DebugLogger): Command {
  const program = new Command();

  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../package.json');
  const cliVersion: string = CLI_VERSION || JSON.parse(readFileSync(packageJsonPath).toString()).version;
  const logJson = (obj: object) => log(JSON.stringify(obj, null, 2));

  program.name('aztec-cli').description('CLI for interacting with Aztec.').version(cliVersion);

  const pxeOption = new Option('-u, --rpc-url <string>', 'URL of the PXE')
    .env('PXE_URL')
    .default(`http://${LOCALHOST}:8080`)
    .makeOptionMandatory(true);

  const createPrivateKeyOption = (description: string, mandatory: boolean) =>
    new Option('-e, --private-key <string>', description)
      .env('PRIVATE_KEY')
      .argParser(parsePrivateKey)
      .makeOptionMandatory(mandatory);

  program
    .command('deploy-l1-contracts')
    .description('Deploys all necessary Ethereum contracts for Aztec.')
    .requiredOption(
      '-u, --rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST,
    )
    .option('-a, --api-key <string>', 'Api key for the ethereum host', API_KEY)
    .requiredOption('-p, --private-key <string>', 'The private key to use for deployment', PRIVATE_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use in deployment',
      'test test test test test test test test test test test junk',
    )
    .action(async options => {
      const { deployL1Contracts } = await import('./cmds/deploy_l1_contracts.js');
      await deployL1Contracts(
        options.rpcUrl,
        options.apiKey ?? '',
        options.privateKey,
        options.mnemonic,
        log,
        debugLogger,
      );
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
      const { deployMockVerifier, deployUltraVerifier } = await import('./cmds/deploy_l1_verifier.js');
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
    .option('-a, --api-key <string>', 'Api key for the ethereum host', API_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic to use for deriving the Ethereum address that will mint and bridge',
      'test test test test test test test test test test test junk',
    )
    .addOption(pxeOption)
    .action(async (amount, recipient, options) => {
      const { bridgeL1Gas } = await import('./cmds/bridge_l1_gas.js');
      await bridgeL1Gas(
        amount,
        recipient,
        options.rpcUrl,
        options.l1RpcUrl,
        options.apiKey ?? '',
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
    .option('-a, --api-key <string>', 'Api key for the ethereum host', API_KEY)
    .addOption(pxeOption)
    .action(async (who, options) => {
      const { getL1Balance } = await import('./cmds/get_l1_balance.js');
      await getL1Balance(who, options.rpcUrl, options.l1RpcUrl, options.apiKey ?? '', log, debugLogger);
    });

  program
    .command('generate-keys')
    .summary('Generates encryption and signing private keys.')
    .description('Generates and encryption and signing private key pair.')
    .option(
      '-m, --mnemonic',
      'An optional mnemonic string used for the private key generation. If not provided, random private key will be generated.',
    )
    .action(async _options => {
      const { generateKeys } = await import('./cmds/generate_private_key.js');
      const { privateEncryptionKey, privateSigningKey } = generateKeys();
      log(`Encryption Private Key: ${privateEncryptionKey}\nSigning Private key: ${privateSigningKey}\n`);
    });

  program
    .command('generate-p2p-private-key')
    .summary('Generates a LibP2P peer private key.')
    .description('Generates a private key that can be used for running a node on a LibP2P network.')
    .action(async () => {
      const { generateP2PPrivateKey } = await import('./cmds/generate_p2p_private_key.js');
      await generateP2PPrivateKey(log);
    });

  program
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
    .addOption(createPrivateKeyOption('Private key for account. Uses random by default.', false))
    .addOption(pxeOption)
    .addOptions(FeeOpts.getOptions())
    .option(
      '--register-only',
      'Just register the account on the PXE. Do not deploy or initialize the account contract.',
    )
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction')
    .action(async args => {
      const { createAccount } = await import('./cmds/create_account.js');
      const { rpcUrl, privateKey, wait, registerOnly, skipInitialization, publicDeploy } = args;
      await createAccount(
        rpcUrl,
        privateKey,
        registerOnly,
        skipInitialization,
        publicDeploy,
        wait,
        FeeOpts.fromCli(args, log),
        debugLogger,
        log,
      );
    });

  program
    .command('bootstrap')
    .description('Bootstrap the blockchain')
    .addOption(pxeOption)
    .action(async options => {
      const { bootstrap } = await import('./cmds/bootstrap.js');
      await bootstrap(options.rpcUrl, log);
    });

  program
    .command('deploy')
    .description('Deploys a compiled Aztec.nr contract to Aztec.')
    .argument(
      '<artifact>',
      "A compiled Aztec.nr contract's artifact in JSON format or name of a contract artifact exported by @aztec/noir-contracts.js",
    )
    .option('--initialize <string>', 'The contract initializer function to call', 'constructor')
    .option('--no-initialize')
    .option('-a, --args <constructorArgs...>', 'Contract constructor arguments', [])
    .addOption(pxeOption)
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
    .addOption(createPrivateKeyOption("The sender's private key.", true))
    .option('--json', 'Emit output as json')
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction')
    .option('--class-registration', 'Register the contract class. Only has to be done once')
    .option('--no-class-registration', 'Skip registering the contract class')
    .option('--public-deployment', 'Deploy the public bytecode of contract')
    .option('--no-public-deployment', "Skip deploying the contract's public bytecode")
    .addOptions(FeeOpts.getOptions())
    .action(async (artifactPath, opts) => {
      const { deploy } = await import('./cmds/deploy.js');
      const {
        json,
        rpcUrl,
        publicKey,
        args: rawArgs,
        salt,
        wait,
        privateKey,
        classRegistration,
        initialize,
        publicDeployment,
        universal,
      } = opts;
      await deploy(
        artifactPath,
        json,
        rpcUrl,
        publicKey ? PublicKeys.fromString(publicKey) : undefined,
        rawArgs,
        salt,
        privateKey,
        typeof initialize === 'string' ? initialize : undefined,
        !publicDeployment,
        !classRegistration,
        typeof initialize === 'string' ? false : initialize,
        universal,
        wait,
        FeeOpts.fromCli(opts, log),
        debugLogger,
        log,
        logJson,
      );
    });

  program
    .command('add-contract')
    .description(
      'Adds an existing contract to the PXE. This is useful if you have deployed a contract outside of the PXE and want to use it with the PXE.',
    )
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts.js",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.', parseAztecAddress)
    .requiredOption('--init-hash <init hash>', 'Initialization hash', parseFieldFromHexString)
    .option('--salt <salt>', 'Optional deployment salt', parseFieldFromHexString)
    .option('-p, --public-key <public key>', 'Optional public key for this contract', parsePublicKey)
    .option('--portal-address <address>', 'Optional address to a portal contract on L1', parseEthereumAddress)
    .option('--deployer-address <address>', 'Optional address of the contract deployer', parseAztecAddress)
    .addOption(pxeOption)
    .action(async options => {
      const { addContract } = await import('./cmds/add_contract.js');
      await addContract(
        options.rpcUrl,
        options.contractArtifact,
        options.contractAddress,
        options.initHash,
        options.salt ?? Fr.ZERO,
        options.publicKey,
        options.deployerAddress,
        debugLogger,
        log,
      );
    });

  program
    .command('get-tx')
    .description('Gets the receipt for the specified transaction hash.')
    .argument('<txHash>', 'A transaction hash to get the receipt for.', parseTxHash)
    .addOption(pxeOption)
    .action(async (txHash, options) => {
      const { getTx } = await import('./cmds/get_tx.js');
      await getTx(options.rpcUrl, txHash, debugLogger, log);
    });

  program
    .command('get-block')
    .description('Gets info for a given block or latest.')
    .argument('[blockNumber]', 'Block height', parseOptionalInteger)
    .option('-f, --follow', 'Keep polling for new blocks')
    .addOption(pxeOption)
    .action(async (blockNumber, options) => {
      const { getBlock } = await import('./cmds/get_block.js');
      await getBlock(options.rpcUrl, blockNumber, options.follow, debugLogger, log);
    });

  program
    .command('get-contract-data')
    .description('Gets information about the Aztec contract deployed at the specified address.')
    .argument('<contractAddress>', 'Aztec address of the contract.', parseAztecAddress)
    .addOption(pxeOption)
    .option('-b, --include-bytecode <boolean>', "Include the contract's public function bytecode, if any.", false)
    .action(async (contractAddress, options) => {
      const { getContractData } = await import('./cmds/get_contract_data.js');
      await getContractData(options.rpcUrl, contractAddress, options.includeBytecode, debugLogger, log);
    });

  program
    .command('get-logs')
    .description('Gets all the unencrypted logs from an intersection of all the filter params.')
    .option('-tx, --tx-hash <txHash>', 'A transaction hash to get the receipt for.', parseOptionalTxHash)
    .option(
      '-fb, --from-block <blockNum>',
      'Initial block number for getting logs (defaults to 1).',
      parseOptionalInteger,
    )
    .option('-tb, --to-block <blockNum>', 'Up to which block to fetch logs (defaults to latest).', parseOptionalInteger)
    .option('-al --after-log <logId>', 'ID of a log after which to fetch the logs.', parseOptionalLogId)
    .option('-ca, --contract-address <address>', 'Contract address to filter logs by.', parseOptionalAztecAddress)
    .addOption(pxeOption)
    .option('--follow', 'If set, will keep polling for new logs until interrupted.')
    .action(async ({ txHash, fromBlock, toBlock, afterLog, contractAddress, rpcUrl, follow }) => {
      const { getLogs } = await import('./cmds/get_logs.js');
      await getLogs(txHash, fromBlock, toBlock, afterLog, contractAddress, rpcUrl, follow, debugLogger, log);
    });

  program
    .command('register-recipient')
    .description('Register a recipient in the PXE.')
    .requiredOption('-a, --address <aztecAddress>', "The account's Aztec address.", parseAztecAddress)
    .requiredOption('-p, --public-key <publicKey>', 'The account public key.', parsePublicKey)
    .requiredOption(
      '-pa, --partial-address <partialAddress>',
      'The partially computed address of the account contract.',
      parsePartialAddress,
    )
    .addOption(pxeOption)
    .action(async ({ address, publicKey, partialAddress, rpcUrl }) => {
      const { registerRecipient } = await import('./cmds/register_recipient.js');
      await registerRecipient(address, publicKey, partialAddress, rpcUrl, debugLogger, log);
    });

  program
    .command('get-accounts')
    .description('Gets all the Aztec accounts stored in the PXE.')
    .addOption(pxeOption)
    .option('--json', 'Emit output as json')
    .action(async (options: any) => {
      const { getAccounts } = await import('./cmds/get_accounts.js');
      await getAccounts(options.rpcUrl, options.json, debugLogger, log, logJson);
    });

  program
    .command('get-account')
    .description('Gets an account given its Aztec address.')
    .argument('<address>', 'The Aztec address to get account for', parseAztecAddress)
    .addOption(pxeOption)
    .action(async (address, options) => {
      const { getAccount } = await import('./cmds/get_account.js');
      await getAccount(address, options.rpcUrl, debugLogger, log);
    });

  program
    .command('get-recipients')
    .description('Gets all the recipients stored in the PXE.')
    .addOption(pxeOption)
    .action(async (options: any) => {
      const { getRecipients } = await import('./cmds/get_recipients.js');
      await getRecipients(options.rpcUrl, debugLogger, log);
    });

  program
    .command('get-recipient')
    .description('Gets a recipient given its Aztec address.')
    .argument('<address>', 'The Aztec address to get recipient for', parseAztecAddress)
    .addOption(pxeOption)
    .action(async (address, options) => {
      const { getRecipient } = await import('./cmds/get_recipient.js');
      await getRecipient(address, options.rpcUrl, debugLogger, log);
    });

  program
    .command('get-balance')
    .description('Gets the token balance for an account. Does NOT format according to decimals.')
    .argument('<address>', 'Aztec address to query balance for.', parseAztecAddress)
    .option('-t, --token-address <address>', 'Token address to query balance for (defaults to gas token).')
    .addOption(pxeOption)
    .action(async (address, options) => {
      const { getBalance } = await import('./cmds/get_balance.js');
      await getBalance(address, options.tokenAddress, options.rpcUrl, debugLogger, log);
    });

  program
    .command('send')
    .description('Calls a function on an Aztec contract.')
    .argument('<functionName>', 'Name of function to execute')
    .option('-a, --args [functionArgs...]', 'Function arguments', [])
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts.js",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.', parseAztecAddress)
    .addOption(createPrivateKeyOption("The sender's private key.", true))
    .addOption(pxeOption)
    .option('--no-wait', 'Print transaction hash without waiting for it to be mined')
    .addOptions(FeeOpts.getOptions())
    .action(async (functionName, options) => {
      const { send } = await import('./cmds/send.js');
      await send(
        functionName,
        options.args,
        options.contractArtifact,
        options.contractAddress,
        options.privateKey,
        options.rpcUrl,
        !options.noWait,
        FeeOpts.fromCli(options, log),
        debugLogger,
        log,
      );
    });

  program
    .command('call')
    .description(
      'Simulates the execution of a view (read-only) function on a deployed contract, without modifying state.',
    )
    .argument('<functionName>', 'Name of function to call')
    .option('-a, --args [functionArgs...]', 'Function arguments', [])
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts.js",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.', parseAztecAddress)
    .option('-f, --from <string>', 'Aztec address of the caller. If empty, will use the first account from RPC.')
    .addOption(pxeOption)
    .action(async (functionName, options) => {
      const { call } = await import('./cmds/call.js');
      await call(
        functionName,
        options.args,
        options.contractArtifact,
        options.contractAddress,
        options.from,
        options.rpcUrl,
        debugLogger,
        log,
      );
    });

  program
    .command('add-note')
    .description('Adds a note to the database in the PXE.')
    .argument('<address>', 'The Aztec address of the note owner.', parseAztecAddress)
    .argument('<contractAddress>', 'Aztec address of the contract.', parseAztecAddress)
    .argument('<storageSlot>', 'The storage slot of the note.', parseField)
    .argument('<noteTypeId>', 'The type ID of the note.', parseField)
    .argument('<txHash>', 'The tx hash of the tx containing the note.', parseTxHash)
    .requiredOption('-n, --note [note...]', 'The members of a Note serialized as hex strings.', [])
    .addOption(pxeOption)
    .action(async (address, contractAddress, storageSlot, noteTypeId, txHash, options) => {
      const { addNote } = await import('./cmds/add_note.js');
      await addNote(
        address,
        contractAddress,
        storageSlot,
        noteTypeId,
        txHash,
        options.note,
        options.rpcUrl,
        debugLogger,
      );
    });

  program
    .command('add-pending-shield')
    .description('Adds a pending shield note to the database in the PXE.')
    .argument('<address>', 'Aztec address of the note owner.', parseAztecAddress)
    .argument('<amount>', 'Amount of the pending shield note.', parseBigint)
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the token contract.', parseAztecAddress)
    .requiredOption('-tx, --tx-hash <txHash>', 'Tx hash in which the note was created.', parseOptionalTxHash)
    .requiredOption('--secret <secret>', 'Secret used for shielding the note.', parseField)
    .addOption(pxeOption)
    .action(async (address, amount, options) => {
      const { addPendingShield } = await import('./cmds/add_pending_shield.js');
      await addPendingShield(
        address,
        options.contractAddress,
        amount,
        options.secret,
        options.txHash,
        options.rpcUrl,
        debugLogger,
        log,
      );
    });

  // Helper for users to decode hex strings into structs if needed.
  program
    .command('parse-parameter-struct')
    .description("Helper for parsing an encoded string into a contract's parameter struct.")
    .argument('<encodedString>', 'The encoded hex string')
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts.js",
    )
    .requiredOption('-p, --parameter <parameterName>', 'The name of the struct parameter to decode into')
    .action(async (encodedString, options) => {
      const { parseParameterStruct } = await import('./cmds/parse_parameter_struct.js');
      await parseParameterStruct(encodedString, options.contractArtifact, options.parameter, log);
    });

  program
    .command('block-number')
    .description('Gets the current Aztec L2 block number.')
    .addOption(pxeOption)
    .action(async (options: any) => {
      const { blockNumber } = await import('./cmds/block_number.js');
      await blockNumber(options.rpcUrl, debugLogger, log);
    });

  program
    .command('example-contracts')
    .description('Lists the example contracts available to deploy from @aztec/noir-contracts.js')
    .action(async () => {
      const { exampleContracts } = await import('./cmds/example_contracts.js');
      await exampleContracts(log);
    });

  program
    .command('get-node-info')
    .description('Gets the information of an aztec node at a URL.')
    .addOption(pxeOption)
    .action(async options => {
      const { getNodeInfo } = await import('./cmds/get_node_info.js');
      await getNodeInfo(options.rpcUrl, debugLogger, log);
    });

  program
    .command('get-pxe-info')
    .description('Gets the information of a PXE at a URL.')
    .addOption(pxeOption)
    .action(async options => {
      const { getPXEInfo } = await import('./cmds/get_pxe_info.js');
      await getPXEInfo(options.rpcUrl, debugLogger, log);
    });

  program
    .command('inspect-contract')
    .description('Shows list of external callable functions for a contract')
    .argument(
      '<contractArtifactFile>',
      `A compiled Noir contract's artifact in JSON format or name of a contract artifact exported by @aztec/noir-contracts.js`,
    )
    .action(async (contractArtifactFile: string) => {
      const { inspectContract } = await import('./cmds/inspect_contract.js');
      await inspectContract(contractArtifactFile, debugLogger, log);
    });

  program
    .command('compute-selector')
    .description('Given a function signature, it computes a selector')
    .argument('<functionSignature>', 'Function signature to compute selector for e.g. foo(Field)')
    .action(async (functionSignature: string) => {
      const { computeSelector } = await import('./cmds/compute_selector.js');
      computeSelector(functionSignature, log);
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
    .option('-a, --api-key <string>', 'Api key for the ethereum host', API_KEY)
    .option(
      '-m, --mnemonic <string>',
      'The mnemonic for the sender of the tx',
      'test test test test test test test test test test test junk',
    )
    .option('--block-number <number>', 'Block number to query next sequencer for', parseOptionalInteger)
    .addOption(pxeOption)
    .action(async (command, who, options) => {
      const { sequencers } = await import('./cmds/sequencers.js');
      await sequencers({
        command: command,
        who,
        mnemonic: options.mnemonic,
        rpcUrl: options.rpcUrl,
        l1RpcUrl: options.l1RpcUrl,
        apiKey: options.apiKey ?? '',
        blockNumber: options.blockNumber,
        log,
        debugLogger,
      });
    });

  return program;
}
