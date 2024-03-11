import { Body, ExtendedContractData, L2Block, L2BlockL2Logs, LogType } from '@aztec/circuit-types';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { sleep } from '@aztec/foundation/sleep';
import {
  AvailabilityOracleAbi,
  ContractDeploymentEmitterAbi,
  InboxAbi,
  NewInboxAbi,
  RollupAbi,
} from '@aztec/l1-artifacts';

import { MockProxy, mock } from 'jest-mock-extended';
import { Chain, HttpTransport, Log, PublicClient, Transaction, encodeFunctionData, toHex } from 'viem';

import { Archiver } from './archiver.js';
import { ArchiverDataStore } from './archiver_store.js';
import { MemoryArchiverStore } from './memory_archiver_store/memory_archiver_store.js';

describe('Archiver', () => {
  const rollupAddress = EthAddress.ZERO;
  const inboxAddress = EthAddress.ZERO;
  // TODO(#4492): Nuke this once the old inbox is purged
  const newInboxAddress = EthAddress.ZERO;
  const registryAddress = EthAddress.ZERO;
  const availabilityOracleAddress = EthAddress.ZERO;
  const contractDeploymentEmitterAddress = EthAddress.fromString('0x0000000000000000000000000000000000000001');
  const blockNumbers = [1, 2, 3];
  let publicClient: MockProxy<PublicClient<HttpTransport, Chain>>;
  let archiverStore: ArchiverDataStore;

  beforeEach(() => {
    publicClient = mock<PublicClient<HttpTransport, Chain>>();
    archiverStore = new MemoryArchiverStore(1000);
  });

  it('can start, sync and stop and handle l1 to l2 messages and logs', async () => {
    const archiver = new Archiver(
      publicClient,
      rollupAddress,
      availabilityOracleAddress,
      inboxAddress,
      newInboxAddress,
      registryAddress,
      contractDeploymentEmitterAddress,
      archiverStore,
      1000,
    );

    let latestBlockNum = await archiver.getBlockNumber();
    expect(latestBlockNum).toEqual(0);

    const blocks = blockNumbers.map(x => L2Block.random(x, 4, x, x + 1, x * 2, x * 3));
    const publishTxs = blocks.map(block => block.body).map(makePublishTx);
    const rollupTxs = blocks.map(makeRollupTx);

    // `L2Block.random(x)` creates some l1 to l2 messages. We add those,
    // since it is expected by the test that these would be consumed.
    // Archiver removes such messages from pending store.
    // Also create some more messages to cancel and some that will stay pending.

    const messageToCancel1 = Fr.random().toString();
    const messageToCancel2 = Fr.random().toString();
    const l1ToL2MessagesToCancel = [messageToCancel1, messageToCancel2];
    const messageToStayPending1 = Fr.random().toString();
    const messageToStayPending2 = Fr.random().toString();

    const l1ToL2MessageAddedEvents = [
      makeL1ToL2MessageAddedEvents(
        100n,
        blocks[0].body.l1ToL2Messages.flatMap(key => (key.isZero() ? [] : key.toString())),
      ),
      makeL1ToL2MessageAddedEvents(
        100n,
        blocks[1].body.l1ToL2Messages.flatMap(key => (key.isZero() ? [] : key.toString())),
      ),
      makeL1ToL2MessageAddedEvents(
        2501n,
        blocks[2].body.l1ToL2Messages.flatMap(key => (key.isZero() ? [] : key.toString())),
      ),
      makeL1ToL2MessageAddedEvents(2502n, [
        messageToCancel1,
        messageToCancel2,
        messageToStayPending1,
        messageToStayPending2,
      ]),
    ];
    publicClient.getBlockNumber.mockResolvedValueOnce(2500n).mockResolvedValueOnce(2600n).mockResolvedValueOnce(2700n);
    // logs should be created in order of how archiver syncs.
    publicClient.getLogs
      .mockResolvedValueOnce(l1ToL2MessageAddedEvents.slice(0, 2).flat())
      .mockResolvedValueOnce([]) // no messages to cancel
      .mockResolvedValueOnce([makeLeafInsertedEvent(98n, 1n, 0n), makeLeafInsertedEvent(99n, 1n, 1n)])
      .mockResolvedValueOnce([makeTxsPublishedEvent(101n, blocks[0].body.getTxsEffectsHash())])
      .mockResolvedValueOnce([makeL2BlockProcessedEvent(101n, 1n)])
      .mockResolvedValueOnce([makeContractDeploymentEvent(103n, blocks[0])]) // the first loop of the archiver ends here at block 2500
      .mockResolvedValueOnce(l1ToL2MessageAddedEvents.slice(2, 4).flat())
      .mockResolvedValueOnce(makeL1ToL2MessageCancelledEvents(2503n, l1ToL2MessagesToCancel))
      .mockResolvedValueOnce([
        makeLeafInsertedEvent(2504n, 2n, 0n),
        makeLeafInsertedEvent(2505n, 2n, 1n),
        makeLeafInsertedEvent(2505n, 2n, 2n),
        makeLeafInsertedEvent(2506n, 3n, 1n),
      ])
      .mockResolvedValueOnce([
        makeTxsPublishedEvent(2510n, blocks[1].body.getTxsEffectsHash()),
        makeTxsPublishedEvent(2520n, blocks[2].body.getTxsEffectsHash()),
      ])
      .mockResolvedValueOnce([makeL2BlockProcessedEvent(2510n, 2n), makeL2BlockProcessedEvent(2520n, 3n)])
      .mockResolvedValueOnce([makeContractDeploymentEvent(2540n, blocks[1])])
      .mockResolvedValue([]);
    publicClient.getTransaction.mockResolvedValueOnce(publishTxs[0]);
    publicClient.getTransaction.mockResolvedValueOnce(rollupTxs[0]);

    publishTxs.slice(1).forEach(tx => publicClient.getTransaction.mockResolvedValueOnce(tx));
    rollupTxs.slice(1).forEach(tx => publicClient.getTransaction.mockResolvedValueOnce(tx));

    await archiver.start(false);

    // Wait until block 3 is processed. If this won't happen the test will fail with timeout.
    while ((await archiver.getBlockNumber()) !== 3) {
      await sleep(100);
    }

    latestBlockNum = await archiver.getBlockNumber();
    expect(latestBlockNum).toEqual(3);

    // New L1 to L2 messages
    {
      // Checks that I get correct amount of sequenced new messages for L2 blocks 1 and 2
      let newL1ToL2Messages = await archiver.getNewL1ToL2Messages(1n);
      expect(newL1ToL2Messages.length).toEqual(2);

      newL1ToL2Messages = await archiver.getNewL1ToL2Messages(2n);
      expect(newL1ToL2Messages.length).toEqual(3);

      // Check that I cannot get messages for block 3 because there is a message gap (message with index 0 was not
      // processed)
      await expect(async () => {
        await archiver.getNewL1ToL2Messages(3n);
      }).rejects.toThrow(`L1 to L2 message gap found in block ${3}`);
    }

    // Check that only 2 messages (l1ToL2MessageAddedEvents[3][2] and l1ToL2MessageAddedEvents[3][3]) are pending.
    // Other two (l1ToL2MessageAddedEvents[3][0..2]) were cancelled. And the previous messages were confirmed.
    const expectedPendingEntryKeys = [
      l1ToL2MessageAddedEvents[3][2].args.entryKey,
      l1ToL2MessageAddedEvents[3][3].args.entryKey,
    ];
    const actualPendingEntryKeys = (await archiver.getPendingL1ToL2EntryKeys(10)).map(key => key.toString());
    expect(expectedPendingEntryKeys).toEqual(actualPendingEntryKeys);

    // Expect logs to correspond to what is set by L2Block.random(...)
    const encryptedLogs = await archiver.getLogs(1, 100, LogType.ENCRYPTED);
    expect(encryptedLogs.length).toEqual(blockNumbers.length);

    for (const [index, x] of blockNumbers.entries()) {
      const expectedTotalNumEncryptedLogs = 4 * x * (x * 2);
      const totalNumEncryptedLogs = L2BlockL2Logs.unrollLogs([encryptedLogs[index]]).length;
      expect(totalNumEncryptedLogs).toEqual(expectedTotalNumEncryptedLogs);
    }

    const unencryptedLogs = await archiver.getLogs(1, 100, LogType.UNENCRYPTED);
    expect(unencryptedLogs.length).toEqual(blockNumbers.length);

    blockNumbers.forEach((x, index) => {
      const expectedTotalNumUnencryptedLogs = 4 * (x + 1) * (x * 3);
      const totalNumUnencryptedLogs = L2BlockL2Logs.unrollLogs([unencryptedLogs[index]]).length;
      expect(totalNumUnencryptedLogs).toEqual(expectedTotalNumUnencryptedLogs);
    });

    await archiver.stop();
  }, 10_000);

  it('does not sync past current block number', async () => {
    const numL2BlocksInTest = 2;
    const archiver = new Archiver(
      publicClient,
      rollupAddress,
      availabilityOracleAddress,
      inboxAddress,
      newInboxAddress,
      registryAddress,
      contractDeploymentEmitterAddress,
      archiverStore,
      1000,
    );

    let latestBlockNum = await archiver.getBlockNumber();
    expect(latestBlockNum).toEqual(0);

    const createL1ToL2Messages = () => {
      return [Fr.random().toString(), Fr.random().toString()];
    };

    const blocks = blockNumbers.map(x => L2Block.random(x, 4, x, x + 1, x * 2, x * 3));

    const publishTxs = blocks.map(block => block.body).map(makePublishTx);
    const rollupTxs = blocks.map(makeRollupTx);

    // `L2Block.random(x)` creates some l1 to l2 messages. We add those,
    // since it is expected by the test that these would be consumed.
    // Archiver removes such messages from pending store.
    // Also create some more messages to cancel and some that will stay pending.

    const additionalL1ToL2MessagesBlock102 = createL1ToL2Messages();
    const additionalL1ToL2MessagesBlock103 = createL1ToL2Messages();

    const l1ToL2MessageAddedEvents = [
      makeL1ToL2MessageAddedEvents(
        100n,
        blocks[0].body.l1ToL2Messages.flatMap(key => (key.isZero() ? [] : key.toString())),
      ),
      makeL1ToL2MessageAddedEvents(
        101n,
        blocks[1].body.l1ToL2Messages.flatMap(key => (key.isZero() ? [] : key.toString())),
      ),
      makeL1ToL2MessageAddedEvents(102n, additionalL1ToL2MessagesBlock102),
      makeL1ToL2MessageAddedEvents(103n, additionalL1ToL2MessagesBlock103),
    ];

    // Here we set the current L1 block number to 102. L1 to L2 messages after this should not be read.
    publicClient.getBlockNumber.mockResolvedValue(102n);
    // add all of the L1 to L2 messages to the mock
    publicClient.getLogs
      .mockImplementationOnce((args?: any) => {
        return Promise.resolve(
          l1ToL2MessageAddedEvents
            .flat()
            .filter(x => x.blockNumber! >= args.fromBlock && x.blockNumber! < args.toBlock),
        );
      })
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([makeLeafInsertedEvent(66n, 1n, 0n), makeLeafInsertedEvent(68n, 1n, 1n)])
      .mockResolvedValueOnce([
        makeTxsPublishedEvent(70n, blocks[0].body.getTxsEffectsHash()),
        makeTxsPublishedEvent(80n, blocks[1].body.getTxsEffectsHash()),
      ])
      .mockResolvedValueOnce([makeL2BlockProcessedEvent(70n, 1n), makeL2BlockProcessedEvent(80n, 2n)])
      .mockResolvedValue([]);
    publishTxs.slice(0, numL2BlocksInTest).forEach(tx => publicClient.getTransaction.mockResolvedValueOnce(tx));
    rollupTxs.slice(0, numL2BlocksInTest).forEach(tx => publicClient.getTransaction.mockResolvedValueOnce(tx));

    await archiver.start(false);

    // Wait until block 3 is processed. If this won't happen the test will fail with timeout.
    while ((await archiver.getBlockNumber()) !== numL2BlocksInTest) {
      await sleep(100);
    }

    latestBlockNum = await archiver.getBlockNumber();
    expect(latestBlockNum).toEqual(numL2BlocksInTest);

    // Check that the only pending L1 to L2 messages are those from eth bock 102
    const expectedPendingEntryKeys = additionalL1ToL2MessagesBlock102;
    const actualPendingEntryKeys = (await archiver.getPendingL1ToL2EntryKeys(100)).map(key => key.toString());
    expect(actualPendingEntryKeys).toEqual(expectedPendingEntryKeys);

    await archiver.stop();
  }, 10_000);

  it('pads L1 to L2 messages', async () => {
    const archiver = new Archiver(
      publicClient,
      rollupAddress,
      availabilityOracleAddress,
      inboxAddress,
      newInboxAddress,
      registryAddress,
      contractDeploymentEmitterAddress,
      archiverStore,
      1000,
    );

    let latestBlockNum = await archiver.getBlockNumber();
    expect(latestBlockNum).toEqual(0);

    const block = L2Block.random(1, 4, 1, 2, 4, 6);
    const rollupTx = makeRollupTx(block);
    const publishTx = makePublishTx(block.body);

    publicClient.getBlockNumber.mockResolvedValueOnce(2500n);
    // logs should be created in order of how archiver syncs.
    publicClient.getLogs
      .mockResolvedValueOnce(
        makeL1ToL2MessageAddedEvents(
          100n,
          block.body.l1ToL2Messages.map(x => x.toString()),
        ),
      )
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([makeTxsPublishedEvent(101n, block.body.getTxsEffectsHash())])
      .mockResolvedValueOnce([makeL2BlockProcessedEvent(101n, 1n)])
      .mockResolvedValue([]);
    publicClient.getTransaction.mockResolvedValueOnce(publishTx);
    publicClient.getTransaction.mockResolvedValueOnce(rollupTx);

    await archiver.start(false);

    // Wait until block 1 is processed. If this won't happen the test will fail with timeout.
    while ((await archiver.getBlockNumber()) !== 1) {
      await sleep(100);
    }

    latestBlockNum = await archiver.getBlockNumber();
    expect(latestBlockNum).toEqual(1);

    const expectedL1Messages = block.body.l1ToL2Messages.map(x => x.value);
    const receivedBlock = await archiver.getBlock(1);

    expect(receivedBlock?.body.l1ToL2Messages.map(x => x.value)).toEqual(expectedL1Messages);

    await archiver.stop();
  }, 10_000);
});

