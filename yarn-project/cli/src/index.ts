import { DebugLogger, LogFn } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';
import { addNoirCompilerCommanderActions } from '@aztec/noir-compiler/cli';

import { Command, Option } from 'commander';
import { resolve as dnsResolve } from 'dns';
import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';

import {
  parseAztecAddress,
  parseEthereumAddress,
  parseField,
  parseOptionalAztecAddress,
  parseOptionalInteger,
  parseOptionalLogId,
  parseOptionalSelector,
  parseOptionalTxHash,
  parsePartialAddress,
  parsePrivateKey,
  parsePublicKey,
  parseSaltFromHexString,
  parseTxHash,
} from './parse_args.js';

/**
 * If we can successfully resolve 'host.docker.internal', then we are running in a container, and we should treat
 * localhost as being host.docker.internal.
 */
function getLocalhost() {
  return new Promise(resolve =>
    dnsResolve('host.docker.internal', err => (err ? resolve('localhost') : resolve('host.docker.internal'))),
  );
}

const LOCALHOST = await getLocalhost();
const { ETHEREUM_HOST = `http://${LOCALHOST}:8545`, PRIVATE_KEY, API_KEY } = process.env;

/**
 * Returns commander program that defines the CLI.
 * @param log - Console logger.
 * @param debugLogger - Debug logger.
 * @returns The CLI.
 */
