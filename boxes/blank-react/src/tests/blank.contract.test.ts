import { BlankContract } from '../artifacts/Blank.js';
import { callContractFunction, deployContract, getWallet } from '../scripts/index.js';
import {
  AccountWallet,
  AztecAddress,
  CompleteAddress,
  Contract,
  Fr,
  PXE,
  TxStatus,
  Wallet,
  createDebugLogger,
} from '@aztec/aztec.js';
import { setupEnvironment } from '../environment/index.js';

const logger = createDebugLogger('aztec:http-pxe-client');


async function deployZKContract(owner: CompleteAddress, wallet: Wallet, pxe: PXE) {
  logger('Deploying Blank contract...');
  const contractAddress = await deployContract(owner, BlankContract.artifact, [], Fr.random(), pxe);

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
    pxe = await setupEnvironment();
    const accounts = await pxe.getRegisteredAccounts();
    [owner, _account2, _account3] = accounts;

    wallet = await getWallet(owner, pxe);

    contract = await deployZKContract(owner, wallet, pxe);
    contractAddress = contract.address;
  }, 60000);

  test('call succeeds after deploy', async () => {
    const callTxReceipt = await callContractFunction(
      contractAddress,
      contract.artifact,
      'getPublicKey',
      [owner.address.toField()],
      pxe,
      owner,
    );

    expect(callTxReceipt.status).toBe(TxStatus.MINED);
  }, 40000);
});