/**
 * Makes a fake L2BlockProcessed event for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param l2BlockNum - L2 Block number.
 * @returns An L2BlockProcessed event log.
 */
function makeL2BlockProcessedEvent(l1BlockNum: bigint, l2BlockNum: bigint) {
  return {
    blockNumber: l1BlockNum,
    args: { blockNumber: l2BlockNum },
    transactionHash: `0x${l2BlockNum}`,
  } as Log<bigint, number, false, undefined, true, typeof RollupAbi, 'L2BlockProcessed'>;
}

/**
 * Makes a fake TxsPublished event for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param txsEffectsHash - txsEffectsHash for the body.
 * @returns A TxsPublished event log.
 */
function makeTxsPublishedEvent(l1BlockNum: bigint, txsEffectsHash: Buffer) {
  return {
    blockNumber: l1BlockNum,
    args: {
      txsEffectsHash: txsEffectsHash.toString('hex'),
    },
  } as Log<bigint, number, false, undefined, true, typeof AvailabilityOracleAbi, 'TxsPublished'>;
}

/**
 * Makes a fake ContractDeployment event for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param l2Block - The l2Block this event is associated with.
 * @returns An ContractDeployment event.
 */
function makeContractDeploymentEvent(l1BlockNum: bigint, l2Block: L2Block) {
  const extendedContractData = ExtendedContractData.random();
  const acir = extendedContractData.bytecode?.toString('hex');
  return {
    blockNumber: l1BlockNum,
    args: {
      l2BlockNum: BigInt(l2Block.number),
      aztecAddress: extendedContractData.contractData.contractAddress.toString(),
      portalAddress: extendedContractData.contractData.portalContractAddress.toString(),
      l2BlockHash: `0x${l2Block.body.getTxsEffectsHash().toString('hex')}`,
      contractClassId: extendedContractData.contractClassId.toString(),
      saltedInitializationHash: extendedContractData.saltedInitializationHash.toString(),
      publicKeyHash: extendedContractData.publicKeyHash.toString(),
      acir: '0x' + acir,
    },
    transactionHash: `0x${l2Block.number}`,
  } as Log<bigint, number, false, undefined, true, typeof ContractDeploymentEmitterAbi, 'ContractDeployment'>;
}

