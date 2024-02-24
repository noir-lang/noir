import {
  AccountWalletWithPrivateKey,
  AztecAddress,
  FeePaymentMethod,
  Fr,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  SentTx,
  computeAuthWitMessageHash,
} from '@aztec/aztec.js';
import { DefaultDappEntrypoint } from '@aztec/entrypoints/dapp';
import {
  AppSubscriptionContractContract,
  TokenContract as BananaCoin,
  CounterContract,
  FPCContract,
  GasTokenContract,
} from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import {
  EndToEndContext,
  PublicBalancesFn,
  assertPublicBalances,
  getPublicBalancesFn,
  setup,
} from './fixtures/utils.js';
import { GasBridgingTestHarness } from './shared/gas_portal_test_harness.js';

jest.setTimeout(1_000_000);

const TOKEN_NAME = 'BananaCoin';
const TOKEN_SYMBOL = 'BAC';
const TOKEN_DECIMALS = 18n;

describe('e2e_fees', () => {
  let aliceWallet: AccountWalletWithPrivateKey;
  let bobWallet: AccountWalletWithPrivateKey;
  let aliceAddress: AztecAddress; // Dapp subscriber.
  let bobAddress: AztecAddress; // Dapp owner.
  let sequencerAddress: AztecAddress;
  // let gasTokenContract: GasTokenContract;
  let bananaCoin: BananaCoin;
  let counterContract: CounterContract;
  let subscriptionContract: AppSubscriptionContractContract;
  let gasTokenContract: GasTokenContract;
  let bananaFPC: FPCContract;
  let e2eContext: EndToEndContext;
  let gasBridgeTestHarness: GasBridgingTestHarness;
  let gasBalances: PublicBalancesFn;

  const FEE_AMOUNT = 1n;
  const SUBSCRIPTION_AMOUNT = 100n;
  const BRIDGED_GAS_BALANCE = 1000n;
  const MINTED_BANANAS = 1000n;

  beforeAll(async () => {
    process.env.PXE_URL = '';
    e2eContext = await setup(3);

    const { wallets, accounts, aztecNode, deployL1ContractsValues, logger, pxe } = e2eContext;

    aliceAddress = accounts.at(0)!.address;
    bobAddress = accounts.at(1)!.address;
    sequencerAddress = accounts.at(2)!.address;

    gasBridgeTestHarness = await GasBridgingTestHarness.new(
      pxe,
      deployL1ContractsValues.publicClient,
      deployL1ContractsValues.walletClient,
      wallets[0],
      logger,
    );

    gasTokenContract = gasBridgeTestHarness.l2Token;

    await aztecNode.setConfig({
      feeRecipient: sequencerAddress,
    });

    [aliceWallet, bobWallet] = wallets;

    bananaCoin = await BananaCoin.deploy(aliceWallet, aliceAddress, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_DECIMALS)
      .send()
      .deployed();
    bananaFPC = await FPCContract.deploy(aliceWallet, bananaCoin.address, gasTokenContract.address).send().deployed();

    counterContract = await CounterContract.deploy(bobWallet, 0, bobAddress).send().deployed();

    subscriptionContract = await AppSubscriptionContractContract.deploy(
      bobWallet,
      counterContract.address,
      bobAddress,
      // anyone can purchase a subscription for 100 test tokens
      bananaCoin.address,
      SUBSCRIPTION_AMOUNT,
      // I had to pass this in because the address kept changing
      gasTokenContract.address,
    )
      .send()
      .deployed();

    // mint some test tokens for Alice
    // she'll pay for the subscription with these
    await bananaCoin.methods.privately_mint_private_note(MINTED_BANANAS).send().wait();
    await bananaCoin.methods.mint_public(aliceAddress, MINTED_BANANAS).send().wait();
    await gasBridgeTestHarness.bridgeFromL1ToL2(BRIDGED_GAS_BALANCE, BRIDGED_GAS_BALANCE, subscriptionContract.address);
    await gasBridgeTestHarness.bridgeFromL1ToL2(BRIDGED_GAS_BALANCE, BRIDGED_GAS_BALANCE, bananaFPC.address);

    gasBalances = getPublicBalancesFn('⛽', gasTokenContract, e2eContext.logger);

    await assertPublicBalances(
      gasBalances,
      [sequencerAddress, subscriptionContract.address, bananaFPC.address],
      [0n, BRIDGED_GAS_BALANCE, BRIDGED_GAS_BALANCE],
    );
  });

  it('should allow Alice to subscribe by paying privately with bananas', async () => {
    // Authorize the subscription contract to transfer the subscription amount from the subscriber.
    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet));
    expect(await bananaCoin.methods.balance_of_private(aliceAddress).view()).toBe(
      BRIDGED_GAS_BALANCE - SUBSCRIPTION_AMOUNT - FEE_AMOUNT,
    );
    expect(await bananaCoin.methods.balance_of_private(bobAddress).view()).toBe(SUBSCRIPTION_AMOUNT);
    expect(await bananaCoin.methods.balance_of_public(bananaFPC).view()).toBe(FEE_AMOUNT);

    // remains unchanged
    await assertPublicBalances(
      gasBalances,
      [subscriptionContract.address, bananaFPC.address, sequencerAddress],
      [BRIDGED_GAS_BALANCE, BRIDGED_GAS_BALANCE - FEE_AMOUNT, FEE_AMOUNT],
    );
  });

  it('should allow Alice to subscribe by paying with bananas in public', async () => {
    // Authorize the subscription contract to transfer the subscription amount from the subscriber.
    await subscribe(new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet));

    // assert that Alice paid 100n for the subscription
    expect(await bananaCoin.methods.balance_of_private(aliceAddress).view()).toBe(
      BRIDGED_GAS_BALANCE - 2n * SUBSCRIPTION_AMOUNT - FEE_AMOUNT,
    );
    expect(await bananaCoin.methods.balance_of_private(bobAddress).view()).toBe(2n * SUBSCRIPTION_AMOUNT);

    // assert that Alice has paid one banana publicly for the tx above
    expect(await bananaCoin.methods.balance_of_public(aliceAddress).view()).toBe(MINTED_BANANAS - FEE_AMOUNT);
    expect(await bananaCoin.methods.balance_of_public(bananaFPC).view()).toBe(2n * FEE_AMOUNT);

    await assertPublicBalances(
      gasBalances,
      [subscriptionContract.address, bananaFPC.address, sequencerAddress],
      [BRIDGED_GAS_BALANCE, BRIDGED_GAS_BALANCE - 2n * FEE_AMOUNT, 2n * FEE_AMOUNT],
    );
  });

  it('should call dapp subscription entrypoint', async () => {
    const { pxe } = e2eContext;
    const dappPayload = new DefaultDappEntrypoint(aliceAddress, aliceWallet, subscriptionContract.address);
    const action = counterContract.methods.increment(bobAddress).request();
    const txExReq = await dappPayload.createTxExecutionRequest([action]);
    const tx = await pxe.simulateTx(txExReq, true);
    const sentTx = new SentTx(pxe, pxe.sendTx(tx));
    await sentTx.wait();

    expect(await counterContract.methods.get_counter(bobAddress).view()).toBe(1n);

    await assertPublicBalances(
      gasBalances,
      [subscriptionContract.address, sequencerAddress],
      [BRIDGED_GAS_BALANCE - FEE_AMOUNT, FEE_AMOUNT * 3n],
    );
  });

  it('should reject after the sub runs out', async () => {
    // subscribe again. This will overwrite the subscription
    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet), 0);
    await expect(dappIncrement()).rejects.toThrow(
      "Failed to solve brillig function, reason: explicit trap hit in brillig '(context.block_number()) as u64 < expiry_block_number as u64'",
    );
  });

  it('should reject after the txs run out', async () => {
    // subscribe again. This will overwrite the subscription
    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet), 5, 1);
    await expect(dappIncrement()).resolves.toBeDefined();
    await expect(dappIncrement()).rejects.toThrow(/note.remaining_txs as u64 > 0/);
  });

  async function subscribe(paymentMethod: FeePaymentMethod, blockDelta: number = 5, txCount: number = 4) {
    {
      const nonce = Fr.random();
      const action = bananaCoin.methods.transfer(aliceAddress, bobAddress, SUBSCRIPTION_AMOUNT, nonce);
      const messageHash = computeAuthWitMessageHash(subscriptionContract.address, action.request());
      await aliceWallet.createAuthWitness(messageHash);

      return subscriptionContract
        .withWallet(aliceWallet)
        .methods.subscribe(aliceAddress, nonce, (await e2eContext.pxe.getBlockNumber()) + blockDelta, txCount)
        .send({
          fee: {
            maxFee: 1n,
            paymentMethod,
          },
        })
        .wait();
    }
  }

  async function dappIncrement() {
    const { pxe } = e2eContext;
    const dappEntrypoint = new DefaultDappEntrypoint(aliceAddress, aliceWallet, subscriptionContract.address);
    const action = counterContract.methods.increment(bobAddress).request();
    const txExReq = await dappEntrypoint.createTxExecutionRequest([action]);
    const tx = await pxe.simulateTx(txExReq, true);
    const sentTx = new SentTx(pxe, pxe.sendTx(tx));
    return sentTx.wait();
  }
});
