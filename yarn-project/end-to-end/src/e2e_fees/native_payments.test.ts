import { type AztecAddress, NativeFeePaymentMethod } from '@aztec/aztec.js';
import { type GasSettings } from '@aztec/circuits.js';
import { type TokenContract as BananaCoin, type GasTokenContract } from '@aztec/noir-contracts.js';

import { FeesTest } from './fees_test.js';

describe('e2e_fees native_payments', () => {
  let aliceAddress: AztecAddress;
  let bobAddress: AztecAddress;
  let bananaCoin: BananaCoin;
  let gasSettings: GasSettings;
  let gasTokenContract: GasTokenContract;

  const t = new FeesTest('native_payments');

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyFundAliceWithBananas();
    await t.applyFundAliceWithGasToken();
    ({ gasTokenContract, aliceAddress, bobAddress, bananaCoin, gasSettings } = await t.setup());
  });

  afterAll(async () => {
    await t.teardown();
  });

  it('sends tx with native fee payment method with a public call', async () => {
    const paymentMethod = new NativeFeePaymentMethod(aliceAddress);
    const initialBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();
    const receipt = await bananaCoin.methods
      .transfer_public(aliceAddress, bobAddress, 1n, 0n)
      .send({ fee: { gasSettings, paymentMethod } })
      .wait();

    expect(receipt.transactionFee).toBeGreaterThan(0n);
    const endBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();
    expect(endBalance).toBeLessThan(initialBalance);
  });

  it('sends tx with native fee payment method with no public call', async () => {
    const paymentMethod = new NativeFeePaymentMethod(aliceAddress);
    const initialBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();
    const receipt = await bananaCoin.methods
      .transfer(aliceAddress, bobAddress, 1n, 0n)
      .send({ fee: { gasSettings, paymentMethod } })
      .wait();

    expect(receipt.transactionFee).toBeGreaterThan(0n);
    const endBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();
    expect(endBalance).toBeLessThan(initialBalance);
  });
});
