import {
  AztecAddress,
  Contract,
  ContractDeployer,
  Fr,
  GrumpkinScalar,
  Point,
  generatePublicKey,
  getSchnorrAccount,
  isContractDeployed,
} from '@aztec/aztec.js';
import { StructType, decodeFunctionSignatureWithParameterNames } from '@aztec/foundation/abi';
import { JsonStringify } from '@aztec/foundation/json-rpc';
import { DebugLogger, LogFn } from '@aztec/foundation/log';
import { fileURLToPath } from '@aztec/foundation/url';
import { compileContract, generateNoirInterface, generateTypescriptInterface } from '@aztec/noir-compiler/cli';
import { CompleteAddress, ContractData, L2BlockL2Logs, TxHash } from '@aztec/types';

import { Command } from 'commander';
import { readFileSync } from 'fs';
import startCase from 'lodash.startcase';
import { dirname, resolve } from 'path';
import { mnemonicToAccount } from 'viem/accounts';

import { createCompatibleClient } from './client.js';
import { encodeArgs, parseStructString } from './encoding.js';
import { unboxContract } from './unbox.js';
import {
  deployAztecContracts,
  getAbiFunction,
  getContractAbi,
  getExampleContractArtifacts,
  getSaltFromHexString,
  getTxSender,
  prepTx,
  stripLeadingHex,
} from './utils.js';

const accountCreationSalt = Fr.ZERO;

