import {
  AztecAddress,
  Fr,
  NoteHash,
  NoteLogHash,
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
  let logs: NoteLogHash[];
  let futureNoteHashReads: ScopedReadRequest[];
  let futureNullifierReads: ScopedReadRequest[];

  beforeEach(() => {
    noteHashes = [
      new NoteHash(new Fr(11), 100).scope(700, contractAddress),
      new NoteHash(new Fr(22), 200).scope(0, contractAddress), // Not linked to a nullifier
      new NoteHash(new Fr(33), 300).scope(500, contractAddress), // Linked to a nullifier that will be read
      new NoteHash(new Fr(44), 350).scope(600, contractAddress), // Linked to a nullifier, but the note hash will be read
      new NoteHash(new Fr(50), 375).scope(800, contractAddress), // Linked to a nullifier not yet known
    ];
    nullifiers = [
      new Nullifier(new Fr(55), 400, new Fr(0)).scope(contractAddress),
      new Nullifier(new Fr(66), 500, new Fr(33)).scope(contractAddress),
      new Nullifier(new Fr(77), 600, new Fr(44)).scope(contractAddress),
      new Nullifier(new Fr(88), 700, new Fr(11)).scope(contractAddress),
    ];
    logs = [
      new NoteLogHash(new Fr(99), 350, new Fr(64), 300),
      new NoteLogHash(new Fr(111), 375, new Fr(64), 300),
      new NoteLogHash(new Fr(122), 150, new Fr(64), 100),
      new NoteLogHash(new Fr(133), 250, new Fr(64), 200),
    ];
    futureNoteHashReads = [new ScopedReadRequest(new ReadRequest(new Fr(44), 351), contractAddress)];
    futureNullifierReads = [new ScopedReadRequest(new ReadRequest(new Fr(66), 502), contractAddress)];
  });

  it('builds index hints that link transient note hashes and nullifiers', () => {
    const [nullifierIndexes, noteHashIndexesForNullifiers, noteHashIndexesForLogs] = buildTransientDataHints(
      noteHashes,
      nullifiers,
      logs,
      futureNoteHashReads,
      futureNullifierReads,
    );
    // Only first one is squashed, since:
    // second one is not linked to a nullifier
    // third one's nullifier will be read
    // and fourth note hash will be read.
    expect(nullifierIndexes).toEqual([3, 4, 4, 4, 4]);
    expect(noteHashIndexesForNullifiers).toEqual([5, 5, 5, 0]);
    expect(noteHashIndexesForLogs).toEqual([5, 5, 0, 5]);
  });

  it('throws if note hash does not match', () => {
    nullifiers[1].nullifier.noteHash = new Fr(11);
    expect(() => buildTransientDataHints(noteHashes, nullifiers, logs, [], [])).toThrow(
      'Hinted note hash does not match.',
    );
  });

  it('throws if contract address does not match', () => {
    nullifiers[1].contractAddress = AztecAddress.fromBigInt(123456n);
    expect(() => buildTransientDataHints(noteHashes, nullifiers, logs, [], [])).toThrow(
      'Contract address of hinted note hash does not match.',
    );
  });
});
