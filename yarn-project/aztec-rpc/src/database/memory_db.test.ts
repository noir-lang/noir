import { AztecAddress, Fr } from '@aztec/circuits.js';

import { MemoryDB } from './memory_db.js';
import { NoteSpendingInfoDao, createRandomNoteSpendingInfoDao } from './note_spending_info_dao.js';

describe('Memory DB', () => {
  let db: MemoryDB;

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
  });
});
