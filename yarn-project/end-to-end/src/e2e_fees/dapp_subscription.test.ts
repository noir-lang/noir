import {
  type AccountWallet,
  type AztecAddress,
  type FeePaymentMethod,
  Fr,
  type PXE,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  SentTx,
} from '@aztec/aztec.js';
import { DefaultDappEntrypoint } from '@aztec/entrypoints/dapp';
import {
  type AppSubscriptionContract,
  type TokenContract as BananaCoin,
  type CounterContract,
  type FPCContract,
} from '@aztec/noir-contracts.js';

import { expectMapping, expectMappingDelta } from '../fixtures/utils.js';
import { FeesTest } from './fees_test.js';

type Balances = [bigint, bigint, bigint];

describe('e2e_fees dapp_subscription', () => {
  let pxe: PXE;

  let aliceWallet: AccountWallet;
  let aliceAddress: AztecAddress; // Dapp subscriber.
  let bobAddress: AztecAddress; // Dapp owner.
  let sequencerAddress: AztecAddress;

  let bananaCoin: BananaCoin;
  let counterContract: CounterContract;
  let subscriptionContract: AppSubscriptionContract;
  let bananaFPC: FPCContract;

  let initialSubscriptionContractGasBalance: bigint;
  let initialSequencerGasBalance: bigint;
  let initialFPCGasBalance: bigint;
  let initialBananasPublicBalances: Balances; // alice, bob, fpc
  let initialBananasPrivateBalances: Balances; // alice, bob, fpc

  const t = new FeesTest('dapp_subscription');

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.applyFPCSetupSnapshot();
    await t.applyFundAliceWithBananas();
    await t.applySetupSubscription();

    ({
      aliceWallet,
      aliceAddress,
      bobAddress,
      sequencerAddress,
      bananaCoin,
      bananaFPC,
      subscriptionContract,
      counterContract,
      pxe,
    } = await t.setup());
  });

  afterAll(async () => {
    await t.teardown();
  });

  beforeAll(async () => {
    await expectMapping(
      t.getGasBalanceFn,
      [aliceAddress, sequencerAddress, subscriptionContract.address, bananaFPC.address],
      [0n, 0n, t.INITIAL_GAS_BALANCE, t.INITIAL_GAS_BALANCE],
    );

    await expectMapping(
      t.getBananaPrivateBalanceFn,
      [aliceAddress, bobAddress, bananaFPC.address],
      [t.ALICE_INITIAL_BANANAS, 0n, 0n],
    );

    await expectMapping(
      t.getBananaPublicBalanceFn,
      [aliceAddress, bobAddress, bananaFPC.address],
      [t.ALICE_INITIAL_BANANAS, 0n, 0n],
    );
  });

  beforeEach(async () => {
    [initialSubscriptionContractGasBalance, initialSequencerGasBalance, initialFPCGasBalance] =
      (await t.getGasBalanceFn(subscriptionContract, sequencerAddress, bananaFPC)) as Balances;
    initialBananasPublicBalances = (await t.getBananaPublicBalanceFn(aliceAddress, bobAddress, bananaFPC)) as Balances;
    initialBananasPrivateBalances = (await t.getBananaPrivateBalanceFn(
      aliceAddress,
      bobAddress,
      bananaFPC,
    )) as Balances;
  });

  it('should allow Alice to subscribe by paying privately with bananas', async () => {
    /**
    PRIVATE SETUP
    we first unshield `MAX_FEE` BC from alice's private balance to the FPC's public balance

    PUBLIC APP LOGIC
    we then privately transfer `SUBSCRIPTION_AMOUNT` BC from alice to bob's subscription contract

    PUBLIC TEARDOWN
    the FPC publicly sends `REFUND` BC to alice
    */

    const { transactionFee } = await subscribe(
      new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet),
    );

    await expectMapping(
      t.getGasBalanceFn,
      [sequencerAddress, bananaFPC.address],
      [initialSequencerGasBalance, initialFPCGasBalance - transactionFee!],
    );

    // alice, bob, fpc
    await expectBananasPrivateDelta(-t.SUBSCRIPTION_AMOUNT - t.maxFee, t.SUBSCRIPTION_AMOUNT, 0n);
    await expectBananasPublicDelta(0n, 0n, transactionFee!);

    // REFUND_AMOUNT is a transparent note note
  });

  it('should allow Alice to subscribe by paying with bananas in public', async () => {
    /**
    PRIVATE SETUP
    we publicly transfer `MAX_FEE` BC from alice's public balance to the FPC's public balance

    PUBLIC APP LOGIC
    we then privately transfer `SUBSCRIPTION_AMOUNT` BC from alice to bob's subscription contract

    PUBLIC TEARDOWN
    the FPC publicly sends `REFUND` BC to alice
    */
    const { transactionFee } = await subscribe(
      new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet),
    );

    await expectMapping(
      t.getGasBalanceFn,
      [sequencerAddress, bananaFPC.address],
      [initialSequencerGasBalance, initialFPCGasBalance - transactionFee!],
    );

    // alice, bob, fpc
    // we pay the fee publicly, but the subscription payment is still private.
    await expectBananasPrivateDelta(-t.SUBSCRIPTION_AMOUNT, t.SUBSCRIPTION_AMOUNT, 0n);
    // we have the refund from the previous test,
    // but since we paid publicly this time, the refund should have been "squashed"
    await expectBananasPublicDelta(-transactionFee!, 0n, transactionFee!);
  });

  it('should call dapp subscription entrypoint', async () => {
    // Subscribe again, so this test does not depend on the previous ones being run.
    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet));

    expect(await subscriptionContract.methods.is_initialized(aliceAddress).simulate()).toBe(true);

    const dappPayload = new DefaultDappEntrypoint(aliceAddress, aliceWallet, subscriptionContract.address);
    // Emitting the outgoing logs to Alice below
    const action = counterContract.methods.increment(bobAddress, aliceAddress).request();
    const txExReq = await dappPayload.createTxExecutionRequest({ calls: [action] });
    const tx = await pxe.proveTx(txExReq, true);
    const sentTx = new SentTx(pxe, pxe.sendTx(tx));
    const { transactionFee } = await sentTx.wait();

    expect(await counterContract.methods.get_counter(bobAddress).simulate()).toBe(1n);

    await expectMapping(
      t.getGasBalanceFn,
      [sequencerAddress, subscriptionContract.address],
      [initialSequencerGasBalance, initialSubscriptionContractGasBalance - transactionFee!],
    );
  });

  it('should reject after the sub runs out', async () => {
    // Subscribe again. This will overwrite the previous subscription.
    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet), 0);
    // TODO(#6651): Change back to /(context.block_number()) as u64 < expiry_block_number as u64/ when fixed
    await expect(dappIncrement()).rejects.toThrow(/Note encrypted logs hash mismatch/);
  });

  it('should reject after the txs run out', async () => {
    // Subscribe again. This will overwrite the previous subscription.
    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet), 5, 1);
    await expect(dappIncrement()).resolves.toBeDefined();
    await expect(dappIncrement()).rejects.toThrow(/note.remaining_txs as u64 > 0/);
  });

  async function subscribe(paymentMethod: FeePaymentMethod, blockDelta: number = 5, txCount: number = 4) {
    const nonce = Fr.random();
    const action = bananaCoin.methods.transfer_from(aliceAddress, bobAddress, t.SUBSCRIPTION_AMOUNT, nonce);
    await aliceWallet.createAuthWit({ caller: subscriptionContract.address, action });

    return subscriptionContract
      .withWallet(aliceWallet)
      .methods.subscribe(aliceAddress, nonce, (await pxe.getBlockNumber()) + blockDelta, txCount)
      .send({ fee: { gasSettings: t.gasSettings, paymentMethod } })
      .wait();
  }

  async function dappIncrement() {
    const dappEntrypoint = new DefaultDappEntrypoint(aliceAddress, aliceWallet, subscriptionContract.address);
    // Emitting the outgoing logs to Alice below
    const action = counterContract.methods.increment(bobAddress, aliceAddress).request();
    const txExReq = await dappEntrypoint.createTxExecutionRequest({ calls: [action] });
    const tx = await pxe.proveTx(txExReq, true);
    expect(tx.data.feePayer).toEqual(subscriptionContract.address);
    const sentTx = new SentTx(pxe, pxe.sendTx(tx));
    return sentTx.wait();
  }

  const expectBananasPrivateDelta = (aliceAmount: bigint, bobAmount: bigint, fpcAmount: bigint) =>
    expectMappingDelta(
      initialBananasPrivateBalances,
      t.getBananaPrivateBalanceFn,
      [aliceAddress, bobAddress, bananaFPC.address],
      [aliceAmount, bobAmount, fpcAmount],
    );

  const expectBananasPublicDelta = (aliceAmount: bigint, bobAmount: bigint, fpcAmount: bigint) =>
    expectMappingDelta(
      initialBananasPublicBalances,
      t.getBananaPublicBalanceFn,
      [aliceAddress, bobAddress, bananaFPC.address],
      [aliceAmount, bobAmount, fpcAmount],
    );
});
