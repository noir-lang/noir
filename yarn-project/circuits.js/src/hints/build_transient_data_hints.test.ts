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
    noteHashNullifierCounterMap.set(100, 700); // Linked to a nullifier.
    noteHashNullifierCounterMap.set(300, 500); // Linked to a nullifier that will be read.
    noteHashNullifierCounterMap.set(350, 600); // Linked to a nullifier, but the note hash will be read.
    noteHashNullifierCounterMap.set(375, 800); // Linked to a nullifier not yet known.
  });

  it('builds index hints that link transient note hashes and nullifiers', () => {
    const [nullifierIndexes, noteHashIndexesForNullifiers] = buildTransientDataHints(
      noteHashes,
      nullifiers,
      futureNoteHashReads,
      futureNullifierReads,
      noteHashNullifierCounterMap,
    );
    // Only first one is squashed, since:
    // second one is not linked to a nullifier
    // third one's nullifier will be read
    // and fourth note hash will be read.
    expect(nullifierIndexes).toEqual([3, 4, 4, 4, 4]);
    expect(noteHashIndexesForNullifiers).toEqual([5, 5, 5, 0]);
  });

  it('throws if note hash does not match', () => {
    nullifiers[1].nullifier.noteHash = new Fr(11);
    expect(() => buildTransientDataHints(noteHashes, nullifiers, [], [], noteHashNullifierCounterMap)).toThrow(
      'Hinted note hash does not match.',
    );
  });

  it('throws if contract address does not match', () => {
    nullifiers[1].contractAddress = AztecAddress.fromBigInt(123456n);
    expect(() => buildTransientDataHints(noteHashes, nullifiers, [], [], noteHashNullifierCounterMap)).toThrow(
      'Contract address of hinted note hash does not match.',
    );
  });
});
