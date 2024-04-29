import { Fr, NoteHashContext, Nullifier } from '@aztec/circuits.js';

import { buildTransientDataHints } from './build_transient_data_hints.js';

describe('buildTransientDataHints', () => {
  let noteHashes: NoteHashContext[];
  let nullifiers: Nullifier[];

  beforeEach(() => {
    noteHashes = [
      new NoteHashContext(new Fr(11), 100, 700),
      new NoteHashContext(new Fr(22), 200, 0),
      new NoteHashContext(new Fr(33), 300, 500),
    ];
    nullifiers = [
      new Nullifier(new Fr(44), 400, new Fr(0)),
      new Nullifier(new Fr(55), 500, new Fr(33)),
      new Nullifier(new Fr(66), 600, new Fr(0)),
      new Nullifier(new Fr(77), 700, new Fr(11)),
    ];
  });

  it('builds index hints that link transient note hashes and nullifiers', () => {
    const [nullifierIndexes, noteHashIndexes] = buildTransientDataHints(noteHashes, nullifiers);
    expect(nullifierIndexes).toEqual([3, 4, 1]);
    expect(noteHashIndexes).toEqual([3, 2, 3, 0]);
  });

  it('throws if no matching nullifier', () => {
    noteHashes[0].nullifierCounter = 450;
    expect(() => buildTransientDataHints(noteHashes, nullifiers)).toThrow('Unknown nullifier counter.');
  });

  it('throws if note hash does not match', () => {
    nullifiers[1].noteHash = new Fr(11);
    expect(() => buildTransientDataHints(noteHashes, nullifiers)).toThrow('Hinted note hash does not match.');
  });
});
