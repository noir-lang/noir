import { AztecAddress, createDebugLogger, sleep } from '@aztec/aztec.js';
import { getProgram } from '@aztec/cli';
import { TxHash } from '@aztec/types';

import stringArgv from 'string-argv';
import { format } from 'util';

const debug = createDebugLogger('aztec:e2e_cli');

const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

describe('CLI docs sandbox', () => {
  let cli: ReturnType<typeof getProgram>;
  let log: (...args: any[]) => void;

  // All logs emitted by the cli will be collected here, and reset between tests
  const logs: string[] = [];

  beforeAll(async () => {
    log = (...args: any[]) => {
      logs.push(format(...args));
      debug(...args);
    };

    await waitForSandboxWithCli();
  }, 60_000);

  const waitForSandboxWithCli = async () => {
    while (true) {
      resetCli();
      try {
        await run('get-node-info');
        break;
      } catch (err) {
        await sleep(1000);
      }
    }
  };

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
      args.push('--rpc-url', SANDBOX_URL);
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

  it('prints example contracts', async () => {
    const docs = `
// docs:start:example-contracts
% aztec-cli example-contracts
CardGameContractAbi
ChildContractAbi
DocsExampleContractAbi
EasyPrivateTokenContractAbi
EcdsaAccountContractAbi
EscrowContractAbi
ImportTestContractAbi
LendingContractAbi
MultiTransferContractAbi
NativeTokenContractAbi
NonNativeTokenContractAbi
ParentContractAbi
PendingCommitmentsContractAbi
PokeableTokenContractAbi
PriceFeedContractAbi
PrivateTokenAirdropContractAbi
PrivateTokenContractAbi
PublicTokenContractAbi
SchnorrAccountContractAbi
SchnorrAuthWitnessAccountContractAbi
SchnorrHardcodedAccountContractAbi
SchnorrSingleKeyAccountContractAbi
TestContractAbi
UniswapContractAbi
// docs:end:example-contracts
`;

    const command = docs.split('\n')[2].split('aztec-cli ')[1];
    const expectedConsoleOutput = docs.split('\n').slice(3, -2);

    await run(command, false);
    expect(logs).toEqual(expectedConsoleOutput);
  });

  it('gets a block number', async () => {
    const docs = `
// docs:start:block-number
% aztec-cli block-number
1
// docs:end:block-number
`;

    const command = docs.split('\n')[2].split('aztec-cli ')[1];

    await run(command);
    // expect logs to contain a number and nothing else
    expect(logs.length).toEqual(1);
    expect(logs[0]).toMatch(/\d+/);
  });

  it('creates an account from private key', async () => {
    const docs = `
// docs:start:create-account-from-private-key
% aztec-cli generate-private-key

Private Key: 0x12684562c8676e66be100878434b01286a757dea468233f818b906f66fb34984
Public Key: 0x1003732857c052c1d6af4dd74b5631863a056c90a586c4e3ea6d94782ee712d317cdb713ed1ba02d3df0ac2b581d269490f9e24916c1b677c7259444aa0ad66b


% aztec-cli create-account --private-key 0x12684562c8676e66be100878434b01286a757dea468233f818b906f66fb34984

Created new account:

Address:         0x26e831b1b146d1faf0c1d27fc72f2243887e9963cc87a6b3af64fe6481920a80
Public key:      0x1003732857c052c1d6af4dd74b5631863a056c90a586c4e3ea6d94782ee712d317cdb713ed1ba02d3df0ac2b581d269490f9e24916c1b677c7259444aa0ad66b
Partial address: 0x01e5e7b2abbfb98a93b7549ae80faa6886f8ea8e8f412416fb330b565fd2b4ed
// docs:end:create-account-from-private-key
`;

    const generateCommand = docs.split('\n')[2].split('aztec-cli ')[1];
    await run(generateCommand, false);

    const foundPrivateKey = findInLogs(/Private\sKey:\s+(?<privateKey>0x[a-fA-F0-9]+)/)?.groups?.privateKey;
    expect(foundPrivateKey).toBeDefined();
    const foundPublicKeyGenerate = findInLogs(/Public\sKey:\s+(?<publicKey>0x[a-fA-F0-9]+)/)?.groups?.publicKey;
    expect(foundPublicKeyGenerate).toBeDefined();

    clearLogs();

    const createCommand = docs.split('\n')[8].split('aztec-cli ')[1];

    await run(createCommand);
    const foundAddress = findInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(foundAddress).toBeDefined();
    const foundPublicKey = findInLogs(/Public\skey:\s+(?<publicKey>0x[a-fA-F0-9]+)/)?.groups?.publicKey;
    expect(foundPublicKey).toBeDefined();
    const foundPartialAddress = findInLogs(/Partial\saddress:\s+(?<partialAddress>0x[a-fA-F0-9]+)/)?.groups
      ?.partialAddress;
    expect(foundPartialAddress).toBeDefined();
  });

  it('creates an account, gets account, deploys, checks deployed, view method, sending a tx... [SEQUENTIAL]', async () => {
    // Test create-account
    let docs = `
// docs:start:create-account
% aztec-cli create-account
Created new account:

Address:         0x20d3321707d53cebb168568e25c5c62a853ae1f0766d965e00d6f6c4eb05d599
Public key:      0x02d18745eadddd496be95274367ee2cbf0bf667b81373fb6bed715c18814a09022907c273ec1c469fcc678738bd8efc3e9053fe1acbb11fa32da0d6881a1370e
Private key:     0x2aba9e7de7075deee3e3f4ad1e47749f985f0f72543ed91063cc97a40d851f1e
Partial address: 0x72bf7c9537875b0af267b4a8c497927e251f5988af6e30527feb16299042ed
// docs:end:create-account
`;

    let command = docs.split('\n')[2].split('aztec-cli ')[1];

    await run(command);
    const foundAddress = findInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(foundAddress).toBeDefined();
    const foundPublicKey = findInLogs(/Public\skey:\s+(?<publicKey>0x[a-fA-F0-9]+)/)?.groups?.publicKey;
    expect(foundPublicKey).toBeDefined();
    const foundPrivateKey = findInLogs(/Private\skey:\s+(?<privateKey>0x[a-fA-F0-9]+)/)?.groups?.privateKey;
    expect(foundPrivateKey).toBeDefined();
    const foundPartialAddress = findInLogs(/Partial\saddress:\s+(?<partialAddress>0x[a-fA-F0-9]+)/)?.groups
      ?.partialAddress;
    expect(foundPartialAddress).toBeDefined();
    const newAddress = AztecAddress.fromString(foundAddress!);

    clearLogs();

    // Test get-account
    docs = `
// docs:start:get-accounts
% aztec-cli get-accounts
Accounts found:

Address:         0x20d3321707d53cebb168568e25c5c62a853ae1f0766d965e00d6f6c4eb05d599
Public key:      0x02d18745eadddd496be95274367ee2cbf0bf667b81373fb6bed715c18814a09022907c273ec1c469fcc678738bd8efc3e9053fe1acbb11fa32da0d6881a1370e
Partial address: 0x72bf7c9537875b0af267b4a8c497927e251f5988af6e30527feb16299042ed

Address:         0x175310d40cd3412477db1c2a2188efd586b63d6830115fbb46c592a6303dbf6c
Public key:      0x08aad54f32f1b6621ee5f25267166e160147cd355a2dfc129fa646a651dd29471d814ac749c2cda831fcca361c830ba56db4b4bd5951d4953c81865d0ae0cbe7
Partial address: 0x72bf7c9537875b0af267b4a8c497927e251f5988af6e30527feb16299042ed
// docs:end:get-accounts
`;

    command = docs.split('\n')[2].split('aztec-cli ')[1];
    await run(command);

    const fetchedAddresses = findMultipleInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/);
    const foundFetchedAddress = fetchedAddresses.find(match => match.groups?.address === newAddress.toString());
    expect(foundFetchedAddress).toBeDefined();

    clearLogs();

    // Set some of the found addresses as address2 for later use
    const address2 = AztecAddress.fromString(fetchedAddresses[1].groups?.address as string);

    // Test deploy
    docs = `
// docs:start:deploy
% aztec-cli deploy PrivateTokenContractAbi --args 1000000 $ADDRESS

Contract deployed at 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f
// docs:end:deploy
    `;

    command = docs.split('\n')[2].split('aztec-cli ')[1].replace('$ADDRESS', newAddress.toString());
    await run(command);

    let foundContractAddress = findInLogs(/Contract\sdeployed\sat\s(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(foundContractAddress).toBeDefined();
    const contractAddress = AztecAddress.fromString(foundContractAddress!);

    clearLogs();

    // Test check-deploy
    docs = `
// docs:start:check-deploy
% aztec-cli check-deploy --contract-address $CONTRACT_ADDRESS

Contract found at 0x1ae8eea0dc265fb7f160dae62cc8912686d8a9ed78e821fbdd8bcedc54c06d0f
// docs:end:check-deploy
`;
    command = docs.split('\n')[2].split('aztec-cli ')[1].replace('$CONTRACT_ADDRESS', contractAddress.toString());
    await run(command);

    foundContractAddress = findInLogs(/Contract\sfound\sat\s(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(foundContractAddress).toEqual(contractAddress.toString());

    clearLogs();

    // Test call
    docs = `
// docs:start:call
% aztec-cli call getBalance \
  --args $ADDRESS \
  --contract-abi PrivateTokenContractAbi \
  --contract-address $CONTRACT_ADDRESS

View result:  1000000n
// docs:end:call
`;
    command = docs
      .split('\n')[2]
      .split('aztec-cli ')[1]
      .replace('$ADDRESS', newAddress.toString())
      .replace('$CONTRACT_ADDRESS', contractAddress.toString());
    await run(command);

    let foundBalance = findInLogs(/View\sresult:\s+(?<data>\S+)/)?.groups?.data;
    expect(foundBalance!).toEqual(`${BigInt(1000000).toString()}n`);

    clearLogs();

    // We reset CLI so that we can call the same command again later on
    resetCli();

    // Test send
    docs = `
// docs:start:send
% aztec-cli send transfer \
  --args 543 $ADDRESS2 \
  --contract-abi PrivateTokenContractAbi \
  --contract-address $CONTRACT_ADDRESS \
  --private-key $PRIVATE_KEY

Transaction has been mined
Transaction hash: 15c5a8e58d5f895c7e3017a706efbad693635e01f67345fa60a64a340d83c78c
Status: mined
Block number: 5
Block hash: 163697608599543b2bee9652f543938683e4cdd0f94ac506e5764d8b908d43d4
// docs:end:send
`;

    command = docs
      .split('\n')[2]
      .split('aztec-cli ')[1]
      .replace('$ADDRESS2', address2.toString())
      .replace('$CONTRACT_ADDRESS', contractAddress.toString())
      .replace('$PRIVATE_KEY', foundPrivateKey!);
    await run(command);

    let foundTxHash = findInLogs(/Transaction\shash:\s+(?<txHash>\S+)/)?.groups?.txHash;
    expect(foundTxHash).toBeDefined();

    clearLogs();

    // Save the tx hash for later use
    const transferTxHash = TxHash.fromString(foundTxHash!);

    // Test get-tx-receipt
    docs = `
// docs:start:get-tx-receipt
% aztec-cli get-tx-receipt 15c5a8e58d5f895c7e3017a706efbad693635e01f67345fa60a64a340d83c78c

Transaction receipt:
{
  "txHash": "15c5a8e58d5f895c7e3017a706efbad693635e01f67345fa60a64a340d83c78c",
  "status": "mined",
  "error": "",
  "blockHash": "163697608599543b2bee9652f543938683e4cdd0f94ac506e5764d8b908d43d4",
  "blockNumber": 5,
  "origin": "0x2337f1d5cfa6c03796db5539b0b2d5a57e9aed42665df2e0907f66820cb6eebe"
}
// docs:end:get-tx-receipt
`;

    command = docs
      .split('\n')[2]
      .split('aztec-cli ')[1]
      .replace('15c5a8e58d5f895c7e3017a706efbad693635e01f67345fa60a64a340d83c78c', transferTxHash.toString());
    await run(command);

    foundTxHash = findInLogs(/"txHash":\s+"(?<txHash>\S+)"/)?.groups?.txHash;
    expect(foundTxHash).toEqual(transferTxHash.toString());
    const status = findInLogs(/"status":\s+"(?<status>\S+)"/)?.groups?.status;
    expect(status).toEqual('mined');
    const error = findInLogs(/"error":\s+"(?<error>\S*)"/)?.groups?.error;
    expect(error).toEqual('');

    clearLogs();

    // get balance
    docs = `
// docs:start:calls
% aztec-cli call getBalance -a $ADDRESS -c PrivateTokenContractAbi -ca $CONTRACT_ADDRESS

View result:  999457n

% aztec-cli call getBalance -a $ADDRESS2 -c PrivateTokenContractAbi -ca $CONTRACT_ADDRESS

View result:  543n
// docs:end:calls
`;
    command = docs
      .split('\n')[2]
      .split('aztec-cli ')[1]
      .replace('$ADDRESS', newAddress.toString())
      .replace('$CONTRACT_ADDRESS', contractAddress.toString());

    await run(command);

    foundBalance = findInLogs(/View\sresult:\s+(?<data>\S+)/)?.groups?.data;
    expect(foundBalance!).toEqual(`${BigInt(999457).toString()}n`);

    clearLogs();
    resetCli();

    command = docs
      .split('\n')[6]
      .split('aztec-cli ')[1]
      .replace('$ADDRESS2', address2.toString())
      .replace('$CONTRACT_ADDRESS', contractAddress.toString());

    await run(command);

    foundBalance = findInLogs(/View\sresult:\s+(?<data>\S+)/)?.groups?.data;
    expect(foundBalance!).toEqual(`${BigInt(543).toString()}n`);

    clearLogs();
  }, 60_000);
});
