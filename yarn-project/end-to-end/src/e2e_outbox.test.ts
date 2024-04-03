import {
  type AccountWalletWithPrivateKey,
  type AztecNode,
  BatchCall,
  type DeployL1Contracts,
  EthAddress,
  Fr,
  type SiblingPath,
} from '@aztec/aztec.js';
import { sha256ToField } from '@aztec/foundation/crypto';
import { truncateAndPad } from '@aztec/foundation/serialize';
import { SHA256 } from '@aztec/merkle-tree';
import { TestContract } from '@aztec/noir-contracts.js';

import { beforeEach, describe, expect, it } from '@jest/globals';

import { setup } from './fixtures/utils.js';

// @remark - This does not test the Outbox Contract yet. All this test does is create L2 to L1 messages in a block,
// verify their existence, and produce a sibling path that is also checked for validity against the circuit produced
// out_hash in the header.
describe('E2E Outbox Tests', () => {
  let teardown: () => void;
  let aztecNode: AztecNode;
  const merkleSha256 = new SHA256();
  let contract: TestContract;
  let wallets: AccountWalletWithPrivateKey[];
  let deployL1ContractsValues: DeployL1Contracts;

  beforeEach(async () => {
    ({ teardown, aztecNode, wallets, deployL1ContractsValues } = await setup(1));

    const receipt = await TestContract.deploy(wallets[0]).send({ contractAddressSalt: Fr.ZERO }).wait();
    contract = receipt.contract;
  }, 100_000);

  afterAll(() => teardown());

  it('Inserts a new transaction with two out messages, and verifies sibling paths of both the new messages', async () => {
    const [[recipient1, content1], [recipient2, content2]] = [
      [EthAddress.random(), Fr.random()],
      [EthAddress.random(), Fr.random()],
    ];

    // We can't put any more l2 to L1 messages here There are a max of 2 L2 to L1 messages per transaction
    const call = new BatchCall(wallets[0], [
      contract.methods.create_l2_to_l1_message_arbitrary_recipient_private(content1, recipient1).request(),
      contract.methods.create_l2_to_l1_message_arbitrary_recipient_private(content2, recipient2).request(),
    ]);

    // TODO (#5104): When able to guarantee multiple txs in a single block, make this populate a full tree. Right now we are
    // unable to do this because in CI, for some reason, the tx's are handled in different blocks, so it is impossible
    // to make a full tree of L2 -> L1 messages as we are only able to set one tx's worth of L1 -> L2 messages in a block (2 messages out of 4)
    const txReceipt = await call.send().wait();

    const block = await aztecNode.getBlock(txReceipt.blockNumber!);

    const l2ToL1Messages = block?.body.txEffects.flatMap(txEffect => txEffect.l2ToL1Msgs);

    expect(l2ToL1Messages?.map(l2ToL1Message => l2ToL1Message.toString())).toStrictEqual(
      [makeL2ToL1Message(recipient2, content2), makeL2ToL1Message(recipient1, content1)].map(expectedL2ToL1Message =>
        expectedL2ToL1Message.toString(),
      ),
    );

    // For each individual message, we are using our node API to grab the index and sibling path. We expect
    // the index to match the order of the block we obtained earlier. We also then use this sibling path to hash up to the root,
    // verifying that the expected root obtained through the message and the sibling path match the actual root
    // that was returned by the circuits in the header as out_hash.
    const [index, siblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
      txReceipt.blockNumber!,
      l2ToL1Messages![0],
    );
    expect(siblingPath.pathSize).toBe(2);
    expect(index).toBe(0n);
    const expectedRoot = calculateExpectedRoot(l2ToL1Messages![0], siblingPath as SiblingPath<2>, index);
    expect(expectedRoot.toString('hex')).toEqual(block?.header.contentCommitment.outHash.toString('hex'));

    const [index2, siblingPath2] = await aztecNode.getL2ToL1MessageMembershipWitness(
      txReceipt.blockNumber!,
      l2ToL1Messages![1],
    );
    expect(siblingPath2.pathSize).toBe(2);
    expect(index2).toBe(1n);
    const expectedRoot2 = calculateExpectedRoot(l2ToL1Messages![1], siblingPath2 as SiblingPath<2>, index2);
    expect(expectedRoot2.toString('hex')).toEqual(block?.header.contentCommitment.outHash.toString('hex'));
  }, 360_000);

  function calculateExpectedRoot(l2ToL1Message: Fr, siblingPath: SiblingPath<2>, index: bigint): Buffer {
    const firstLayerInput: [Buffer, Buffer] =
      index & 0x1n
        ? [siblingPath.toBufferArray()[0], l2ToL1Message.toBuffer()]
        : [l2ToL1Message.toBuffer(), siblingPath.toBufferArray()[0]];
    const firstLayer = merkleSha256.hash(...firstLayerInput);
    index /= 2n;
    // In the circuit, the 'firstLayer' is the kernel out hash, which is truncated to 31 bytes
    // To match the result, the below preimages and the output are truncated to 31 then padded
    const secondLayerInput: [Buffer, Buffer] =
      index & 0x1n
        ? [siblingPath.toBufferArray()[1], truncateAndPad(firstLayer)]
        : [truncateAndPad(firstLayer), siblingPath.toBufferArray()[1]];
    return truncateAndPad(merkleSha256.hash(...secondLayerInput));
  }

  function makeL2ToL1Message(recipient: EthAddress, content: Fr = Fr.ZERO): Fr {
    const leaf = sha256ToField([
      contract.address,
      new Fr(1), // aztec version
      recipient.toBuffer32(),
      new Fr(deployL1ContractsValues.publicClient.chain.id), // chain id
      content,
    ]);

    return leaf;
  }
});
