import { type AztecAddress, Comparator, Fr, type Wallet, toBigInt } from '@aztec/aztec.js';
import { DocsExampleContract, TestContract } from '@aztec/noir-contracts.js';

import { setup } from './fixtures/utils.js';

interface NoirOption<T> {
  _is_some: boolean;
  _value: T;
}

const sortFunc = (a: any, b: any) =>
  a.points > b.points ? 1 : a.points < b.points ? -1 : a.randomness > b.randomness ? 1 : -1;

function unwrapOptions<T>(options: NoirOption<T>[]): T[] {
  return options.filter((option: any) => option._is_some).map((option: any) => option._value);
}

describe('e2e_note_getter', () => {
  let wallet: Wallet;
  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, wallet } = await setup());
  }, 25_000);

  afterAll(() => teardown());

  describe('comparators', () => {
    let contract: DocsExampleContract;

    beforeAll(async () => {
      contract = await DocsExampleContract.deploy(wallet).send().deployed();
      // sets card value to 1 and leader to sender.
      await contract.methods.initialize_private(Fr.random(), 1).send().wait();
    }, 25_000);

    it('inserts notes from 0-9, then makes multiple queries specifying the total suite of comparators', async () => {
      // ISSUE #4243
      // Calling this function does not work like this
      // const numbers = [...Array(10).keys()];
      // await Promise.all(numbers.map(number => contract.methods.insert_note(number).send().wait()));
      // It causes a race condition complaining about root mismatch

      await contract.methods
        .insert_notes([...Array(10).keys()])
        .send()
        .wait();
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
        unwrapOptions(returnEq)
          .map(({ points, randomness }: any) => ({ points, randomness }))
          .sort(sortFunc),
      ).toStrictEqual(
        [
          { points: 5n, randomness: 1n },
          { points: 5n, randomness: 0n },
        ].sort(sortFunc),
      );

      expect(
        unwrapOptions(returnNeq)
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
        unwrapOptions(returnLt)
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
        unwrapOptions(returnGt)
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
        unwrapOptions(returnLte)
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
        unwrapOptions(returnGte)
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
    }, 300_000);
  });

  describe('status filter', () => {
    let contract: TestContract;
    let owner: AztecAddress;

    beforeAll(async () => {
      contract = await TestContract.deploy(wallet).send().deployed();
      owner = wallet.getCompleteAddress().address;
    }, 100_000);

    const VALUE = 5;

    // To prevent tests from interacting with one another, we'll have each use a different storage slot.
    let storageSlot: number = 2;

    beforeEach(() => {
      storageSlot += 1;
    });

    async function assertNoteIsReturned(storageSlot: number, expectedValue: number, activeOrNullified: boolean) {
      const viewNotesResult = await contract.methods.call_view_notes(storageSlot, activeOrNullified).simulate();
      const getNotesResult = await callGetNotes(storageSlot, activeOrNullified);

      expect(viewNotesResult).toEqual(getNotesResult);
      expect(viewNotesResult).toEqual(BigInt(expectedValue));
    }

    async function assertNoReturnValue(storageSlot: number, activeOrNullified: boolean) {
      await expect(contract.methods.call_view_notes(storageSlot, activeOrNullified).simulate()).rejects.toThrow(
        'is_some',
      );
      await expect(contract.methods.call_get_notes(storageSlot, activeOrNullified).send().wait()).rejects.toThrow(
        `Assertion failed: Cannot return zero notes`,
      );
    }

    async function callGetNotes(storageSlot: number, activeOrNullified: boolean): Promise<bigint> {
      // call_get_notes exposes the return value via an event since we cannot use simulate() with it.
      const tx = contract.methods.call_get_notes(storageSlot, activeOrNullified).send();
      await tx.wait();

      const logs = (await tx.getUnencryptedLogs()).logs;
      expect(logs.length).toBe(1);

      return toBigInt(logs[0].log.data);
    }

    async function callGetNotesMany(storageSlot: number, activeOrNullified: boolean): Promise<Array<bigint>> {
      // call_get_notes_many exposes the return values via event since we cannot use simulate() with it.
      const tx = contract.methods.call_get_notes_many(storageSlot, activeOrNullified).send();
      await tx.wait();

      const logs = (await tx.getUnencryptedLogs()).logs;
      expect(logs.length).toBe(2);

      return [toBigInt(logs[0].log.data), toBigInt(logs[1].log.data)];
    }

    describe('active note only', () => {
      const activeOrNullified = false;

      it('returns active notes', async () => {
        await contract.methods.call_create_note(VALUE, owner, storageSlot).send().wait();
        await assertNoteIsReturned(storageSlot, VALUE, activeOrNullified);
      }, 30_000);

      it('does not return nullified notes', async () => {
        await contract.methods.call_create_note(VALUE, owner, storageSlot).send().wait();
        await contract.methods.call_destroy_note(storageSlot).send().wait();

        await assertNoReturnValue(storageSlot, activeOrNullified);
      }, 30_000);
    });

    describe('active and nullified notes', () => {
      const activeOrNullified = true;

      it('returns active notes', async () => {
        await contract.methods.call_create_note(VALUE, owner, storageSlot).send().wait();
        await assertNoteIsReturned(storageSlot, VALUE, activeOrNullified);
      }, 30_000);

      it('returns nullified notes', async () => {
        await contract.methods.call_create_note(VALUE, owner, storageSlot).send().wait();
        await contract.methods.call_destroy_note(storageSlot).send().wait();

        await assertNoteIsReturned(storageSlot, VALUE, activeOrNullified);
      }, 30_000);

      it('returns both active and nullified notes', async () => {
        // We store two notes with two different values in the same storage slot, and then delete one of them. Note that
        // we can't be sure which one was deleted since we're just deleting based on the storage slot.
        await contract.methods.call_create_note(VALUE, owner, storageSlot).send().wait();
        await contract.methods
          .call_create_note(VALUE + 1, owner, storageSlot)
          .send()
          .wait();
        await contract.methods.call_destroy_note(storageSlot).send().wait();

        // We now fetch multiple notes, and get both the active and the nullified one.
        const viewNotesManyResult = await contract.methods
          .call_view_notes_many(storageSlot, activeOrNullified)
          .simulate();
        const getNotesManyResult = await callGetNotesMany(storageSlot, activeOrNullified);

        // We can't be sure in which order the notes will be returned, so we simply sort them to test equality. Note
        // however that both view_notes and get_notes get the exact same result.
        expect(viewNotesManyResult).toEqual(getNotesManyResult);
        expect(viewNotesManyResult.sort()).toEqual([BigInt(VALUE), BigInt(VALUE + 1)]);
      }, 45_000);
    });
  });
});
