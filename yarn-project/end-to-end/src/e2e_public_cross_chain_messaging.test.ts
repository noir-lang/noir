import {
  AccountWallet,
  AztecAddress,
  DebugLogger,
  EthAddress,
  Fr,
  TxStatus,
  computeAuthWitMessageHash,
  sleep,
} from '@aztec/aztec.js';
import { TokenContract } from '@aztec/noir-contracts/Token';
import { TokenBridgeContract } from '@aztec/noir-contracts/TokenBridge';

import { setup } from './fixtures/utils.js';
import { CrossChainTestHarness } from './shared/cross_chain_test_harness.js';

describe('e2e_public_cross_chain_messaging', () => {
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let user1Wallet: AccountWallet;
  let user2Wallet: AccountWallet;
  let ethAccount: EthAddress;
  let ownerAddress: AztecAddress;

  let crossChainTestHarness: CrossChainTestHarness;
  let l2Token: TokenContract;
  let l2Bridge: TokenBridgeContract;
  let outbox: any;

  beforeEach(async () => {
    const { pxe, deployL1ContractsValues, wallets, logger: logger_, teardown: teardown_ } = await setup(2);
    crossChainTestHarness = await CrossChainTestHarness.new(
      pxe,
      deployL1ContractsValues.publicClient,
      deployL1ContractsValues.walletClient,
      wallets[0],
      logger_,
    );
    l2Token = crossChainTestHarness.l2Token;
    l2Bridge = crossChainTestHarness.l2Bridge;
    ethAccount = crossChainTestHarness.ethAccount;
    ownerAddress = crossChainTestHarness.ownerAddress;
    outbox = crossChainTestHarness.outbox;
    teardown = teardown_;
    user1Wallet = wallets[0];
    user2Wallet = wallets[1];

    logger = logger_;
    logger('Successfully deployed contracts and initialized portal');
  }, 100_000);

  afterEach(async () => {
    await teardown();
  });

  // docs:start:e2e_public_cross_chain
  it('Publicly deposit funds from L1 -> L2 and withdraw back to L1', async () => {
    // Generate a claim secret using pedersen
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;

    const [secret, secretHash] = crossChainTestHarness.generateClaimSecret();

    // 1. Mint tokens on L1
    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);

    // 2. Deposit tokens to the TokenPortal
    const messageKey = await crossChainTestHarness.sendTokensToPortalPublic(bridgeAmount, secretHash);
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    // Wait for the archiver to process the message
    await sleep(5000); /// waiting 5 seconds.

    // Perform an unrelated transaction on L2 to progress the rollup. Here we mint public tokens.
    const unrelatedMintAmount = 99n;
    await crossChainTestHarness.mintTokensPublicOnL2(unrelatedMintAmount);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, unrelatedMintAmount);
    const balanceBefore = unrelatedMintAmount;

    // 3. Consume L1-> L2 message and mint public tokens on L2
    await crossChainTestHarness.consumeMessageOnAztecAndMintPublicly(bridgeAmount, messageKey, secret);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, balanceBefore + bridgeAmount);
    const afterBalance = balanceBefore + bridgeAmount;

    // time to withdraw the funds again!
    logger('Withdrawing funds from L2');

    // 4. Give approval to bridge to burn owner's funds:
    const withdrawAmount = 9n;
    const nonce = Fr.random();
    const burnMessageHash = computeAuthWitMessageHash(
      l2Bridge.address,
      l2Token.methods.burn_public(ownerAddress, withdrawAmount, nonce).request(),
    );
    await user1Wallet.setPublicAuth(burnMessageHash, true).send().wait();

    // 5. Withdraw owner's funds from L2 to L1
    const entryKey = await crossChainTestHarness.checkEntryIsNotInOutbox(withdrawAmount);
    await crossChainTestHarness.withdrawPublicFromAztecToL1(withdrawAmount, nonce);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, afterBalance - withdrawAmount);

    // Check balance before and after exit.
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);
    await crossChainTestHarness.withdrawFundsFromBridgeOnL1(withdrawAmount, entryKey);
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount + withdrawAmount);

    expect(await outbox.read.contains([entryKey.toString()])).toBeFalsy();
  }, 120_000);
  // docs:end:e2e_public_cross_chain

  // Unit tests for TokenBridge's public methods.

  it('Someone else can mint funds to me on my behalf (publicly)', async () => {
    // Generate a claim secret using pedersen
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;

    const [secret, secretHash] = crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);
    const messageKey = await crossChainTestHarness.sendTokensToPortalPublic(bridgeAmount, secretHash);
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    // Wait for the archiver to process the message
    await sleep(5000); /// waiting 5 seconds.

    // Perform an unrelated transaction on L2 to progress the rollup. Here we mint public tokens.
    const unrelatedMintAmount = 99n;
    await crossChainTestHarness.mintTokensPublicOnL2(unrelatedMintAmount);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, unrelatedMintAmount);

    // user2 tries to consume this message and minting to itself -> should fail since the message is intended to be consumed only by owner.
    await expect(
      l2Bridge
        .withWallet(user2Wallet)
        .methods.claim_public(user2Wallet.getAddress(), bridgeAmount, ethAccount, messageKey, secret)
        .simulate(),
    ).rejects.toThrow("Invalid Content 'l1_to_l2_message_data.message.content == content'");

    // user2 consumes owner's L1-> L2 message on bridge contract and mints public tokens on L2
    logger("user2 consumes owner's message on L2 Publicly");
    const tx = l2Bridge
      .withWallet(user2Wallet)
      .methods.claim_public(ownerAddress, bridgeAmount, ethAccount, messageKey, secret)
      .send();
    const receipt = await tx.wait();
    expect(receipt.status).toBe(TxStatus.MINED);
    // ensure funds are gone to owner and not user2.
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount + unrelatedMintAmount);
    await crossChainTestHarness.expectPublicBalanceOnL2(user2Wallet.getAddress(), 0n);
  }, 90_000);

  it("Bridge can't withdraw my funds if I don't give approval", async () => {
    const mintAmountToOwner = 100n;
    await crossChainTestHarness.mintTokensPublicOnL2(mintAmountToOwner);

    const withdrawAmount = 9n;
    const nonce = Fr.random();
    // Should fail as owner has not given approval to bridge burn their funds.
    await expect(
      l2Bridge
        .withWallet(user1Wallet)
        .methods.exit_to_l1_public(ethAccount, withdrawAmount, EthAddress.ZERO, nonce)
        .simulate(),
    ).rejects.toThrowError('Assertion failed: Message not authorized by account');
  }, 60_000);

  it("can't claim funds privately which were intended for public deposit from the token portal", async () => {
    const bridgeAmount = 100n;
    const [secret, secretHash] = crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(bridgeAmount);
    const messageKey = await crossChainTestHarness.sendTokensToPortalPublic(bridgeAmount, secretHash);
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(0n);

    // Wait for the archiver to process the message
    await sleep(5000); /// waiting 5 seconds.

    // Perform an unrelated transaction on L2 to progress the rollup. Here we mint public tokens.
    await crossChainTestHarness.mintTokensPublicOnL2(0n);

    await expect(
      l2Bridge
        .withWallet(user2Wallet)
        .methods.claim_private(secretHash, bridgeAmount, ethAccount, messageKey, secret)
        .simulate(),
    ).rejects.toThrowError("Invalid Content 'l1_to_l2_message_data.message.content == content'");
  }, 60_000);
});
