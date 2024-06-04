import { type AztecNode, EncryptedL2NoteLog, L2Block, TaggedNote } from '@aztec/circuit-types';
import {
  AztecAddress,
  Fr,
  type GrumpkinPrivateKey,
  INITIAL_L2_BLOCK_NUM,
  KeyValidationRequest,
  MAX_NEW_NOTE_HASHES_PER_TX,
  type PublicKey,
  computeOvskApp,
  deriveKeys,
} from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';
import { GrumpkinScalar, Point } from '@aztec/foundation/fields';
import { type KeyStore } from '@aztec/key-store';
import { openTmpStore } from '@aztec/kv-store/utils';
import { type AcirSimulator } from '@aztec/simulator';

import { jest } from '@jest/globals';
import { type MockProxy, mock } from 'jest-mock-extended';

import { type PxeDatabase } from '../database/index.js';
import { KVPxeDatabase } from '../database/kv_pxe_database.js';
import { type NoteDao } from '../database/note_dao.js';
import { NoteProcessor } from './note_processor.js';

const TXS_PER_BLOCK = 4;
const NUM_NOTE_HASHES_PER_BLOCK = TXS_PER_BLOCK * MAX_NEW_NOTE_HASHES_PER_TX;

/** A wrapper containing info about a note we want to mock and insert into a block */
class MockNoteRequest {
  constructor(
    /** Note we want to insert into a block. */
    public readonly note: TaggedNote,
    /** Block number this note corresponds to. */
    public readonly blockNumber: number,
    /** Index of a tx within a block this note corresponds to. */
    public readonly txIndex: number,
    /** Index of a note hash within a list of note hashes for 1 tx. */
    public readonly noteHashIndex: number,
    /** ivpk we use when encrypting a note. */
    public readonly ivpk: PublicKey,
    /** ovKeys we use when encrypting a note. */
    public readonly ovKeys: KeyValidationRequest,
  ) {
    if (blockNumber < INITIAL_L2_BLOCK_NUM) {
      throw new Error(`Block number should be greater than or equal to ${INITIAL_L2_BLOCK_NUM}.`);
    }
    if (noteHashIndex >= MAX_NEW_NOTE_HASHES_PER_TX) {
      throw new Error(`Data index should be less than ${MAX_NEW_NOTE_HASHES_PER_TX}.`);
    }
    if (txIndex >= TXS_PER_BLOCK) {
      throw new Error(`Tx index should be less than ${TXS_PER_BLOCK}.`);
    }
  }

  encrypt(): EncryptedL2NoteLog {
    const ephSk = GrumpkinScalar.random();
    const recipient = AztecAddress.random();
    const log = this.note.encrypt(ephSk, recipient, this.ivpk, this.ovKeys);
    return new EncryptedL2NoteLog(log);
  }

  get indexWithinNoteHashTree(): bigint {
    return BigInt(
      (this.blockNumber - 1) * NUM_NOTE_HASHES_PER_BLOCK +
        this.txIndex * MAX_NEW_NOTE_HASHES_PER_TX +
        this.noteHashIndex,
    );
  }
}

