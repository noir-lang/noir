import {
  AztecAddress,
  Fr,
  NoteHash,
  NoteLogHash,
  Nullifier,
  type ScopedNoteHash,
  type ScopedNullifier,
} from '@aztec/circuits.js';

import { buildTransientDataHints } from './build_transient_data_hints.js';

describe('buildTransientDataHints', () => {
  const contractAddress = AztecAddress.fromBigInt(987654n);

  let noteHashes: ScopedNoteHash[];
  let nullifiers: ScopedNullifier[];
  let logs: NoteLogHash[];

  beforeEach(() => {
    noteHashes = [
      new NoteHash(new Fr(11), 100).scope(700, contractAddress),
      new NoteHash(new Fr(22), 200).scope(0, contractAddress),
      new NoteHash(new Fr(33), 300).scope(500, contractAddress),
    ];
    nullifiers = [
      new Nullifier(new Fr(44), 400, new Fr(0)).scope(contractAddress),
      new Nullifier(new Fr(55), 500, new Fr(33)).scope(contractAddress),
      new Nullifier(new Fr(66), 600, new Fr(0)).scope(contractAddress),
      new Nullifier(new Fr(77), 700, new Fr(11)).scope(contractAddress),
    ];
    logs = [
      new NoteLogHash(new Fr(88), 350, new Fr(64), 300),
      new NoteLogHash(new Fr(99), 375, new Fr(64), 300),
      new NoteLogHash(new Fr(111), 150, new Fr(64), 100),
      new NoteLogHash(new Fr(122), 250, new Fr(64), 200),
    ];
  });

  it('builds index hints that link transient note hashes and nullifiers', () => {
    const [nullifierIndexes, noteHashIndexesForNullifiers, noteHashIndexesForLogs] = buildTransientDataHints(
      noteHashes,
      nullifiers,
      logs,
    );
    expect(nullifierIndexes).toEqual([3, 4, 1]);
    expect(noteHashIndexesForNullifiers).toEqual([3, 2, 3, 0]);
    expect(noteHashIndexesForLogs).toEqual([2, 2, 0, 3]);
  });

  it('throws if no matching nullifier', () => {
    noteHashes[0].nullifierCounter = 450;
    expect(() => buildTransientDataHints(noteHashes, nullifiers, logs)).toThrow('Unknown nullifier counter.');
  });

  it('throws if note hash does not match', () => {
    nullifiers[1].nullifier.noteHash = new Fr(11);
    expect(() => buildTransientDataHints(noteHashes, nullifiers, logs)).toThrow('Hinted note hash does not match.');
  });

  it('throws if contract address does not match', () => {
    nullifiers[1].contractAddress = AztecAddress.fromBigInt(123456n);
    expect(() => buildTransientDataHints(noteHashes, nullifiers, logs)).toThrow(
      'Contract address of hinted note hash does not match.',
    );
  });
});
