import {
  ExtendedContractData,
  INITIAL_L2_BLOCK_NUM,
  L1ToL2Message,
  L2Block,
  L2BlockContext,
  LogId,
  LogType,
  TxHash,
  UnencryptedL2Log,
} from '@aztec/circuit-types';
import '@aztec/circuit-types/jest';
import { AztecAddress, Fr } from '@aztec/circuits.js';
import { randomBytes } from '@aztec/foundation/crypto';
import {
  ContractClassWithId,
  ContractInstanceWithAddress,
  SerializableContractClass,
  SerializableContractInstance,
} from '@aztec/types/contracts';

import { ArchiverDataStore } from './archiver_store.js';

/**
 * @param testName - The name of the test suite.
 * @param getStore - Returns an instance of a store that's already been initialized.
 */
export function describeArchiverDataStore(testName: string, getStore: () => ArchiverDataStore) {
  describe(testName, () => {
    let store: ArchiverDataStore;
    let blocks: L2Block[];
    const blockTests: [number, number, () => L2Block[]][] = [
      [1, 1, () => blocks.slice(0, 1)],
      [10, 1, () => blocks.slice(9, 10)],
      [1, 10, () => blocks.slice(0, 10)],
      [2, 5, () => blocks.slice(1, 6)],
      [5, 2, () => blocks.slice(4, 6)],
    ];

    beforeEach(() => {
      store = getStore();
      blocks = Array.from({ length: 10 }).map((_, i) => {
        const block = L2Block.random(i + 1);
        block.setL1BlockNumber(BigInt(i + 1));
        return block;
      });
    });

    describe('addBlocks', () => {
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
      });

      it.each(blockTests)('retrieves previously stored blocks', async (start, limit, getExpectedBlocks) => {
        await expect(store.getBlocks(start, limit)).resolves.toEqual(getExpectedBlocks());
      });

      it('returns an empty array if no blocks are found', async () => {
        await expect(store.getBlocks(12, 1)).resolves.toEqual([]);
      });

      it('throws an error if limit is invalid', async () => {
        await expect(store.getBlocks(1, 0)).rejects.toThrowError('Invalid limit: 0');
      });

      it('resets `from` to the first block if it is out of range', async () => {
        await expect(store.getBlocks(INITIAL_L2_BLOCK_NUM - 100, 1)).resolves.toEqual(blocks.slice(0, 1));
      });
    });

    describe('getBlockNumber', () => {
      it('returns the block number before INITIAL_L2_BLOCK_NUM if no blocks have been added', async () => {
        await expect(store.getBlockNumber()).resolves.toEqual(INITIAL_L2_BLOCK_NUM - 1);
      });

      it("returns the most recently added block's number", async () => {
        await store.addBlocks(blocks);
        await expect(store.getBlockNumber()).resolves.toEqual(blocks.at(-1)!.number);
      });
    });

    describe('getL1BlockNumber', () => {
      it('returns 0n if no blocks have been added', async () => {
        await expect(store.getL1BlockNumber()).resolves.toEqual({
          addedBlock: 0n,
          addedMessages: 0n,
          cancelledMessages: 0n,
        });
      });

      it('returns the L1 block number in which the most recent L2 block was published', async () => {
        await store.addBlocks(blocks);
        await expect(store.getL1BlockNumber()).resolves.toEqual({
          addedBlock: blocks.at(-1)!.getL1BlockNumber(),
          addedMessages: 0n,
          cancelledMessages: 0n,
        });
      });

      it('returns the L1 block number that most recently added pending messages', async () => {
        await store.addPendingL1ToL2Messages([L1ToL2Message.random(Fr.random())], 1n);
        await expect(store.getL1BlockNumber()).resolves.toEqual({
          addedBlock: 0n,
          addedMessages: 1n,
          cancelledMessages: 0n,
        });
      });
      it('returns the L1 block number that most recently cancelled pending messages', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message], 1n);
        await store.cancelPendingL1ToL2Messages([message.entryKey!], 2n);
        await expect(store.getL1BlockNumber()).resolves.toEqual({
          addedBlock: 0n,
          addedMessages: 1n,
          cancelledMessages: 2n,
        });
      });
    });

    describe('addLogs', () => {
      it('adds encrypted & unencrypted logs', async () => {
        await expect(
          store.addLogs(blocks[0].newEncryptedLogs, blocks[0].newUnencryptedLogs, blocks[0].number),
        ).resolves.toEqual(true);
      });
    });

    describe.each([
      ['encrypted', LogType.ENCRYPTED],
      ['unencrypted', LogType.UNENCRYPTED],
    ])('getLogs (%s)', (_, logType) => {
      beforeEach(async () => {
        await Promise.all(
          blocks.map(block => store.addLogs(block.newEncryptedLogs, block.newUnencryptedLogs, block.number)),
        );
      });

      it.each(blockTests)('retrieves previously stored logs', async (from, limit, getExpectedBlocks) => {
        const expectedLogs = getExpectedBlocks().map(block =>
          logType === LogType.ENCRYPTED ? block.newEncryptedLogs : block.newUnencryptedLogs,
        );
        const actualLogs = await store.getLogs(from, limit, logType);
        expect(actualLogs).toEqual(expectedLogs);
      });
    });

    describe('getL2Tx', () => {
      beforeEach(async () => {
        await Promise.all(
          blocks.map(block => store.addLogs(block.newEncryptedLogs, block.newUnencryptedLogs, block.number)),
        );
        await store.addBlocks(blocks);
      });

      it.each([
        () => blocks[0].getTx(0),
        () => blocks[9].getTx(3),
        () => blocks[3].getTx(1),
        () => blocks[5].getTx(2),
        () => blocks[1].getTx(0),
      ])('retrieves a previously stored transaction', async getExpectedTx => {
        const expectedTx = getExpectedTx();
        const actualTx = await store.getL2Tx(expectedTx.txHash);
        expect(actualTx).toEqual(expectedTx);
      });

      it('returns undefined if tx is not found', async () => {
        await expect(store.getL2Tx(new TxHash(Fr.random().toBuffer()))).resolves.toBeUndefined();
      });
    });

    describe('addPendingL1ToL2Messages', () => {
      it('stores pending L1 to L2 messages', async () => {
        await expect(store.addPendingL1ToL2Messages([L1ToL2Message.random(Fr.random())], 1n)).resolves.toEqual(true);
      });

      it('allows duplicate pending messages in different positions in the same block', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await expect(store.addPendingL1ToL2Messages([message, message], 1n)).resolves.toEqual(true);

        await expect(store.getPendingL1ToL2MessageKeys(2)).resolves.toEqual([message.entryKey!, message.entryKey!]);
      });

      it('allows duplicate pending messages in different blocks', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await expect(store.addPendingL1ToL2Messages([message], 1n)).resolves.toEqual(true);
        await expect(store.addPendingL1ToL2Messages([message], 2n)).resolves.toEqual(true);

        await expect(store.getPendingL1ToL2MessageKeys(2)).resolves.toEqual([message.entryKey!, message.entryKey!]);
      });

      it('is idempotent', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await expect(store.addPendingL1ToL2Messages([message], 1n)).resolves.toEqual(true);
        await expect(store.addPendingL1ToL2Messages([message], 1n)).resolves.toEqual(false);
        await expect(store.getPendingL1ToL2MessageKeys(2)).resolves.toEqual([message.entryKey!]);
      });
    });

    describe('getPendingL1ToL2Messages', () => {
      it('returns previously stored pending L1 to L2 messages', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message], 1n);
        await expect(store.getPendingL1ToL2MessageKeys(1)).resolves.toEqual([message.entryKey!]);
      });

      it('returns messages ordered by fee', async () => {
        const messages = Array.from({ length: 3 }, () => L1ToL2Message.random(Fr.random()));
        // add a duplicate message
        messages.push(messages[0]);

        await store.addPendingL1ToL2Messages(messages, 1n);

        messages.sort((a, b) => b.fee - a.fee);
        await expect(store.getPendingL1ToL2MessageKeys(messages.length)).resolves.toEqual(
          messages.map(message => message.entryKey!),
        );
      });

      it('returns an empty array if no messages are found', async () => {
        await expect(store.getPendingL1ToL2MessageKeys(1)).resolves.toEqual([]);
      });
    });

    describe('confirmL1ToL2Messages', () => {
      it('updates a message from pending to confirmed', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message], 1n);
        await expect(store.confirmL1ToL2Messages([message.entryKey!])).resolves.toEqual(true);
      });

      it('once confirmed, a message is no longer pending', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message], 1n);
        await store.confirmL1ToL2Messages([message.entryKey!]);
        await expect(store.getPendingL1ToL2MessageKeys(1)).resolves.toEqual([]);
      });

      it('once confirmed a message can also be pending if added again', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message], 1n);
        await store.confirmL1ToL2Messages([message.entryKey!]);
        await store.addPendingL1ToL2Messages([message], 2n);
        await expect(store.getPendingL1ToL2MessageKeys(2)).resolves.toEqual([message.entryKey!]);
      });

      it('once confirmed a message can remain pending if more of it were pending', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message, message], 1n);
        await store.confirmL1ToL2Messages([message.entryKey!]);
        await expect(store.getPendingL1ToL2MessageKeys(1)).resolves.toEqual([message.entryKey!]);
      });
    });

    describe('cancelL1ToL2Messages', () => {
      it('cancels a pending message', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message], 1n);
        await store.cancelPendingL1ToL2Messages([message.entryKey!], 1n);
        await expect(store.getPendingL1ToL2MessageKeys(1)).resolves.toEqual([]);
      });

      it('cancels only one of the pending messages if duplicates exist', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message, message], 1n);
        await store.cancelPendingL1ToL2Messages([message.entryKey!], 1n);
        await expect(store.getPendingL1ToL2MessageKeys(2)).resolves.toEqual([message.entryKey]);
      });

      it('once canceled a message can also be pending if added again', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message], 1n);

        await store.cancelPendingL1ToL2Messages([message.entryKey!], 1n);
        await expect(store.getPendingL1ToL2MessageKeys(1)).resolves.toEqual([]);

        await store.addPendingL1ToL2Messages([message], 2n);
        await expect(store.getPendingL1ToL2MessageKeys(1)).resolves.toEqual([message.entryKey!]);
      });

      it('allows adding and cancelling in the same block', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message], 1n);
        await store.cancelPendingL1ToL2Messages([message.entryKey!], 1n);
        await expect(store.getPendingL1ToL2MessageKeys(1)).resolves.toEqual([]);
      });

      it('allows duplicates cancellations in different positions in the same block', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message, message], 1n);

        await store.cancelPendingL1ToL2Messages([message.entryKey!, message.entryKey!], 1n);

        await expect(store.getPendingL1ToL2MessageKeys(2)).resolves.toEqual([]);
      });

      it('allows duplicates cancellations in different blocks', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message, message], 1n);

        await store.cancelPendingL1ToL2Messages([message.entryKey!], 2n);
        await store.cancelPendingL1ToL2Messages([message.entryKey!], 3n);

        await expect(store.getPendingL1ToL2MessageKeys(2)).resolves.toEqual([]);
      });

      it('is idempotent', async () => {
        const message = L1ToL2Message.random(Fr.random());
        await store.addPendingL1ToL2Messages([message, message], 1n);

        await store.cancelPendingL1ToL2Messages([message.entryKey!], 2n);
        await store.cancelPendingL1ToL2Messages([message.entryKey!], 2n);

        await expect(store.getPendingL1ToL2MessageKeys(2)).resolves.toEqual([message.entryKey!]);
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
      let contractClass: ContractClassWithId;
      const blockNum = 10;

      beforeEach(async () => {
        contractClass = { ...SerializableContractClass.random(), id: Fr.random() };
        await store.addContractClasses([contractClass], blockNum);
      });

      it('returns previously stored contract class', async () => {
        await expect(store.getContractClass(contractClass.id)).resolves.toMatchObject(contractClass);
      });

      it('returns undefined if contract class is not found', async () => {
        await expect(store.getContractClass(Fr.random())).resolves.toBeUndefined();
      });
    });

    describe('getContractData', () => {
      let block: L2Block;
      beforeEach(async () => {
        block = L2Block.random(1);
        await store.addBlocks([block]);
      });

      it('returns previously stored contract data', async () => {
        await expect(store.getContractData(block.newContractData[0].contractAddress)).resolves.toEqual(
          block.newContractData[0],
        );
      });

      it('returns undefined if contract data is not found', async () => {
        await expect(store.getContractData(AztecAddress.random())).resolves.toBeUndefined();
      });
    });

    describe('getContractDataInBlock', () => {
      let block: L2Block;
      beforeEach(async () => {
        block = L2Block.random(1);
        await store.addBlocks([block]);
      });

      it('returns the contract data for a known block', async () => {
        await expect(store.getContractDataInBlock(block.number)).resolves.toEqual(block.newContractData);
      });

      it('returns an empty array if contract data is not found', async () => {
        await expect(store.getContractDataInBlock(block.number + 1)).resolves.toEqual([]);
      });
    });

    describe('addExtendedContractData', () => {
      it('stores extended contract data', async () => {
        const block = L2Block.random(1);
        await store.addBlocks([block]);
        await expect(store.addExtendedContractData([ExtendedContractData.random()], block.number)).resolves.toEqual(
          true,
        );
      });

      it('stores extended contract data for an unknown block', async () => {
        await expect(store.addExtendedContractData([ExtendedContractData.random()], 1)).resolves.toEqual(true);
      });

      it('"pushes" extended contract data and does not overwrite', async () => {
        const block = L2Block.random(1);
        await store.addBlocks([block]);

        const firstContract = ExtendedContractData.random(block.newContractData[0]);
        await store.addExtendedContractData([firstContract], block.number);

        const secondContract = ExtendedContractData.random(block.newContractData[1]);
        await store.addExtendedContractData([secondContract], block.number);

        await expect(store.getExtendedContractDataInBlock(block.number)).resolves.toEqual([
          firstContract,
          secondContract,
        ]);
      });
    });

    describe('getExtendedContractData', () => {
      let block: L2Block;
      let extendedContractData: ExtendedContractData;
      beforeEach(async () => {
        block = L2Block.random(1);
        extendedContractData = ExtendedContractData.random(block.newContractData[0]);
        await store.addBlocks([block]);
        await store.addExtendedContractData([extendedContractData], block.number);
      });

      it('returns previously stored extended contract data', async () => {
        await expect(store.getExtendedContractData(extendedContractData.contractData.contractAddress)).resolves.toEqual(
          extendedContractData,
        );
      });

      it('returns undefined if extended contract data is not found', async () => {
        await expect(store.getExtendedContractData(AztecAddress.random())).resolves.toBeUndefined();
      });
    });

    describe('getExtendedContractDataInBlock', () => {
      let block: L2Block;
      let extendedContractData: ExtendedContractData;
      beforeEach(async () => {
        block = L2Block.random(1);
        extendedContractData = ExtendedContractData.random(block.newContractData[0]);
        await store.addBlocks([block]);
        await store.addExtendedContractData([extendedContractData], block.number);
      });

      it('returns previously stored extended contract data', async () => {
        await expect(store.getExtendedContractDataInBlock(block.number)).resolves.toEqual([extendedContractData]);
      });

      it('returns an empty array if extended contract data is not found for the block', async () => {
        await expect(store.getExtendedContractDataInBlock(block.number + 1)).resolves.toEqual([]);
      });
    });

    describe('getUnencryptedLogs', () => {
      const txsPerBlock = 4;
      const numPublicFunctionCalls = 3;
      const numUnencryptedLogs = 4;
      const numBlocks = 10;
      let blocks: L2Block[];

      beforeEach(async () => {
        blocks = Array(numBlocks)
          .fill(0)
          .map((_, index: number) =>
            L2Block.random(index + 1, txsPerBlock, 2, numPublicFunctionCalls, 2, numUnencryptedLogs),
          );

        await store.addBlocks(blocks);
        await Promise.all(
          blocks.map(block => store.addLogs(block.newEncryptedLogs, block.newUnencryptedLogs, block.number)),
        );
      });

      it('"txHash" filter param is respected', async () => {
        // get random tx
        const targetBlockIndex = Math.floor(Math.random() * numBlocks);
        const targetTxIndex = Math.floor(Math.random() * txsPerBlock);
        const targetTxHash = new L2BlockContext(blocks[targetBlockIndex]).getTxHash(targetTxIndex);

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
        const targetBlockIndex = Math.floor(Math.random() * numBlocks);
        const targetTxIndex = Math.floor(Math.random() * txsPerBlock);
        const targetFunctionLogIndex = Math.floor(Math.random() * numPublicFunctionCalls);
        const targetLogIndex = Math.floor(Math.random() * numUnencryptedLogs);
        const targetContractAddress = UnencryptedL2Log.fromBuffer(
          blocks[targetBlockIndex].newUnencryptedLogs!.txLogs[targetTxIndex].functionLogs[targetFunctionLogIndex].logs[
            targetLogIndex
          ],
        ).contractAddress;

        const response = await store.getUnencryptedLogs({ contractAddress: targetContractAddress });

        expect(response.maxLogsHit).toBeFalsy();

        for (const extendedLog of response.logs) {
          expect(extendedLog.log.contractAddress.equals(targetContractAddress)).toBeTruthy();
        }
      });

      it('"selector" filter param is respected', async () => {
        // Get a random selector from the logs
        const targetBlockIndex = Math.floor(Math.random() * numBlocks);
        const targetTxIndex = Math.floor(Math.random() * txsPerBlock);
        const targetFunctionLogIndex = Math.floor(Math.random() * numPublicFunctionCalls);
        const targetLogIndex = Math.floor(Math.random() * numUnencryptedLogs);
        const targetSelector = UnencryptedL2Log.fromBuffer(
          blocks[targetBlockIndex].newUnencryptedLogs!.txLogs[targetTxIndex].functionLogs[targetFunctionLogIndex].logs[
            targetLogIndex
          ],
        ).selector;

        const response = await store.getUnencryptedLogs({ selector: targetSelector });

        expect(response.maxLogsHit).toBeFalsy();

        for (const extendedLog of response.logs) {
          expect(extendedLog.log.selector.equals(targetSelector)).toBeTruthy();
        }
      });

      it('"afterLog" filter param is respected', async () => {
        // Get a random log as reference
        const targetBlockIndex = Math.floor(Math.random() * numBlocks);
        const targetTxIndex = Math.floor(Math.random() * txsPerBlock);
        const targetLogIndex = Math.floor(Math.random() * numUnencryptedLogs);

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
        const targetBlockIndex = Math.floor(Math.random() * numBlocks);
        const targetTxIndex = Math.floor(Math.random() * txsPerBlock);
        const targetLogIndex = Math.floor(Math.random() * numUnencryptedLogs);

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
