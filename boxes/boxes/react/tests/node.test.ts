import { AccountWallet, CompleteAddress, Contract, Fr, createDebugLogger } from '@aztec/aztec.js';
import { BoxReactContract } from '../artifacts/BoxReact.js';
import { deployerEnv } from '../src/config.js';

const logger = createDebugLogger('aztec:http-pxe-client');

describe('BoxReact Contract Tests', () => {
  let wallet: AccountWallet;
  let contract: Contract;
  const numberToSet = Fr.random();
  let accountCompleteAddress: CompleteAddress;

  beforeAll(async () => {
    wallet = await deployerEnv.getWallet();
    accountCompleteAddress = wallet.getCompleteAddress();
    const salt = Fr.random();
    const { masterNullifierPublicKey, masterIncomingViewingPublicKey, masterOutgoingViewingPublicKey } =
      accountCompleteAddress.publicKeys;
    contract = await BoxReactContract.deploy(
      wallet,
      Fr.random(),
      accountCompleteAddress.address,
      masterNullifierPublicKey.hash(),
      masterOutgoingViewingPublicKey.toWrappedNoirStruct(),
      masterIncomingViewingPublicKey.toWrappedNoirStruct(),
    )
      .send({ contractAddressSalt: salt })
      .deployed();

    logger.info(`L2 contract deployed at ${contract.address}`);
  }, 60000);

  test('Can set a number', async () => {
    logger.info(`${await wallet.getRegisteredAccounts()}`);
    const { masterNullifierPublicKey, masterIncomingViewingPublicKey, masterOutgoingViewingPublicKey } =
      accountCompleteAddress.publicKeys;
    await contract.methods
      .setNumber(
        numberToSet,
        accountCompleteAddress.address,
        masterNullifierPublicKey.hash(),
        masterOutgoingViewingPublicKey.toWrappedNoirStruct(),
        masterIncomingViewingPublicKey.toWrappedNoirStruct(),
      )
      .send()
      .wait();
  }, 40000);

  test('Can read a number', async () => {
    const viewTxReceipt = await contract.methods.getNumber(accountCompleteAddress.address).simulate();
    expect(numberToSet.toBigInt()).toEqual(viewTxReceipt.value);
  }, 40000);
});
