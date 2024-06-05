import { type NoteFilter, NoteStatus, randomTxHash } from '@aztec/circuit-types';
import { AztecAddress, CompleteAddress, INITIAL_L2_BLOCK_NUM, PublicKeys } from '@aztec/circuits.js';
import { makeHeader } from '@aztec/circuits.js/testing';
import { randomInt } from '@aztec/foundation/crypto';
import { Fr, Point } from '@aztec/foundation/fields';
import { BenchmarkingContractArtifact } from '@aztec/noir-contracts.js/Benchmarking';
import { SerializableContractInstance } from '@aztec/types/contracts';

import { type IncomingNoteDao } from './incoming_note_dao.js';
import { randomIncomingNoteDao } from './incoming_note_dao.test.js';
import { type PxeDatabase } from './pxe_database.js';

/**
 * A common test suite for a PXE database.
 * @param getDatabase - A function that returns a database instance.
 */
export function describePxeDatabase(getDatabase: () => PxeDatabase) {
  let database: PxeDatabase;

  beforeEach(() => {
    database = getDatabase();
  });

  describe('Database', () => {
    describe('auth witnesses', () => {
      it('stores and retrieves auth witnesses', async () => {
        const messageHash = Fr.random();
        const witness = [Fr.random(), Fr.random()];

        await database.addAuthWitness(messageHash, witness);
        await expect(database.getAuthWitness(messageHash)).resolves.toEqual(witness);
      });

      it("returns undefined if it doesn't have auth witnesses for the message", async () => {
        const messageHash = Fr.random();
        await expect(database.getAuthWitness(messageHash)).resolves.toBeUndefined();
      });

      it.skip('refuses to overwrite auth witnesses for the same message', async () => {
        const messageHash = Fr.random();
        const witness = [Fr.random(), Fr.random()];

        await database.addAuthWitness(messageHash, witness);
        await expect(database.addAuthWitness(messageHash, witness)).rejects.toThrow();
      });
    });

    describe('capsules', () => {
      it('stores and retrieves capsules', async () => {
        const capsule = [Fr.random(), Fr.random()];

        await database.addCapsule(capsule);
        await expect(database.popCapsule()).resolves.toEqual(capsule);
      });

      it("returns undefined if it doesn't have capsules", async () => {
        await expect(database.popCapsule()).resolves.toBeUndefined();
      });

      it('behaves like a stack when storing capsules', async () => {
        const capsule1 = [Fr.random(), Fr.random()];
        const capsule2 = [Fr.random(), Fr.random()];

        await database.addCapsule(capsule1);
        await database.addCapsule(capsule2);
        await expect(database.popCapsule()).resolves.toEqual(capsule2);
        await expect(database.popCapsule()).resolves.toEqual(capsule1);
      });
    });

    describe('incoming notes', () => {
      let owners: CompleteAddress[];
      let contractAddresses: AztecAddress[];
      let storageSlots: Fr[];
      let notes: IncomingNoteDao[];

      const filteringTests: [() => NoteFilter, () => IncomingNoteDao[]][] = [
        [() => ({}), () => notes],

        [
          () => ({ contractAddress: contractAddresses[0] }),
          () => notes.filter(note => note.contractAddress.equals(contractAddresses[0])),
        ],
        [() => ({ contractAddress: AztecAddress.random() }), () => []],

        [
          () => ({ storageSlot: storageSlots[0] }),
          () => notes.filter(note => note.storageSlot.equals(storageSlots[0])),
        ],
        [() => ({ storageSlot: Fr.random() }), () => []],

        [() => ({ txHash: notes[0].txHash }), () => [notes[0]]],
        [() => ({ txHash: randomTxHash() }), () => []],

        [
          () => ({ owner: owners[0].address }),
          () => notes.filter(note => note.ivpkM.equals(owners[0].publicKeys.masterIncomingViewingPublicKey)),
        ],

        [
          () => ({ contractAddress: contractAddresses[0], storageSlot: storageSlots[0] }),
          () =>
            notes.filter(
              note => note.contractAddress.equals(contractAddresses[0]) && note.storageSlot.equals(storageSlots[0]),
            ),
        ],
        [() => ({ contractAddress: contractAddresses[0], storageSlot: storageSlots[1] }), () => []],
      ];

      beforeEach(() => {
        owners = Array.from({ length: 2 }).map(() => CompleteAddress.random());
        contractAddresses = Array.from({ length: 2 }).map(() => AztecAddress.random());
        storageSlots = Array.from({ length: 2 }).map(() => Fr.random());

        notes = Array.from({ length: 10 }).map((_, i) =>
          randomIncomingNoteDao({
            contractAddress: contractAddresses[i % contractAddresses.length],
            storageSlot: storageSlots[i % storageSlots.length],
            ivpkM: owners[i % owners.length].publicKeys.masterIncomingViewingPublicKey,
            index: BigInt(i),
          }),
        );
      });

      beforeEach(async () => {
        for (const owner of owners) {
          await database.addCompleteAddress(owner);
        }
      });

      it.each(filteringTests)('stores notes in bulk and retrieves notes', async (getFilter, getExpected) => {
        await database.addNotes(notes, []);
        await expect(database.getNotes(getFilter())).resolves.toEqual(getExpected());
      });

      it.each(filteringTests)('stores notes one by one and retrieves notes', async (getFilter, getExpected) => {
        for (const note of notes) {
          await database.addNote(note);
        }
        await expect(database.getNotes(getFilter())).resolves.toEqual(getExpected());
      });

      it.each(filteringTests)('retrieves nullified notes', async (getFilter, getExpected) => {
        await database.addNotes(notes, []);

        // Nullify all notes and use the same filter as other test cases
        for (const owner of owners) {
          const notesToNullify = notes.filter(note =>
            note.ivpkM.equals(owner.publicKeys.masterIncomingViewingPublicKey),
          );
          const nullifiers = notesToNullify.map(note => note.siloedNullifier);
          await expect(
            database.removeNullifiedNotes(nullifiers, owner.publicKeys.masterIncomingViewingPublicKey),
          ).resolves.toEqual(notesToNullify);
        }

        await expect(database.getNotes({ ...getFilter(), status: NoteStatus.ACTIVE_OR_NULLIFIED })).resolves.toEqual(
          getExpected(),
        );
      });

      it('skips nullified notes by default or when requesting active', async () => {
        await database.addNotes(notes, []);

        const notesToNullify = notes.filter(note =>
          note.ivpkM.equals(owners[0].publicKeys.masterIncomingViewingPublicKey),
        );
        const nullifiers = notesToNullify.map(note => note.siloedNullifier);
        await expect(database.removeNullifiedNotes(nullifiers, notesToNullify[0].ivpkM)).resolves.toEqual(
          notesToNullify,
        );

        const actualNotesWithDefault = await database.getNotes({});
        const actualNotesWithActive = await database.getNotes({ status: NoteStatus.ACTIVE });

        expect(actualNotesWithDefault).toEqual(actualNotesWithActive);
        expect(actualNotesWithActive).toEqual(notes.filter(note => !notesToNullify.includes(note)));
      });

      it('returns active and nullified notes when requesting either', async () => {
        await database.addNotes(notes, []);

        const notesToNullify = notes.filter(note =>
          note.ivpkM.equals(owners[0].publicKeys.masterIncomingViewingPublicKey),
        );
        const nullifiers = notesToNullify.map(note => note.siloedNullifier);
        await expect(database.removeNullifiedNotes(nullifiers, notesToNullify[0].ivpkM)).resolves.toEqual(
          notesToNullify,
        );

        const result = await database.getNotes({
          status: NoteStatus.ACTIVE_OR_NULLIFIED,
        });

        // We have to compare the sorted arrays since the database does not return the same order as when originally
        // inserted combining active and nullified results.
        expect(result.sort()).toEqual([...notes].sort());
      });
    });

    // TODO(#6867): Add tests for outgoing notes

    describe('block header', () => {
      it('stores and retrieves the block header', async () => {
        const header = makeHeader(randomInt(1000), INITIAL_L2_BLOCK_NUM);

        await database.setHeader(header);
        expect(database.getHeader()).toEqual(header);
      });

      it('rejects getting header if no block set', () => {
        expect(() => database.getHeader()).toThrow();
      });
    });

    describe('addresses', () => {
      it('stores and retrieves addresses', async () => {
        const address = CompleteAddress.random();
        await expect(database.addCompleteAddress(address)).resolves.toBe(true);
        await expect(database.getCompleteAddress(address.address)).resolves.toEqual(address);
      });

      it('silently ignores an address it already knows about', async () => {
        const address = CompleteAddress.random();
        await expect(database.addCompleteAddress(address)).resolves.toBe(true);
        await expect(database.addCompleteAddress(address)).resolves.toBe(false);
      });

      it.skip('refuses to overwrite an address with a different public key', async () => {
        const address = CompleteAddress.random();
        const otherAddress = new CompleteAddress(
          address.address,
          new PublicKeys(Point.random(), Point.random(), Point.random(), Point.random()),
          address.partialAddress,
        );

        await database.addCompleteAddress(address);
        await expect(database.addCompleteAddress(otherAddress)).rejects.toThrow();
      });

      it('returns all addresses', async () => {
        const addresses = Array.from({ length: 10 }).map(() => CompleteAddress.random());
        for (const address of addresses) {
          await database.addCompleteAddress(address);
        }

        const result = await database.getCompleteAddresses();
        expect(result).toEqual(expect.arrayContaining(addresses));
      });

      it("returns an empty array if it doesn't have addresses", async () => {
        expect(await database.getCompleteAddresses()).toEqual([]);
      });

      it("returns undefined if it doesn't have an address", async () => {
        expect(await database.getCompleteAddress(CompleteAddress.random().address)).toBeUndefined();
      });
    });

    describe('contracts', () => {
      it('stores a contract artifact', async () => {
        const artifact = BenchmarkingContractArtifact;
        const id = Fr.random();
        await database.addContractArtifact(id, artifact);
        await expect(database.getContractArtifact(id)).resolves.toEqual(artifact);
      });

      it('stores a contract instance', async () => {
        const address = AztecAddress.random();
        const instance = SerializableContractInstance.random().withAddress(address);
        await database.addContractInstance(instance);
        await expect(database.getContractInstance(address)).resolves.toEqual(instance);
      });
    });
  });
}
