import {
  type AccountWalletWithSecretKey,
  type AztecNode,
  BatchCall,
  type DeployL1Contracts,
  EthAddress,
  Fr,
  type SiblingPath,
} from '@aztec/aztec.js';
import { sha256ToField } from '@aztec/foundation/crypto';
import { truncateAndPad } from '@aztec/foundation/serialize';
import { OutboxAbi, RollupAbi } from '@aztec/l1-artifacts';
import { SHA256 } from '@aztec/merkle-tree';
import { TestContract } from '@aztec/noir-contracts.js';

import { beforeEach, describe, expect, it } from '@jest/globals';
import { decodeEventLog, getContract } from 'viem';

import { setup } from './fixtures/utils.js';

describe('E2E Outbox Tests', () => {
  let teardown: () => void;
  let aztecNode: AztecNode;
  const merkleSha256 = new SHA256();
  let contract: TestContract;
  let wallets: AccountWalletWithSecretKey[];
  let deployL1ContractsValues: DeployL1Contracts;
  let outbox: any;
  let rollup: any;

  beforeEach(async () => {
    ({ teardown, aztecNode, wallets, deployL1ContractsValues } = await setup(1));
    outbox = getContract({
      address: deployL1ContractsValues.l1ContractAddresses.outboxAddress.toString(),
      abi: OutboxAbi,
      client: deployL1ContractsValues.walletClient,
    });

    const receipt = await TestContract.deploy(wallets[0]).send({ contractAddressSalt: Fr.ZERO }).wait();
    contract = receipt.contract;

    rollup = getContract({
      address: deployL1ContractsValues.l1ContractAddresses.rollupAddress.toString(),
      abi: RollupAbi,
      client: deployL1ContractsValues.walletClient,
    });
  });

  afterAll(() => teardown());

  it('Inserts a new transaction with two out messages, and verifies sibling paths of both the new messages', async () => {
    // recipient2 = msg.sender, so we can consume it later
    const [[recipient1, content1], [recipient2, content2]] = [
      [EthAddress.random(), Fr.random()],
      [EthAddress.fromString(deployL1ContractsValues.walletClient.account.address), Fr.random()],
    ];

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
      [makeL2ToL1Message(recipient1, content1), makeL2ToL1Message(recipient2, content2)].map(expectedL2ToL1Message =>
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

    // Outbox L1 tests

    // Since the outbox is only consumable when the block is proven, we need to set the block to be proven
    await rollup.write.setAssumeProvenUntilBlockNumber([1 + (txReceipt.blockNumber ?? 0)]);

    // Check L1 has expected message tree
    const [l1Root, l1MinHeight] = await outbox.read.getRootData([txReceipt.blockNumber]);
    expect(l1Root).toEqual(`0x${block?.header.contentCommitment.outHash.toString('hex')}`);
    // The path for the message should have the shortest possible height, since we only have 2 msgs
    expect(l1MinHeight).toEqual(BigInt(siblingPath.pathSize));

    // Consume msg 2
    // Taken from l2_to_l1.test
    const msg2 = {
      sender: { actor: contract.address.toString() as `0x${string}`, version: 1n },
      recipient: {
        actor: recipient2.toString() as `0x${string}`,
        chainId: BigInt(deployL1ContractsValues.publicClient.chain.id),
      },
      content: content2.toString() as `0x${string}`,
    };

    const txHash = await outbox.write.consume(
      [
        msg2,
        BigInt(txReceipt.blockNumber!),
        BigInt(index2),
        siblingPath2.toBufferArray().map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      ],
      {} as any,
    );
    const l1Receipt = await deployL1ContractsValues.publicClient.waitForTransactionReceipt({
      hash: txHash,
    });
    // Consume call goes through
    expect(l1Receipt.status).toEqual('success');

    const txLog = l1Receipt.logs[0];
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
    // Consumed the expected message
    expect(topics.args.messageHash).toStrictEqual(l2ToL1Messages?.[1].toString());
    expect(topics.args.leafIndex).toStrictEqual(BigInt(index2));

    const consumeAgain = outbox.write.consume(
      [
        msg2,
        BigInt(txReceipt.blockNumber!),
        BigInt(index2),
        siblingPath2.toBufferArray().map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      ],
      {} as any,
    );
    // Ensuring we cannot consume the same message again
    await expect(consumeAgain).rejects.toThrow();
  });

  it('Inserts two transactions with total four out messages, and verifies sibling paths of two new messages', async () => {
    // Force txs to be in the same block
    await aztecNode.setConfig({ minTxsPerBlock: 2 });
    const [[recipient1, content1], [recipient2, content2], [recipient3, content3], [recipient4, content4]] = [
      [EthAddress.random(), Fr.random()],
      [EthAddress.fromString(deployL1ContractsValues.walletClient.account.address), Fr.random()],
      [EthAddress.random(), Fr.random()],
      [EthAddress.random(), Fr.random()],
    ];

    const call0 = new BatchCall(wallets[0], [
      contract.methods.create_l2_to_l1_message_arbitrary_recipient_private(content1, recipient1).request(),
      contract.methods.create_l2_to_l1_message_arbitrary_recipient_private(content2, recipient2).request(),
      contract.methods.create_l2_to_l1_message_arbitrary_recipient_private(content3, recipient3).request(),
    ]);

    const call1 = contract.methods.create_l2_to_l1_message_arbitrary_recipient_private(content4, recipient4);

    const [l2TxReceipt0, l2TxReceipt1] = await Promise.all([call0.send().wait(), call1.send().wait()]);
    expect(l2TxReceipt0.blockNumber).toEqual(l2TxReceipt1.blockNumber);

    const block = await aztecNode.getBlock(l2TxReceipt0.blockNumber!);

    const l2ToL1Messages = block?.body.txEffects.flatMap(txEffect => txEffect.l2ToL1Msgs);
    // Not checking strict equality as ordering is not guaranteed - this should be covered in that we can recalculate the out hash below
    expect(l2ToL1Messages?.length).toEqual(4);

    // For each individual message, we are using our node API to grab the index and sibling path. We expect
    // the index to match the order of the block we obtained earlier. We also then use this sibling path to hash up to the root,
    // verifying that the expected root obtained through the message and the sibling path match the actual root
    // that was returned by the circuits in the header as out_hash.
    const singleMessage = makeL2ToL1Message(recipient4, content4);
    const [index, siblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
      l2TxReceipt0.blockNumber!,
      singleMessage,
    );
    // The solo message is the only one in the tx, so it only requires a subtree of height 1
    // +1 for being rolled up
    expect(siblingPath.pathSize).toBe(2);
    const expectedRoot = calculateExpectedRoot(singleMessage, siblingPath as SiblingPath<2>, index);
    expect(expectedRoot.toString('hex')).toEqual(block?.header.contentCommitment.outHash.toString('hex'));

    const messageToConsume = makeL2ToL1Message(recipient2, content2);
    const [index2, siblingPath2] = await aztecNode.getL2ToL1MessageMembershipWitness(
      l2TxReceipt0.blockNumber!,
      messageToConsume,
    );
    // This message is in a group of 3, => it needs a subtree of height 2
    // +1 for being rolled up
    expect(siblingPath2.pathSize).toBe(3);

    // Outbox L1 tests
    // Since the outbox is only consumable when the block is proven, we need to set the block to be proven
    await rollup.write.setAssumeProvenUntilBlockNumber([1 + (l2TxReceipt0.blockNumber ?? 0)]);

    // Check L1 has expected message tree
    const [l1Root, l1MinHeight] = await outbox.read.getRootData([l2TxReceipt0.blockNumber]);
    expect(l1Root).toEqual(`0x${block?.header.contentCommitment.outHash.toString('hex')}`);

    // The path for the single message should have the shortest possible height
    expect(l1MinHeight).toEqual(BigInt(siblingPath.pathSize));

    // Consume msg 2
    // Taken from l2_to_l1.test
    const msg2 = {
      sender: { actor: contract.address.toString() as `0x${string}`, version: 1n },
      recipient: {
        actor: recipient2.toString() as `0x${string}`,
        chainId: BigInt(deployL1ContractsValues.publicClient.chain.id),
      },
      content: content2.toString() as `0x${string}`,
    };

    const txHash = await outbox.write.consume(
      [
        msg2,
        BigInt(l2TxReceipt0.blockNumber!),
        BigInt(index2),
        siblingPath2.toBufferArray().map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      ],
      {} as any,
    );
    const l1Receipt = await deployL1ContractsValues.publicClient.waitForTransactionReceipt({
      hash: txHash,
    });
    // Consume call goes through
    expect(l1Receipt.status).toEqual('success');

    const txLog = l1Receipt.logs[0];
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
    // Consumed the expected message
    expect(topics.args.messageHash).toStrictEqual(messageToConsume.toString());
    expect(topics.args.leafIndex).toStrictEqual(BigInt(index2));

    const consumeAgain = outbox.write.consume(
      [
        msg2,
        BigInt(l2TxReceipt0.blockNumber!),
        BigInt(index2),
        siblingPath2.toBufferArray().map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      ],
      {} as any,
    );
    // Ensuring we cannot consume the same message again
    await expect(consumeAgain).rejects.toThrow();
  });

  it('Inserts two out messages in two transactions and verifies sibling paths of both the new messages', async () => {
    // Force txs to be in the same block
    await aztecNode.setConfig({ minTxsPerBlock: 2 });
    // recipient2 = msg.sender, so we can consume it later
    const [[recipient1, content1], [recipient2, content2]] = [
      [EthAddress.random(), Fr.random()],
      [EthAddress.fromString(deployL1ContractsValues.walletClient.account.address), Fr.random()],
    ];

    const call0 = contract.methods.create_l2_to_l1_message_arbitrary_recipient_private(content1, recipient1);
    const call1 = contract.methods.create_l2_to_l1_message_arbitrary_recipient_private(content2, recipient2);

    // resolve together to force the txs to be in the same block
    const [l2TxReceipt0, l2TxReceipt1] = await Promise.all([call0.send().wait(), call1.send().wait()]);
    expect(l2TxReceipt0.blockNumber).toEqual(l2TxReceipt1.blockNumber);

    const block = await aztecNode.getBlock(l2TxReceipt0.blockNumber!);

    const l2ToL1Messages = block?.body.txEffects.flatMap(txEffect => txEffect.l2ToL1Msgs);
    const messageToConsume = makeL2ToL1Message(recipient2, content2);

    // We cannot guarantee the order of txs in blocks
    expect(
      l2ToL1Messages?.map(l2ToL1Message =>
        l2ToL1Message.toString().includes(makeL2ToL1Message(recipient1, content1).toString()),
      ),
    );
    expect(l2ToL1Messages?.map(l2ToL1Message => l2ToL1Message.toString().includes(messageToConsume.toString())));

    // For each individual message, we are using our node API to grab the index and sibling path. We expect
    // the index to match the order of the block we obtained earlier. We also then use this sibling path to hash up to the root,
    // verifying that the expected root obtained through the message and the sibling path match the actual root
    // that was returned by the circuits in the header as out_hash.
    const [index, siblingPath] = await aztecNode.getL2ToL1MessageMembershipWitness(
      l2TxReceipt0.blockNumber!,
      l2ToL1Messages![0],
    );
    expect(siblingPath.pathSize).toBe(2);
    // We can only confirm the below index because we have taken the msg hash as the first of the block.body
    // It is not necesssarily the msg constructed from [recipient1, content1] above
    expect(index).toBe(0n);

    const [index2, siblingPath2] = await aztecNode.getL2ToL1MessageMembershipWitness(
      l2TxReceipt0.blockNumber!,
      l2ToL1Messages![1],
    );
    expect(siblingPath2.pathSize).toBe(2);
    // See above comment for confirming index
    expect(index2).toBe(2n);

    // Outbox L1 tests
    // Since the outbox is only consumable when the block is proven, we need to set the block to be proven
    await rollup.write.setAssumeProvenUntilBlockNumber([1 + (l2TxReceipt0.blockNumber ?? 0)]);

    // Check L1 has expected message tree
    const [l1Root, l1MinHeight] = await outbox.read.getRootData([l2TxReceipt0.blockNumber]);
    expect(l1Root).toEqual(`0x${block?.header.contentCommitment.outHash.toString('hex')}`);
    // The path for the message should have the shortest possible height, since we only have one msg per tx
    expect(l1MinHeight).toEqual(BigInt(siblingPath.pathSize));

    // Consume msg 2
    // Taken from l2_to_l1.test
    const msg2 = {
      sender: { actor: contract.address.toString() as `0x${string}`, version: 1n },
      recipient: {
        actor: recipient2.toString() as `0x${string}`,
        chainId: BigInt(deployL1ContractsValues.publicClient.chain.id),
      },
      content: content2.toString() as `0x${string}`,
    };
    const [inputIndex, inputPath] = messageToConsume.equals(l2ToL1Messages![0])
      ? [index, siblingPath]
      : [index2, siblingPath2];
    const txHash = await outbox.write.consume(
      [
        msg2,
        BigInt(l2TxReceipt0.blockNumber!),
        BigInt(inputIndex),
        inputPath.toBufferArray().map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      ],
      {} as any,
    );
    const l1Receipt = await deployL1ContractsValues.publicClient.waitForTransactionReceipt({
      hash: txHash,
    });
    // Consume call goes through
    expect(l1Receipt.status).toEqual('success');

    const txLog = l1Receipt.logs[0];
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
    // Consumed the expected message
    expect(topics.args.messageHash).toStrictEqual(messageToConsume.toString());
    expect(topics.args.leafIndex).toStrictEqual(BigInt(inputIndex));

    const consumeAgain = outbox.write.consume(
      [
        msg2,
        BigInt(l2TxReceipt0.blockNumber!),
        BigInt(index2),
        siblingPath2.toBufferArray().map((buf: Buffer) => `0x${buf.toString('hex')}`) as readonly `0x${string}`[],
      ],
      {} as any,
    );
    // Ensuring we cannot consume the same message again
    await expect(consumeAgain).rejects.toThrow();
  });

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