/**
 * Makes fake L1ToL2 MessageAdded events for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param entryKeys - The entry keys of the messages to add.
 * @returns MessageAdded event logs.
 */
function makeL1ToL2MessageAddedEvents(l1BlockNum: bigint, entryKeys: string[]) {
  return entryKeys.map(entryKey => {
    return {
      blockNumber: l1BlockNum,
      args: {
        sender: EthAddress.random().toString(),
        senderChainId: 1n,
        recipient: AztecAddress.random().toString(),
        recipientVersion: 1n,
        content: Fr.random().toString(),
        secretHash: Fr.random().toString(),
        deadline: 100,
        fee: 1n,
        entryKey: entryKey,
      },
      transactionHash: `0x${l1BlockNum}`,
    } as Log<bigint, number, false, undefined, true, typeof InboxAbi, 'MessageAdded'>;
  });
}

/**
 * Makes fake L1ToL2 MessageCancelled events for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param entryKey - The entry keys of the message to cancel.
 * @returns MessageCancelled event logs.
 */
function makeL1ToL2MessageCancelledEvents(l1BlockNum: bigint, entryKeys: string[]) {
  return entryKeys.map(entryKey => {
    return {
      blockNumber: l1BlockNum,
      args: {
        entryKey,
      },
      transactionHash: `0x${l1BlockNum}`,
    } as Log<bigint, number, false, undefined, true, typeof InboxAbi, 'L1ToL2MessageCancelled'>;
  });
}

