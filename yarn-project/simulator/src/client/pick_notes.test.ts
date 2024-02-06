import { Comparator, Note } from '@aztec/circuit-types';
import { Fr } from '@aztec/foundation/fields';

import { SortOrder, pickNotes } from './pick_notes.js';

describe('getNotes', () => {
  const expectNotesFields = (notes: { note: Note }[], ...expected: [number, bigint[]][]) => {
    expect(notes.length).toBe(expected[0][1].length);
    expected.forEach(([fieldIndex, fields]) => {
      for (let i = 0; i < notes.length; ++i) {
        expect(notes[i].note.items[fieldIndex].value).toBe(fields[i]);
      }
    });
  };

  const expectNotes = (notes: { note: Note }[], expected: bigint[][]) => {
    expect(notes.length).toBe(expected.length);
    notes.forEach((note, i) => {
      expect(note.note.items.map(p => p.value)).toEqual(expected[i]);
    });
  };

  const createNote = (items: bigint[]) => ({
    note: new Note(items.map(f => new Fr(f))),
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
      const options = { sorts: [{ index: 1, order: SortOrder.ASC }] };
      const result = pickNotes(notes, options);
      expectNotesFields(result, [1, [0n, 1n, 5n, 5n, 5n, 6n]]);
    }

    // Sort 1st field in descending order.
    {
      const options = { sorts: [{ index: 1, order: SortOrder.DESC }] };
      const result = pickNotes(notes, options);
      expectNotesFields(result, [1, [6n, 5n, 5n, 5n, 1n, 0n]], [0, [7n, 4n, 6n, 6n, 2n, 0n]]);
    }

    // Sort 1st and 0th fields in descending order.
    {
      const options = {
        sorts: [
          { index: 1, order: SortOrder.DESC },
          { index: 0, order: SortOrder.DESC },
        ],
      };
      const result = pickNotes(notes, options);
      expectNotesFields(result, [1, [6n, 5n, 5n, 5n, 1n, 0n]], [0, [7n, 6n, 6n, 4n, 2n, 0n]]);
    }

    // Sort 1st field in descending order
    // Then 0th field in ascending order
    {
      const options = {
        sorts: [
          { index: 1, order: SortOrder.DESC },
          { index: 0, order: SortOrder.ASC },
        ],
      };
      const result = pickNotes(notes, options);
      expectNotesFields(
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
      const options = {
        sorts: [
          { index: 1, order: SortOrder.DESC },
          { index: 0, order: SortOrder.ASC },
          { index: 2, order: SortOrder.DESC },
        ],
      };
      const result = pickNotes(notes, options);
      expectNotesFields(
        result,
        [1, [6n, 5n, 5n, 5n, 1n, 0n]],
        [0, [7n, 4n, 6n, 6n, 2n, 0n]],
        [2, [8n, 3n, 7n, 2n, 3n, 0n]],
      );
    }
  });

  it('should get sorted notes in a range', () => {
    const notes = [createNote([2n]), createNote([8n]), createNote([6n]), createNote([5n]), createNote([0n])];

    const sorts = [{ index: 0, order: SortOrder.DESC }];
    // Sorted values: [8n, 6n, 5n, 2n, 0n]

    {
      const options = { sorts, limit: 3 };
      const result = pickNotes(notes, options);
      expectNotesFields(result, [0, [8n, 6n, 5n]]);
    }

    {
      const options = { sorts, limit: 3, offset: 1 };
      const result = pickNotes(notes, options);
      expectNotesFields(result, [0, [6n, 5n, 2n]]);
    }

    {
      const options = { sorts, limit: 3, offset: 4 };
      const result = pickNotes(notes, options);
      expectNotesFields(result, [0, [0n]]);
    }
  });

  it('should not change order if sortOrder is NADA', () => {
    const notes = [createNote([2n]), createNote([8n]), createNote([6n]), createNote([5n]), createNote([0n])];
    const options = { sorts: [{ index: 0, order: SortOrder.NADA }] };
    const result = pickNotes(notes, options);
    expectNotesFields(result, [0, [2n, 8n, 6n, 5n, 0n]]);
  });

  it('should get notes that have the required fields', () => {
    const notes = [
      createNote([2n, 1n, 3n]),
      createNote([1n, 2n, 3n]),
      createNote([3n, 2n, 0n]),
      createNote([2n, 2n, 0n]),
      createNote([2n, 3n, 3n]),
    ];

    {
      const options = { selects: [{ index: 0, value: new Fr(2n), comparator: Comparator.EQ }] };
      const result = pickNotes(notes, options);
      expectNotes(result, [
        [2n, 1n, 3n],
        [2n, 2n, 0n],
        [2n, 3n, 3n],
      ]);
    }

    {
      const options = {
        selects: [
          { index: 0, value: new Fr(2n), comparator: Comparator.EQ },
          { index: 2, value: new Fr(3n), comparator: Comparator.EQ },
        ],
      };
      const result = pickNotes(notes, options);
      expectNotes(result, [
        [2n, 1n, 3n],
        [2n, 3n, 3n],
      ]);
    }

    {
      const options = {
        selects: [
          { index: 1, value: new Fr(2n), comparator: Comparator.EQ },
          { index: 2, value: new Fr(3n), comparator: Comparator.EQ },
        ],
      };
      const result = pickNotes(notes, options);
      expectNotes(result, [[1n, 2n, 3n]]);
    }

    {
      const options = { selects: [{ index: 1, value: new Fr(5n), comparator: Comparator.EQ }] };
      const result = pickNotes(notes, options);
      expectNotes(result, []);
    }

    {
      const options = {
        selects: [
          { index: 0, value: new Fr(2n), comparator: Comparator.EQ },
          { index: 1, value: new Fr(5n), comparator: Comparator.EQ },
        ],
      };
      const result = pickNotes(notes, options);
      expectNotes(result, []);
    }
  });

  it('should get sorted matching notes', () => {
    const notes = [
      createNote([2n, 1n, 3n]),
      createNote([4n, 5n, 8n]),
      createNote([7n, 6n, 8n]),
      createNote([6n, 5n, 2n]),
      createNote([0n, 0n, 8n]),
      createNote([6n, 5n, 8n]),
    ];

    const options = {
      selects: [{ index: 2, value: new Fr(8n), comparator: Comparator.EQ }],
      sorts: [{ index: 1, order: SortOrder.ASC }],
    };
    const result = pickNotes(notes, options);
    expectNotes(result, [
      [0n, 0n, 8n],
      [4n, 5n, 8n],
      [6n, 5n, 8n],
      [7n, 6n, 8n],
    ]);
  });

  it('should get sorted matching notes with GTE and LTE', () => {
    const notes = [
      createNote([2n, 1n, 3n]),
      createNote([4n, 5n, 8n]),
      createNote([7n, 6n, 8n]),
      createNote([6n, 5n, 2n]),
      createNote([0n, 0n, 8n]),
      createNote([6n, 5n, 8n]),
    ];

    const options = {
      selects: [
        {
          index: 2,
          value: new Fr(7n),
          comparator: Comparator.GTE,
        },
        {
          index: 2,
          value: new Fr(8n),
          comparator: Comparator.LTE,
        },
      ],
      sorts: [
        {
          index: 1,
          order: SortOrder.ASC,
        },
      ],
    };
    const result = pickNotes(notes, options);
    expectNotes(result, [
      [0n, 0n, 8n],
      [4n, 5n, 8n],
      [6n, 5n, 8n],
      [7n, 6n, 8n],
    ]);
  });

  it('should get sorted matching notes with GTE and LTE', () => {
    const notes = [
      createNote([2n, 1n, 1n]),
      createNote([4n, 5n, 2n]),
      createNote([7n, 6n, 3n]),
      createNote([6n, 5n, 4n]),
      createNote([0n, 0n, 5n]),
      createNote([6n, 5n, 6n]),
    ];

    const options1 = {
      selects: [
        {
          index: 2,
          value: new Fr(3n),
          comparator: Comparator.GT,
        },
      ],
      sorts: [
        {
          index: 1,
          order: SortOrder.ASC,
        },
      ],
    };

    const result1 = pickNotes(notes, options1);

    expectNotes(result1, [
      [0n, 0n, 5n],
      [6n, 5n, 4n],
      [6n, 5n, 6n],
    ]);

    const options2 = {
      selects: [
        {
          index: 2,
          value: new Fr(4n),
          comparator: Comparator.LT,
        },
      ],
      sorts: [
        {
          index: 1,
          order: SortOrder.ASC,
        },
      ],
    };

    const result2 = pickNotes(notes, options2);

    expectNotes(result2, [
      [2n, 1n, 1n],
      [4n, 5n, 2n],
      [7n, 6n, 3n],
    ]);
  });
});
