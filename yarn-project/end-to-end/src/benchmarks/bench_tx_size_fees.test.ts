import {
  type AccountWalletWithSecretKey,
  type AztecAddress,
  type EthAddress,
  type FeePaymentMethod,
  NativeFeePaymentMethod,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  TxStatus,
} from '@aztec/aztec.js';
import { GasSettings } from '@aztec/circuits.js';
import { FPCContract, GasTokenContract, TokenContract } from '@aztec/noir-contracts.js';
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from '../fixtures/utils.js';

jest.setTimeout(50_000);

describe('benchmarks/tx_size_fees', () => {
  let aliceWallet: AccountWalletWithSecretKey;
  let bobAddress: AztecAddress;
  let sequencerAddress: AztecAddress;
  let gas: GasTokenContract;
  let fpc: FPCContract;
  let token: TokenContract;
  let gasPortalAddress: EthAddress;

  // setup the environment
  beforeAll(async () => {
    const { wallets, aztecNode, deployL1ContractsValues } = await setup(3, {}, {}, true);

    gasPortalAddress = deployL1ContractsValues.l1ContractAddresses.gasPortalAddress;

    aliceWallet = wallets[0];
    bobAddress = wallets[1].getAddress();
    sequencerAddress = wallets[2].getAddress();

    await aztecNode.setConfig({
      feeRecipient: sequencerAddress,
    });

    await publicDeployAccounts(aliceWallet, wallets);
  });

  // deploy the contracts
  beforeAll(async () => {
    gas = await GasTokenContract.at(getCanonicalGasTokenAddress(gasPortalAddress), aliceWallet);
    token = await TokenContract.deploy(aliceWallet, aliceWallet.getAddress(), 'test', 'test', 18).send().deployed();
    fpc = await FPCContract.deploy(aliceWallet, token.address, gas.address).send().deployed();
  });

  // mint tokens
  beforeAll(async () => {
    await Promise.all([
      gas.methods.mint_public(aliceWallet.getAddress(), 100e9).send().wait(),
      gas.methods.mint_public(fpc.address, 100e9).send().wait(),
    ]);
    await token.methods.privately_mint_private_note(100e9).send().wait();
    await token.methods.mint_public(aliceWallet.getAddress(), 100e9).send().wait();
  });

  it.each<[string, () => Promise<FeePaymentMethod | undefined>]>([
    ['no', () => Promise.resolve(undefined)],
    ['native fee', () => NativeFeePaymentMethod.create(aliceWallet)],
    ['public fee', () => Promise.resolve(new PublicFeePaymentMethod(token.address, fpc.address, aliceWallet))],
    ['private fee', () => Promise.resolve(new PrivateFeePaymentMethod(token.address, fpc.address, aliceWallet))],
  ] as const)('sends a tx with a fee with %s payment method', async (_name, createPaymentMethod) => {
    const paymentMethod = await createPaymentMethod();
    const gasSettings = GasSettings.default();
    const tx = await token.methods
      .transfer(aliceWallet.getAddress(), bobAddress, 1n, 0)
      .send({ fee: paymentMethod ? { gasSettings, paymentMethod } : undefined })
      .wait();

    expect(tx.status).toEqual(TxStatus.MINED);
  });
});
