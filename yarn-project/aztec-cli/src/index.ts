#!/usr/bin/env -S node --no-warnings
import {
  AztecAddress,
  Contract,
  ContractDeployer,
  Fr,
  Point,
  createAztecRpcClient,
  generatePublicKey,
  getAccountWallets,
  getSchnorrAccount,
  isContractDeployed,
} from '@aztec/aztec.js';
import { StructType } from '@aztec/foundation/abi';
import { JsonStringify } from '@aztec/foundation/json-rpc';
import { createConsoleLogger, createDebugLogger } from '@aztec/foundation/log';
import { compileContract } from '@aztec/noir-compiler/cli';
import { SchnorrAccountContractAbi } from '@aztec/noir-contracts/artifacts';
import { CompleteAddress, ContractData, L2BlockL2Logs, PrivateKey, TxHash } from '@aztec/types';

import { Command } from 'commander';
import { mnemonicToAccount } from 'viem/accounts';

import { encodeArgs, parseStructString } from './cli_encoder.js';
import {
  deployAztecContracts,
  getAbiFunction,
  getContractAbi,
  getExampleContractArtifacts,
  getTxSender,
  prepTx,
} from './utils.js';

const accountCreationSalt = Fr.ZERO;

const debugLogger = createDebugLogger('aztec:cli');
const log = createConsoleLogger();
const stripLeadingHex = (hex: string) => {
  if (hex.length > 2 && hex.startsWith('0x')) {
    return hex.substring(2);
  }
  return hex;
};

const program = new Command();

program.name('aztec-cli').description('CLI for interacting with Aztec.').version('0.1.0');

const { ETHEREUM_HOST, AZTEC_RPC_HOST, PRIVATE_KEY, API_KEY } = process.env;

/**
 * Main function for the Aztec CLI.
 */
