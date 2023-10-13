import {
  Contract,
  ContractDeployer,
  Fr,
  GrumpkinScalar,
  NotePreimage,
  generatePublicKey,
  getSchnorrAccount,
  isContractDeployed,
} from '@aztec/aztec.js';
import { StructType, decodeFunctionSignatureWithParameterNames } from '@aztec/foundation/abi';
import { JsonStringify } from '@aztec/foundation/json-rpc';
import { DebugLogger, LogFn } from '@aztec/foundation/log';
import { sleep } from '@aztec/foundation/sleep';
import { fileURLToPath } from '@aztec/foundation/url';
import { compileContract, generateNoirInterface, generateTypescriptInterface } from '@aztec/noir-compiler/cli';
import { CompleteAddress, ContractData, LogFilter } from '@aztec/types';

import { createSecp256k1PeerId } from '@libp2p/peer-id-factory';
import { Command, Option } from 'commander';
import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { format } from 'util';
import { mnemonicToAccount } from 'viem/accounts';

import { createCompatibleClient } from './client.js';
import { encodeArgs, parseStructString } from './encoding.js';
import { unboxContract } from './unbox.js';
import {
  deployAztecContracts,
  getContractArtifact,
  getExampleContractArtifacts,
  getFunctionArtifact,
  getTxSender,
  parseAztecAddress,
  parseField,
  parseFields,
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
  prepTx,
} from './utils.js';

const accountCreationSalt = Fr.ZERO;

const { ETHEREUM_HOST = 'http://localhost:8545', PRIVATE_KEY, API_KEY } = process.env;

/**
 * Returns commander program that defines the CLI.
 * @param log - Console logger.
 * @param debugLogger - Debug logger.
 * @returns The CLI.
 */
