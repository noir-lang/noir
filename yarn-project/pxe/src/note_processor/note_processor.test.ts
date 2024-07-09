import { type AztecNode, EncryptedL2NoteLog, L1NotePayload, L2Block, TaggedLog } from '@aztec/circuit-types';
import {
  AztecAddress,
  CompleteAddress,
  Fr,
  INITIAL_L2_BLOCK_NUM,
  KeyValidationRequest,
  MAX_NOTE_HASHES_PER_TX,
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

import { type IncomingNoteDao } from '../database/incoming_note_dao.js';
import { type PxeDatabase } from '../database/index.js';
import { KVPxeDatabase } from '../database/kv_pxe_database.js';
import { type OutgoingNoteDao } from '../database/outgoing_note_dao.js';
import { NoteProcessor } from './note_processor.js';

const TXS_PER_BLOCK = 4;
const NUM_NOTE_HASHES_PER_BLOCK = TXS_PER_BLOCK * MAX_NOTE_HASHES_PER_TX;

/** A wrapper containing info about a note we want to mock and insert into a block. */
class MockNoteRequest {
  constructor(
    /** Note we want to insert into a block. */
    public readonly note: TaggedLog<L1NotePayload>,
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
    if (noteHashIndex >= MAX_NOTE_HASHES_PER_TX) {
      throw new Error(`Data index should be less than ${MAX_NOTE_HASHES_PER_TX}.`);
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
      (this.blockNumber - 1) * NUM_NOTE_HASHES_PER_BLOCK + this.txIndex * MAX_NOTE_HASHES_PER_TX + this.noteHashIndex,
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

  const app = AztecAddress.random();

  let ownerIvskM: GrumpkinScalar;
  let ownerIvpkM: PublicKey;
  let ownerOvskM: GrumpkinScalar;
  let ownerOvKeys: KeyValidationRequest;
  let account: CompleteAddress;

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
        const noteHash = pedersenHash(note.payload.note.items);
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
    const partialAddress = Fr.random();

    account = CompleteAddress.fromSecretKeyAndPartialAddress(ownerSk, partialAddress);
    ownerIvpkM = account.publicKeys.masterIncomingViewingPublicKey;

    ({ masterIncomingViewingSecretKey: ownerIvskM, masterOutgoingViewingSecretKey: ownerOvskM } = deriveKeys(ownerSk));

    ownerOvKeys = new KeyValidationRequest(
      account.publicKeys.masterOutgoingViewingPublicKey,
      computeOvskApp(ownerOvskM, app),
    );
  });

  beforeEach(async () => {
    database = new KVPxeDatabase(openTmpStore());
    addNotesSpy = jest.spyOn(database, 'addNotes');

    aztecNode = mock<AztecNode>();
    keyStore = mock<KeyStore>();
    simulator = mock<AcirSimulator>();

    keyStore.getMasterSecretKey.mockImplementation((pkM: PublicKey) => {
      if (pkM.equals(ownerIvpkM)) {
        return Promise.resolve(ownerIvskM);
      }
      if (pkM.equals(ownerOvKeys.pkM)) {
        return Promise.resolve(ownerOvskM);
      }
      throw new Error(`Unknown public key: ${pkM}`);
    });

    keyStore.getMasterIncomingViewingPublicKey.mockResolvedValue(account.publicKeys.masterIncomingViewingPublicKey);
    keyStore.getMasterOutgoingViewingPublicKey.mockResolvedValue(account.publicKeys.masterOutgoingViewingPublicKey);

    noteProcessor = await NoteProcessor.create(
      account.address,
      keyStore,
      database,
      aztecNode,
      INITIAL_L2_BLOCK_NUM,
      simulator,
    );

    simulator.computeNoteHashAndOptionallyANullifier.mockImplementation((...args) =>
      Promise.resolve({
        innerNoteHash: Fr.random(),
        uniqueNoteHash: Fr.random(),
        siloedNoteHash: pedersenHash(args[5].items), // args[5] is note
        innerNullifier: Fr.random(),
      }),
    );
  });

  afterEach(() => {
    addNotesSpy.mockReset();
  });

  it('should store an incoming note that belongs to us', async () => {
    const request = new MockNoteRequest(
      TaggedLog.random(L1NotePayload, app),
      4,
      0,
      2,
      ownerIvpkM,
      KeyValidationRequest.random(),
    );

    const blocks = mockBlocks([request]);
    await noteProcessor.process(blocks);

    expect(addNotesSpy).toHaveBeenCalledTimes(1);
    expect(addNotesSpy).toHaveBeenCalledWith(
      [
        expect.objectContaining({
          ...request.note.payload,
          index: request.indexWithinNoteHashTree,
        }),
      ],
      [],
    );
  }, 25_000);

  it('should store an outgoing note that belongs to us', async () => {
    const request = new MockNoteRequest(TaggedLog.random(L1NotePayload, app), 4, 0, 2, Point.random(), ownerOvKeys);

    const blocks = mockBlocks([request]);
    await noteProcessor.process(blocks);

    expect(addNotesSpy).toHaveBeenCalledTimes(1);
    // For outgoing notes, the resulting DAO does not contain index.
    expect(addNotesSpy).toHaveBeenCalledWith([], [expect.objectContaining(request.note.payload)]);
  }, 25_000);

  it('should store multiple notes that belong to us', async () => {
    const requests = [
      new MockNoteRequest(TaggedLog.random(L1NotePayload, app), 1, 1, 1, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(TaggedLog.random(L1NotePayload, app), 2, 3, 0, Point.random(), ownerOvKeys),
      new MockNoteRequest(TaggedLog.random(L1NotePayload, app), 6, 3, 2, ownerIvpkM, KeyValidationRequest.random()),
      new MockNoteRequest(TaggedLog.random(L1NotePayload, app), 9, 3, 2, Point.random(), KeyValidationRequest.random()),
      new MockNoteRequest(TaggedLog.random(L1NotePayload, app), 12, 3, 2, ownerIvpkM, ownerOvKeys),
    ];

    const blocks = mockBlocks(requests);
    await noteProcessor.process(blocks);

    expect(addNotesSpy).toHaveBeenCalledTimes(1);
    expect(addNotesSpy).toHaveBeenCalledWith(
      // Incoming should contain notes from requests 0, 2, 4 because in those requests we set owner ivpk.
      [
        expect.objectContaining({
          ...requests[0].note.payload,
          index: requests[0].indexWithinNoteHashTree,
        }),
        expect.objectContaining({
          ...requests[2].note.payload,
          index: requests[2].indexWithinNoteHashTree,
        }),
        expect.objectContaining({
          ...requests[4].note.payload,
          index: requests[4].indexWithinNoteHashTree,
        }),
      ],
      // Outgoing should contain notes from requests 0, 1, 4 because in those requests we set owner ovKeys.
      [
        expect.objectContaining(requests[0].note.payload),
        expect.objectContaining(requests[1].note.payload),
        expect.objectContaining(requests[4].note.payload),
      ],
    );
  }, 30_000);

  it('should not store notes that do not belong to us', async () => {
    // Both notes should be ignored because the encryption keys do not belong to owner (they are random).
    const blocks = mockBlocks([
      new MockNoteRequest(TaggedLog.random(), 2, 1, 1, Point.random(), KeyValidationRequest.random()),
      new MockNoteRequest(TaggedLog.random(), 2, 3, 0, Point.random(), KeyValidationRequest.random()),
    ]);
    await noteProcessor.process(blocks);

    expect(addNotesSpy).toHaveBeenCalledTimes(0);
  });

  it('should be able to recover two note payloads containing the same note', async () => {
    const note = TaggedLog.random(L1NotePayload, app);
    const note2 = TaggedLog.random(L1NotePayload, app);
    // All note payloads except one have the same contract address, storage slot, and the actual note.
    const requests = [
      new MockNoteRequest(note, 3, 0, 0, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(note, 4, 0, 2, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(note, 4, 2, 0, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(note2, 5, 2, 1, ownerIvpkM, ownerOvKeys),
      new MockNoteRequest(note, 6, 2, 3, ownerIvpkM, ownerOvKeys),
    ];

    const blocks = mockBlocks(requests);
    await noteProcessor.process(blocks);

    // First we check incoming
    {
      const addedIncoming: IncomingNoteDao[] = addNotesSpy.mock.calls[0][0];
      expect(addedIncoming.map(dao => dao)).toEqual([
        expect.objectContaining({ ...requests[0].note.payload, index: requests[0].indexWithinNoteHashTree }),
        expect.objectContaining({ ...requests[1].note.payload, index: requests[1].indexWithinNoteHashTree }),
        expect.objectContaining({ ...requests[2].note.payload, index: requests[2].indexWithinNoteHashTree }),
        expect.objectContaining({ ...requests[3].note.payload, index: requests[3].indexWithinNoteHashTree }),
        expect.objectContaining({ ...requests[4].note.payload, index: requests[4].indexWithinNoteHashTree }),
      ]);

      // Check that every note has a different nonce.
      const nonceSet = new Set<bigint>();
      addedIncoming.forEach(info => nonceSet.add(info.nonce.value));
      expect(nonceSet.size).toBe(requests.length);
    }

    // Then we check outgoing
    {
      const addedOutgoing: OutgoingNoteDao[] = addNotesSpy.mock.calls[0][1];
      expect(addedOutgoing.map(dao => dao)).toEqual([
        expect.objectContaining(requests[0].note.payload),
        expect.objectContaining(requests[1].note.payload),
        expect.objectContaining(requests[2].note.payload),
        expect.objectContaining(requests[3].note.payload),
        expect.objectContaining(requests[4].note.payload),
      ]);

      // Outgoing note daos do not have a nonce so we don't check it.
    }
  });

  it('advances the block number', async () => {
    const request = new MockNoteRequest(TaggedLog.random(), 6, 0, 2, ownerIvpkM, ownerOvKeys);

    const blocks = mockBlocks([request]);
    await noteProcessor.process(blocks);

    expect(noteProcessor.status.syncedToBlock).toEqual(blocks.at(-1)?.number);
  });

  it('should restore the last block number processed and ignore the starting block', async () => {
    const request = new MockNoteRequest(TaggedLog.random(), 6, 0, 2, Point.random(), KeyValidationRequest.random());

    const blocks = mockBlocks([request]);
    await noteProcessor.process(blocks);

    const newNoteProcessor = await NoteProcessor.create(
      account.address,
      keyStore,
      database,
      aztecNode,
      INITIAL_L2_BLOCK_NUM,
      simulator,
    );

    expect(newNoteProcessor.status).toEqual(noteProcessor.status);
  });
});
