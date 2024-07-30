import {
  type EthAddressLike,
  type FieldLike,
  Fr,
  L1Actor,
  L1ToL2Message,
  L2Actor,
  computeSecretHash,
} from '@aztec/aztec.js';
import { TestContract } from '@aztec/noir-contracts.js';

import { sendL1ToL2Message } from '../fixtures/l1_to_l2_messaging.js';
import { PublicCrossChainMessagingContractTest } from './public_cross_chain_messaging_contract_test.js';

describe('e2e_public_cross_chain_messaging l1_to_l2', () => {
  const t = new PublicCrossChainMessagingContractTest('l1_to_l2');

  let { crossChainTestHarness, aztecNode, user1Wallet } = t;

  beforeAll(async () => {
    await t.applyBaseSnapshots();
    await t.setup();
    // Have to destructure again to ensure we have latest refs.
    ({ crossChainTestHarness, user1Wallet } = t);

    aztecNode = crossChainTestHarness.aztecNode;
  }, 300_000);

  afterAll(async () => {
    await t.teardown();
  });

  // Note: We register one portal address when deploying contract but that address is no-longer the only address
  // allowed to send messages to the given contract. In the following test we'll test that it's really the case.
  it.each([true, false])(
    'can send an L1 -> L2 message from a non-registered portal address consumed from private or public and then sends and claims exactly the same message again',
    async (isPrivate: boolean) => {
      const testContract = await TestContract.deploy(user1Wallet).send().deployed();

      const consumeMethod = isPrivate
        ? (content: FieldLike, secret: FieldLike, sender: EthAddressLike, _leafIndex: FieldLike) =>
            testContract.methods.consume_message_from_arbitrary_sender_private(content, secret, sender)
        : testContract.methods.consume_message_from_arbitrary_sender_public;

      const secret = Fr.random();

      const message = new L1ToL2Message(
        new L1Actor(crossChainTestHarness.ethAccount, crossChainTestHarness.publicClient.chain.id),
        new L2Actor(testContract.address, 1),
        Fr.random(), // content
        computeSecretHash(secret), // secretHash
      );

      await sendL2Message(message);

      const [message1Index, _1] = (await aztecNode.getL1ToL2MessageMembershipWitness('latest', message.hash(), 0n))!;

      // Finally, we consume the L1 -> L2 message using the test contract either from private or public
      await consumeMethod(message.content, secret, message.sender.sender, message1Index).send().wait();

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
      await consumeMethod(message.content, secret, message.sender.sender, message2Index).send().wait();
    },
    120_000,
  );

  const sendL2Message = async (message: L1ToL2Message) => {
    const msgHash = await sendL1ToL2Message(message, crossChainTestHarness);
    await crossChainTestHarness.makeMessageConsumable(msgHash);
  };
});
