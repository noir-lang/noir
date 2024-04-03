import { BoxReactContract } from '../artifacts/BoxReact.js';
import { AccountWallet, Fr, Contract, TxStatus, createDebugLogger, ContractDeployer } from '@aztec/aztec.js';
import { deployerEnv } from '../src/config.js';

const logger = createDebugLogger('aztec:http-pxe-client');

describe('BoxReact Contract Tests', () => {
  let wallet: AccountWallet;
  let contract: Contract;
  const numberToSet = Fr.random();

  beforeAll(async () => {
    wallet = await deployerEnv.getWallet();
    const salt = Fr.random();
    contract = await BoxReactContract.deploy(wallet, Fr.random(), wallet.getCompleteAddress().address)
      .send({ contractAddressSalt: salt })
      .deployed();

    logger(`L2 contract deployed at ${contract.address}`);
  }, 60000);

  test('Can set a number', async () => {
    logger(`${await wallet.getRegisteredAccounts()}`);
    await contract.methods.setNumber(numberToSet, wallet.getCompleteAddress()).send().wait();
  }, 40000);

  test('Can read a number', async () => {
    const viewTxReceipt = await contract.methods.getNumber(wallet.getCompleteAddress()).simulate();
    expect(numberToSet.toBigInt()).toEqual(viewTxReceipt.value);
  }, 40000);
});