export function getProgram(log: LogFn, debugLogger: DebugLogger): Command {
  const program = new Command();

  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../package.json');
  const cliVersion: string = JSON.parse(readFileSync(packageJsonPath).toString()).version;
  const logJson = (obj: object) => log(JSON.stringify(obj, null, 2));

  program.name('aztec-cli').description('CLI for interacting with Aztec.').version(cliVersion);

  const pxeOption = new Option('-u, --rpc-url <string>', 'URL of the PXE')
    .env('PXE_URL')
    .default(`http://${LOCALHOST}:8080`)
    .makeOptionMandatory(true);

  const createPrivateKeyOption = (description: string, mandatory: boolean) =>
    new Option('-k, --private-key <string>', description)
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
    .command('generate-private-key')
    .summary('Generates an encryption private key.')
    .description(
      'Generates a private key which fits into the scalar field used by Grumpkin curve, can be used as an encryption private key.',
    )
    .option(
      '-m, --mnemonic',
      'An optional mnemonic string used for the private key generation. If not provided, random private key will be generated.',
    )
    .action(async options => {
      const { generatePrivateKey } = await import('./cmds/generate_private_key.js');
      generatePrivateKey(options.mnemonic, log);
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
    .addOption(
      createPrivateKeyOption('Private key for note encryption and transaction signing. Uses random by default.', false),
    )
    .addOption(pxeOption)
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction')
    .action(async ({ rpcUrl, privateKey, wait }) => {
      const { createAccount } = await import('./cmds/create_account.js');
      await createAccount(rpcUrl, privateKey, wait, debugLogger, log);
    });

  program
    .command('register-account')
    .description(
      'Registers an aztec account that can be used for sending transactions. Registers the account on the PXE. Uses a Schnorr single-key account which uses the same key for encryption and authentication (not secure for production usage).',
    )
    .summary('Registers an aztec account that can be used for sending transactions.')
    .addOption(createPrivateKeyOption('Private key for note encryption and transaction signing.', true))
    .requiredOption(
      '-pa, --partial-address <partialAddress>',
      'The partially computed address of the account contract.',
      parsePartialAddress,
    )
    .addOption(pxeOption)
    .action(async ({ rpcUrl, privateKey, partialAddress }) => {
      const { registerAccount } = await import('./cmds/register_account.js');
      await registerAccount(rpcUrl, privateKey, partialAddress, debugLogger, log);
    });

  program
    .command('deploy')
    .description('Deploys a compiled Aztec.nr contract to Aztec.')
    .argument(
      '<artifact>',
      "A compiled Aztec.nr contract's artifact in JSON format or name of a contract artifact exported by @aztec/noir-contracts",
    )
    .option('-a, --args <constructorArgs...>', 'Contract constructor arguments', [])
    .addOption(pxeOption)
    .option(
      '-k, --public-key <string>',
      'Optional encryption public key for this address. Set this value only if this contract is expected to receive private notes, which will be encrypted using this public key.',
      parsePublicKey,
    )
    .option(
      '-p, --portal-address <hex string>',
      'Optional L1 portal address to link the contract to.',
      parseEthereumAddress,
    )
    .option(
      '-s, --salt <hex string>',
      'Optional deployment salt as a hex string for generating the deployment address.',
      parseSaltFromHexString,
    )
    .option('--json', 'Emit output as json')
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction')
    .action(async (artifactPath, { json, rpcUrl, publicKey, args: rawArgs, portalAddress, salt, wait }) => {
      const { deploy } = await import('./cmds/deploy.js');
      await deploy(
        artifactPath,
        json,
        rpcUrl,
        publicKey,
        rawArgs,
        portalAddress,
        salt,
        wait,
        debugLogger,
        log,
        logJson,
      );
    });

  program
    .command('check-deploy')
    .description('Checks if a contract is deployed to the specified Aztec address.')
    .requiredOption(
      '-ca, --contract-address <address>',
      'An Aztec address to check if contract has been deployed to.',
      parseAztecAddress,
    )
    .addOption(pxeOption)
    .action(async options => {
      const { checkDeploy } = await import('./cmds/check_deploy.js');
      await checkDeploy(options.rpcUrl, options.contractAddress, debugLogger, log);
    });

  program
    .command('add-contract')
    .description(
      'Adds an existing contract to the PXE. This is useful if you have deployed a contract outside of the PXE and want to use it with the PXE.',
    )
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.', parseAztecAddress)
    .requiredOption('-pa, --partial-address <address>', 'Partial address of the contract', parsePartialAddress)
    .option('-p, --public-key <public key>', 'Optional public key for this contract', parsePublicKey)
    .option('--portal-address <address>', 'Optional address to a portal contract on L1', parseEthereumAddress)
    .addOption(pxeOption)
    .action(async options => {
      const { addContract } = await import('./cmds/add_contract.js');
      await addContract(
        options.rpcUrl,
        options.contractArtifact,
        options.contractAddress,
        options.partialAddress,
        options.publicKey,
        options.portalContract,
        debugLogger,
        log,
      );
    });

  program
    .command('get-tx-receipt')
    .description('Gets the receipt for the specified transaction hash.')
    .argument('<txHash>', 'A transaction hash to get the receipt for.', parseTxHash)
    .addOption(pxeOption)
    .action(async (txHash, options) => {
      const { getTxReceipt } = await import('./cmds/get_tx_receipt.js');
      await getTxReceipt(options.rpcUrl, txHash, debugLogger, log);
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
    .option('-s, --selector <hex string>', 'Event selector to filter logs by.', parseOptionalSelector)
    .addOption(pxeOption)
    .option('--follow', 'If set, will keep polling for new logs until interrupted.')
    .action(async ({ txHash, fromBlock, toBlock, afterLog, contractAddress, selector, rpcUrl, follow }) => {
      const { getLogs } = await import('./cmds/get_logs.js');
      await getLogs(txHash, fromBlock, toBlock, afterLog, contractAddress, selector, rpcUrl, follow, debugLogger, log);
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
    .action(async (options: any) => {
      const { getAccounts } = await import('./cmds/get_accounts.js');
      await getAccounts(options.rpcUrl, debugLogger, log);
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
    .command('send')
    .description('Calls a function on an Aztec contract.')
    .argument('<functionName>', 'Name of function to execute')
    .option('-a, --args [functionArgs...]', 'Function arguments', [])
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.', parseAztecAddress)
    .addOption(createPrivateKeyOption("The sender's private key.", true))
    .addOption(pxeOption)
    .option('--no-wait', 'Print transaction hash without waiting for it to be mined')
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
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
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
    .argument('<txHash>', 'The tx hash of the tx containing the note.', parseTxHash)
    .requiredOption('-n, --note [note...]', 'The members of a Note serialized as hex strings.', [])
    .addOption(pxeOption)
    .action(async (address, contractAddress, storageSlot, txHash, options) => {
      const { addNote } = await import('./cmds/add_note.js');
      await addNote(address, contractAddress, storageSlot, txHash, options.note, options.rpcUrl, debugLogger);
    });

  // Helper for users to decode hex strings into structs if needed.
  program
    .command('parse-parameter-struct')
    .description("Helper for parsing an encoded string into a contract's parameter struct.")
    .argument('<encodedString>', 'The encoded hex string')
    .requiredOption(
      '-c, --contract-artifact <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
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
    .description('Lists the example contracts available to deploy from @aztec/noir-contracts')
    .action(async () => {
      const { exampleContracts } = await import('./cmds/example_contracts.js');
      await exampleContracts(log);
    });

  program
    .command('unbox')
    .description(
      'Unboxes an example contract from @aztec/boxes.  Also Copies `noir-libs` dependencies and setup simple frontend for the contract using its ABI.',
    )
    .argument('<contractName>', 'Name of the contract to unbox, e.g. "PrivateToken"')
    .argument(
      '[localDirectory]',
      'Local directory to unbox source folder to (relative or absolute), optional - defaults to `<contractName>/`',
    )
    .action(async (contractName, localDirectory) => {
      const { unbox } = await import('./cmds/unbox.js');
      await unbox(contractName, localDirectory, cliVersion, log);
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
    .command('inspect-contract')
    .description('Shows list of external callable functions for a contract')
    .argument(
      '<contractArtifactFile>',
      `A compiled Noir contract's artifact in JSON format or name of a contract artifact exported by @aztec/noir-contracts`,
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
    .command('update')
    .description('Updates Nodejs and Noir dependencies')
    .argument('[projectPath]', 'Path to the project directory', process.cwd())
    .option('--contract [paths...]', 'Paths to contracts to update dependencies', [])
    .option('--aztec-version <semver>', 'The version to update Aztec packages to. Defaults to latest', 'latest')
    .addOption(pxeOption)
    .action(async (projectPath: string, options) => {
      const { update } = await import('./update/update.js');
      const { contract, aztecVersion, rpcUrl } = options;
      await update(projectPath, contract, rpcUrl, aztecVersion, log);
    });

  addNoirCompilerCommanderActions(program, log);

  return program;
}
