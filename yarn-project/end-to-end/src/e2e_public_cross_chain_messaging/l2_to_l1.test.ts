import { Fr } from '@aztec/aztec.js';
import { sha256ToField } from '@aztec/foundation/crypto';
import { OutboxAbi } from '@aztec/l1-artifacts';
import { TestContract } from '@aztec/noir-contracts.js';

import { type Hex, decodeEventLog } from 'viem';

import { PublicCrossChainMessagingContractTest } from './public_cross_chain_messaging_contract_test.js';

describe('e2e_public_cross_chain_messaging l2_to_l1', () => {
  const t = new PublicCrossChainMessagingContractTest('l2_to_l1');

  let { crossChainTestHarness, aztecNode, user1Wallet, outbox } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
    // Have to destructure again to ensure we have latest refs.
    ({ crossChainTestHarness, user1Wallet } = t);

    aztecNode = crossChainTestHarness.aztecNode;

    outbox = crossChainTestHarness.outbox;
  }, 300_000);

  afterAll(async () => {
    await t.teardown();
  });

  // Note: We register one portal address when deploying contract but that address is no-longer the only address
  // allowed to receive messages from the given contract. In the following test we'll test that it's really the case.
  it.each([[true], [false]])(
    `can send an L2 -> L1 message to a non-registered portal address from public or private`,
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

      await t.assumeProven();

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
});
