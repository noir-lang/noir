import { makeCombinedAccumulatedData, makeFinalAccumulatedData } from '../../tests/factories.js';
import { Fr } from '../index.js';
import {
  CombinedAccumulatedData,
  PrivateAccumulatedRevertibleData,
  PublicAccumulatedNonRevertibleData,
  PublicAccumulatedRevertibleData,
} from './combined_accumulated_data.js';

describe('CombinedAccumulatedData', () => {
  it('Data after serialization and deserialization is equal to the original', () => {
    const original = makeCombinedAccumulatedData();
    const afterSerialization = CombinedAccumulatedData.fromBuffer(original.toBuffer());
    expect(original).toEqual(afterSerialization);
  });

  it('recombines notes correctly', () => {
    const nonRevertible = PublicAccumulatedNonRevertibleData.empty();
    nonRevertible.newNoteHashes[0].counter = new Fr(1); // a note created in private fee setup
    nonRevertible.newNoteHashes[1].counter = new Fr(5); // a note created in public setup
    nonRevertible.newNoteHashes[2].counter = new Fr(10); // a note created in public teardown

    const end = PublicAccumulatedRevertibleData.empty();
    end.newNoteHashes[0].counter = new Fr(2); // a note created in private app logic
    end.newNoteHashes[1].counter = new Fr(8); // a note created in public app logic

    const combined = CombinedAccumulatedData.recombine(nonRevertible, end);

    expect(combined.newNoteHashes.map(x => x.counter.toNumber()).slice(0, 5)).toEqual([1, 2, 5, 8, 10]);
  });

  it('recombines nullifiers correctly', () => {
    const nonRevertible = PublicAccumulatedNonRevertibleData.empty();
    nonRevertible.newNullifiers[0].counter = new Fr(1); // a nullifier created in private fee setup
    nonRevertible.newNullifiers[1].counter = new Fr(5); // a nullifier created in public setup
    nonRevertible.newNullifiers[2].counter = new Fr(10); // a nullifier created in public teardown

    const end = PublicAccumulatedRevertibleData.empty();
    end.newNullifiers[0].counter = new Fr(2); // a nullifier created in private app logic
    end.newNullifiers[1].counter = new Fr(8); // a nullifier created in public app logic

    const combined = CombinedAccumulatedData.recombine(nonRevertible, end);

    expect(combined.newNullifiers.map(x => x.counter.toNumber()).slice(0, 5)).toEqual([1, 2, 5, 8, 10]);
  });
});

describe('FinalAccumulatedData', () => {
  it('Data after serialization and deserialization is equal to the original', () => {
    const original = makeFinalAccumulatedData();
    const afterSerialization = PrivateAccumulatedRevertibleData.fromBuffer(original.toBuffer());
    expect(original).toEqual(afterSerialization);
  });
});
