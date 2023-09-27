import {
  AccountWallet,
  AztecAddress,
  PXE,
  CompleteAddress,
  Contract,
  Fr,
  TxStatus,
  Wallet,
  createPXEClient,
  waitForSandbox,
} from '@aztec/aztec.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { BlankContract } from '../artifacts/blank.js';
import { callContractFunction, deployContract, getWallet } from '../index.js';
const logger = createDebugLogger('aztec:blank-box-test');

// assumes sandbox is running locally, which this script does not trigger
// as well as anvil.  anvil can be started with yarn test:integration
const setupSandbox = async () => {
  const { SANDBOX_URL = 'http://localhost:8080' } = process.env;
  const pxe = createPXEClient(SANDBOX_URL);
  await waitForSandbox(pxe);
  return pxe;
};

async function deployZKContract(owner: CompleteAddress, wallet: Wallet, pxe: PXE) {
  logger('Deploying Blank contract...');
  const contractAddress = await deployContract(owner, BlankContract.abi, [], Fr.random(), pxe);

  logger(`L2 contract deployed at ${contractAddress}`);
  return BlankContract.at(contractAddress, wallet);
}

describe('ZK Contract Tests', () => {
  let wallet: AccountWallet;
  let owner: CompleteAddress;
  let _account2: CompleteAddress;
  let _account3: CompleteAddress;
  let contract: Contract;
  let contractAddress: AztecAddress;
  let pxe: PXE;

  beforeAll(async () => {
    pxe = await setupSandbox();
    const accounts = await pxe.getRegisteredAccounts();
    [owner, _account2, _account3] = accounts;

    wallet = await getWallet(owner, pxe);

    contract = await deployZKContract(owner, wallet, pxe);
    contractAddress = contract.address;
  }, 60000);

  test('call succeeds after deploy', async () => {
    const callTxReceipt = await callContractFunction(
      contractAddress,
      contract.abi,
      'getPublicKey',
      [owner.address.toField()],
      pxe,
      owner,
    );
    expect(callTxReceipt.status).toBe(TxStatus.MINED);
  }, 40000);
});
