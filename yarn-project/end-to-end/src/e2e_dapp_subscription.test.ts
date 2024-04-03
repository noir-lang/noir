import {
  type AccountWalletWithPrivateKey,
  type AztecAddress,
  type FeePaymentMethod,
  Fr,
  PrivateFeePaymentMethod,
  PublicFeePaymentMethod,
  SentTx,
  getContractClassFromArtifact,
} from '@aztec/aztec.js';
import { DefaultDappEntrypoint } from '@aztec/entrypoints/dapp';
import {
  AppSubscriptionContract,
  TokenContract as BananaCoin,
  CounterContract,
  FPCContract,
  GasTokenContract,
} from '@aztec/noir-contracts.js';
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { jest } from '@jest/globals';

import {
  type BalancesFn,
  type EndToEndContext,
  expectMapping,
  getBalancesFn,
  publicDeployAccounts,
  setup,
} from './fixtures/utils.js';

jest.setTimeout(100_000);

const TOKEN_NAME = 'BananaCoin';
const TOKEN_SYMBOL = 'BC';
const TOKEN_DECIMALS = 18n;

describe('e2e_dapp_subscription', () => {
  let aliceWallet: AccountWalletWithPrivateKey;
  let bobWallet: AccountWalletWithPrivateKey;
  let aliceAddress: AztecAddress; // Dapp subscriber.
  let bobAddress: AztecAddress; // Dapp owner.
  let sequencerAddress: AztecAddress;
  // let gasTokenContract: GasTokenContract;
  let bananaCoin: BananaCoin;
  let counterContract: CounterContract;
  let subscriptionContract: AppSubscriptionContract;
  let gasTokenContract: GasTokenContract;
  let bananaFPC: FPCContract;
  let e2eContext: EndToEndContext;
  let gasBalances: BalancesFn;
  let bananasPublicBalances: BalancesFn;
  let bananasPrivateBalances: BalancesFn;

  const SUBSCRIPTION_AMOUNT = 100n;
  const INITIAL_GAS_BALANCE = 1000n;
  const PUBLICLY_MINTED_BANANAS = 500n;
  const PRIVATELY_MINTED_BANANAS = 600n;

  const FEE_AMOUNT = 1n;
  const REFUND = 2n; // intentionally overpay the gas fee. This is the expected refund.
  const MAX_FEE = FEE_AMOUNT + REFUND;

  beforeAll(async () => {
    process.env.PXE_URL = '';
    e2eContext = await setup(3);
    await publicDeployAccounts(e2eContext.wallet, e2eContext.accounts);

    const { wallets, accounts, aztecNode, deployL1ContractsValues } = e2eContext;

    await aztecNode.setConfig({
      allowedFeePaymentContractClasses: [getContractClassFromArtifact(FPCContract.artifact).id],
    });

    // this should be a SignerlessWallet but that can't call public functions directly
    gasTokenContract = await GasTokenContract.at(
      getCanonicalGasTokenAddress(deployL1ContractsValues.l1ContractAddresses.gasPortalAddress),
      wallets[0],
    );

    aliceAddress = accounts.at(0)!.address;
    bobAddress = accounts.at(1)!.address;
    sequencerAddress = accounts.at(2)!.address;

    await aztecNode.setConfig({
      feeRecipient: sequencerAddress,
    });

    [aliceWallet, bobWallet] = wallets;

    bananaCoin = await BananaCoin.deploy(aliceWallet, aliceAddress, TOKEN_NAME, TOKEN_SYMBOL, TOKEN_DECIMALS)
      .send()
      .deployed();
    bananaFPC = await FPCContract.deploy(aliceWallet, bananaCoin.address, gasTokenContract.address).send().deployed();

    counterContract = await CounterContract.deploy(bobWallet, 0, bobAddress).send().deployed();

    subscriptionContract = await AppSubscriptionContract.deploy(
      bobWallet,
      counterContract.address,
      bobAddress,
      // anyone can purchase a subscription for 100 test tokens
      bananaCoin.address,
      SUBSCRIPTION_AMOUNT,
      gasTokenContract.address,
    )
      .send()
      .deployed();

    // mint some test tokens for Alice
    // she'll pay for the subscription with these
    await bananaCoin.methods.privately_mint_private_note(PRIVATELY_MINTED_BANANAS).send().wait();
    await bananaCoin.methods.mint_public(aliceAddress, PUBLICLY_MINTED_BANANAS).send().wait();
    await gasTokenContract.methods.mint_public(subscriptionContract.address, INITIAL_GAS_BALANCE).send().wait();
    await gasTokenContract.methods.mint_public(bananaFPC.address, INITIAL_GAS_BALANCE).send().wait();

    gasBalances = getBalancesFn('⛽', gasTokenContract.methods.balance_of_public, e2eContext.logger);
    bananasPublicBalances = getBalancesFn('Public 🍌', bananaCoin.methods.balance_of_public, e2eContext.logger);
    bananasPrivateBalances = getBalancesFn('Private 🍌', bananaCoin.methods.balance_of_private, e2eContext.logger);

    await expectMapping(
      gasBalances,
      [aliceAddress, sequencerAddress, subscriptionContract.address, bananaFPC.address],
      [0n, 0n, INITIAL_GAS_BALANCE, INITIAL_GAS_BALANCE],
    );
  });

  it('should allow Alice to subscribe by paying privately with bananas', async () => {
    /**
    PRIVATE SETUP
    we first unshield `MAX_FEE` BC from alice's private balance to the FPC's public balance

    PUBLIC APP LOGIC
    we then privately transfer `SUBSCRIPTION_AMOUNT` BC from alice to bob's subscription contract

    PUBLIC TEARDOWN
    then the FPC calls `pay_fee`, reducing its gas balance by `FEE_AMOUNT`, and increasing the sequencer's gas balance by `FEE_AMOUNT`
    the FPC also publicly sends `REFUND` BC to alice
    */

    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet), MAX_FEE);

    await expectMapping(
      bananasPrivateBalances,
      [aliceAddress, bobAddress, bananaFPC.address],
      [PRIVATELY_MINTED_BANANAS - SUBSCRIPTION_AMOUNT - MAX_FEE, SUBSCRIPTION_AMOUNT, 0n],
    );

    await expectMapping(
      bananasPublicBalances,
      [aliceAddress, bobAddress, bananaFPC.address],
      // refund is done via a transparent note for now
      [PUBLICLY_MINTED_BANANAS, 0n, FEE_AMOUNT],
    );

    await expectMapping(
      gasBalances,
      // note the subscription contract hasn't paid any fees yet
      [bananaFPC.address, subscriptionContract.address, sequencerAddress],
      [INITIAL_GAS_BALANCE - FEE_AMOUNT, INITIAL_GAS_BALANCE, FEE_AMOUNT],
    );

    // REFUND_AMOUNT is a transparent note note
  });

  it('should allow Alice to subscribe by paying with bananas in public', async () => {
    /**
    PRIVATE SETUP
    we publicly transfer `MAX_FEE` BC from alice's public balance to the FPC's public balance

    PUBLIC APP LOGIC
    we then privately transfer `SUBSCRIPTION_AMOUNT` BC from alice to bob's subscription contract

    PUBLIC TEARDOWN
    then the FPC calls `pay_fee`, reducing its gas balance by `FEE_AMOUNT`, and increasing the sequencer's gas balance by `FEE_AMOUNT`
    the FPC also publicly sends `REFUND` BC to alice
    */
    await subscribe(new PublicFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet), MAX_FEE);

    await expectMapping(
      bananasPrivateBalances,
      [aliceAddress, bobAddress, bananaFPC.address],
      // we pay the fee publicly, but the subscription payment is still private.
      // Also, minus 1 x MAX_FEE as leftover from the previous test, since we paid publicly this time
      [PRIVATELY_MINTED_BANANAS - 2n * SUBSCRIPTION_AMOUNT - MAX_FEE, 2n * SUBSCRIPTION_AMOUNT, 0n],
    );

    await expectMapping(
      bananasPublicBalances,
      [aliceAddress, bobAddress, bananaFPC.address],
      [
        // we have the refund from the previous test,
        // but since we paid publicly this time, the refund should have been "squashed"
        PUBLICLY_MINTED_BANANAS - FEE_AMOUNT,
        0n, // Bob still has no public bananas
        2n * FEE_AMOUNT, // because this is the second time we've used the FPC
      ],
    );

    await expectMapping(
      gasBalances,
      [subscriptionContract.address, bananaFPC.address, sequencerAddress],
      [INITIAL_GAS_BALANCE, INITIAL_GAS_BALANCE - 2n * FEE_AMOUNT, 2n * FEE_AMOUNT],
    );
  });

  it('should call dapp subscription entrypoint', async () => {
    const { pxe } = e2eContext;
    const dappPayload = new DefaultDappEntrypoint(aliceAddress, aliceWallet, subscriptionContract.address);
    const action = counterContract.methods.increment(bobAddress).request();
    const txExReq = await dappPayload.createTxExecutionRequest([action]);
    const tx = await pxe.proveTx(txExReq, true);
    const sentTx = new SentTx(pxe, pxe.sendTx(tx));
    await sentTx.wait();

    expect(await counterContract.methods.get_counter(bobAddress).simulate()).toBe(1n);

    await expectMapping(
      gasBalances,
      [subscriptionContract.address, bananaFPC.address, sequencerAddress],
      [INITIAL_GAS_BALANCE - FEE_AMOUNT, INITIAL_GAS_BALANCE - 2n * FEE_AMOUNT, FEE_AMOUNT * 3n],
    );
  });

  it('should reject after the sub runs out', async () => {
    // subscribe again. This will overwrite the subscription
    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet), MAX_FEE, 0);
    await expect(dappIncrement()).rejects.toThrow(
      "Failed to solve brillig function, reason: explicit trap hit in brillig '(context.block_number()) as u64 < expiry_block_number as u64'",
    );
  });

  it('should reject after the txs run out', async () => {
    // subscribe again. This will overwrite the subscription
    await subscribe(new PrivateFeePaymentMethod(bananaCoin.address, bananaFPC.address, aliceWallet), FEE_AMOUNT, 5, 1);
    await expect(dappIncrement()).resolves.toBeDefined();
    await expect(dappIncrement()).rejects.toThrow(/note.remaining_txs as u64 > 0/);
  });

  async function subscribe(
    paymentMethod: FeePaymentMethod,
    maxFee: bigint,
    blockDelta: number = 5,
    txCount: number = 4,
  ) {
    const nonce = Fr.random();
    const action = bananaCoin.methods.transfer(aliceAddress, bobAddress, SUBSCRIPTION_AMOUNT, nonce);
    await aliceWallet.createAuthWit({ caller: subscriptionContract.address, action });

    return subscriptionContract
      .withWallet(aliceWallet)
      .methods.subscribe(aliceAddress, nonce, (await e2eContext.pxe.getBlockNumber()) + blockDelta, txCount)
      .send({
        fee: {
          maxFee,
          paymentMethod,
        },
      })
      .wait();
  }

  async function dappIncrement() {
    const { pxe } = e2eContext;
    const dappEntrypoint = new DefaultDappEntrypoint(aliceAddress, aliceWallet, subscriptionContract.address);
    const action = counterContract.methods.increment(bobAddress).request();
    const txExReq = await dappEntrypoint.createTxExecutionRequest([action]);
    const tx = await pxe.proveTx(txExReq, true);
    const sentTx = new SentTx(pxe, pxe.sendTx(tx));
    return sentTx.wait();
  }
});
