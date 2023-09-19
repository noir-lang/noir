import {
  AccountWallet,
  AztecAddress,
  AztecRPC,
  CompleteAddress,
  Contract,
  Fr,
  Wallet,
  createAztecRpcClient,
  waitForSandbox,
} from '@aztec/aztec.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { BlankContract } from '../artifacts/blank.js';
import { callContractFunction, deployContract, getWallet, rpcClient } from '../index.js';
const logger = createDebugLogger('aztec:blank-box-test');

// assumes sandbox is running locally, which this script does not trigger
// as well as anvil.  anvil can be started with yarn test:integration
const setupSandbox = async () => {
  const { SANDBOX_URL = 'http://localhost:8080' } = process.env;
  const aztecRpc = createAztecRpcClient(SANDBOX_URL);
  await waitForSandbox(aztecRpc);
  return aztecRpc;
};

async function deployZKContract(owner: CompleteAddress, wallet: Wallet, rpcClient: AztecRPC) {
  logger('Deploying Blank contract...');
  const contractAddress = await deployContract(owner, BlankContract.abi, [], Fr.random(), rpcClient);

  logger(`L2 contract deployed at ${contractAddress}`);
  const contract = await BlankContract.at(contractAddress, wallet);
  return contract;
}

async function call(contractAddress: AztecAddress, testTokenContract: Contract, address: CompleteAddress) {
  return await callContractFunction(
    contractAddress,
    testTokenContract.abi,
    'getPublicKey',
    [address.address.toField()],
    rpcClient,
    address,
  );
}

describe('ZK Contract Tests', () => {
  let wallet: AccountWallet;
  let owner: CompleteAddress;
  let _account2: CompleteAddress;
  let _account3: CompleteAddress;
  let contract: Contract;
  let contractAddress: AztecAddress;
  let rpcClient: AztecRPC;

  beforeAll(async () => {
    rpcClient = await setupSandbox();
    const accounts = await rpcClient.getRegisteredAccounts();
    [owner, _account2, _account3] = accounts;

    wallet = await getWallet(owner, rpcClient);

    contract = await deployZKContract(owner, wallet, rpcClient);
    contractAddress = contract.address;
  }, 60000);

  test('call succeeds after deploy', async () => {
    const callTx = call(contractAddress, contract, owner);
    await callTx;
  }, 40000);
});
