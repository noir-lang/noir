import { AcirSimulator } from '@aztec/acir-simulator';
import {
  AztecNode,
  FunctionL2Logs,
  INITIAL_L2_BLOCK_NUM,
  KeyPair,
  KeyStore,
  L1NotePayload,
  L2Block,
  L2BlockContext,
  L2BlockL2Logs,
  Note,
  TxL2Logs,
} from '@aztec/circuit-types';
import { Fr, MAX_NEW_COMMITMENTS_PER_TX } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Point } from '@aztec/foundation/fields';
import { ConstantKeyPair } from '@aztec/key-store';
import { AztecLmdbStore } from '@aztec/kv-store';

import { jest } from '@jest/globals';
import { MockProxy, mock } from 'jest-mock-extended';

import { PxeDatabase } from '../database/index.js';
import { KVPxeDatabase } from '../database/kv_pxe_database.js';
import { NoteDao } from '../database/note_dao.js';
import { NoteProcessor } from './note_processor.js';

const TXS_PER_BLOCK = 4;

describe('Note Processor', () => {
  let grumpkin: Grumpkin;
  let database: PxeDatabase;
  let aztecNode: ReturnType<typeof mock<AztecNode>>;
  let addNotesSpy: any;
  let noteProcessor: NoteProcessor;
  let owner: KeyPair;
  let keyStore: MockProxy<KeyStore>;
  let simulator: MockProxy<AcirSimulator>;
  const firstBlockNum = 123;
  const numCommitmentsPerBlock = TXS_PER_BLOCK * MAX_NEW_COMMITMENTS_PER_TX;
  const firstBlockDataStartIndex = (firstBlockNum - 1) * numCommitmentsPerBlock;
  const firstBlockDataEndIndex = firstBlockNum * numCommitmentsPerBlock;

  const computeMockNoteHash = (note: Note) => Fr.fromBuffer(pedersenHash(note.items.map(i => i.toBuffer())));

  // ownedData: [tx1, tx2, ...], the numbers in each tx represents the indices of the note hashes the account owns.
  const createEncryptedLogsAndOwnedL1NotePayloads = (ownedData: number[][], ownedNotes: L1NotePayload[]) => {
    const newNotes: L1NotePayload[] = [];
    const ownedL1NotePayloads: L1NotePayload[] = [];
    const txLogs: TxL2Logs[] = [];
    let usedOwnedNote = 0;
    for (let i = 0; i < TXS_PER_BLOCK; ++i) {
      const ownedDataIndices = ownedData[i] || [];
      if (ownedDataIndices.some(index => index >= MAX_NEW_COMMITMENTS_PER_TX)) {
        throw new Error(`Data index should be less than ${MAX_NEW_COMMITMENTS_PER_TX}.`);
      }

      const logs: FunctionL2Logs[] = [];
      for (let noteIndex = 0; noteIndex < MAX_NEW_COMMITMENTS_PER_TX; ++noteIndex) {
        const isOwner = ownedDataIndices.includes(noteIndex);
        const publicKey = isOwner ? owner.getPublicKey() : Point.random();
        const note = (isOwner && ownedNotes[usedOwnedNote]) || L1NotePayload.random();
        usedOwnedNote += note === ownedNotes[usedOwnedNote] ? 1 : 0;
        newNotes.push(note);
        if (isOwner) {
          ownedL1NotePayloads.push(note);
        }
        const log = note.toEncryptedBuffer(publicKey, grumpkin);
        // 1 tx containing 1 function invocation containing 1 log
        logs.push(new FunctionL2Logs([log]));
      }
      txLogs.push(new TxL2Logs(logs));
    }

    const encryptedLogs = new L2BlockL2Logs(txLogs);
    return { newNotes, ownedL1NotePayloads, encryptedLogs };
  };

  const mockData = (
    ownedData: number[][],
    prependedBlocks = 0,
    appendedBlocks = 0,
    ownedNotes: L1NotePayload[] = [],
  ) => {
    if (ownedData.length > TXS_PER_BLOCK) {
      throw new Error(`Tx size should be less than ${TXS_PER_BLOCK}.`);
    }

    const blockContexts: L2BlockContext[] = [];
    const encryptedLogsArr: L2BlockL2Logs[] = [];
    const ownedL1NotePayloads: L1NotePayload[] = [];
    const numberOfBlocks = prependedBlocks + appendedBlocks + 1;
    for (let i = 0; i < numberOfBlocks; ++i) {
      const block = L2Block.random(firstBlockNum + i, TXS_PER_BLOCK);
      block.header.state.partial.noteHashTree.nextAvailableLeafIndex =
        firstBlockDataEndIndex + i * numCommitmentsPerBlock;

      const isTargetBlock = i === prependedBlocks;
      const {
        newNotes,
        encryptedLogs,
        ownedL1NotePayloads: payloads,
      } = createEncryptedLogsAndOwnedL1NotePayloads(isTargetBlock ? ownedData : [], isTargetBlock ? ownedNotes : []);
      encryptedLogsArr.push(encryptedLogs);
      ownedL1NotePayloads.push(...payloads);
      block.newCommitments = newNotes.map(n => computeMockNoteHash(n.note));

      const randomBlockContext = new L2BlockContext(block);
      blockContexts.push(randomBlockContext);
    }
    return { blockContexts, encryptedLogsArr, ownedL1NotePayloads };
  };

  beforeAll(() => {
    grumpkin = new Grumpkin();
    owner = ConstantKeyPair.random(grumpkin);
  });

  beforeEach(async () => {
    database = new KVPxeDatabase(await AztecLmdbStore.openTmp());
    addNotesSpy = jest.spyOn(database, 'addNotes');

    aztecNode = mock<AztecNode>();
    keyStore = mock<KeyStore>();
    simulator = mock<AcirSimulator>();
    keyStore.getAccountPrivateKey.mockResolvedValue(owner.getPrivateKey());
    noteProcessor = new NoteProcessor(
      owner.getPublicKey(),
      keyStore,
      database,
      aztecNode,
      INITIAL_L2_BLOCK_NUM,
      simulator,
    );

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
    addNotesSpy.mockReset();
  });

  it('should store a note that belongs to us', async () => {
    const { blockContexts, encryptedLogsArr, ownedL1NotePayloads } = mockData([[2]]);
    await noteProcessor.process(blockContexts, encryptedLogsArr);

    expect(addNotesSpy).toHaveBeenCalledTimes(1);
    expect(addNotesSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...ownedL1NotePayloads[0],
        index: BigInt(firstBlockDataStartIndex + 2),
      }),
    ]);
  });

  it('should store multiple notes that belong to us', async () => {
    const prependedBlocks = 2;
    const appendedBlocks = 1;
    const thisBlockDataStartIndex = firstBlockDataStartIndex + prependedBlocks * numCommitmentsPerBlock;

    const { blockContexts, encryptedLogsArr, ownedL1NotePayloads } = mockData(
      [[], [1], [], [0, 2]],
      prependedBlocks,
      appendedBlocks,
    );
    await noteProcessor.process(blockContexts, encryptedLogsArr);

    expect(addNotesSpy).toHaveBeenCalledTimes(1);
    expect(addNotesSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...ownedL1NotePayloads[0],
        // Index 1 log in the 2nd tx.
        index: BigInt(thisBlockDataStartIndex + MAX_NEW_COMMITMENTS_PER_TX * (2 - 1) + 1),
      }),
      expect.objectContaining({
        ...ownedL1NotePayloads[1],
        // Index 0 log in the 4th tx.
        index: BigInt(thisBlockDataStartIndex + MAX_NEW_COMMITMENTS_PER_TX * (4 - 1) + 0),
      }),
      expect.objectContaining({
        ...ownedL1NotePayloads[2],
        // Index 2 log in the 4th tx.
        index: BigInt(thisBlockDataStartIndex + MAX_NEW_COMMITMENTS_PER_TX * (4 - 1) + 2),
      }),
    ]);
  });

  it('should not store notes that do not belong to us', async () => {
    const { blockContexts, encryptedLogsArr } = mockData([]);
    await noteProcessor.process(blockContexts, encryptedLogsArr);
  });

  it('should be able to recover two note payloads with containing the same note', async () => {
    const note = L1NotePayload.random();
    const note2 = L1NotePayload.random();
    // All note payloads except one have the same contract address, storage slot, and the actual note.
    const notes = [note, note, note, note2, note];
    const { blockContexts, encryptedLogsArr, ownedL1NotePayloads } = mockData([[0, 2], [], [0, 1, 3]], 0, 0, notes);
    await noteProcessor.process(blockContexts, encryptedLogsArr);

    const addedNoteDaos: NoteDao[] = addNotesSpy.mock.calls[0][0];
    expect(addedNoteDaos.map(dao => dao)).toEqual([
      expect.objectContaining({ ...ownedL1NotePayloads[0] }),
      expect.objectContaining({ ...ownedL1NotePayloads[1] }),
      expect.objectContaining({ ...ownedL1NotePayloads[2] }),
      expect.objectContaining({ ...ownedL1NotePayloads[3] }),
      expect.objectContaining({ ...ownedL1NotePayloads[4] }),
    ]);
    expect(ownedL1NotePayloads[0]).toEqual(ownedL1NotePayloads[1]);
    expect(ownedL1NotePayloads[1]).toEqual(ownedL1NotePayloads[2]);
    expect(ownedL1NotePayloads[2]).toEqual(ownedL1NotePayloads[4]);
    expect(ownedL1NotePayloads[3]).not.toEqual(ownedL1NotePayloads[4]);

    // Check that every note has a different nonce.
    const nonceSet = new Set<bigint>();
    addedNoteDaos.forEach(info => nonceSet.add(info.nonce.value));
    expect(nonceSet.size).toBe(notes.length);
  });

  it('advances the block number', async () => {
    const { blockContexts, encryptedLogsArr } = mockData([[2]]);
    await noteProcessor.process(blockContexts, encryptedLogsArr);
    expect(noteProcessor.status.syncedToBlock).toEqual(blockContexts.at(-1)?.block.number);
  });

  it('should restore the last block number processed and ignore the starting block', async () => {
    const { blockContexts, encryptedLogsArr } = mockData([[2]]);
    await noteProcessor.process(blockContexts, encryptedLogsArr);

    const newNoteProcessor = new NoteProcessor(
      owner.getPublicKey(),
      keyStore,
      database,
      aztecNode,
      INITIAL_L2_BLOCK_NUM,
      simulator,
    );

    expect(newNoteProcessor.status).toEqual(noteProcessor.status);
  });
});
