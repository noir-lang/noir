import { AztecAddress, TxHash, createDebugLogger, sleep } from '@aztec/aztec.js';
import { getProgram } from '@aztec/cli';

import stringArgv from 'string-argv';

const debug = createDebugLogger('aztec:e2e_cli');

const { PXE_URL = 'http://localhost:8080' } = process.env;

describe('CLI docs sandbox', () => {
  let cli: ReturnType<typeof getProgram>;
  let log: (msg: string) => void;

  // All logs emitted by the cli will be collected here, and reset between tests
  const logs: string[] = [];

  beforeAll(async () => {
    log = (msg: string) => {
      logs.push(msg);
      debug(msg);
    };

    await waitForSandboxWithCli();
  }, 60_000);

  const waitForSandboxWithCli = async () => {
    const docs = `
// docs:start:node-info
% aztec-cli get-node-info
Node Info:

Node Version: #include_aztec_short_version
Chain Id: 31337
Protocol Version: 1
Rollup Address: 0x0dcd1bf9a1b36ce34237eeafef220932846bcd82
// docs:end:node-info
`;

    const command = docs.split('\n')[2].split('aztec-cli ')[1];
    while (true) {
      resetCli();
      try {
        await run(command);
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
      args.push('--rpc-url', PXE_URL);
    }
    return cli.parseAsync(args);
  };

  // Returns first match across all logs collected so far
  const findInLogs = (regex: RegExp) => {
    for (const log of logs) {
      const match = regex.exec(log);
      if (match) {
        return match;
      }
    }
  };

  const findMultipleInLogs = (regex: RegExp) => {
    const matches = [];
    for (const log of logs) {
      const match = regex.exec(log);
      if (match) {
        matches.push(match);
      }
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
BenchmarkingContractArtifact
CardGameContractArtifact
ChildContractArtifact
ContractClassRegistererContractArtifact
ContractInstanceDeployerContractArtifact
CounterContractArtifact
DocsExampleContractArtifact
EasyPrivateTokenContractArtifact
EasyPrivateVotingContractArtifact
EcdsaAccountContractArtifact
EscrowContractArtifact
ImportTestContractArtifact
InclusionProofsContractArtifact
LendingContractArtifact
ParentContractArtifact
PendingCommitmentsContractArtifact
PriceFeedContractArtifact
ReaderContractArtifact
SchnorrAccountContractArtifact
SchnorrHardcodedAccountContractArtifact
SchnorrSingleKeyAccountContractArtifact
SlowTreeContractArtifact
StatefulTestContractArtifact
TestContractArtifact
TokenBlacklistContractArtifact
TokenBridgeContractArtifact
TokenContractArtifact
UniswapContractArtifact
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

  Address: 0x0c8a6673d7676cc80aaebe7fa7504cf51daa90ba906861bfad70a58a98bf5a7d
  Public Key: 0x27c20118733174347b8082f578a7d8fb84b3ad38be293715eee8119ee5cd8a6d0d6b7d8124b37359663e75bcd2756f544a93b821a06f8e33fba68cc8029794d9
  Partial Address: 0x1c6484e22441e5ca43bba53495d0cdc911da299150fde1191bcb330b64716ff9

  Address: 0x226f8087792beff8d5009eb94e65d2a4a505b70baf4a9f28d33c8d620b0ba972
  Public Key: 0x08145e8e8d46f51cda8d4c9cad81920236366abeafb8d387002bad879a3e87a81570b04ac829e4c007141d856d5a36d3b9c464e0f3c1c99cdbadaa6bb93f3257
  Partial Address: 0x1833e53112953e6830a230cfc2895caed604f6395bbfafa730da26c5bf53c0a9

  Address: 0x0e1f60e8566e2c6d32378bdcadb7c63696e853281be798c107266b8c3a88ea9b
  Public Key: 0x13e6151ea8e7386a5e7c4c5221047bf73d0b1b7a2ad14d22b7f73e57c1fa00c614bc6da69da1b581b09ee6cdc195e5d58ae4dce01b63bbb744e58f03855a94dd
  Partial Address: 0x30034aaf5d78821effa4827a132357110a49a4f37b6e384d884e233595bcf342

  Address: 0x01b18c2044bbedd4a2e5f67cf6858370ccfb2b869b2000abe2f4ca12d9cc166e
  Public Key: 0x240845f1179e3fbaa6ce587d44722b3452bbdaa11deb29553196b23534985d432b746bcf2f0e7046eb13f0ca0c4fedd027dc80b64384f50d6a14ad248faa941a
  Partial Address: 0x03834845fc488d1454f195abe7d52b3393f6902eee080c90cd694c63572f7160
// docs:end:get-accounts
`;

    command = docs.split('\n')[2].split('aztec-cli ')[1];
    await run(command);

    const fetchedAddresses = findMultipleInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/);
    const foundFetchedAddress = fetchedAddresses.find(match => match.groups?.address === newAddress.toString());
    expect(foundFetchedAddress).toBeDefined();

    clearLogs();

    // Test deploy
    docs = `
// docs:start:deploy
% aztec-cli deploy TokenContractArtifact --args $ADDRESS TokenName TKN 18

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

    // Test send
    docs = `
// docs:start:send
% aztec-cli send mint_public \
  --args $ADDRESS 543 \
  --contract-artifact TokenContractArtifact \
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
      .replace('$ADDRESS', newAddress.toString())
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

    // Test call
    docs = `
// docs:start:call
% aztec-cli call balance_of_public -a $ADDRESS -c TokenContractArtifact -ca $CONTRACT_ADDRESS

View result:  543n
// docs:end:call
`;
    command = docs
      .split('\n')[2]
      .split('aztec-cli ')[1]
      .replace('$ADDRESS', newAddress.toString())
      .replace('$CONTRACT_ADDRESS', contractAddress.toString());

    await run(command);

    const foundBalance = findInLogs(/View\sresult:\s+(?<data>\S+)/)?.groups?.data;
    expect(foundBalance!).toEqual(`${BigInt(543).toString()}n`);
  }, 60_000);
});
