import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';

import { MAX_NOTE_HASHES_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX } from '../constants.gen.js';
import { siloNoteHash } from '../hash/index.js';
import {
  NoteHash,
  type NoteHashReadRequestHints,
  NoteHashReadRequestHintsBuilder,
  PendingReadHint,
  ReadRequest,
  ReadRequestStatus,
  type ScopedNoteHash,
  ScopedReadRequest,
  SettledReadHint,
} from '../structs/index.js';
import { buildNoteHashReadRequestHints } from './build_note_hash_read_request_hints.js';

describe('buildNoteHashReadRequestHints', () => {
  const contractAddress = AztecAddress.random();
  const settledNoteHashInnerValues = [111, 222, 333];
  const settledNoteHashes = settledNoteHashInnerValues.map(noteHash => siloNoteHash(contractAddress, new Fr(noteHash)));
  const settledLeafIndexes = [1010n, 2020n, 3030n];
  const oracle = {
    getNoteHashMembershipWitness: (leafIndex: bigint) =>
      settledLeafIndexes.includes(leafIndex) ? ({} as any) : undefined,
  };
  let noteHashReadRequests: Tuple<ScopedReadRequest, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX>;
  let noteHashes: Tuple<ScopedNoteHash, typeof MAX_NOTE_HASHES_PER_TX>;
  let noteHashLeafIndexMap: Map<bigint, bigint> = new Map();
  let expectedHints: NoteHashReadRequestHints<
    typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX
  >;
  let numReadRequests = 0;
  let numPendingReads = 0;
  let numSettledReads = 0;
  let futureNoteHashes: ScopedNoteHash[];

  const innerNoteHash = (index: number) => index + 9999;

  const makeReadRequest = (value: number, counter = 2) =>
    new ReadRequest(new Fr(value), counter).scope(contractAddress);

  const makeNoteHash = (value: number, counter = 1) => new NoteHash(new Fr(value), counter).scope(contractAddress);

  const readPendingNoteHash = (noteHashIndex: number) => {
    const readRequestIndex = numReadRequests;
    const hintIndex = numPendingReads;
    noteHashReadRequests[readRequestIndex] = makeReadRequest(innerNoteHash(noteHashIndex));
    expectedHints.readRequestStatuses[readRequestIndex] = ReadRequestStatus.pending(hintIndex);
    expectedHints.pendingReadHints[hintIndex] = new PendingReadHint(readRequestIndex, noteHashIndex);
    numReadRequests++;
    numPendingReads++;
  };

  const readSettledNoteHash = (noteHashIndex: number) => {
    const readRequestIndex = numReadRequests;
    const hintIndex = numSettledReads;
    const value = settledNoteHashes[noteHashIndex];
    noteHashLeafIndexMap.set(value.toBigInt(), settledLeafIndexes[noteHashIndex]);
    noteHashReadRequests[readRequestIndex] = makeReadRequest(settledNoteHashInnerValues[noteHashIndex]);
    expectedHints.readRequestStatuses[readRequestIndex] = ReadRequestStatus.settled(hintIndex);
    expectedHints.settledReadHints[hintIndex] = new SettledReadHint(readRequestIndex, {} as any, value);
    numReadRequests++;
    numSettledReads++;
  };

  const readFutureNoteHash = (noteHashIndex: number) => {
    const readRequestIndex = numReadRequests;
    noteHashReadRequests[readRequestIndex] = makeReadRequest(futureNoteHashes[noteHashIndex].value.toNumber());
    numReadRequests++;
  };

  const buildHints = async () =>
    (
      await buildNoteHashReadRequestHints(
        oracle,
        noteHashReadRequests,
        noteHashes,
        noteHashLeafIndexMap,
        MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
        MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
        futureNoteHashes,
      )
    ).hints;

  beforeEach(() => {
    noteHashReadRequests = makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ScopedReadRequest.empty);
    noteHashes = makeTuple(MAX_NOTE_HASHES_PER_TX, i => makeNoteHash(innerNoteHash(i)));
    noteHashLeafIndexMap = new Map();
    expectedHints = NoteHashReadRequestHintsBuilder.empty(
      MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
      MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    );
    numReadRequests = 0;
    numPendingReads = 0;
    numSettledReads = 0;
    futureNoteHashes = new Array(MAX_NOTE_HASHES_PER_TX)
      .fill(null)
      .map((_, i) => makeNoteHash(innerNoteHash(i + MAX_NOTE_HASHES_PER_TX)));
  });

  it('builds empty hints', async () => {
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for pending note hash read requests', async () => {
    readPendingNoteHash(2);
    readPendingNoteHash(1);
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for settled note hash read requests', async () => {
    readSettledNoteHash(0);
    readSettledNoteHash(1);
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for mixed pending, settled and future note hash read requests', async () => {
    readPendingNoteHash(2);
    readSettledNoteHash(2);
    readSettledNoteHash(0);
    readFutureNoteHash(0);
    readPendingNoteHash(1);
    readFutureNoteHash(1);
    readPendingNoteHash(1);
    readSettledNoteHash(2);
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('throws if cannot find a match in pending set and in the tree', async () => {
    readPendingNoteHash(2);
    // Tweak the value of the read request.
    noteHashReadRequests[0].readRequest.value = new Fr(123);
    await expect(() => buildHints()).rejects.toThrow('Read request is reading an unknown note hash.');
  });
});
