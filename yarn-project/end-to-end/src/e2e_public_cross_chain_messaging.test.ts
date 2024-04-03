import {
  type AccountWallet,
  type AztecAddress,
  type AztecNode,
  type CompleteAddress,
  type DebugLogger,
  type DeployL1Contracts,
  EthAddress,
  Fr,
  L1Actor,
  L1ToL2Message,
  L2Actor,
  type PXE,
  computeAuthWitMessageHash,
  computeMessageSecretHash,
} from '@aztec/aztec.js';
import { sha256ToField } from '@aztec/foundation/crypto';
import { InboxAbi, OutboxAbi } from '@aztec/l1-artifacts';
import { TestContract } from '@aztec/noir-contracts.js';
import { type TokenContract } from '@aztec/noir-contracts.js/Token';
import { type TokenBridgeContract } from '@aztec/noir-contracts.js/TokenBridge';

import { type Chain, type GetContractReturnType, type Hex, type HttpTransport, type PublicClient } from 'viem';
import { decodeEventLog, toFunctionSelector } from 'viem/utils';

import { publicDeployAccounts, setup } from './fixtures/utils.js';
import { CrossChainTestHarness } from './shared/cross_chain_test_harness.js';

describe('e2e_public_cross_chain_messaging', () => {
  let aztecNode: AztecNode;
  let pxe: PXE;
  let deployL1ContractsValues: DeployL1Contracts;
  let logger: DebugLogger;
  let teardown: () => Promise<void>;
  let wallets: AccountWallet[];
  let accounts: CompleteAddress[];

  let user1Wallet: AccountWallet;
  let user2Wallet: AccountWallet;
  let ethAccount: EthAddress;
  let ownerAddress: AztecAddress;

  let crossChainTestHarness: CrossChainTestHarness;
  let l2Token: TokenContract;
  let l2Bridge: TokenBridgeContract;
  let inbox: GetContractReturnType<typeof InboxAbi, PublicClient<HttpTransport, Chain>>;
  let outbox: GetContractReturnType<typeof OutboxAbi, PublicClient<HttpTransport, Chain>>;

  beforeAll(async () => {
    ({ aztecNode, pxe, deployL1ContractsValues, wallets, accounts, logger, teardown } = await setup(2));
    user1Wallet = wallets[0];
    user2Wallet = wallets[1];
    await publicDeployAccounts(wallets[0], accounts.slice(0, 2));
  }, 30_000);

  beforeEach(async () => {
    crossChainTestHarness = await CrossChainTestHarness.new(
      aztecNode,
      pxe,
      deployL1ContractsValues.publicClient,
      deployL1ContractsValues.walletClient,
      wallets[0],
      logger,
    );
    l2Token = crossChainTestHarness.l2Token;
    l2Bridge = crossChainTestHarness.l2Bridge;
    ethAccount = crossChainTestHarness.ethAccount;
    ownerAddress = crossChainTestHarness.ownerAddress;
    inbox = crossChainTestHarness.inbox;
    outbox = crossChainTestHarness.outbox;

    logger('Successfully deployed contracts and initialized portal');
  }, 100_000);

  afterAll(async () => {
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
    const msgHash = await crossChainTestHarness.sendTokensToPortalPublic(bridgeAmount, secretHash);
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    // Wait for the message to be available for consumption
    await crossChainTestHarness.makeMessageConsumable(msgHash);

    // 3. Consume L1 -> L2 message and mint public tokens on L2
    await crossChainTestHarness.consumeMessageOnAztecAndMintPublicly(bridgeAmount, secret);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount);
    const afterBalance = bridgeAmount;

    // time to withdraw the funds again!
    logger('Withdrawing funds from L2');

    // 4. Give approval to bridge to burn owner's funds:
    const withdrawAmount = 9n;
    const nonce = Fr.random();
    const burnMessageHash = computeAuthWitMessageHash(
      l2Bridge.address,
      wallets[0].getChainId(),
      wallets[0].getVersion(),
      l2Token.methods.burn_public(ownerAddress, withdrawAmount, nonce).request(),
    );
    await user1Wallet.setPublicAuthWit(burnMessageHash, true).send().wait();

    // 5. Withdraw owner's funds from L2 to L1
    const l2ToL1Message = crossChainTestHarness.getL2ToL1MessageLeaf(withdrawAmount);
    const l2TxReceipt = await crossChainTestHarness.withdrawPublicFromAztecToL1(withdrawAmount, nonce);
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, afterBalance - withdrawAmount);

    // Check balance before and after exit.
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    const [l2ToL1MessageIndex, siblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
      l2TxReceipt.blockNumber!,
      l2ToL1Message,
    );

    await crossChainTestHarness.withdrawFundsFromBridgeOnL1(
      withdrawAmount,
      l2TxReceipt.blockNumber!,
      l2ToL1MessageIndex,
      siblingPath,
    );
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount + withdrawAmount);
  }, 120_000);
  // docs:end:e2e_public_cross_chain

  // Unit tests for TokenBridge's public methods.

  it('Someone else can mint funds to me on my behalf (publicly)', async () => {
    // Generate a claim secret using pedersen
    const l1TokenBalance = 1000000n;
    const bridgeAmount = 100n;

    const [secret, secretHash] = crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(l1TokenBalance);
    const msgHash = await crossChainTestHarness.sendTokensToPortalPublic(bridgeAmount, secretHash);
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(l1TokenBalance - bridgeAmount);

    await crossChainTestHarness.makeMessageConsumable(msgHash);

    const content = sha256ToField([
      Buffer.from(toFunctionSelector('mint_public(bytes32,uint256)').substring(2), 'hex'),
      user2Wallet.getAddress(),
      new Fr(bridgeAmount),
    ]);
    const wrongMessage = new L1ToL2Message(
      new L1Actor(crossChainTestHarness.tokenPortalAddress, crossChainTestHarness.publicClient.chain.id),
      new L2Actor(l2Bridge.address, 1),
      content,
      secretHash,
    );

    // user2 tries to consume this message and minting to itself -> should fail since the message is intended to be consumed only by owner.
    await expect(
      l2Bridge.withWallet(user2Wallet).methods.claim_public(user2Wallet.getAddress(), bridgeAmount, secret).prove(),
    ).rejects.toThrow(`No non-nullified L1 to L2 message found for message hash ${wrongMessage.hash().toString()}`);

    // user2 consumes owner's L1-> L2 message on bridge contract and mints public tokens on L2
    logger("user2 consumes owner's message on L2 Publicly");
    await l2Bridge.withWallet(user2Wallet).methods.claim_public(ownerAddress, bridgeAmount, secret).send().wait();
    // ensure funds are gone to owner and not user2.
    await crossChainTestHarness.expectPublicBalanceOnL2(ownerAddress, bridgeAmount);
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
        .prove(),
    ).rejects.toThrow('Assertion failed: Message not authorized by account');
  }, 60_000);

  it("can't claim funds privately which were intended for public deposit from the token portal", async () => {
    const bridgeAmount = 100n;
    const [secret, secretHash] = crossChainTestHarness.generateClaimSecret();

    await crossChainTestHarness.mintTokensOnL1(bridgeAmount);
    const msgHash = await crossChainTestHarness.sendTokensToPortalPublic(bridgeAmount, secretHash);
    expect(await crossChainTestHarness.getL1BalanceOf(ethAccount)).toBe(0n);

    await crossChainTestHarness.makeMessageConsumable(msgHash);

    // Wrong message hash
    const content = sha256ToField([
      Buffer.from(toFunctionSelector('mint_private(bytes32,uint256)').substring(2), 'hex'),
      secretHash,
      new Fr(bridgeAmount),
    ]);
    const wrongMessage = new L1ToL2Message(
      new L1Actor(crossChainTestHarness.tokenPortalAddress, crossChainTestHarness.publicClient.chain.id),
      new L2Actor(l2Bridge.address, 1),
      content,
      secretHash,
    );

    await expect(
      l2Bridge.withWallet(user2Wallet).methods.claim_private(secretHash, bridgeAmount, secret).prove(),
    ).rejects.toThrow(`No non-nullified L1 to L2 message found for message hash ${wrongMessage.hash().toString()}`);
  }, 60_000);

  // Note: We register one portal address when deploying contract but that address is no-longer the only address
  // allowed to receive messages from the given contract. In the following test we'll test that it's really the case.
  it.each([true, false])(
    'can send an L2 -> L1 message to a non-registered portal address from private or public',
    async (isPrivate: boolean) => {
      const testContract = await TestContract.deploy(user1Wallet).send().deployed();

      const content = Fr.random();
      const recipient = crossChainTestHarness.ethAccount;

      let l2TxReceipt;

      // We create the L2 -> L1 message using the test contract
      if (isPrivate) {
        l2TxReceipt = await testContract.methods
          .create_l2_to_l1_message_arbitrary_recipient_private(content, recipient)
          .send()
          .wait();
      } else {
        l2TxReceipt = await testContract.methods
          .create_l2_to_l1_message_arbitrary_recipient_public(content, recipient)
          .send()
          .wait();
      }

      const l2ToL1Message = {
        sender: { actor: testContract.address.toString() as Hex, version: 1n },
        recipient: {
          actor: recipient.toString() as Hex,
          chainId: BigInt(crossChainTestHarness.publicClient.chain.id),
        },
        content: content.toString() as Hex,
      };

      const leaf = sha256ToField([
        testContract.address,
        new Fr(1), // aztec version
        recipient.toBuffer32(),
        new Fr(crossChainTestHarness.publicClient.chain.id), // chain id
        content,
      ]);

      const [l2MessageIndex, siblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
        l2TxReceipt.blockNumber!,
        leaf,
      );

      const txHash = await outbox.write.consume(
        [
          l2ToL1Message,
          BigInt(l2TxReceipt.blockNumber!),
          BigInt(l2MessageIndex),
          siblingPath.toBufferArray().map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
        ],
        {} as any,
      );

      const txReceipt = await crossChainTestHarness.publicClient.waitForTransactionReceipt({
        hash: txHash,
      });

      // Exactly 1 event should be emitted in the transaction
      expect(txReceipt.logs.length).toBe(1);

      // We decode the event log before checking it
      const txLog = txReceipt.logs[0];
      const topics = decodeEventLog({
        abi: OutboxAbi,
        data: txLog.data,
        topics: txLog.topics,
      }) as {
        eventName: 'MessageConsumed';
        args: {
          l2BlockNumber: bigint;
          root: `0x${string}`;
          messageHash: `0x${string}`;
          leafIndex: bigint;
        };
      };

      // We check that MessageConsumed event was emitted with the expected message hash and leaf index
      expect(topics.args.messageHash).toStrictEqual(leaf.toString());
      expect(topics.args.leafIndex).toStrictEqual(BigInt(0));
    },
    60_000,
  );

  // Note: We register one portal address when deploying contract but that address is no-longer the only address
  // allowed to send messages to the given contract. In the following test we'll test that it's really the case.
  it.each([true, false])(
    'can send an L1 -> L2 message from a non-registered portal address consumed from private or public and then sends and claims exactly the same message again',
    async (isPrivate: boolean) => {
      const testContract = await TestContract.deploy(user1Wallet).send().deployed();

      const consumeMethod = isPrivate
        ? testContract.methods.consume_message_from_arbitrary_sender_private
        : testContract.methods.consume_message_from_arbitrary_sender_public;

      const secret = Fr.random();

      const message = new L1ToL2Message(
        new L1Actor(crossChainTestHarness.ethAccount, crossChainTestHarness.publicClient.chain.id),
        new L2Actor(testContract.address, 1),
        Fr.random(), // content
        computeMessageSecretHash(secret), // secretHash
      );

      await sendL2Message(message);

      const [message1Index, _1] = (await aztecNode.getL1ToL2MessageMembershipWitness('latest', message.hash(), 0n))!;

      // Finally, we consume the L1 -> L2 message using the test contract either from private or public
      await consumeMethod(message.content, secret, message.sender.sender).send().wait();

      // We send and consume the exact same message the second time to test that oracles correctly return the new
      // non-nullified message
      await sendL2Message(message);

      // We check that the duplicate message was correctly inserted by checking that its message index is defined and
      // larger than the previous message index
      const [message2Index, _2] = (await aztecNode.getL1ToL2MessageMembershipWitness(
        'latest',
        message.hash(),
        message1Index + 1n,
      ))!;

      expect(message2Index).toBeDefined();
      expect(message2Index).toBeGreaterThan(message1Index);

      // Now we consume the message again. Everything should pass because oracle should return the duplicate message
      // which is not nullified
      await consumeMethod(message.content, secret, message.sender.sender).send().wait();
    },
    120_000,
  );

  const sendL2Message = async (message: L1ToL2Message) => {
    // We inject the message to Inbox
    const txHash = await inbox.write.sendL2Message(
      [
        { actor: message.recipient.recipient.toString() as Hex, version: 1n },
        message.content.toString() as Hex,
        message.secretHash.toString() as Hex,
      ] as const,
      {} as any,
    );

    // We check that the message was correctly injected by checking the emitted event
    const msgHash = message.hash();
    {
      const txReceipt = await crossChainTestHarness.publicClient.waitForTransactionReceipt({
        hash: txHash,
      });

      // Exactly 1 event should be emitted in the transaction
      expect(txReceipt.logs.length).toBe(1);

      // We decode the event and get leaf out of it
      const txLog = txReceipt.logs[0];
      const topics = decodeEventLog({
        abi: InboxAbi,
        data: txLog.data,
        topics: txLog.topics,
      });
      const receivedMsgHash = topics.args.hash;

      // We check that the leaf inserted into the subtree matches the expected message hash
      expect(receivedMsgHash).toBe(msgHash.toString());
    }

    await crossChainTestHarness.makeMessageConsumable(msgHash);
  };
});