/**
 * Makes fake L1ToL2 LeafInserted events for testing purposes.
 * @param l1BlockNum - L1 block number.
 * @param l2BlockNumber - The L2 block number of the leaf inserted.
 * @returns LeafInserted event logs.
 */
function makeLeafInsertedEvent(l1BlockNum: bigint, l2BlockNumber: bigint, index: bigint) {
  return {
    blockNumber: l1BlockNum,
    args: {
      blockNumber: l2BlockNumber,
      index,
      value: Fr.random().toString(),
    },
    transactionHash: `0x${l1BlockNum}`,
  } as Log<bigint, number, false, undefined, true, typeof NewInboxAbi, 'LeafInserted'>;
}

/**
 * Makes a fake rollup tx for testing purposes.
 * @param block - The L2Block.
 * @returns A fake tx with calldata that corresponds to calling process in the Rollup contract.
 */
function makeRollupTx(l2Block: L2Block) {
  const header = toHex(l2Block.header.toBuffer());
  const archive = toHex(l2Block.archive.root.toBuffer());
  const body = toHex(l2Block.body.toBuffer());
  const proof = `0x`;
  const input = encodeFunctionData({
    abi: RollupAbi,
    functionName: 'process',
    args: [header, archive, body, proof],
  });
  return { input } as Transaction<bigint, number>;
}

/**
 * Makes a fake availability oracle tx for testing purposes.
 * @param blockBody - The block body posted by the simulated tx.
 * @returns A fake tx with calldata that corresponds to calling publish in the Availability Oracle contract.
 */
function makePublishTx(blockBody: Body) {
  const body = toHex(blockBody.toBuffer());
  const input = encodeFunctionData({
    abi: AvailabilityOracleAbi,
    functionName: 'publish',
    args: [body],
  });
  return { input } as Transaction<bigint, number>;
}
