import { InboxLeaf, L2Block, LogId, LogType, TxHash } from '@aztec/circuit-types';
import '@aztec/circuit-types/jest';
import { AztecAddress, Fr, INITIAL_L2_BLOCK_NUM, L1_TO_L2_MSG_SUBTREE_HEIGHT } from '@aztec/circuits.js';
import {
  makeContractClassPublic,
  makeExecutablePrivateFunctionWithMembershipProof,
  makeUnconstrainedFunctionWithMembershipProof,
} from '@aztec/circuits.js/testing';
import { times } from '@aztec/foundation/collection';
import { randomBytes, randomInt } from '@aztec/foundation/crypto';
import {
  type ContractClassPublic,
  type ContractInstanceWithAddress,
  SerializableContractInstance,
} from '@aztec/types/contracts';

import { type ArchiverDataStore } from './archiver_store.js';
import { type DataRetrieval } from './data_retrieval.js';

/**
 * @param testName - The name of the test suite.
 * @param getStore - Returns an instance of a store that's already been initialized.
 */
export function describeArchiverDataStore(testName: string, getStore: () => ArchiverDataStore) {
  describe(testName, () => {
    let store: ArchiverDataStore;
    let blocks: DataRetrieval<L2Block>;
    const blockTests: [number, number, () => L2Block[]][] = [
      [1, 1, () => blocks.retrievedData.slice(0, 1)],
      [10, 1, () => blocks.retrievedData.slice(9, 10)],
      [1, 10, () => blocks.retrievedData.slice(0, 10)],
      [2, 5, () => blocks.retrievedData.slice(1, 6)],
      [5, 2, () => blocks.retrievedData.slice(4, 6)],
    ];

    beforeEach(() => {
      store = getStore();
      blocks = {
        lastProcessedL1BlockNumber: 5n,
        retrievedData: Array.from({ length: 10 }).map((_, i) => L2Block.random(i + 1)),
      };
    });

    describe('addBlocks', () => {
      it('returns success when adding block bodies', async () => {
        await expect(store.addBlockBodies(blocks.retrievedData.map(block => block.body))).resolves.toBe(true);
      });

      it('returns success when adding blocks', async () => {
        await expect(store.addBlocks(blocks)).resolves.toBe(true);
      });

      it('allows duplicate blocks', async () => {
        await store.addBlocks(blocks);
        await expect(store.addBlocks(blocks)).resolves.toBe(true);
      });
    });

    describe('getBlocks', () => {
      beforeEach(async () => {
        await store.addBlocks(blocks);
        await store.addBlockBodies(blocks.retrievedData.map(block => block.body));
      });

      it.each(blockTests)('retrieves previously stored blocks', async (start, limit, getExpectedBlocks) => {
        await expect(store.getBlocks(start, limit)).resolves.toEqual(getExpectedBlocks());
      });

      it('returns an empty array if no blocks are found', async () => {
        await expect(store.getBlocks(12, 1)).resolves.toEqual([]);
      });

      it('throws an error if limit is invalid', async () => {
        await expect(store.getBlocks(1, 0)).rejects.toThrow('Invalid limit: 0');
      });

      it('resets `from` to the first block if it is out of range', async () => {
        await expect(store.getBlocks(INITIAL_L2_BLOCK_NUM - 100, 1)).resolves.toEqual(blocks.retrievedData.slice(0, 1));
      });
    });

    describe('getSyncedL2BlockNumber', () => {
      it('returns the block number before INITIAL_L2_BLOCK_NUM if no blocks have been added', async () => {
        await expect(store.getSynchedL2BlockNumber()).resolves.toEqual(INITIAL_L2_BLOCK_NUM - 1);
      });

      it("returns the most recently added block's number", async () => {
        await store.addBlocks(blocks);
        await expect(store.getSynchedL2BlockNumber()).resolves.toEqual(blocks.retrievedData.at(-1)!.number);
      });
    });

    describe('getSynchPoint', () => {
      it('returns 0n if no blocks have been added', async () => {
        await expect(store.getSynchPoint()).resolves.toEqual({
          blocksSynchedTo: 0n,
          messagesSynchedTo: 0n,
        });
      });

      it('returns the L1 block number in which the most recent L2 block was published', async () => {
        await store.addBlocks(blocks);
        await expect(store.getSynchPoint()).resolves.toEqual({
          blocksSynchedTo: blocks.lastProcessedL1BlockNumber,
          messagesSynchedTo: 0n,
        });
      });

      it('returns the L1 block number that most recently added messages from inbox', async () => {
        await store.addL1ToL2Messages({
          lastProcessedL1BlockNumber: 1n,
          retrievedData: [new InboxLeaf(0n, 0n, Fr.ZERO)],
        });
        await expect(store.getSynchPoint()).resolves.toEqual({
          blocksSynchedTo: 0n,
          messagesSynchedTo: 1n,
        });
      });
    });

    describe('addLogs', () => {
      it('adds encrypted & unencrypted logs', async () => {
        await expect(
          store.addLogs(
            blocks.retrievedData[0].body.noteEncryptedLogs,
            blocks.retrievedData[0].body.encryptedLogs,
            blocks.retrievedData[0].body.unencryptedLogs,
            blocks.retrievedData[0].number,
          ),
        ).resolves.toEqual(true);
      });
    });

    describe.each([
      ['note_encrypted', LogType.NOTEENCRYPTED],
      ['encrypted', LogType.ENCRYPTED],
      ['unencrypted', LogType.UNENCRYPTED],
    ])('getLogs (%s)', (_, logType) => {
      beforeEach(async () => {
        await Promise.all(
          blocks.retrievedData.map(block =>
            store.addLogs(
              block.body.noteEncryptedLogs,
              block.body.encryptedLogs,
              block.body.unencryptedLogs,
              block.number,
            ),
          ),
        );
      });

      it.each(blockTests)('retrieves previously stored logs', async (from, limit, getExpectedBlocks) => {
        const expectedLogs = getExpectedBlocks().map(block => {
          switch (logType) {
            case LogType.ENCRYPTED:
              return block.body.encryptedLogs;
            case LogType.NOTEENCRYPTED:
              return block.body.noteEncryptedLogs;
            case LogType.UNENCRYPTED:
            default:
              return block.body.unencryptedLogs;
          }
        });
        const actualLogs = await store.getLogs(from, limit, logType);
        expect(actualLogs[0].txLogs[0]).toEqual(expectedLogs[0].txLogs[0]);
      });
    });

    describe('getTxEffect', () => {
      beforeEach(async () => {
        await Promise.all(
          blocks.retrievedData.map(block =>
            store.addLogs(
              block.body.noteEncryptedLogs,
              block.body.encryptedLogs,
              block.body.unencryptedLogs,
              block.number,
            ),
          ),
        );
        await store.addBlocks(blocks);
        await store.addBlockBodies(blocks.retrievedData.map(block => block.body));
      });

      it.each([
        () => blocks.retrievedData[0].body.txEffects[0],
        () => blocks.retrievedData[9].body.txEffects[3],
        () => blocks.retrievedData[3].body.txEffects[1],
        () => blocks.retrievedData[5].body.txEffects[2],
        () => blocks.retrievedData[1].body.txEffects[0],
      ])('retrieves a previously stored transaction', async getExpectedTx => {
        const expectedTx = getExpectedTx();
        const actualTx = await store.getTxEffect(expectedTx.txHash);
        expect(actualTx).toEqual(expectedTx);
      });

      it('returns undefined if tx is not found', async () => {
        await expect(store.getTxEffect(new TxHash(Fr.random().toBuffer()))).resolves.toBeUndefined();
      });
    });

    describe('L1 to L2 Messages', () => {
      const l2BlockNumber = 13n;
      const l1ToL2MessageSubtreeSize = 2 ** L1_TO_L2_MSG_SUBTREE_HEIGHT;

      const generateBlockMessages = (blockNumber: bigint, numMessages: number) =>
        Array.from({ length: numMessages }, (_, i) => new InboxLeaf(blockNumber, BigInt(i), Fr.random()));

      it('returns messages in correct order', async () => {
        const msgs = generateBlockMessages(l2BlockNumber, l1ToL2MessageSubtreeSize);
        const shuffledMessages = msgs.slice().sort(() => randomInt(1) - 0.5);
        await store.addL1ToL2Messages({ lastProcessedL1BlockNumber: 100n, retrievedData: shuffledMessages });
        const retrievedMessages = await store.getL1ToL2Messages(l2BlockNumber);

        const expectedLeavesOrder = msgs.map(msg => msg.leaf);
        expect(expectedLeavesOrder).toEqual(retrievedMessages);
      });

      it('throws if it is impossible to sequence messages correctly', async () => {
        const msgs = generateBlockMessages(l2BlockNumber, l1ToL2MessageSubtreeSize - 1);
        // We replace a message with index 4 with a message with index at the end of the tree
        // --> with that there will be a gap and it will be impossible to sequence the messages
        msgs[4] = new InboxLeaf(l2BlockNumber, BigInt(l1ToL2MessageSubtreeSize - 1), Fr.random());

        await store.addL1ToL2Messages({ lastProcessedL1BlockNumber: 100n, retrievedData: msgs });
        await expect(async () => {
          await store.getL1ToL2Messages(l2BlockNumber);
        }).rejects.toThrow(`L1 to L2 message gap found in block ${l2BlockNumber}`);
      });

      it('throws if adding more messages than fits into a block', async () => {
        const msgs = generateBlockMessages(l2BlockNumber, l1ToL2MessageSubtreeSize + 1);

        await expect(async () => {
          await store.addL1ToL2Messages({ lastProcessedL1BlockNumber: 100n, retrievedData: msgs });
        }).rejects.toThrow(`Message index ${l1ToL2MessageSubtreeSize} out of subtree range`);
      });

      it('correctly handles duplicate messages', async () => {
        const messageHash = Fr.random();

        const msgs = [new InboxLeaf(1n, 0n, messageHash), new InboxLeaf(2n, 0n, messageHash)];

        await store.addL1ToL2Messages({ lastProcessedL1BlockNumber: 100n, retrievedData: msgs });

        const index1 = (await store.getL1ToL2MessageIndex(messageHash, 0n))!;
        const index2 = await store.getL1ToL2MessageIndex(messageHash, index1 + 1n);

        expect(index2).toBeDefined();
        expect(index2).toBeGreaterThan(index1);
      });
    });

    describe('contractInstances', () => {
      let contractInstance: ContractInstanceWithAddress;
      const blockNum = 10;

      beforeEach(async () => {
        contractInstance = { ...SerializableContractInstance.random(), address: AztecAddress.random() };
        await store.addContractInstances([contractInstance], blockNum);
      });

      it('returns previously stored contract instances', async () => {
        await expect(store.getContractInstance(contractInstance.address)).resolves.toMatchObject(contractInstance);
      });

      it('returns undefined if contract instance is not found', async () => {
        await expect(store.getContractInstance(AztecAddress.random())).resolves.toBeUndefined();
      });
    });

    describe('contractClasses', () => {
      let contractClass: ContractClassPublic;
      const blockNum = 10;

      beforeEach(async () => {
        contractClass = makeContractClassPublic();
        await store.addContractClasses([contractClass], blockNum);
      });

      it('returns previously stored contract class', async () => {
        await expect(store.getContractClass(contractClass.id)).resolves.toMatchObject(contractClass);
      });

      it('returns undefined if contract class is not found', async () => {
        await expect(store.getContractClass(Fr.random())).resolves.toBeUndefined();
      });

      it('adds new private functions', async () => {
        const fns = times(3, makeExecutablePrivateFunctionWithMembershipProof);
        await store.addFunctions(contractClass.id, fns, []);
        const stored = await store.getContractClass(contractClass.id);
        expect(stored?.privateFunctions).toEqual(fns);
      });

      it('does not duplicate private functions', async () => {
        const fns = times(3, makeExecutablePrivateFunctionWithMembershipProof);
        await store.addFunctions(contractClass.id, fns.slice(0, 1), []);
        await store.addFunctions(contractClass.id, fns, []);
        const stored = await store.getContractClass(contractClass.id);
        expect(stored?.privateFunctions).toEqual(fns);
      });

      it('adds new unconstrained functions', async () => {
        const fns = times(3, makeUnconstrainedFunctionWithMembershipProof);
        await store.addFunctions(contractClass.id, [], fns);
        const stored = await store.getContractClass(contractClass.id);
        expect(stored?.unconstrainedFunctions).toEqual(fns);
      });

      it('does not duplicate unconstrained functions', async () => {
        const fns = times(3, makeUnconstrainedFunctionWithMembershipProof);
        await store.addFunctions(contractClass.id, [], fns.slice(0, 1));
        await store.addFunctions(contractClass.id, [], fns);
        const stored = await store.getContractClass(contractClass.id);
        expect(stored?.unconstrainedFunctions).toEqual(fns);
      });
    });

    describe('getUnencryptedLogs', () => {
      const txsPerBlock = 4;
      const numPublicFunctionCalls = 3;
      const numUnencryptedLogs = 2;
      const numBlocks = 10;
      let blocks: DataRetrieval<L2Block>;

      beforeEach(async () => {
        blocks = {
          lastProcessedL1BlockNumber: 4n,
          retrievedData: Array(numBlocks)
            .fill(0)
            .map((_, index: number) =>
              L2Block.random(index + 1, txsPerBlock, 2, numPublicFunctionCalls, 2, numUnencryptedLogs),
            ),
        };

        await store.addBlocks(blocks);
        await store.addBlockBodies(blocks.retrievedData.map(block => block.body));

        await Promise.all(
          blocks.retrievedData.map(block =>
            store.addLogs(
              block.body.noteEncryptedLogs,
              block.body.encryptedLogs,
              block.body.unencryptedLogs,
              block.number,
            ),
          ),
        );
      });

      it('"txHash" filter param is respected', async () => {
        // get random tx
        const targetBlockIndex = randomInt(numBlocks);
        const targetTxIndex = randomInt(txsPerBlock);
        const targetTxHash = blocks.retrievedData[targetBlockIndex].body.txEffects[targetTxIndex].txHash;

        const response = await store.getUnencryptedLogs({ txHash: targetTxHash });
        const logs = response.logs;

        expect(response.maxLogsHit).toBeFalsy();

        const expectedNumLogs = numPublicFunctionCalls * numUnencryptedLogs;
        expect(logs.length).toEqual(expectedNumLogs);

        const targeBlockNumber = targetBlockIndex + INITIAL_L2_BLOCK_NUM;
        for (const log of logs) {
          expect(log.id.blockNumber).toEqual(targeBlockNumber);
          expect(log.id.txIndex).toEqual(targetTxIndex);
        }
      });

      it('"fromBlock" and "toBlock" filter params are respected', async () => {
        // Set "fromBlock" and "toBlock"
        const fromBlock = 3;
        const toBlock = 7;

        const response = await store.getUnencryptedLogs({ fromBlock, toBlock });
        const logs = response.logs;

        expect(response.maxLogsHit).toBeFalsy();

        const expectedNumLogs = txsPerBlock * numPublicFunctionCalls * numUnencryptedLogs * (toBlock - fromBlock);
        expect(logs.length).toEqual(expectedNumLogs);

        for (const log of logs) {
          const blockNumber = log.id.blockNumber;
          expect(blockNumber).toBeGreaterThanOrEqual(fromBlock);
          expect(blockNumber).toBeLessThan(toBlock);
        }
      });

      it('"contractAddress" filter param is respected', async () => {
        // Get a random contract address from the logs
        const targetBlockIndex = randomInt(numBlocks);
        const targetTxIndex = randomInt(txsPerBlock);
        const targetFunctionLogIndex = randomInt(numPublicFunctionCalls);
        const targetLogIndex = randomInt(numUnencryptedLogs);
        const targetContractAddress =
          blocks.retrievedData[targetBlockIndex].body.txEffects[targetTxIndex].unencryptedLogs.functionLogs[
            targetFunctionLogIndex
          ].logs[targetLogIndex].contractAddress;

        const response = await store.getUnencryptedLogs({ contractAddress: targetContractAddress });

        expect(response.maxLogsHit).toBeFalsy();

        for (const extendedLog of response.logs) {
          expect(extendedLog.log.contractAddress.equals(targetContractAddress)).toBeTruthy();
        }
      });

      it('"afterLog" filter param is respected', async () => {
        // Get a random log as reference
        const targetBlockIndex = randomInt(numBlocks);
        const targetTxIndex = randomInt(txsPerBlock);
        const targetLogIndex = randomInt(numUnencryptedLogs);

        const afterLog = new LogId(targetBlockIndex + INITIAL_L2_BLOCK_NUM, targetTxIndex, targetLogIndex);

        const response = await store.getUnencryptedLogs({ afterLog });
        const logs = response.logs;

        expect(response.maxLogsHit).toBeFalsy();

        for (const log of logs) {
          const logId = log.id;
          expect(logId.blockNumber).toBeGreaterThanOrEqual(afterLog.blockNumber);
          if (logId.blockNumber === afterLog.blockNumber) {
            expect(logId.txIndex).toBeGreaterThanOrEqual(afterLog.txIndex);
            if (logId.txIndex === afterLog.txIndex) {
              expect(logId.logIndex).toBeGreaterThan(afterLog.logIndex);
            }
          }
        }
      });

      it('"txHash" filter param is ignored when "afterLog" is set', async () => {
        // Get random txHash
        const txHash = new TxHash(randomBytes(TxHash.SIZE));
        const afterLog = new LogId(1, 0, 0);

        const response = await store.getUnencryptedLogs({ txHash, afterLog });
        expect(response.logs.length).toBeGreaterThan(1);
      });

      it('intersecting works', async () => {
        let logs = (await store.getUnencryptedLogs({ fromBlock: -10, toBlock: -5 })).logs;
        expect(logs.length).toBe(0);

        // "fromBlock" gets correctly trimmed to range and "toBlock" is exclusive
        logs = (await store.getUnencryptedLogs({ fromBlock: -10, toBlock: 5 })).logs;
        let blockNumbers = new Set(logs.map(log => log.id.blockNumber));
        expect(blockNumbers).toEqual(new Set([1, 2, 3, 4]));

        // "toBlock" should be exclusive
        logs = (await store.getUnencryptedLogs({ fromBlock: 1, toBlock: 1 })).logs;
        expect(logs.length).toBe(0);

        logs = (await store.getUnencryptedLogs({ fromBlock: 10, toBlock: 5 })).logs;
        expect(logs.length).toBe(0);

        // both "fromBlock" and "toBlock" get correctly capped to range and logs from all blocks are returned
        logs = (await store.getUnencryptedLogs({ fromBlock: -100, toBlock: +100 })).logs;
        blockNumbers = new Set(logs.map(log => log.id.blockNumber));
        expect(blockNumbers.size).toBe(numBlocks);

        // intersecting with "afterLog" works
        logs = (await store.getUnencryptedLogs({ fromBlock: 2, toBlock: 5, afterLog: new LogId(4, 0, 0) })).logs;
        blockNumbers = new Set(logs.map(log => log.id.blockNumber));
        expect(blockNumbers).toEqual(new Set([4]));

        logs = (await store.getUnencryptedLogs({ toBlock: 5, afterLog: new LogId(5, 1, 0) })).logs;
        expect(logs.length).toBe(0);

        logs = (await store.getUnencryptedLogs({ fromBlock: 2, toBlock: 5, afterLog: new LogId(100, 0, 0) })).logs;
        expect(logs.length).toBe(0);
      });

      it('"txIndex" and "logIndex" are respected when "afterLog.blockNumber" is equal to "fromBlock"', async () => {
        // Get a random log as reference
        const targetBlockIndex = randomInt(numBlocks);
        const targetTxIndex = randomInt(txsPerBlock);
        const targetLogIndex = randomInt(numUnencryptedLogs);

        const afterLog = new LogId(targetBlockIndex + INITIAL_L2_BLOCK_NUM, targetTxIndex, targetLogIndex);

        const response = await store.getUnencryptedLogs({ afterLog, fromBlock: afterLog.blockNumber });
        const logs = response.logs;

        expect(response.maxLogsHit).toBeFalsy();

        for (const log of logs) {
          const logId = log.id;
          expect(logId.blockNumber).toBeGreaterThanOrEqual(afterLog.blockNumber);
          if (logId.blockNumber === afterLog.blockNumber) {
            expect(logId.txIndex).toBeGreaterThanOrEqual(afterLog.txIndex);
            if (logId.txIndex === afterLog.txIndex) {
              expect(logId.logIndex).toBeGreaterThan(afterLog.logIndex);
            }
          }
        }
      });
    });
  });
}