async function main() {
  program
    .command('deploy-l1-contracts')
    .description('Deploys all necessary Ethereum contracts for Aztec.')
    .option(
      '-u, --rpc-url <string>',
      'Url of the ethereum host. Chain identifiers localhost and testnet can be used',
      ETHEREUM_HOST || 'http://localhost:8545',
    )
    .option('-a, --api-key <string>', 'Api key for the ethereum host', API_KEY)
    .option('-p, --private-key <string>', 'The private key to use for deployment', PRIVATE_KEY)
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
    .description('Generates a 32-byte private key.')
    .option('-m, --mnemonic', 'A mnemonic string that can be used for the private key generation.')
    .action(async options => {
      let privKey;
      let publicKey;
      if (options.mnemonic) {
        const acc = mnemonicToAccount(options.mnemonic);
        const key = Buffer.from(acc.getHdKey().privateKey!);
        privKey = key.toString('hex');
        publicKey = await generatePublicKey(new PrivateKey(key));
      } else {
        const key = PrivateKey.random();
        privKey = key.toString();
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
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async options => {
      const client = createAztecRpcClient(options.rpcUrl);
      const privateKey = options.privateKey
        ? new PrivateKey(Buffer.from(stripLeadingHex(options.privateKey), 'hex'))
        : PrivateKey.random();

      const account = getSchnorrAccount(client, privateKey, privateKey, accountCreationSalt);
      const wallet = await account.waitDeploy();
      const { address, publicKey, partialAddress } = wallet.getCompleteAddress();

      log(`\nCreated new account:\n`);
      log(`Address:         ${address.toString()}`);
      log(`Public key:      ${publicKey.toString()}`);
      if (!options.privateKey) log(`Private key:     ${privateKey.toString()}`);
      log(`Partial address: ${partialAddress.toString()}`);
    });

  program
    .command('deploy')
    .description('Deploys a compiled Noir contract to Aztec.')
    .option(
      '-c, --contract-abi <file>',
      "A compiled Noir contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
      undefined,
    )
    .option('-a, --args <constructorArgs...>', 'Contract constructor arguments', [])
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .option(
      '-k, --public-key <string>',
      'Optional encryption public key for this address. Set this value only if this contract is expected to receive private notes, which will be encrypted using this public key.',
    )
    .option('-s, --salt <string>', 'Optional deployment salt as a hex string for generating the deployment address.')
    .action(async (options: any) => {
      const contractAbi = await getContractAbi(options.contractAbi, log);
      const constructorAbi = contractAbi.functions.find(({ name }) => name === 'constructor');

      const client = createAztecRpcClient(options.rpcUrl);
      const publicKey = options.publicKey ? Point.fromString(options.publicKey) : undefined;
      const salt = options.salt ? Fr.fromBuffer(Buffer.from(stripLeadingHex(options.salt), 'hex')) : undefined;
      const deployer = new ContractDeployer(contractAbi, client, publicKey);

      const constructor = getAbiFunction(contractAbi, 'constructor');
      if (!constructor) throw new Error(`Constructor not found in contract ABI`);
      if (constructor.parameters.length !== options.args.length) {
        throw new Error(
          `Invalid number of args passed (expected ${constructor.parameters.length} but got ${options.args.length})`,
        );
      }

      const args = encodeArgs(options.args, constructorAbi!.parameters);
      const tx = deployer.deploy(args).send({ contractAddressSalt: salt });
      const deployed = await tx.wait();
      log(`\nContract deployed at ${deployed.contractAddress!.toString()}\n`);
    });

  program
    .command('check-deploy')
    .description('Checks if a contract is deployed to the specified Aztec address.')
    .option('-ca, --contract-address <address>', 'An Aztec address to check if contract has been deployed to.')
    .option('-u, --rpc-url <url>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async options => {
      const client = createAztecRpcClient(options.rpcUrl);
      const address = AztecAddress.fromString(options.contractAddress);
      const isDeployed = await isContractDeployed(client, address);
      if (isDeployed) log(`\nContract found at ${address.toString()}\n`);
      else log(`\nNo contract found at ${address.toString()}\n`);
    });

  program
    .command('get-tx-receipt')
    .argument('<txHash>', 'A transaction hash to get the receipt for.')
    .description('Gets the receipt for the specified transaction hash.')
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async (_txHash, options) => {
      const client = createAztecRpcClient(options.rpcUrl);
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
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .option('-b, --include-bytecode', "Include the contract's public function bytecode, if any.")
    .action(async (contractAddress, options) => {
      const client = createAztecRpcClient(options.rpcUrl);
      const address = AztecAddress.fromString(contractAddress);
      const contractDataWithOrWithoutBytecode = options.includeBytecode
        ? await client.getContractDataAndBytecode(address)
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
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async options => {
      const { from, limit } = options;
      const fromBlock = from ? parseInt(from) : 0;
      const limitCount = limit ? parseInt(limit) : 100;

      const client = createAztecRpcClient(options.rpcUrl);
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
    .option('-a, --address <aztecAddress>', "The account's Aztec address.")
    .option('-p, --public-key <publicKey>', 'The account public key.')
    .option('-pa, --partial-address <partialAddress', 'The partially computed address of the account contract.')
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async options => {
      const client = createAztecRpcClient(options.rpcUrl);
      const address = AztecAddress.fromString(options.address);
      const publicKey = Point.fromString(options.publicKey);
      const partialAddress = Fr.fromString(options.partialAddress);

      await client.registerRecipient(await CompleteAddress.create(address, publicKey, partialAddress));
      log(`\nRegistered details for Address: ${options.address}\n`);
    });

  program
    .command('get-accounts')
    .description('Gets all the Aztec accounts stored in the Aztec RPC.')
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async (options: any) => {
      const client = createAztecRpcClient(options.rpcUrl);
      const accounts = await client.getAccounts();
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
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async (_address, options) => {
      const client = createAztecRpcClient(options.rpcUrl);
      const address = AztecAddress.fromString(_address);
      const account = await client.getAccount(address);

      if (!account) {
        log(`Unknown account ${_address}`);
      } else {
        log(account.toReadableString());
      }
    });

  program
    .command('get-recipients')
    .description('Gets all the recipients stored in the Aztec RPC.')
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async (options: any) => {
      const client = createAztecRpcClient(options.rpcUrl);
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
    .option('-u, --rpc-url <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async (_address, options) => {
      const client = createAztecRpcClient(options.rpcUrl);
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
    .option(
      '-c, --contract-abi <fileLocation>',
      "A compiled Noir contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
      undefined,
    )
    .option('-ca, --contract-address <address>', 'Aztec address of the contract.')
    .option('-k, --private-key <string>', "The sender's private key.", PRIVATE_KEY)
    .option('-u, --rpcUrl <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')

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

      const privateKey = new PrivateKey(Buffer.from(stripLeadingHex(options.privateKey), 'hex'));

      const client = createAztecRpcClient(options.rpcUrl);
      const wallet = await getAccountWallets(
        client,
        SchnorrAccountContractAbi,
        [privateKey],
        [privateKey],
        [accountCreationSalt],
      );
      const contract = await Contract.at(contractAddress, contractAbi, wallet);
      const tx = contract.methods[functionName](...functionArgs).send();
      await tx.isMined();
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
    .option(
      '-c, --contract-abi <fileLocation>',
      "A compiled Noir contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
      undefined,
    )
    .option('-ca, --contract-address <address>', 'Aztec address of the contract.')
    .option('-f, --from <string>', 'Public key of the TX viewer. If empty, will try to find account in RPC.')
    .option('-u, --rpcUrl <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
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
      const client = createAztecRpcClient(options.rpcUrl);
      const from = await getTxSender(client, options.from);
      const result = await client.viewTx(functionName, functionArgs, contractAddress, from);
      log('\nView result: ', JsonStringify(result, true), '\n');
    });

  // Helper for users to decode hex strings into structs if needed
  program
    .command('parse-parameter-struct')
    .description("Helper for parsing an encoded string into a contract's parameter struct.")
    .argument('<encodedString>', 'The encoded hex string')
    .option(
      '-c, --contract-abi <fileLocation>',
      "A compiled Noir contract's ABI in JSON format or name of a contract ABI exported by @aztec/noir-contracts",
      undefined,
    )
    .option('-p, --parameter <parameterName>', 'The name of the struct parameter to decode into')
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
    .option('-u, --rpcUrl <string>', 'URL of the Aztec RPC', AZTEC_RPC_HOST || 'http://localhost:8080')
    .action(async (options: any) => {
      const client = createAztecRpcClient(options.rpcUrl);
      const num = await client.getBlockNum();
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

  compileContract(program, 'compile', log);

  await program.parseAsync(process.argv);
}

main().catch(err => {
  log(`Error thrown: ${err}`);
  process.exit(1);
});
