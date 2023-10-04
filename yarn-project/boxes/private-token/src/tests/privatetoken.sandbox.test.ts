import { PrivateTokenContract } from '../artifacts/private_token.js';
import { pxe } from '../config.js';
import { callContractFunction, deployContract, getWallet, viewContractFunction } from '../scripts/index.js';
import {
  AccountWallet,
  AztecAddress,
  PXE,
  CompleteAddress,
  Contract,
  Fr,
  Wallet,
  createPXEClient,
  waitForSandbox,
} from '@aztec/aztec.js';
import { createDebugLogger } from '@aztec/foundation/log';

const logger = createDebugLogger('aztec:private-token-box-sandbox-test');

const INITIAL_BALANCE = 444n;
const TRANSFER_AMOUNT = 44n;
const MINT_AMOUNT = 11n;

// assumes sandbox is running locally, which this script does not trigger
// as well as anvil.  anvil can be started with yarn test:integration
const setupSandbox = async () => {
  const { PXE_URL = 'http://localhost:8080' } = process.env;
  const pxe = createPXEClient(PXE_URL);
  await waitForSandbox(pxe);
  return pxe;
};

async function deployZKContract(owner: CompleteAddress, wallet: Wallet, pxe: PXE) {
  logger('Deploying PrivateToken contract...');
  const typedArgs = [new Fr(INITIAL_BALANCE), owner.address.toField()];

  const contractAddress = await deployContract(owner, PrivateTokenContract.abi, typedArgs, Fr.random(), pxe);

  logger(`L2 contract deployed at ${contractAddress}`);
  return PrivateTokenContract.at(contractAddress, wallet);
}

async function getBalance(contractAddress: AztecAddress, privateTokenContract: Contract, owner: CompleteAddress) {
  const typedArgs = [owner.address.toField()];

  return await viewContractFunction(contractAddress, privateTokenContract.abi, 'getBalance', typedArgs, pxe, owner);
}

async function mint(
  contractAddress: AztecAddress,
  privateTokenContract: Contract,
  from: CompleteAddress,
  to: CompleteAddress,
  amount: bigint,
) {
  const typedArgs = [new Fr(amount), to.address.toField()];

  return await callContractFunction(contractAddress, privateTokenContract.abi, 'mint', typedArgs, pxe, from);
}

async function transfer(
  contractAddress: AztecAddress,
  privateTokenContract: Contract,
  from: CompleteAddress,
  to: CompleteAddress,
  amount: bigint,
) {
  const typedArgs = [new Fr(amount), to.address.toField()];

  return await callContractFunction(contractAddress, privateTokenContract.abi, 'transfer', typedArgs, pxe, from);
}

describe('ZK Contract Tests', () => {
  let wallet: AccountWallet;
  let owner: CompleteAddress;
  let account2: CompleteAddress;
  let _account3: CompleteAddress;
  let privateTokenContract: Contract;
  let contractAddress: AztecAddress;
  let pxe: PXE;

  beforeAll(async () => {
    pxe = await setupSandbox();
    const accounts = await pxe.getRegisteredAccounts();
    [owner, account2, _account3] = accounts;

    wallet = await getWallet(owner, pxe);

    privateTokenContract = await deployZKContract(owner, wallet, pxe);
    contractAddress = privateTokenContract.address;
  }, 60000);

  test('Initial balance is correct', async () => {
    const balance = await getBalance(contractAddress, privateTokenContract, owner);
    expect(balance).toEqual(INITIAL_BALANCE);
  }, 40000);

  test('Balance after mint is correct', async () => {
    const mintTx = mint(contractAddress, privateTokenContract, owner, owner, MINT_AMOUNT);
    await mintTx;

    const balanceAfterMint = await getBalance(contractAddress, privateTokenContract, owner);
    expect(balanceAfterMint).toEqual(INITIAL_BALANCE + MINT_AMOUNT);
  }, 40000);

  test('Balance after transfer is correct for both sender and receiver', async () => {
    const transferTx = transfer(contractAddress, privateTokenContract, owner, account2, TRANSFER_AMOUNT);
    await transferTx;

    const balanceAfterTransfer = await getBalance(contractAddress, privateTokenContract, owner);
    expect(balanceAfterTransfer).toEqual(INITIAL_BALANCE + MINT_AMOUNT - TRANSFER_AMOUNT);

    const receiverBalance = await getBalance(contractAddress, privateTokenContract, account2);
    expect(receiverBalance).toEqual(TRANSFER_AMOUNT);
  }, 40000);
});
