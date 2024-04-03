import {
  type AccountWallet,
  type AztecAddress,
  type AztecNode,
  type DebugLogger,
  EthAddress,
  Fr,
  L1Actor,
  L1ToL2Message,
  L2Actor,
  computeAuthWitMessageHash,
} from '@aztec/aztec.js';
import { sha256ToField } from '@aztec/foundation/crypto';
import { type TokenBridgeContract, type TokenContract } from '@aztec/noir-contracts.js';

import { toFunctionSelector } from 'viem/utils';

import { setup } from './fixtures/utils.js';
import { CrossChainTestHarness } from './shared/cross_chain_test_harness.js';

describe('e2e_cross_chain_messaging', () => {
  let aztecNode: AztecNode;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;

  let user1Wallet: AccountWallet;
  let user2Wallet: AccountWallet;
  let ethAccount: EthAddress;
  let ownerAddress: AztecAddress;

  let crossChainTestHarness: CrossChainTestHarness;
  let l2Token: TokenContract;
  let l2Bridge: TokenBridgeContract;

  beforeEach(async () => {
    const {
      aztecNode: aztecNode_,
      pxe,
      deployL1ContractsValues,
      wallets,
      logger: logger_,
      teardown: teardown_,
    } = await setup(2);

    crossChainTestHarness = await CrossChainTestHarness.new(
      aztecNode_,
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
    user1Wallet = wallets[0];
    user2Wallet = wallets[1];
    logger = logger_;
    aztecNode = aztecNode_;
    teardown = teardown_;
    logger('Successfully deployed contracts and initialized portal');
  }, 100_000);

  afterEach(async () => {
    await teardown();
  });
  // docs:start:e2e_private_cross_chain
  it('Privately deposit funds from L1 -> L2 and withdraw back to L1', async () => {
    // Generate a claim secret using pedersen
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;

    const [secretForL2MessageConsumption, secretHashForL2MessageConsumption] =
      crossChainTestHarness.generateClaimSecret();
    const [secretForRedeemingMintedNotes, secretHashForRedeemingMintedNotes] =
      crossChainTestHarness.generateClaimSecret();

    // 1. Mint tokens on L1
    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);

    // 2. Deposit tokens to the TokenPortal
    const msgHash = await crossChainTestHarness.sendTokensToPortalPrivate(
      secretHashForRedeemingMintedNotes,
      bridgeAmount,
      secretHashForL2MessageConsumption,
    );
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    await crossChainTestHarness.makeMessageConsumable(msgHash);

    // 3. Consume L1 -> L2 message and mint private tokens on L2
    await crossChainTestHarness.consumeMessageOnAztecAndMintPrivately(
      secretHashForRedeemingMintedNotes,
      bridgeAmount,
      secretForL2MessageConsumption,
    );
    // tokens were minted privately in a TransparentNote which the owner (person who knows the secret) must redeem:
    await crossChainTestHarness.redeemShieldPrivatelyOnL2(bridgeAmount, secretForRedeemingMintedNotes);
    await crossChainTestHarness.expectPrivateBalanceOnL2(ownerAddress, bridgeAmount);

    // time to withdraw the funds again!
    logger('Withdrawing funds from L2');

    // docs:start:authwit_to_another_sc
    // 4. Give approval to bridge to burn owner's funds:
    const withdrawAmount = 9n;
    const nonce = Fr.random();
    await user1Wallet.createAuthWit({
      caller: l2Bridge.address,
      action: l2Token.methods.burn(ownerAddress, withdrawAmount, nonce),
    });
    // docs:end:authwit_to_another_sc

    // 5. Withdraw owner's funds from L2 to L1
    const l2ToL1Message = crossChainTestHarness.getL2ToL1MessageLeaf(withdrawAmount);
    const l2TxReceipt = await crossChainTestHarness.withdrawPrivateFromAztecToL1(withdrawAmount, nonce);
    await crossChainTestHarness.expectPrivateBalanceOnL2(ownerAddress, bridgeAmount - withdrawAmount);

    const [l2ToL1MessageIndex, siblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
      l2TxReceipt.blockNumber!,
      l2ToL1Message,
    );

    // Check balance before and after exit.
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);
    await crossChainTestHarness.withdrawFundsFromBridgeOnL1(
      withdrawAmount,
      l2TxReceipt.blockNumber!,
      l2ToL1MessageIndex,
      siblingPath,
    );
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount + withdrawAmount);
  }, 120_000);
  // docs:end:e2e_private_cross_chain

  // Unit tests for TokenBridge's private methods.
  it('Someone else can mint funds to me on my behalf (privately)', async () => {
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;
    const [secretForL2MessageConsumption, secretHashForL2MessageConsumption] =
      crossChainTestHarness.generateClaimSecret();
    const [secretForRedeemingMintedNotes, secretHashForRedeemingMintedNotes] =
      crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);
    const msgHash = await crossChainTestHarness.sendTokensToPortalPrivate(
      secretHashForRedeemingMintedNotes,
      bridgeAmount,
      secretHashForL2MessageConsumption,
    );
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    // Wait for the message to be available for consumption
    await crossChainTestHarness.makeMessageConsumable(msgHash);

    // 3. Consume L1 -> L2 message and mint private tokens on L2
    const content = sha256ToField([
      Buffer.from(toFunctionSelector('mint_private(bytes32,uint256)').substring(2), 'hex'),
      secretHashForL2MessageConsumption,
      new Fr(bridgeAmount),
    ]);
    const wrongMessage = new L1ToL2Message(
      new L1Actor(crossChainTestHarness.tokenPortalAddress, crossChainTestHarness.publicClient.chain.id),
      new L2Actor(l2Bridge.address, 1),
      content,
      secretHashForL2MessageConsumption,
    );

    // Sending wrong secret hashes should fail:
    await expect(
      l2Bridge
        .withWallet(user2Wallet)
        .methods.claim_private(secretHashForL2MessageConsumption, bridgeAmount, secretForL2MessageConsumption)
        .prove(),
    ).rejects.toThrow(`No non-nullified L1 to L2 message found for message hash ${wrongMessage.hash().toString()}`);

    // send the right one -
    const consumptionReceipt = await l2Bridge
      .withWallet(user2Wallet)
      .methods.claim_private(secretHashForRedeemingMintedNotes, bridgeAmount, secretForL2MessageConsumption)
      .send()
      .wait();

    // Now user1 can claim the notes that user2 minted on their behalf.
    await crossChainTestHarness.addPendingShieldNoteToPXE(
      bridgeAmount,
      secretHashForRedeemingMintedNotes,
      consumptionReceipt.txHash,
    );
    await crossChainTestHarness.redeemShieldPrivatelyOnL2(bridgeAmount, secretForRedeemingMintedNotes);
    await crossChainTestHarness.expectPrivateBalanceOnL2(ownerAddress, bridgeAmount);
  }, 120_000);

  it("Bridge can't withdraw my funds if I don't give approval", async () => {
    const mintAmountToUser1 = 100n;
    await crossChainTestHarness.mintTokensPublicOnL2(mintAmountToUser1);

    const withdrawAmount = 9n;
    const nonce = Fr.random();
    const expectedBurnMessageHash = computeAuthWitMessageHash(
      l2Bridge.address,
      user1Wallet.getChainId(),
      user1Wallet.getVersion(),
      l2Token.methods.burn(user1Wallet.getAddress(), withdrawAmount, nonce).request(),
    );
    // Should fail as owner has not given approval to bridge burn their funds.
    await expect(
      l2Bridge
        .withWallet(user1Wallet)
        .methods.exit_to_l1_private(l2Token.address, ethAccount, withdrawAmount, EthAddress.ZERO, nonce)
        .prove(),
    ).rejects.toThrow(`Unknown auth witness for message hash ${expectedBurnMessageHash.toString()}`);
  }, 120_000);

  it("Can't claim funds publicly if they were deposited privately", async () => {
    // 1. Mint tokens on L1
    const bridgeAmount = 100n;
    await crossChainTestHarness.mintTokensOnL1(bridgeAmount);

    // 2. Deposit tokens to the TokenPortal privately
    const [secretForL2MessageConsumption, secretHashForL2MessageConsumption] =
      crossChainTestHarness.generateClaimSecret();

    const msgHash = await crossChainTestHarness.sendTokensToPortalPrivate(
      Fr.random(),
      bridgeAmount,
      secretHashForL2MessageConsumption,
    );
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(0n);

    // Wait for the message to be available for consumption
    await crossChainTestHarness.makeMessageConsumable(msgHash);

    const content = sha256ToField([
      Buffer.from(toFunctionSelector('mint_public(bytes32,uint256)').substring(2), 'hex'),
      ownerAddress,
      new Fr(bridgeAmount),
    ]);
    const wrongMessage = new L1ToL2Message(
      new L1Actor(crossChainTestHarness.tokenPortalAddress, crossChainTestHarness.publicClient.chain.id),
      new L2Actor(l2Bridge.address, 1),
      content,
      secretHashForL2MessageConsumption,
    );

    // 3. Consume L1 -> L2 message and try to mint publicly on L2  - should fail
    await expect(
      l2Bridge
        .withWallet(user2Wallet)
        .methods.claim_public(ownerAddress, bridgeAmount, secretForL2MessageConsumption)
        .prove(),
    ).rejects.toThrow(`No non-nullified L1 to L2 message found for message hash ${wrongMessage.hash().toString()}`);
  }, 120_000);
});
