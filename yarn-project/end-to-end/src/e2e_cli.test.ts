import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer } from '@aztec/aztec-rpc';
import { startHttpRpcServer } from '@aztec/aztec-sandbox';
import { createDebugLogger } from '@aztec/aztec.js';
import { getProgram } from '@aztec/cli';
import { AztecRPC, CompleteAddress } from '@aztec/types';

import stringArgv from 'string-argv';
import { format } from 'util';

import { setup } from './fixtures/utils.js';

const HTTP_PORT = 9009;
const RPC_URL = `http://localhost:${HTTP_PORT}`;
const debug = createDebugLogger('aztec:e2e_cli');

let http: ReturnType<typeof startHttpRpcServer>;
let aztecNode: AztecNodeService | undefined;
let aztecRpcServer: AztecRPC;

const testSetup = async () => {
  const context = await setup(2);
  debug(`Environment set up`);
  const { deployL1ContractsValues } = context;
  ({ aztecNode, aztecRpcServer } = context);
  http = startHttpRpcServer(aztecRpcServer, deployL1ContractsValues, HTTP_PORT);
  debug(`HTTP RPC server started in port ${HTTP_PORT}`);
  return aztecRpcServer;
};

const cleanup = async () => {
  http.close();
  await aztecNode?.stop();
  await (aztecRpcServer as AztecRPCServer).stop();
};

const INITIAL_BALANCE = 33000;
const TRANSFER_BALANCE = 3000;

describe('CLI e2e test', () => {
  let cli: ReturnType<typeof getProgram>;
  let aztecRpcClient: AztecRPC;
  let existingAccounts: CompleteAddress[];
  let contractAddress: AztecAddress;
  let log: (...args: any[]) => void;

  // All logs emitted by the cli will be collected here, and reset between tests
  const logs: string[] = [];

  beforeAll(async () => {
    aztecRpcClient = await testSetup();
    log = (...args: any[]) => {
      logs.push(format(...args));
      debug(...args);
    };
  }, 30_000);

  afterAll(async () => {
    await cleanup();
  });

  // in order to run the same command twice, we need to create a new CLI instance
  const resetCli = () => {
    cli = getProgram(log, debug);
  };

  beforeEach(() => {
    logs.splice(0);
    resetCli();
  });

  // Run a command on the CLI
  const run = (cmd: string, addRpcUrl = true) => {
    const args = stringArgv(cmd, 'node', 'dest/bin/index.js');
    if (addRpcUrl) {
      args.push('--rpc-url', RPC_URL);
    }
    return cli.parseAsync(args);
  };

  // Returns first match across all logs collected so far
  const findInLogs = (regex: RegExp) => {
    for (const log of logs) {
      const match = regex.exec(log);
      if (match) return match;
    }
  };

  const findMultipleInLogs = (regex: RegExp) => {
    const matches = [];
    for (const log of logs) {
      const match = regex.exec(log);
      if (match) matches.push(match);
    }
    return matches;
  };

  const clearLogs = () => {
    logs.splice(0);
  };

  it('creates & retrieves an account', async () => {
    existingAccounts = await aztecRpcClient.getRegisteredAccounts();
    debug('Create an account');
    await run(`create-account`);
    const foundAddress = findInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(foundAddress).toBeDefined();
    const newAddress = AztecAddress.fromString(foundAddress!);

    const accountsAfter = await aztecRpcClient.getRegisteredAccounts();
    const expectedAccounts = [...existingAccounts.map(a => a.address), newAddress];
    expect(accountsAfter.map(a => a.address)).toEqual(expectedAccounts);
    const newCompleteAddress = accountsAfter[accountsAfter.length - 1];

    // Test get-accounts
    debug('Check that account was added to the list of accs in RPC');
    await run('get-accounts');
    const fetchedAddresses = findMultipleInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/);
    const foundFetchedAddress = fetchedAddresses.find(match => match.groups?.address === newAddress.toString());
    expect(foundFetchedAddress).toBeDefined();

    // Test get-account
    debug('Check we can retrieve the specific account');
    clearLogs();
    await run(`get-account ${newAddress.toString()}`);
    const fetchedAddress = findInLogs(/Public Key:\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(fetchedAddress).toEqual(newCompleteAddress.publicKey.toString());
  });

  it('deploys a contract & sends transactions', async () => {
    // generate a private key
    debug('Create an account using a private key');
    await run('generate-private-key', false);
    const privKey = findInLogs(/Private\sKey:\s+0x(?<privKey>[a-fA-F0-9]+)/)?.groups?.privKey;
    expect(privKey).toHaveLength(64);
    await run(`create-account --private-key ${privKey}`);
    const foundAddress = findInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(foundAddress).toBeDefined();
    const ownerAddress = AztecAddress.fromString(foundAddress!);

    debug('Deploy Private Token Contract using created account.');
    await run(`deploy PrivateTokenContractAbi --args ${INITIAL_BALANCE} ${ownerAddress} --salt 0`);
    const loggedAddress = findInLogs(/Contract\sdeployed\sat\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(loggedAddress).toBeDefined();
    contractAddress = AztecAddress.fromString(loggedAddress!);

    const deployedContract = await aztecRpcClient.getContractData(contractAddress);
    expect(deployedContract?.contractAddress).toEqual(contractAddress);

    debug('Check contract can be found in returned address');
    await run(`check-deploy -ca ${loggedAddress}`);
    const checkResult = findInLogs(/Contract\sfound\sat\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(checkResult).toEqual(deployedContract?.contractAddress.toString());

    // clear logs
    clearLogs();
    await run(`get-contract-data ${loggedAddress}`);
    const contractDataAddress = findInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(contractDataAddress).toEqual(deployedContract?.contractAddress.toString());

    debug("Check owner's balance");
    await run(
      `call getBalance --args ${ownerAddress} --contract-abi PrivateTokenContractAbi --contract-address ${contractAddress.toString()}`,
    );
    const balance = findInLogs(/View\sresult:\s+(?<data>\S+)/)?.groups?.data;
    expect(balance!).toEqual(`${BigInt(INITIAL_BALANCE).toString()}n`);

    debug('Transfer some tokens');
    const existingAccounts = await aztecRpcClient.getRegisteredAccounts();
    // ensure we pick a different acc
    const receiver = existingAccounts.find(acc => acc.address.toString() !== ownerAddress.toString());

    await run(
      `send transfer --args ${TRANSFER_BALANCE} ${receiver?.address.toString()} --contract-address ${contractAddress.toString()} --contract-abi PrivateTokenContractAbi --private-key ${privKey}`,
    );
    const txHash = findInLogs(/Transaction\shash:\s+(?<txHash>\S+)/)?.groups?.txHash;

    debug('Check the transfer receipt');
    await run(`get-tx-receipt ${txHash}`);
    const txResult = findInLogs(/Transaction receipt:\s*(?<txHash>[\s\S]*?\})/)?.groups?.txHash;
    const parsedResult = JSON.parse(txResult!);
    expect(parsedResult.txHash).toEqual(txHash);
    expect(parsedResult.status).toEqual('mined');
    debug("Check Receiver's balance");
    // Reset CLI as we're calling getBalance again
    resetCli();
    clearLogs();
    await run(
      `call getBalance --args ${receiver?.address.toString()} --contract-abi PrivateTokenContractAbi --contract-address ${contractAddress.toString()}`,
    );
    const receiverBalance = findInLogs(/View\sresult:\s+(?<data>\S+)/)?.groups?.data;
    expect(receiverBalance).toEqual(`${BigInt(TRANSFER_BALANCE).toString()}n`);
  }, 30_000);
});
