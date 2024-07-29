import { type AztecAddress, Comparator, Fr, type Wallet } from '@aztec/aztec.js';
import { DocsExampleContract, TestContract } from '@aztec/noir-contracts.js';

import { setup } from './fixtures/utils.js';

interface NoirBoundedVec<T> {
  storage: T[];
  len: bigint;
}

function boundedVecToArray<T>(boundedVec: NoirBoundedVec<T>): T[] {
  return boundedVec.storage.slice(0, Number(boundedVec.len));
}

const sortFunc = (a: any, b: any) =>
  a.points > b.points ? 1 : a.points < b.points ? -1 : a.randomness > b.randomness ? 1 : -1;

describe('e2e_note_getter', () => {
  let wallet: Wallet;
  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
  });

  afterAll(() => teardown());

  describe('comparators', () => {
    let contract: DocsExampleContract;

    beforeAll(async () => {
      contract = await DocsExampleContract.deploy(wallet).send().deployed();
      // sets card value to 1 and leader to sender.
      await contract.methods.initialize_private(Fr.random(), 1).send().wait();
    });

    it('inserts notes from 0-9, then makes multiple queries specifying the total suite of comparators', async () => {
      // ISSUE #4243
      // Calling this function does not work like this
      // const numbers = [...Array(10).keys()];
      // await Promise.all(numbers.map(number => contract.methods.insert_note(number).send().wait()));
      // It causes a race condition complaining about root mismatch

      // Note: Separated the below into calls of 3 to avoid reaching logs per call limit
      await contract.methods.insert_notes([0, 1, 2]).send().wait();
      await contract.methods.insert_notes([3, 4, 5]).send().wait();
      await contract.methods.insert_notes([6, 7, 8]).send().wait();
      await contract.methods.insert_note(9, new Fr(1n)).send().wait();
      await contract.methods.insert_note(5, Fr.ZERO).send().wait();

      const [returnEq, returnNeq, returnLt, returnGt, returnLte, returnGte] = await Promise.all([
        contract.methods.read_note(5, Comparator.EQ).simulate(),
        contract.methods.read_note(5, Comparator.NEQ).simulate(),
        contract.methods.read_note(5, Comparator.LT).simulate(),
        contract.methods.read_note(5, Comparator.GT).simulate(),
        contract.methods.read_note(5, Comparator.LTE).simulate(),
        // docs:start:state_vars-NoteGetterOptionsComparatorExampleTs
        contract.methods.read_note(5, Comparator.GTE).simulate(),
        // docs:end:state_vars-NoteGetterOptionsComparatorExampleTs
      ]);

      expect(
        boundedVecToArray(returnEq)
          .map(({ points, randomness }: any) => ({ points, randomness }))
          .sort(sortFunc),
      ).toStrictEqual(
        [
          { points: 5n, randomness: 1n },
          { points: 5n, randomness: 0n },
        ].sort(sortFunc),
      );

      expect(
        boundedVecToArray(returnNeq)
          .map(({ points, randomness }: any) => ({ points, randomness }))
          .sort(sortFunc),
      ).toStrictEqual(
        [
          { points: 0n, randomness: 1n },
          { points: 1n, randomness: 1n },
          { points: 7n, randomness: 1n },
          { points: 9n, randomness: 1n },
          { points: 2n, randomness: 1n },
          { points: 6n, randomness: 1n },
          { points: 8n, randomness: 1n },
          { points: 4n, randomness: 1n },
          { points: 3n, randomness: 1n },
        ].sort(sortFunc),
      );

      expect(
        boundedVecToArray(returnLt)
          .map(({ points, randomness }: any) => ({ points, randomness }))
          .sort(sortFunc),
      ).toStrictEqual(
        [
          { points: 0n, randomness: 1n },
          { points: 1n, randomness: 1n },
          { points: 2n, randomness: 1n },
          { points: 4n, randomness: 1n },
          { points: 3n, randomness: 1n },
        ].sort(sortFunc),
      );

      expect(
        boundedVecToArray(returnGt)
          .map(({ points, randomness }: any) => ({ points, randomness }))
          .sort(sortFunc),
      ).toStrictEqual(
        [
          { points: 7n, randomness: 1n },
          { points: 9n, randomness: 1n },
          { points: 6n, randomness: 1n },
          { points: 8n, randomness: 1n },
        ].sort(sortFunc),
      );

      expect(
        boundedVecToArray(returnLte)
          .map(({ points, randomness }: any) => ({ points, randomness }))
          .sort(sortFunc),
      ).toStrictEqual(
        [
          { points: 5n, randomness: 1n },
          { points: 5n, randomness: 0n },
          { points: 0n, randomness: 1n },
          { points: 1n, randomness: 1n },
          { points: 2n, randomness: 1n },
          { points: 4n, randomness: 1n },
          { points: 3n, randomness: 1n },
        ].sort(sortFunc),
      );

      expect(
        boundedVecToArray(returnGte)
          .map(({ points, randomness }: any) => ({ points, randomness }))
          .sort(sortFunc),
      ).toStrictEqual(
        [
          { points: 5n, randomness: 0n },
          { points: 5n, randomness: 1n },
          { points: 7n, randomness: 1n },
          { points: 9n, randomness: 1n },
          { points: 6n, randomness: 1n },
          { points: 8n, randomness: 1n },
        ].sort(sortFunc),
      );
    });
  });

  describe('status filter', () => {
    let contract: TestContract;
    let owner: AztecAddress;
    let outgoingViewer: AztecAddress;

    beforeAll(async () => {
      contract = await TestContract.deploy(wallet).send().deployed();
      owner = wallet.getCompleteAddress().address;
      // Setting the outgoing viewer to owner not have to bother with setting up another account.
      outgoingViewer = owner;
    });

    const VALUE = 5;

    // To prevent tests from interacting with one another, we'll have each use a different storage slot.
    let storageSlot = TestContract.storage.example_set.slot.toNumber();

    beforeEach(() => {
      storageSlot += 1;
    });

    async function assertNoteIsReturned(storageSlot: number, expectedValue: number, activeOrNullified: boolean) {
      const viewNotesResult = await contract.methods.call_view_notes(storageSlot, activeOrNullified).simulate();
      const getNotesResult = await contract.methods.call_get_notes(storageSlot, activeOrNullified).simulate();

      expect(viewNotesResult).toEqual(getNotesResult);
      expect(viewNotesResult).toEqual(BigInt(expectedValue));
    }

    async function assertNoReturnValue(storageSlot: number, activeOrNullified: boolean) {
      await expect(contract.methods.call_view_notes(storageSlot, activeOrNullified).simulate()).rejects.toThrow(
        'index < self.len', // from BoundedVec::get
      );
      await expect(contract.methods.call_get_notes(storageSlot, activeOrNullified).prove()).rejects.toThrow(
        `Assertion failed: Attempted to read past end of BoundedVec`,
      );
    }

    describe('active note only', () => {
      const activeOrNullified = false;

      it('returns active notes', async () => {
        await contract.methods.call_create_note(VALUE, owner, outgoingViewer, storageSlot).send().wait();
        await assertNoteIsReturned(storageSlot, VALUE, activeOrNullified);
      });

      it('does not return nullified notes', async () => {
        await contract.methods.call_create_note(VALUE, owner, outgoingViewer, storageSlot).send().wait();
        await contract.methods.call_destroy_note(storageSlot).send().wait();

        await assertNoReturnValue(storageSlot, activeOrNullified);
      });
    });

    describe('active and nullified notes', () => {
      const activeOrNullified = true;

      it('returns active notes', async () => {
        await contract.methods.call_create_note(VALUE, owner, outgoingViewer, storageSlot).send().wait();
        await assertNoteIsReturned(storageSlot, VALUE, activeOrNullified);
      });

      it('returns nullified notes', async () => {
        await contract.methods.call_create_note(VALUE, owner, outgoingViewer, storageSlot).send().wait();
        await contract.methods.call_destroy_note(storageSlot).send().wait();

        await assertNoteIsReturned(storageSlot, VALUE, activeOrNullified);
      });

      it('returns both active and nullified notes', async () => {
        // We store two notes with two different values in the same storage slot, and then delete one of them. Note that
        // we can't be sure which one was deleted since we're just deleting based on the storage slot.
        await contract.methods.call_create_note(VALUE, owner, outgoingViewer, storageSlot).send().wait();
        await contract.methods
          .call_create_note(VALUE + 1, owner, outgoingViewer, storageSlot)
          .send()
          .wait();
        await contract.methods.call_destroy_note(storageSlot).send().wait();

        // We now fetch multiple notes, and get both the active and the nullified one.
        const viewNotesManyResult = await contract.methods
          .call_view_notes_many(storageSlot, activeOrNullified)
          .simulate();
        const getNotesManyResult = await contract.methods
          .call_get_notes_many(storageSlot, activeOrNullified)
          .simulate();

        // We can't be sure in which order the notes will be returned, so we simply sort them to test equality. Note
        // however that both view_notes and get_notes get the exact same result.
        expect(viewNotesManyResult).toEqual(getNotesManyResult);
        expect(viewNotesManyResult.sort()).toEqual([BigInt(VALUE), BigInt(VALUE + 1)]);
      });
    });
  });
});
