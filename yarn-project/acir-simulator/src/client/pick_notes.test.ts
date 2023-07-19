import { Fr } from '@aztec/foundation/fields';

import { SortOrder, pickNotes } from './pick_notes.js';

describe('getNotes', () => {
  const expectSortedNotes = (notes: { preimage: Fr[] }[], ...expected: [number, bigint[]][]) => {
    expect(notes.length).toBe(expected[0][1].length);
    expected.forEach(([fieldIndex, fields]) => {
      for (let i = 0; i < notes.length; ++i) {
        expect(notes[i].preimage[fieldIndex].value).toBe(fields[i]);
      }
    });
  };

  const createNote = (preimage: bigint[]) => ({
    preimage: preimage.map(f => new Fr(f)),
  });

  it('should get sorted notes', () => {
    const notes = [
      createNote([2n, 1n, 3n]),
      createNote([4n, 5n, 3n]),
      createNote([7n, 6n, 8n]),
      createNote([6n, 5n, 2n]),
      createNote([0n, 0n, 0n]),
      createNote([6n, 5n, 7n]),
    ];

    // Sort 1st field in ascending order.
    {
      const options = { sortBy: [1], sortOrder: [SortOrder.ASC] };
      const result = pickNotes(notes, options);
      expectSortedNotes(result, [1, [0n, 1n, 5n, 5n, 5n, 6n]]);
    }

    // Sort 1st field in descending order.
    {
      const options = { sortBy: [1] };
      const result = pickNotes(notes, options);
      expectSortedNotes(result, [1, [6n, 5n, 5n, 5n, 1n, 0n]], [0, [7n, 4n, 6n, 6n, 2n, 0n]]);
    }

    // Sort 1st and 0th fields in descending order.
    {
      const options = { sortBy: [1, 0] };
      const result = pickNotes(notes, options);
      expectSortedNotes(result, [1, [6n, 5n, 5n, 5n, 1n, 0n]], [0, [7n, 6n, 6n, 4n, 2n, 0n]]);
    }

    // Sort 1st field in descending order
    // Then 0th field in ascending order
    {
      const options = { sortBy: [1, 0], sortOrder: [SortOrder.DESC, SortOrder.ASC] };
      const result = pickNotes(notes, options);
      expectSortedNotes(
        result,
        [1, [6n, 5n, 5n, 5n, 1n, 0n]],
        [0, [7n, 4n, 6n, 6n, 2n, 0n]],
        [2, [8n, 3n, 2n, 7n, 3n, 0n]],
      );
    }

    // Sort 1st field in descending order.
    // Then 0th field in ascending order
    // Then 2nd field in descending order.
    {
      const options = { sortBy: [1, 0, 2], sortOrder: [SortOrder.DESC, SortOrder.ASC, SortOrder.DESC] };
      const result = pickNotes(notes, options);
      expectSortedNotes(
        result,
        [1, [6n, 5n, 5n, 5n, 1n, 0n]],
        [0, [7n, 4n, 6n, 6n, 2n, 0n]],
        [2, [8n, 3n, 7n, 2n, 3n, 0n]],
      );
    }
  });

  it('should get sorted notes in a range', () => {
    const notes = [createNote([2n]), createNote([8n]), createNote([6n]), createNote([5n]), createNote([0n])];

    const sortBy = [0];
    // Sorted values: [8n, 6n, 5n, 2n, 0n]

    {
      const options = { sortBy, limit: 3 };
      const result = pickNotes(notes, options);
      expectSortedNotes(result, [0, [8n, 6n, 5n]]);
    }

    {
      const options = { sortBy, limit: 3, offset: 1 };
      const result = pickNotes(notes, options);
      expectSortedNotes(result, [0, [6n, 5n, 2n]]);
    }

    {
      const options = { sortBy, limit: 3, offset: 4 };
      const result = pickNotes(notes, options);
      expectSortedNotes(result, [0, [0n]]);
    }
  });

  it('should not change order if sortOrder is NADA', () => {
    const notes = [createNote([2n]), createNote([8n]), createNote([6n]), createNote([5n]), createNote([0n])];
    const options = { sortBy: [0], sortOrder: [SortOrder.NADA] };
    const result = pickNotes(notes, options);
    expectSortedNotes(result, [0, [2n, 8n, 6n, 5n, 0n]]);
  });
});
