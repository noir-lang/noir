import {
  AztecAddress,
  Fr,
  NoteHash,
  Nullifier,
  ReadRequest,
  type ScopedNoteHash,
  type ScopedNullifier,
  ScopedReadRequest,
} from '@aztec/circuits.js';

import { buildTransientDataHints } from './build_transient_data_hints.js';

describe('buildTransientDataHints', () => {
  const contractAddress = AztecAddress.fromBigInt(987654n);

  let noteHashes: ScopedNoteHash[];
  let nullifiers: ScopedNullifier[];
  let futureNoteHashReads: ScopedReadRequest[];
  let futureNullifierReads: ScopedReadRequest[];
  let noteHashNullifierCounterMap: Map<number, number>;

  const buildHints = () =>
    buildTransientDataHints(
      noteHashes,
      nullifiers,
      futureNoteHashReads,
      futureNullifierReads,
      noteHashNullifierCounterMap,
    );

  beforeEach(() => {
    noteHashes = [
      new NoteHash(new Fr(11), 100).scope(contractAddress),
      new NoteHash(new Fr(22), 200).scope(contractAddress),
      new NoteHash(new Fr(33), 300).scope(contractAddress),
      new NoteHash(new Fr(44), 350).scope(contractAddress),
      new NoteHash(new Fr(50), 375).scope(contractAddress),
    ];
    nullifiers = [
      new Nullifier(new Fr(55), 400, new Fr(0)).scope(contractAddress),
      new Nullifier(new Fr(66), 500, new Fr(33)).scope(contractAddress),
      new Nullifier(new Fr(77), 600, new Fr(44)).scope(contractAddress),
      new Nullifier(new Fr(88), 700, new Fr(11)).scope(contractAddress),
    ];
    futureNoteHashReads = [new ScopedReadRequest(new ReadRequest(new Fr(44), 351), contractAddress)];
    futureNullifierReads = [new ScopedReadRequest(new ReadRequest(new Fr(66), 502), contractAddress)];
    noteHashNullifierCounterMap = new Map();
    noteHashNullifierCounterMap.set(100, 700);
    noteHashNullifierCounterMap.set(300, 500);
    noteHashNullifierCounterMap.set(350, 600);
    noteHashNullifierCounterMap.set(375, 800);
    /**
     * nullifiers[0] not nullifying any note hashes.
     * nullifiers[1] <> noteHashes[2], nullifier is read.
     * nullifiers[2] <> noteHashes[3], note hash is read.
     * nullifiers[3] <> noteHashes[0].
     * noteHashes[1] and noteHashes[4] not being nullified.
     */
  });

  it('builds index hints that link transient note hashes and nullifiers', () => {
    const [nullifierIndexes, noteHashIndexesForNullifiers] = buildHints();
    // Only first one is squashed, since:
    // second one is not linked to a nullifier
    // third one's nullifier will be read
    // and fourth note hash will be read.
    expect(nullifierIndexes).toEqual([3, 4, 4, 4, 4]);
    expect(noteHashIndexesForNullifiers).toEqual([5, 5, 5, 0]);
  });

  it('keeps the pair if note hash does not match', () => {
    nullifiers[3].nullifier.noteHash = new Fr(9999);
    const [nullifierIndexes, noteHashIndexesForNullifiers] = buildHints();
    expect(nullifierIndexes).toEqual([4, 4, 4, 4, 4]);
    expect(noteHashIndexesForNullifiers).toEqual([5, 5, 5, 5]);
  });

  it('throws if contract address does not match', () => {
    nullifiers[3].contractAddress = AztecAddress.fromBigInt(123456n);
    expect(buildHints).toThrow('Contract address of hinted note hash does not match.');
  });

  it('throws if note hash counter is larger than nullifier counter', () => {
    nullifiers[3].nullifier.counter = noteHashes[0].counter - 1;
    noteHashNullifierCounterMap.set(noteHashes[0].counter, noteHashes[0].counter - 1);
    expect(buildHints).toThrow('Hinted nullifier has smaller counter than note hash.');
  });
});
