import { AztecAddress, Fr } from '@aztec/circuits.js';
import { MemoryDB } from './memory_db.js';
import { NoteSpendingInfoDao, createRandomNoteSpendingInfoDao } from './note_spending_info_dao.js';
import { NotePreimage } from '@aztec/types';

describe('Memory DB', () => {
  let db: MemoryDB;

  const fr = (val: bigint) => new Fr(val);

  beforeEach(() => {
    db = new MemoryDB();
  });

  describe('NoteSpendingInfo', () => {
    const contractAddress = AztecAddress.random();
    const storageSlot = Fr.random();

    const createNote = (attributes: Partial<NoteSpendingInfoDao> = {}, sameStorage = true) =>
      createRandomNoteSpendingInfoDao({
        ...attributes,
        contractAddress: sameStorage ? contractAddress : AztecAddress.random(),
        storageSlot: sameStorage ? storageSlot : Fr.random(),
      });

    const createNotes = (numberOfNotes: number, sameStorage = true) =>
      Array(numberOfNotes)
        .fill(0)
        .map(() => createNote({}, sameStorage));

    const expectSortedNotes = (notes: NoteSpendingInfoDao[], ...expected: [number, bigint[]][]) => {
      expect(notes.length).toBe(expected[0][1].length);
      expected.forEach(([fieldIndex, fields]) => {
        for (let i = 0; i < notes.length; ++i) {
          expect(notes[i].notePreimage.items[fieldIndex].value).toBe(fields[i]);
        }
      });
    };

    it('should add and get notes', async () => {
      const notes = createNotes(3, false);
      for (let i = 0; i < notes.length; ++i) {
        await db.addNoteSpendingInfo(notes[i]);
      }

      for (let i = 0; i < notes.length; ++i) {
        const result = await db.getNoteSpendingInfo(notes[i].contractAddress, notes[i].storageSlot);
        expect(result).toEqual([notes[i]]);
      }
    });

    it('should batch add notes', async () => {
      const notes = createNotes(3, false);
      await db.addNoteSpendingInfoBatch(notes);

      for (let i = 0; i < notes.length; ++i) {
        const result = await db.getNoteSpendingInfo(notes[i].contractAddress, notes[i].storageSlot);
        expect(result).toEqual([notes[i]]);
      }
    });

    it('should get all notes with the same contract storage slot', async () => {
      const notes = createNotes(3);
      await db.addNoteSpendingInfoBatch(notes);

      const result = await db.getNoteSpendingInfo(contractAddress, storageSlot);
      expect(result.length).toBe(notes.length);
      expect(result).toEqual(expect.objectContaining(notes));
    });

    it('should get sorted notes', async () => {
      const notes = [
        createNote({ notePreimage: new NotePreimage([fr(2n), fr(1n), fr(3n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(4n), fr(5n), fr(3n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(7n), fr(6n), fr(8n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(6n), fr(5n), fr(2n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(0n), fr(0n), fr(0n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(6n), fr(5n), fr(7n)]) }),
      ];
      await db.addNoteSpendingInfoBatch(notes);

      // Sort 1st field in ascending order.
      {
        const options = { sortBy: [1], sortOrder: [2] };
        const result = await db.getNoteSpendingInfo(contractAddress, storageSlot, options);
        expectSortedNotes(result, [1, [0n, 1n, 5n, 5n, 5n, 6n]]);
      }

      // Sort 1st field in descending order.
      {
        const options = { sortBy: [1] };
        const result = await db.getNoteSpendingInfo(contractAddress, storageSlot, options);
        expectSortedNotes(result, [1, [6n, 5n, 5n, 5n, 1n, 0n]], [0, [7n, 4n, 6n, 6n, 2n, 0n]]);
      }

      // Sort 1st and 0th fields in descending order.
      {
        const options = { sortBy: [1, 0] };
        const result = await db.getNoteSpendingInfo(contractAddress, storageSlot, options);
        expectSortedNotes(result, [1, [6n, 5n, 5n, 5n, 1n, 0n]], [0, [7n, 6n, 6n, 4n, 2n, 0n]]);
      }

      // Sort 1st field in descending order
      // Then 0th field in ascending order
      {
        const options = { sortBy: [1, 0], sortOrder: [1, 2] };
        const result = await db.getNoteSpendingInfo(contractAddress, storageSlot, options);
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
        const options = { sortBy: [1, 0, 2], sortOrder: [1, 2, 1] };
        const result = await db.getNoteSpendingInfo(contractAddress, storageSlot, options);
        expectSortedNotes(
          result,
          [1, [6n, 5n, 5n, 5n, 1n, 0n]],
          [0, [7n, 4n, 6n, 6n, 2n, 0n]],
          [2, [8n, 3n, 7n, 2n, 3n, 0n]],
        );
      }
    });

    it('should get sorted notes in a range', async () => {
      const notes = [
        createNote({ notePreimage: new NotePreimage([fr(2n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(8n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(6n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(5n)]) }),
        createNote({ notePreimage: new NotePreimage([fr(0n)]) }),
      ];
      await db.addNoteSpendingInfoBatch(notes);

      const sortBy = [0];
      // Sorted values: [8n, 6n, 5n, 2n, 0n]

      {
        const options = { sortBy, limit: 3 };
        const result = await db.getNoteSpendingInfo(contractAddress, storageSlot, options);
        expectSortedNotes(result, [0, [8n, 6n, 5n]]);
      }

      {
        const options = { sortBy, limit: 3, offset: 1 };
        const result = await db.getNoteSpendingInfo(contractAddress, storageSlot, options);
        expectSortedNotes(result, [0, [6n, 5n, 2n]]);
      }

      {
        const options = { sortBy, limit: 3, offset: 4 };
        const result = await db.getNoteSpendingInfo(contractAddress, storageSlot, options);
        expectSortedNotes(result, [0, [0n]]);
      }
    });
  });
});