describe('Note Processor', () => {
  let database: PxeDatabase;
  let aztecNode: ReturnType<typeof mock<AztecNode>>;
  let addNotesSpy: any;
  let noteProcessor: NoteProcessor;
  let keyStore: MockProxy<KeyStore>;
  let simulator: MockProxy<AcirSimulator>;

  let ownerIvskM: GrumpkinPrivateKey;
  let ownerIvpkM: PublicKey;
  let ownerOvKeys: KeyValidationRequest;

  function mockBlocks(requests: MockNoteRequest[]) {
    const blocks = [];

    // The number of blocks we create starts from INITIAL_L2_BLOCK_NUM and ends at the highest block number in requests
    const numBlocks = requests.reduce((maxBlockNum, request) => Math.max(maxBlockNum, request.blockNumber), 0);

    for (let i = 0; i < numBlocks; i++) {
      // First we get a random block with correct block number
      const block = L2Block.random(INITIAL_L2_BLOCK_NUM + i, TXS_PER_BLOCK, 1, 0, 4);

      // We have to update the next available leaf index in note hash tree to match the block number
      block.header.state.partial.noteHashTree.nextAvailableLeafIndex = block.number * NUM_NOTE_HASHES_PER_BLOCK;

      // Then we get all the note requests for the block
      const noteRequestsForBlock = requests.filter(request => request.blockNumber === block.number);

      // Then we update the relevant note hashes to match the note requests
      for (const request of noteRequestsForBlock) {
        const note = request.note;
        const noteHash = pedersenHash(note.notePayload.note.items);
        block.body.txEffects[request.txIndex].noteHashes[request.noteHashIndex] = noteHash;

        // Now we populate the log - to simplify we say that there is only 1 function invocation in each tx
        block.body.txEffects[request.txIndex].noteEncryptedLogs.functionLogs[0].logs[request.noteHashIndex] =
          request.encrypt();
      }

      // The block is finished so we add it to the list of blocks
      blocks.push(block);
    }

    return blocks;
  }

  beforeAll(() => {
    const ownerSk = Fr.random();
    const allOwnerKeys = deriveKeys(ownerSk);
    const app = AztecAddress.random();

    ownerIvskM = allOwnerKeys.masterIncomingViewingSecretKey;
    ownerIvpkM = allOwnerKeys.publicKeys.masterIncomingViewingPublicKey;
    ownerOvKeys = new KeyValidationRequest(
      allOwnerKeys.publicKeys.masterOutgoingViewingPublicKey,
      computeOvskApp(allOwnerKeys.masterOutgoingViewingSecretKey, app),
    );
  });

  beforeEach(() => {
    database = new KVPxeDatabase(openTmpStore());
    addNotesSpy = jest.spyOn(database, 'addNotes');

    aztecNode = mock<AztecNode>();
    keyStore = mock<KeyStore>();
    simulator = mock<AcirSimulator>();
    keyStore.getMasterIncomingViewingSecretKeyForPublicKey.mockResolvedValue(ownerIvskM);
    noteProcessor = new NoteProcessor(ownerIvpkM, keyStore, database, aztecNode, INITIAL_L2_BLOCK_NUM, simulator);

    simulator.computeNoteHashAndNullifier.mockImplementation((...args) =>
      Promise.resolve({
        innerNoteHash: Fr.random(),
        uniqueNoteHash: Fr.random(),
        siloedNoteHash: pedersenHash(args[4].items), // args[4] is note
        innerNullifier: Fr.random(),
      }),
    );
  });

  afterEach(() => {
    addNotesSpy.mockReset();
  });

  it('should store a note that belongs to us', async () => {
    const request = new MockNoteRequest(TaggedNote.random(), 4, 0, 2, ownerIvpkM, ownerOvKeys);

    const blocks = mockBlocks([request]);

    // TODO(#6830): pass in only the blocks
    const encryptedLogs = blocks.flatMap(block => block.body.noteEncryptedLogs);
    await noteProcessor.process(blocks, encryptedLogs);

    expect(addNotesSpy).toHaveBeenCalledTimes(1);
    expect(addNotesSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...request.note.notePayload,
        index: request.indexWithinNoteHashTree,
      }),
    ]);
  }, 25_000);

  it('should store multiple notes that belong to us', async () => {
    const requests = [
      new MockNoteRequest(TaggedNote.random(), 1, 1, 1, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(TaggedNote.random(), 2, 3, 0, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(TaggedNote.random(), 6, 3, 2, ownerIvpkM, ownerOvKeys),
    ];

    const blocks = mockBlocks(requests);

    // TODO(#6830): pass in only the blocks
    const encryptedLogs = blocks.flatMap(block => block.body.noteEncryptedLogs);
    await noteProcessor.process(blocks, encryptedLogs);

    expect(addNotesSpy).toHaveBeenCalledTimes(1);
    expect(addNotesSpy).toHaveBeenCalledWith([
      expect.objectContaining({
        ...requests[0].note.notePayload,
        // Index 1 log in the 2nd tx.
        index: requests[0].indexWithinNoteHashTree,
      }),
      expect.objectContaining({
        ...requests[1].note.notePayload,
        // Index 0 log in the 4th tx.
        index: requests[1].indexWithinNoteHashTree,
      }),
      expect.objectContaining({
        ...requests[2].note.notePayload,
        // Index 2 log in the 4th tx.
        index: requests[2].indexWithinNoteHashTree,
      }),
    ]);
  }, 30_000);

  it('should not store notes that do not belong to us', async () => {
    // Both notes should be ignored because the encryption keys do not belong to owner (they are random).
    const blocks = mockBlocks([
      new MockNoteRequest(TaggedNote.random(), 2, 1, 1, Point.random(), KeyValidationRequest.random()),
      new MockNoteRequest(TaggedNote.random(), 2, 3, 0, Point.random(), KeyValidationRequest.random()),
    ]);

    // TODO(#6830): pass in only the blocks
    const encryptedLogs = blocks.flatMap(block => block.body.noteEncryptedLogs);
    await noteProcessor.process(blocks, encryptedLogs);

    expect(addNotesSpy).toHaveBeenCalledTimes(0);
  });

  it('should be able to recover two note payloads containing the same note', async () => {
    const note = TaggedNote.random();
    const note2 = TaggedNote.random();
    // All note payloads except one have the same contract address, storage slot, and the actual note.
    const requests = [
      new MockNoteRequest(note, 3, 0, 0, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(note, 4, 0, 2, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(note, 4, 2, 0, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(note2, 5, 2, 1, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(note, 6, 2, 3, ownerIvpkM, ownerOvKeys),
    ];

    const blocks = mockBlocks(requests);

    // TODO(#6830): pass in only the blocks
    const encryptedLogs = blocks.flatMap(block => block.body.noteEncryptedLogs);
    await noteProcessor.process(blocks, encryptedLogs);

    const addedNoteDaos: NoteDao[] = addNotesSpy.mock.calls[0][0];
    expect(addedNoteDaos.map(dao => dao)).toEqual([
      expect.objectContaining({ ...requests[0].note.notePayload }),
      expect.objectContaining({ ...requests[1].note.notePayload }),
      expect.objectContaining({ ...requests[2].note.notePayload }),
      expect.objectContaining({ ...requests[3].note.notePayload }),
      expect.objectContaining({ ...requests[4].note.notePayload }),
    ]);

    // Check that every note has a different nonce.
    const nonceSet = new Set<bigint>();
    addedNoteDaos.forEach(info => nonceSet.add(info.nonce.value));
    expect(nonceSet.size).toBe(requests.length);
  });

  it('advances the block number', async () => {
    const request = new MockNoteRequest(TaggedNote.random(), 6, 0, 2, ownerIvpkM, ownerOvKeys);

    const blocks = mockBlocks([request]);

    // TODO(#6830): pass in only the blocks
    const encryptedLogs = blocks.flatMap(block => block.body.noteEncryptedLogs);
    await noteProcessor.process(blocks, encryptedLogs);

    expect(noteProcessor.status.syncedToBlock).toEqual(blocks.at(-1)?.number);
  });

  it('should restore the last block number processed and ignore the starting block', async () => {
    const request = new MockNoteRequest(TaggedNote.random(), 6, 0, 2, ownerIvpkM, ownerOvKeys);

    const blocks = mockBlocks([request]);

    // TODO(#6830): pass in only the blocks
    const encryptedLogs = blocks.flatMap(block => block.body.noteEncryptedLogs);
    await noteProcessor.process(blocks, encryptedLogs);

    const newNoteProcessor = new NoteProcessor(
      ownerIvpkM,
      keyStore,
      database,
      aztecNode,
      INITIAL_L2_BLOCK_NUM,
      simulator,
    );

    expect(newNoteProcessor.status).toEqual(noteProcessor.status);
  });
});
