import {
  type AccountWallet,
  type AztecAddress,
  type FeePaymentMethod,
  NativeFeePaymentMethod,
  PublicFeePaymentMethod,
} from '@aztec/aztec.js';
import { GasFees, type GasSettings } from '@aztec/circuits.js';
import { type TokenContract as BananaCoin, type FPCContract } from '@aztec/noir-contracts.js';

import { FeesTest } from './fees_test.js';

describe('e2e_fees gas_estimation', () => {
  let aliceWallet: AccountWallet;
  let aliceAddress: AztecAddress;
  let bobAddress: AztecAddress;
  let bananaCoin: BananaCoin;
  let bananaFPC: FPCContract;
  let gasSettings: GasSettings;
  let teardownFixedFee: bigint;

  const t = new FeesTest('gas_estimation');

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyFPCSetupSnapshot();
    await t.applyFundAliceWithBananas();
    await t.applyFundAliceWithGasToken();
    ({ aliceWallet, aliceAddress, bobAddress, bananaCoin, bananaFPC, gasSettings } = await t.setup());

    teardownFixedFee = gasSettings.teardownGasLimits.computeFee(GasFees.default()).toBigInt();
  });

  afterAll(async () => {
    await t.teardown();
  });

  // Sends two tx with transfers of public tokens: one with estimateGas on, one with estimateGas off
  const sendTransfers = (paymentMethod: FeePaymentMethod) =>
    Promise.all(
      [true, false].map(estimateGas =>
        bananaCoin.methods
          .transfer_public(aliceAddress, bobAddress, 1n, 0n)
          .send({ estimateGas, fee: { gasSettings, paymentMethod } })
          .wait(),
      ),
    );

  it('estimates gas with native fee payment method', async () => {
    const paymentMethod = new NativeFeePaymentMethod(aliceAddress);
    const [withEstimate, withoutEstimate] = await sendTransfers(paymentMethod);

    // Estimation should yield that teardown has no cost, so should send the tx with zero for teardown
    expect(withEstimate.transactionFee! + teardownFixedFee).toEqual(withoutEstimate.transactionFee!);
  });

  it('estimates gas with public payment method', async () => {
    const paymentMethod = new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet);
    const [withEstimate, withoutEstimate] = await sendTransfers(paymentMethod);

    // Estimation should yield that teardown has reduced cost, but is not zero
    // TODO(palla/gas): We set toBeGreaterThanOrEqual because gas in public functions is zero for now (we only meter on AVM).
    // We should be able to change this to a strict equality once we meter gas in public functions or we replace brillig with the AVM.
    expect(withEstimate.transactionFee!).toBeLessThan(withoutEstimate.transactionFee!);
    expect(withEstimate.transactionFee! + teardownFixedFee).toBeGreaterThanOrEqual(withoutEstimate.transactionFee!);
  });
});
