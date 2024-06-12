import { type AztecAddress, NativeFeePaymentMethod, NativeFeePaymentMethodWithClaim } from '@aztec/aztec.js';
import { type GasSettings } from '@aztec/circuits.js';
import { type TokenContract as BananaCoin, type GasTokenContract } from '@aztec/noir-contracts.js';

import { FeesTest } from './fees_test.js';

describe('e2e_fees native_payments', () => {
  let aliceAddress: AztecAddress;
  let bobAddress: AztecAddress;
  let bananaCoin: BananaCoin;
  let gasSettings: GasSettings;
  let gasTokenContract: GasTokenContract;
  let paymentMethod: NativeFeePaymentMethod;

  const t = new FeesTest('native_payments');

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyFundAliceWithBananas();
    ({ gasTokenContract, aliceAddress, bobAddress, bananaCoin, gasSettings } = await t.setup());

    paymentMethod = new NativeFeePaymentMethod(aliceAddress);
  });

  afterAll(async () => {
    await t.teardown();
  });

  describe('without initial funds', () => {
    beforeAll(async () => {
      expect(await gasTokenContract.methods.balance_of_public(aliceAddress).simulate()).toEqual(0n);
    });

    it('fails to send a tx', async () => {
      await expect(
        bananaCoin.methods
          .transfer_public(aliceAddress, bobAddress, 1n, 0n)
          .send({ fee: { gasSettings, paymentMethod } })
          .wait(),
      ).rejects.toThrow(/Not enough balance for fee payer to pay for transaction/i);
    });

    it('claims bridged funds and pays with them on the same tx', async () => {
      const { secret } = await t.gasBridgeTestHarness.prepareTokensOnL1(
        t.INITIAL_GAS_BALANCE,
        t.INITIAL_GAS_BALANCE,
        aliceAddress,
      );
      const paymentMethod = new NativeFeePaymentMethodWithClaim(aliceAddress, t.INITIAL_GAS_BALANCE, secret);
      const receipt = await bananaCoin.methods
        .transfer_public(aliceAddress, bobAddress, 1n, 0n)
        .send({ fee: { gasSettings, paymentMethod } })
        .wait();
      const endBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();

      expect(endBalance).toBeGreaterThan(0n);
      expect(endBalance).toBeLessThan(t.INITIAL_GAS_BALANCE);
      expect(endBalance).toEqual(t.INITIAL_GAS_BALANCE - receipt.transactionFee!);
    });
  });

  describe('with initial funds', () => {
    beforeAll(async () => {
      await t.applyFundAliceWithGasToken();
    });

    it('sends tx with native fee payment method with public calls', async () => {
      const initialBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();
      const { transactionFee } = await bananaCoin.methods
        .transfer_public(aliceAddress, bobAddress, 1n, 0n)
        .send({ fee: { gasSettings, paymentMethod } })
        .wait();
      expect(transactionFee).toBeGreaterThan(0n);
      const endBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();
      expect(endBalance).toBeLessThan(initialBalance);
    });

    it('sends tx with native fee payment method with no public calls', async () => {
      const initialBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();
      const { transactionFee } = await bananaCoin.methods
        .transfer(bobAddress, 1n)
        .send({ fee: { gasSettings, paymentMethod } })
        .wait();
      expect(transactionFee).toBeGreaterThan(0n);
      const endBalance = await gasTokenContract.methods.balance_of_public(aliceAddress).simulate();
      expect(endBalance).toBeLessThan(initialBalance);
    });
  });
});