const {
  ETHEREUM_HOST = 'http://localhost:8545',
  AZTEC_RPC_HOST = 'http://localhost:8080',
  PRIVATE_KEY,
  API_KEY,
} = process.env;

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
      const { rollupAddress, registryAddress, inboxAddress, outboxAddress, contractDeploymentEmitterAddress } =
        await deployAztecContracts(
          options.rpcUrl,
          options.apiKey ?? '',
          options.privateKey,
          options.mnemonic,
          debugLogger,
        );
      log('\n');
      log(`Rollup Address: ${rollupAddress.toString()}`);
      log(`Registry Address: ${registryAddress.toString()}`);
      log(`L1 -> L2 Inbox Address: ${inboxAddress.toString()}`);
      log(`L2 -> L1 Outbox address: ${outboxAddress.toString()}`);
      log(`Contract Deployment Emitter Address: ${contractDeploymentEmitterAddress.toString()}`);
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
    .command('create-account')
    .description(
      'Creates an aztec account that can be used for sending transactions. Registers the account on the RPC server and deploys an account contract. Uses a Schnorr single-key account which uses the same key for encryption and authentication (not secure for production usage).',
    )
    .summary('Creates an aztec account that can be used for sending transactions.')
    .option(
      '-k, --private-key <string>',
      'Private key for note encryption and transaction signing. Uses random by default.',
      PRIVATE_KEY,
    )
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async options => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const privateKey = options.privateKey
        ? GrumpkinScalar.fromString(stripLeadingHex(options.privateKey))
        : GrumpkinScalar.random();

      const account = getSchnorrAccount(client, privateKey, privateKey, accountCreationSalt);
      const wallet = await account.waitDeploy();
      const { address, publicKey, partialAddress } = wallet.getCompleteAddress();

      log(`\nCreated new account:\n`);
      log(`Address:         ${address.toString()}`);
      log(`Public key:      ${publicKey.toString()}`);
      if (!options.privateKey) log(`Private key:     ${privateKey.toString(true)}`);
      log(`Partial address: ${partialAddress.toString()}`);
    });

  program
    .command('deploy')
    .description('Deploys a compiled Aztec.nr contract to Aztec.')
    .argument(
      '<abi>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
    )
    .option('-a, --args <constructorArgs...>', 'Contract constructor arguments', [])
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .option(
      '-k, --public-key <string>',
      'Optional encryption public key for this address. Set this value only if this contract is expected to receive private notes, which will be encrypted using this public key.',
    )
    .option(
      '-s, --salt <hex string>',
      'Optional deployment salt as a hex string for generating the deployment address.',
      getSaltFromHexString,
    )
    .action(async (abiPath, options: any) => {
      const contractAbi = await getContractAbi(abiPath, log);
      const constructorAbi = contractAbi.functions.find(({ name }) => name === 'constructor');

      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const publicKey = options.publicKey ? Point.fromString(options.publicKey) : undefined;

      const deployer = new ContractDeployer(contractAbi, client, publicKey);

      const constructor = getAbiFunction(contractAbi, 'constructor');
      if (!constructor) throw new Error(`Constructor not found in contract ABI`);

      debugLogger(`Input arguments: ${options.args.map((x: any) => `"${x}"`).join(', ')}`);
      const args = encodeArgs(options.args, constructorAbi!.parameters);
      debugLogger(`Encoded arguments: ${args.join(', ')}`);
      const tx = deployer.deploy(...args).send({ contractAddressSalt: options.salt });
      debugLogger(`Deploy tx sent with hash ${await tx.getTxHash()}`);
      const deployed = await tx.wait();
      log(`\nContract deployed at ${deployed.contractAddress!.toString()}\n`);
    });

  program
    .command('check-deploy')
    .description('Checks if a contract is deployed to the specified Aztec address.')
    .requiredOption('-ca, --contract-address <address>', 'An Aztec address to check if contract has been deployed to.')
    .requiredOption('-u, --rpc-url <url>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async options => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const address = AztecAddress.fromString(options.contractAddress);
      const isDeployed = await isContractDeployed(client, address);
      if (isDeployed) log(`\nContract found at ${address.toString()}\n`);
      else log(`\nNo contract found at ${address.toString()}\n`);
    });

  program
    .command('get-tx-receipt')
    .description('Gets the receipt for the specified transaction hash.')
    .argument('<txHash>', 'A transaction hash to get the receipt for.')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async (_txHash, options) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const txHash = TxHash.fromString(_txHash);
      const receipt = await client.getTxReceipt(txHash);
      if (!receipt) {
        log(`No receipt found for transaction hash ${_txHash}`);
      } else {
        log(`\nTransaction receipt: \n${JsonStringify(receipt, true)}\n`);
      }
    });

  program
    .command('get-contract-data')
    .description('Gets information about the Aztec contract deployed at the specified address.')
    .argument('<contractAddress>', 'Aztec address of the contract.')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .option('-b, --include-bytecode <boolean>', "Include the contract's public function bytecode, if any.", false)
    .action(async (contractAddress, options) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const address = AztecAddress.fromString(contractAddress);
      const contractDataWithOrWithoutBytecode = options.includeBytecode
        ? await client.getExtendedContractData(address)
        : await client.getContractData(address);

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
    .description('Gets all the unencrypted logs from L2 blocks in the range specified.')
    .option('-f, --from <blockNum>', 'Initial block number for getting logs (defaults to 1).')
    .option('-l, --limit <blockCount>', 'How many blocks to fetch (defaults to 100).')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async options => {
      const { from, limit } = options;
      const fromBlock = from ? parseInt(from) : 1;
      const limitCount = limit ? parseInt(limit) : 100;

      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const logs = await client.getUnencryptedLogs(fromBlock, limitCount);
      if (!logs.length) {
        log(`No logs found in blocks ${fromBlock} to ${fromBlock + limitCount}`);
      } else {
        log('Logs found: \n');
        L2BlockL2Logs.unrollLogs(logs).forEach(fnLog => log(`${fnLog.toString('ascii')}\n`));
      }
    });

  program
    .command('register-recipient')
    .description('Register a recipient in the Aztec RPC.')
    .requiredOption('-a, --address <aztecAddress>', "The account's Aztec address.")
    .requiredOption('-p, --public-key <publicKey>', 'The account public key.')
    .requiredOption('-pa, --partial-address <partialAddress', 'The partially computed address of the account contract.')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async options => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const address = AztecAddress.fromString(options.address);
      const publicKey = Point.fromString(options.publicKey);
      const partialAddress = Fr.fromString(options.partialAddress);

      await client.registerRecipient(await CompleteAddress.create(address, publicKey, partialAddress));
      log(`\nRegistered details for account with address: ${options.address}\n`);
    });

  program
    .command('get-accounts')
    .description('Gets all the Aztec accounts stored in the Aztec RPC.')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
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
    .argument('<address>', 'The Aztec address to get account for')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async (_address, options) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const address = AztecAddress.fromString(_address);
      const account = await client.getRegisteredAccount(address);

      if (!account) {
        log(`Unknown account ${_address}`);
      } else {
        log(account.toReadableString());
      }
    });

  program
    .command('get-recipients')
    .description('Gets all the recipients stored in the Aztec RPC.')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
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
    .argument('<address>', 'The Aztec address to get recipient for')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async (_address, options) => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const address = AztecAddress.fromString(_address);
      const recipient = await client.getRecipient(address);

      if (!recipient) {
        log(`Unknown recipient ${_address}`);
      } else {
        log(recipient.toReadableString());
      }
    });

  program
    .command('send')
    .description('Calls a function on an Aztec contract.')
    .argument('<functionName>', 'Name of Function to view')
    .option('-a, --args [functionArgs...]', 'Function arguments', [])
    .requiredOption(
      '-c, --contract-abi <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.')
    .requiredOption('-k, --private-key <string>', "The sender's private key.", PRIVATE_KEY)
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)

    .action(async (functionName, options) => {
      const { contractAddress, functionArgs, contractAbi } = await prepTx(
        options.contractAbi,
        options.contractAddress,
        functionName,
        options.args,
        log,
      );

      const privateKey = GrumpkinScalar.fromString(stripLeadingHex(options.privateKey));

      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const wallet = await getSchnorrAccount(client, privateKey, privateKey, accountCreationSalt).getWallet();
      const contract = await Contract.at(contractAddress, contractAbi, wallet);
      const tx = contract.methods[functionName](...functionArgs).send();
      await tx.wait();
      log('\nTransaction has been mined');
      const receipt = await tx.getReceipt();
      log(`Transaction hash: ${(await tx.getTxHash()).toString()}`);
      log(`Status: ${receipt.status}\n`);
      log(`Block number: ${receipt.blockNumber}`);
      log(`Block hash: ${receipt.blockHash?.toString('hex')}`);
    });

  program
    .command('call')
    .description(
      'Simulates the execution of a view (read-only) function on a deployed contract, without modifying state.',
    )
    .argument('<functionName>', 'Name of Function to view')
    .option('-a, --args [functionArgs...]', 'Function arguments', [])
    .requiredOption(
      '-c, --contract-abi <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
    )
    .requiredOption('-ca, --contract-address <address>', 'Aztec address of the contract.')
    .option('-f, --from <string>', 'Aztec address of the caller. If empty, will use the first account from RPC.')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async (functionName, options) => {
      const { contractAddress, functionArgs, contractAbi } = await prepTx(
        options.contractAbi,
        options.contractAddress,
        functionName,
        options.args,
        log,
      );
      const fnAbi = getAbiFunction(contractAbi, functionName);
      if (fnAbi.parameters.length !== options.args.length) {
        throw Error(
          `Invalid number of args passed. Expected ${fnAbi.parameters.length}; Received: ${options.args.length}`,
        );
      }
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const from = await getTxSender(client, options.from);
      const result = await client.viewTx(functionName, functionArgs, contractAddress, from);
      log('\nView result: ', result, '\n');
    });

  // Helper for users to decode hex strings into structs if needed.
  program
    .command('parse-parameter-struct')
    .description("Helper for parsing an encoded string into a contract's parameter struct.")
    .argument('<encodedString>', 'The encoded hex string')
    .requiredOption(
      '-c, --contract-abi <fileLocation>',
      "A compiled Aztec.nr contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
    )
    .requiredOption('-p, --parameter <parameterName>', 'The name of the struct parameter to decode into')
    .action(async (encodedString, options) => {
      const contractAbi = await getContractAbi(options.contractAbi, log);
      const parameterAbitype = contractAbi.functions
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
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
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
    .argument('[localDirectory]', 'Local directory to unbox to (relative or absolute), defaults to `<contractName>`')
    .action(async (contractName, localDirectory) => {
      const unboxTo: string = localDirectory ? localDirectory : contractName;
      await unboxContract(contractName, unboxTo, version, log);
    });

  program
    .command('get-node-info')
    .description('Gets the information of an aztec node at a URL.')
    .requiredOption('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST)
    .action(async options => {
      const client = await createCompatibleClient(options.rpcUrl, debugLogger);
      const info = await client.getNodeInfo();
      log(`\nNode Info:\n`);
      Object.entries(info).map(([key, value]) => log(`${startCase(key)}: ${value}`));
    });

  program
    .command('inspect-contract')
    .description('Shows list of external callable functions for a contract')
    .argument(
      '<contractAbiFile>',
      `A compiled Noir contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts`,
    )
    .action(async (contractAbiFile: string) => {
      const contractAbi = await getContractAbi(contractAbiFile, debugLogger);
      const contractFns = contractAbi.functions.filter(
        f => !f.isInternal && f.name !== 'compute_note_hash_and_nullifier',
      );
      if (contractFns.length === 0) {
        log(`No external functions found for contract ${contractAbi.name}`);
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