export function getProgram(log: LogFn, debugLogger: DebugLogger): Command {
  const program = new Command();

  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../package.json');
  const version: string = JSON.parse(readFileSync(packageJsonPath).toString()).version;

  program.name('aztec-cli').description('CLI for interacting with Aztec.').version(version);

  const pxeOption = new Option('-u, --rpc-url <string>', 'URL of the PXE')
    .env('PXE_URL')
    .default('http://localhost:8080')
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
      const { l1ContractAddresses } = await deployAztecContracts(
        options.rpcUrl,
        options.apiKey ?? '',
        options.privateKey,
        options.mnemonic,
        debugLogger,
      );
      log('\n');
      log(`Rollup Address: ${l1ContractAddresses.rollupAddress.toString()}`);
      log(`Registry Address: ${l1ContractAddresses.registryAddress.toString()}`);
      log(`L1 -> L2 Inbox Address: ${l1ContractAddresses.inboxAddress.toString()}`);
      log(`L2 -> L1 Outbox address: ${l1ContractAddresses.outboxAddress.toString()}`);
      log(`Contract Deployment Emitter Address: ${l1ContractAddresses.contractDeploymentEmitterAddress.toString()}`);
      log('\n');
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
      let privKey;
      let publicKey;
      if (options.mnemonic) {
        const acc = mnemonicToAccount(options.mnemonic);
        // TODO(#2052): This reduction is not secure enough. TACKLE THIS ISSUE BEFORE MAINNET.
        const key = GrumpkinScalar.fromBufferWithReduction(Buffer.from(acc.getHdKey().privateKey!));
        publicKey = await generatePublicKey(key);
      } else {
        const key = GrumpkinScalar.random();
        privKey = key.toString(true);
        publicKey = await generatePublicKey(key);
      }
      log(`\nPrivate Key: ${privKey}\nPublic Key: ${publicKey.toString()}\n`);
    });

  program
    .command('generate-p2p-private-key')
    .summary('Generates a LibP2P peer private key.')
    .description('Generates a private key that can be used for running a node on a LibP2P network.')
    .action(async () => {
      const peerId = await createSecp256k1PeerId();
      const exportedPeerId = Buffer.from(peerId.privateKey!).toString('hex');
      log(`Private key: ${exportedPeerId}`);
      log(`Peer Id: ${peerId}`);
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
      const client = await createCompatibleClient(rpcUrl, debugLogger);
      const actualPrivateKey = privateKey ?? GrumpkinScalar.random();

      const account = getSchnorrAccount(client, actualPrivateKey, actualPrivateKey, accountCreationSalt);
      const { address, publicKey, partialAddress } = await account.getCompleteAddress();
      const tx = await account.deploy();
      const txHash = await tx.getTxHash();
      debugLogger(`Account contract tx sent with hash ${txHash}`);
      if (wait) {
        log(`\nWaiting for account contract deployment...`);
        await tx.wait();
      } else {
        log(`\nAccount deployment transaction hash: ${txHash}\n`);
      }

      log(`\nNew account:\n`);
      log(`Address:         ${address.toString()}`);
      log(`Public key:      ${publicKey.toString()}`);
      if (!privateKey) log(`Private key:     ${actualPrivateKey.toString(true)}`);
      log(`Partial address: ${partialAddress.toString()}`);
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
      '-s, --salt <hex string>',
      'Optional deployment salt as a hex string for generating the deployment address.',
      parseSaltFromHexString,
    )
    // `options.wait` is default true. Passing `--no-wait` will set it to false.
    // https://github.com/tj/commander.js#other-option-types-negatable-boolean-and-booleanvalue
    .option('--no-wait', 'Skip waiting for the contract to be deployed. Print the hash of deployment transaction')
    .action(async (artifactPath, { rpcUrl, publicKey, args: rawArgs, salt, wait }) => {
      const contractArtifact = await getContractArtifact(artifactPath, log);
      const constructorArtifact = contractArtifact.functions.find(({ name }) => name === 'constructor');

      const client = await createCompatibleClient(rpcUrl, debugLogger);
      const deployer = new ContractDeployer(contractArtifact, client, publicKey);

      const constructor = getFunctionArtifact(contractArtifact, 'constructor');
      if (!constructor) throw new Error(`Constructor not found in contract ABI`);

      debugLogger(`Input arguments: ${rawArgs.map((x: any) => `"${x}"`).join(', ')}`);
      const args = encodeArgs(rawArgs, constructorArtifact!.parameters);
      debugLogger(`Encoded arguments: ${args.join(', ')}`);

      const tx = deployer.deploy(...args).send({ contractAddressSalt: salt });
      const txHash = await tx.getTxHash();
      debugLogger(`Deploy tx sent with hash ${txHash}`);
      if (wait) {
        const deployed = await tx.wait();
        log(`\nContract deployed at ${deployed.contractAddress!.toString()}\n`);
      } else {
        log(`\nContract Address: ${tx.completeContractAddress?.address.toString() ?? 'N/A'}`);
        log(`Deployment transaction hash: ${txHash}\n`);
      }
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
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const address = options.contractAddress;
      const isDeployed = await isContractDeployed(client, address);
      if (isDeployed) log(`\nContract found at ${address.toString()}\n`);
      else log(`\nNo contract found at ${address.toString()}\n`);
    });

  program
    .command('get-tx-receipt')
    .description('Gets the receipt for the specified transaction hash.')
    .argument('<txHash>', 'A transaction hash to get the receipt for.', parseTxHash)
    .addOption(pxeOption)
    .action(async (txHash, options) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const receipt = await client.getTxReceipt(txHash);
      if (!receipt) {
        log(`No receipt found for transaction hash ${txHash.toString()}`);
      } else {
        log(`\nTransaction receipt: \n${JsonStringify(receipt, true)}\n`);
      }
    });

  program
    .command('get-contract-data')
    .description('Gets information about the Aztec contract deployed at the specified address.')
    .argument('<contractAddress>', 'Aztec address of the contract.', parseAztecAddress)
    .addOption(pxeOption)
    .option('-b, --include-bytecode <boolean>', "Include the contract's public function bytecode, if any.", false)
    .action(async (contractAddress, options) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const contractDataWithOrWithoutBytecode = options.includeBytecode
        ? await client.getExtendedContractData(contractAddress)
        : await client.getContractData(contractAddress);

      if (!contractDataWithOrWithoutBytecode) {
        log(`No contract data found at ${contractAddress}`);
        return;
      }
      let contractData: ContractData;

      if ('contractData' in contractDataWithOrWithoutBytecode) {
        contractData = contractDataWithOrWithoutBytecode.contractData;
      } else {
        contractData = contractDataWithOrWithoutBytecode;
      }
      log(`\nContract Data: \nAddress: ${contractData.contractAddress.toString()}`);
      log(`Portal:  ${contractData.portalContractAddress.toString()}`);
      if ('bytecode' in contractDataWithOrWithoutBytecode) {
        log(`Bytecode: ${contractDataWithOrWithoutBytecode.bytecode}`);
      }
      log('\n');
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
      const pxe = await createCompatibleClient(rpcUrl, debugLogger);

      if (follow) {
        if (txHash) throw Error('Cannot use --follow with --tx-hash');
        if (toBlock) throw Error('Cannot use --follow with --to-block');
      }

      const filter: LogFilter = { txHash, fromBlock, toBlock, afterLog, contractAddress, selector };

      const fetchLogs = async () => {
        const response = await pxe.getUnencryptedLogs(filter);
        const logs = response.logs;

        if (!logs.length) {
          const filterOptions = Object.entries(filter)
            .filter(([, value]) => value !== undefined)
            .map(([key, value]) => `${key}: ${value}`)
            .join(', ');
          if (!follow) log(`No logs found for filter: {${filterOptions}}`);
        } else {
          if (!follow && !filter.afterLog) log('Logs found: \n');
          logs.forEach(unencryptedLog => log(unencryptedLog.toHumanReadable()));
          // Set the continuation parameter for the following requests
          filter.afterLog = logs[logs.length - 1].id;
        }
        return response.maxLogsHit;
      };

      if (follow) {
        log('Fetching logs...');
        while (true) {
          const maxLogsHit = await fetchLogs();
          if (!maxLogsHit) await sleep(1000);
        }
      } else {
        while (await fetchLogs()) {
          // Keep fetching logs until we reach the end.
        }
      }
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
      const client = await createCompatibleClient(rpcUrl, debugLogger);
      await client.registerRecipient(await CompleteAddress.create(address, publicKey, partialAddress));
      log(`\nRegistered details for account with address: ${address}\n`);
    });

  program
    .command('get-accounts')
    .description('Gets all the Aztec accounts stored in the PXE.')
    .addOption(pxeOption)
    .action(async (options: any) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const accounts = await client.getRegisteredAccounts();
      if (!accounts.length) {
        log('No accounts found.');
      } else {
        log(`Accounts found: \n`);
        for (const account of accounts) {
          log(account.toReadableString());
        }
      }
    });

  program
    .command('get-account')
    .description('Gets an account given its Aztec address.')
    .argument('<address>', 'The Aztec address to get account for', parseAztecAddress)
    .addOption(pxeOption)
    .action(async (address, options) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const account = await client.getRegisteredAccount(address);

      if (!account) {
        log(`Unknown account ${address.toString()}`);
      } else {
        log(account.toReadableString());
      }
    });

  program
    .command('get-recipients')
    .description('Gets all the recipients stored in the PXE.')
    .addOption(pxeOption)
    .action(async (options: any) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const recipients = await client.getRecipients();
      if (!recipients.length) {
        log('No recipients found.');
      } else {
        log(`Recipients found: \n`);
        for (const recipient of recipients) {
          log(recipient.toReadableString());
        }
      }
    });

  program
    .command('get-recipient')
    .description('Gets a recipient given its Aztec address.')
    .argument('<address>', 'The Aztec address to get recipient for', parseAztecAddress)
    .addOption(pxeOption)
    .action(async (address, options) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const recipient = await client.getRecipient(address);

      if (!recipient) {
        log(`Unknown recipient ${address.toString()}`);
      } else {
        log(recipient.toReadableString());
      }
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
      const { functionArgs, contractArtifact } = await prepTx(
        options.contractArtifact,
        functionName,
        options.args,
        log,
      );
      const { contractAddress, privateKey } = options;

      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const wallet = await getSchnorrAccount(client, privateKey, privateKey, accountCreationSalt).getWallet();
      const contract = await Contract.at(contractAddress, contractArtifact, wallet);
      const tx = contract.methods[functionName](...functionArgs).send();
      log(`\nTransaction hash: ${(await tx.getTxHash()).toString()}`);
      if (options.wait) {
        await tx.wait();

        log('Transaction has been mined');

        const receipt = await tx.getReceipt();
        log(`Status: ${receipt.status}\n`);
        log(`Block number: ${receipt.blockNumber}`);
        log(`Block hash: ${receipt.blockHash?.toString('hex')}`);
      } else {
        log('Transaction pending. Check status with get-tx-receipt');
      }
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
      const { functionArgs, contractArtifact } = await prepTx(
        options.contractArtifact,
        functionName,
        options.args,
        log,
      );

      const fnArtifact = getFunctionArtifact(contractArtifact, functionName);
      if (fnArtifact.parameters.length !== options.args.length) {
        throw Error(
          `Invalid number of args passed. Expected ${fnArtifact.parameters.length}; Received: ${options.args.length}`,
        );
      }
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const from = await getTxSender(client, options.from);
      const result = await client.viewTx(functionName, functionArgs, options.contractAddress, from);
      log(format('\nView result: ', result, '\n'));
    });

  program
    .command('add-note')
    .description('Adds a note to the database in the PXE.')
    .argument('<address>', 'The Aztec address of the note owner.', parseAztecAddress)
    .argument('<contractAddress>', 'Aztec address of the contract.', parseAztecAddress)
    .argument('<storageSlot>', 'The storage slot of the note.', parseField)
    .argument('<txHash>', 'The tx hash of the tx containing the note.', parseTxHash)
    .requiredOption('-p, --preimage [notePreimage...]', 'Note preimage.', [])
    .addOption(pxeOption)
    .action(async (address, contractAddress, storageSlot, txHash, options) => {
      const preimage = new NotePreimage(parseFields(options.preimage));
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      await client.addNote(address, contractAddress, storageSlot, preimage, txHash);
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
      const contractArtifact = await getContractArtifact(options.contractArtifact, log);
      const parameterAbitype = contractArtifact.functions
        .map(({ parameters }) => parameters)
        .flat()
        .find(({ name, type }) => name === options.parameter && type.kind === 'struct');
      if (!parameterAbitype) {
        log(`No struct parameter found with name ${options.parameter}`);
        return;
      }
      const data = parseStructString(encodedString, parameterAbitype.type as StructType);
      log(`\nStruct Data: \n${JsonStringify(data, true)}\n`);
    });

  program
    .command('block-number')
    .description('Gets the current Aztec L2 block number.')
    .addOption(pxeOption)
    .action(async (options: any) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const num = await client.getBlockNumber();
      log(`${num}\n`);
    });

  program
    .command('example-contracts')
    .description('Lists the example contracts available to deploy from @aztec/noir-contracts')
    .action(async () => {
      const abisList = await getExampleContractArtifacts();
      const names = Object.keys(abisList);
      names.forEach(name => log(name));
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
      const unboxTo: string = localDirectory ? localDirectory : contractName;
      await unboxContract(contractName, unboxTo, version, log);
    });

  program
    .command('get-node-info')
    .description('Gets the information of an aztec node at a URL.')
    .addOption(pxeOption)
    .action(async options => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const info = await client.getNodeInfo();
      log(`\nNode Info:\n`);
      log(`Sandbox Version: ${info.sandboxVersion}\n`);
      log(`Compatible Nargo Version: ${info.compatibleNargoVersion}\n`);
      log(`Chain Id: ${info.chainId}\n`);
      log(`Protocol Version: ${info.protocolVersion}\n`);
      log(`Rollup Address: ${info.l1ContractAddresses.rollupAddress.toString()}`);
    });

  program
    .command('inspect-contract')
    .description('Shows list of external callable functions for a contract')
    .argument(
      '<contractArtifactFile>',
      `A compiled Noir contract's artifact in JSON format or name of a contract artifact exported by @aztec/noir-contracts`,
    )
    .action(async (contractArtifactFile: string) => {
      const contractArtifact = await getContractArtifact(contractArtifactFile, debugLogger);
      const contractFns = contractArtifact.functions.filter(
        f => !f.isInternal && f.name !== 'compute_note_hash_and_nullifier',
      );
      if (contractFns.length === 0) {
        log(`No external functions found for contract ${contractArtifact.name}`);
      }
      for (const fn of contractFns) {
        const signature = decodeFunctionSignatureWithParameterNames(fn.name, fn.parameters);
        log(`${fn.functionType} ${signature}`);
      }
    });

  compileContract(program, 'compile', log);
  generateTypescriptInterface(program, 'generate-typescript', log);
  generateNoirInterface(program, 'generate-noir-interface', log);

  return program;
}
