import { AcirSimulator } from '@aztec/acir-simulator';
import { CircuitsWasm, Fr, MAX_NEW_COMMITMENTS_PER_TX } from '@aztec/circuits.js';
import { Grumpkin, pedersenCompressInputs } from '@aztec/circuits.js/barretenberg';
import { Point } from '@aztec/foundation/fields';
import { ConstantKeyPair } from '@aztec/key-store';
import {
  AztecNode,
  FunctionL2Logs,
  KeyPair,
  KeyStore,
  L2Block,
  L2BlockContext,
  L2BlockL2Logs,
  NoteSpendingInfo,
  TxL2Logs,
} from '@aztec/types';

import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';

import { Database, MemoryDB } from '../database/index.js';
import { NoteProcessor } from './note_processor.js';

const TXS_PER_BLOCK = 4;

describe('Note Processor', () => {
  let wasm: CircuitsWasm;
  let grumpkin: Grumpkin;
  let database: Database;
  let aztecNode: ReturnType<typeof mock<AztecNode>>;
  let addNoteSpendingInfoBatchSpy: any;
  let noteProcessor: NoteProcessor;
  let owner: KeyPair;
  let keyStore: MockProxy<KeyStore>;
  let simulator: MockProxy<AcirSimulator>;
  const firstBlockNum = 123;
  const numCommitmentsPerBlock = TXS_PER_BLOCK * MAX_NEW_COMMITMENTS_PER_TX;
  const firstBlockDataStartIndex = (firstBlockNum - 1) * numCommitmentsPerBlock;

  const computeMockNoteHash = (preimage: Fr[]) =>
    Fr.fromBuffer(
      pedersenCompressInputs(
        wasm,
        preimage.map(p => p.toBuffer()),
      ),
    );

  // ownedData: [tx1, tx2, ...], the numbers in each tx represents the indices of the note hashes the account owns.
  const createEncryptedLogsAndOwnedNoteSpendingInfo = (newNotes: NoteSpendingInfo[], ownedData: number[][]) => {
    const txLogs: TxL2Logs[] = [];
    const ownedNoteSpendingInfo: NoteSpendingInfo[] = [];
    for (let i = 0; i < TXS_PER_BLOCK; ++i) {
      const ownedDataIndices = ownedData[i] || [];
      if (ownedDataIndices.some(index => index >= MAX_NEW_COMMITMENTS_PER_TX)) {
        throw new Error(`Data index should be less than ${MAX_NEW_COMMITMENTS_PER_TX}.`);
      }

      const logs: FunctionL2Logs[] = [];
      const notesForTx = newNotes.slice(i * MAX_NEW_COMMITMENTS_PER_TX, (i + 1) * MAX_NEW_COMMITMENTS_PER_TX);
      notesForTx.forEach((note, noteIndex) => {
        const isOwner = ownedDataIndices.includes(noteIndex);
        const publicKey = isOwner ? owner.getPublicKey() : Point.random();
        const log = note.toEncryptedBuffer(publicKey, grumpkin);
        // 1 tx containing 1 function invocation containing 1 log
        logs.push(new FunctionL2Logs([log]));
        if (isOwner) {
          ownedNoteSpendingInfo.push(note);
        }
      });
      txLogs.push(new TxL2Logs(logs));
    }

    const encryptedLogs = new L2BlockL2Logs(txLogs);
    return { encryptedLogs, ownedNoteSpendingInfo };
  };

  const mockData = (ownedData: number[][], prependedBlocks = 0, appendedBlocks = 0) => {
    if (ownedData.length > TXS_PER_BLOCK) {
      throw new Error(`Tx size should be less than ${TXS_PER_BLOCK}.`);
    }

    const blockContexts: L2BlockContext[] = [];
    const encryptedLogsArr: L2BlockL2Logs[] = [];
    const ownedNoteSpendingInfos: NoteSpendingInfo[] = [];
    const numberOfBlocks = prependedBlocks + appendedBlocks + 1;
    for (let i = 0; i < numberOfBlocks; ++i) {
      const block = L2Block.random(firstBlockNum + i, TXS_PER_BLOCK);
      block.startPrivateDataTreeSnapshot.nextAvailableLeafIndex = firstBlockDataStartIndex + i * numCommitmentsPerBlock;

      const newNotes = Array(numCommitmentsPerBlock).fill(0).map(NoteSpendingInfo.random);

      block.newCommitments = newNotes.map(n => computeMockNoteHash(n.notePreimage.items));

      const isTargetBlock = i === prependedBlocks;
      const { encryptedLogs, ownedNoteSpendingInfo } = createEncryptedLogsAndOwnedNoteSpendingInfo(
        newNotes,
        isTargetBlock ? ownedData : [],
      );
      encryptedLogsArr.push(encryptedLogs);
      ownedNoteSpendingInfos.push(...ownedNoteSpendingInfo);

      const randomBlockContext = new L2BlockContext(block);
      blockContexts.push(randomBlockContext);
    }
    return { blockContexts, encryptedLogsArr, ownedNoteSpendingInfos };
  };

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
    grumpkin = new Grumpkin(wasm);
    owner = ConstantKeyPair.random(grumpkin);
  });

  beforeEach(() => {
    database = new MemoryDB();
    addNoteSpendingInfoBatchSpy = jest.spyOn(database, 'addNoteSpendingInfoBatch');

    aztecNode = mock<AztecNode>();
    keyStore = mock<KeyStore>();
    simulator = mock<AcirSimulator>();
    keyStore.getAccountPrivateKey.mockResolvedValue(owner.getPrivateKey());
    noteProcessor = new NoteProcessor(owner.getPublicKey(), keyStore, database, aztecNode, simulator);

    simulator.computeNoteHashAndNullifier.mockImplementation((...args) =>
      Promise.resolve({
        innerNoteHash: Fr.random(),
        siloedNoteHash: Fr.random(),
        uniqueSiloedNoteHash: computeMockNoteHash(args[3]),
        innerNullifier: Fr.random(),
      }),
    );
  });

  afterEach(() => {
    addNoteSpendingInfoBatchSpy.mockReset();
  });

  it('should store a note that belongs to us', async () => {
    const { blockContexts, encryptedLogsArr, ownedNoteSpendingInfos } = mockData([[2]]);
    await noteProcessor.process(blockContexts, encryptedLogsArr);

    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledTimes(1);
    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...ownedNoteSpendingInfos[0],
        index: BigInt(firstBlockDataStartIndex + 2),
      }),
    ]);
  });

  it('should store multiple notes that belong to us', async () => {
    const prependedBlocks = 3;
    const thisBlockDataStartIndex = firstBlockDataStartIndex + prependedBlocks * numCommitmentsPerBlock;

    const { blockContexts, encryptedLogsArr, ownedNoteSpendingInfos } = mockData(
      [[], [1], [], [0, 2]],
      prependedBlocks,
    );
    await noteProcessor.process(blockContexts, encryptedLogsArr);

    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledTimes(1);
    expect(addNoteSpendingInfoBatchSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...ownedNoteSpendingInfos[0],
        // Index 1 log in the 2nd tx.
        index: BigInt(thisBlockDataStartIndex + MAX_NEW_COMMITMENTS_PER_TX * (2 - 1) + 1),
      }),
      expect.objectContaining({
        ...ownedNoteSpendingInfos[1],
        // Index 0 log in the 4th tx.
        index: BigInt(thisBlockDataStartIndex + MAX_NEW_COMMITMENTS_PER_TX * (4 - 1) + 0),
      }),
      expect.objectContaining({
        ...ownedNoteSpendingInfos[2],
        // Index 2 log in the 4th tx.
        index: BigInt(thisBlockDataStartIndex + MAX_NEW_COMMITMENTS_PER_TX * (4 - 1) + 2),
      }),
    ]);
  });

  it('should not store notes that do not belong to us', async () => {
    const { blockContexts, encryptedLogsArr } = mockData([]);
    await noteProcessor.process(blockContexts, encryptedLogsArr);
  });
});
