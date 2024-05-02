import { makeTuple } from '@aztec/foundation/array';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';

import { MAX_NEW_NOTE_HASHES_PER_TX, MAX_NOTE_HASH_READ_REQUESTS_PER_TX } from '../constants.gen.js';
import { siloNoteHash } from '../hash/index.js';
import {
  NoteHashContext,
  type NoteHashReadRequestHints,
  NoteHashReadRequestHintsBuilder,
  PendingReadHint,
  ReadRequestContext,
  ReadRequestStatus,
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
  let noteHashReadRequests: Tuple<ReadRequestContext, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX>;
  let noteHashes: Tuple<NoteHashContext, typeof MAX_NEW_NOTE_HASHES_PER_TX>;
  let noteHashLeafIndexMap: Map<bigint, bigint> = new Map();
  let expectedHints: NoteHashReadRequestHints;
  let numReadRequests = 0;
  let numPendingReads = 0;
  let numSettledReads = 0;

  const innerNoteHash = (index: number) => index + 9999;

  const makeReadRequest = (value: number, counter = 2) =>
    new ReadRequestContext(new Fr(value), counter, contractAddress);

  function makeNoteHash(value: number, counter = 1) {
    const siloedValue = siloNoteHash(contractAddress, new Fr(value));
    return new NoteHashContext(siloedValue, counter, 0);
  }

  const readPendingNoteHash = ({
    noteHashIndex,
    readRequestIndex = numReadRequests,
    hintIndex = numPendingReads,
  }: {
    noteHashIndex: number;
    readRequestIndex?: number;
    hintIndex?: number;
  }) => {
    noteHashReadRequests[readRequestIndex] = makeReadRequest(innerNoteHash(noteHashIndex));
    expectedHints.readRequestStatuses[readRequestIndex] = ReadRequestStatus.pending(hintIndex);
    expectedHints.pendingReadHints[hintIndex] = new PendingReadHint(readRequestIndex, noteHashIndex);
    numReadRequests++;
    numPendingReads++;
  };

  const readSettledNoteHash = ({
    readRequestIndex = numReadRequests,
    hintIndex = numSettledReads,
  }: {
    readRequestIndex?: number;
    hintIndex?: number;
  } = {}) => {
    const value = settledNoteHashes[hintIndex];
    noteHashLeafIndexMap.set(value.toBigInt(), settledLeafIndexes[hintIndex]);
    noteHashReadRequests[readRequestIndex] = new ReadRequestContext(value, 1, contractAddress);
    expectedHints.readRequestStatuses[readRequestIndex] = ReadRequestStatus.settled(hintIndex);
    expectedHints.settledReadHints[hintIndex] = new SettledReadHint(readRequestIndex, {} as any, value);
    numReadRequests++;
    numSettledReads++;
  };

  const buildHints = () =>
    buildNoteHashReadRequestHints(oracle, noteHashReadRequests, noteHashes, noteHashLeafIndexMap);

  beforeEach(() => {
    noteHashReadRequests = makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, ReadRequestContext.empty);
    noteHashes = makeTuple(MAX_NEW_NOTE_HASHES_PER_TX, i => makeNoteHash(innerNoteHash(i)));
    noteHashLeafIndexMap = new Map();
    expectedHints = NoteHashReadRequestHintsBuilder.empty();
    numReadRequests = 0;
    numPendingReads = 0;
    numSettledReads = 0;
  });

  it('builds empty hints', async () => {
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for pending note hash read requests', async () => {
    readPendingNoteHash({ noteHashIndex: 2 });
    readPendingNoteHash({ noteHashIndex: 1 });
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for settled note hash read requests', async () => {
    readSettledNoteHash();
    readSettledNoteHash();
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('builds hints for mixed pending and settled note hash read requests', async () => {
    readPendingNoteHash({ noteHashIndex: 2 });
    readSettledNoteHash();
    readSettledNoteHash();
    readPendingNoteHash({ noteHashIndex: 1 });
    readPendingNoteHash({ noteHashIndex: 1 });
    readSettledNoteHash();
    const hints = await buildHints();
    expect(hints).toEqual(expectedHints);
  });

  it('throws if cannot find a match in pending set and in the tree', async () => {
    readPendingNoteHash({ noteHashIndex: 2 });
    // Tweak the value of the read request.
    noteHashReadRequests[0].value = new Fr(123);
    await expect(() => buildHints()).rejects.toThrow('Read request is reading an unknown note hash.');
  });
});
