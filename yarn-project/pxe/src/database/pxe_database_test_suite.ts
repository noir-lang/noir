import { INITIAL_L2_BLOCK_NUM, MerkleTreeId, NoteFilter, randomTxHash } from '@aztec/circuit-types';
import { AztecAddress, BlockHeader, CompleteAddress } from '@aztec/circuits.js';
import { Fr, Point } from '@aztec/foundation/fields';
import { BenchmarkingContractArtifact } from '@aztec/noir-contracts/Benchmarking';
import { SerializableContractInstance } from '@aztec/types/contracts';

import { NoteDao } from './note_dao.js';
import { randomNoteDao } from './note_dao.test.js';
import { PxeDatabase } from './pxe_database.js';

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

    describe('notes', () => {
      let owners: CompleteAddress[];
      let contractAddresses: AztecAddress[];
      let storageSlots: Fr[];
      let notes: NoteDao[];

      const filteringTests: [() => NoteFilter, () => NoteDao[]][] = [
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

        [() => ({ owner: owners[0].address }), () => notes.filter(note => note.publicKey.equals(owners[0].publicKey))],

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
          randomNoteDao({
            contractAddress: contractAddresses[i % contractAddresses.length],
            storageSlot: storageSlots[i % storageSlots.length],
            publicKey: owners[i % owners.length].publicKey,
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
        await database.addNotes(notes);
        await expect(database.getNotes(getFilter())).resolves.toEqual(getExpected());
      });

      it.each(filteringTests)('stores notes one by one and retrieves notes', async (getFilter, getExpected) => {
        for (const note of notes) {
          await database.addNote(note);
        }
        await expect(database.getNotes(getFilter())).resolves.toEqual(getExpected());
      });

      it('removes nullified notes', async () => {
        const notesToNullify = notes.filter(note => note.publicKey.equals(owners[0].publicKey));
        const nullifiers = notesToNullify.map(note => note.siloedNullifier);

        await database.addNotes(notes);

        await expect(database.removeNullifiedNotes(nullifiers, notesToNullify[0].publicKey)).resolves.toEqual(
          notesToNullify,
        );
        await expect(
          database.getNotes({
            owner: owners[0].address,
          }),
        ).resolves.toEqual([]);
        await expect(database.getNotes({})).resolves.toEqual(notes.filter(note => !notesToNullify.includes(note)));
      });
    });

    describe('block header', () => {
      it('stores and retrieves the block header', async () => {
        const blockHeader = BlockHeader.random();
        blockHeader.privateKernelVkTreeRoot = Fr.zero();

        await database.setBlockData(INITIAL_L2_BLOCK_NUM, blockHeader);
        expect(database.getBlockHeader()).toEqual(blockHeader);
      });

      it('retrieves the merkle tree roots from the block', async () => {
        const blockHeader = BlockHeader.random();
        await database.setBlockData(INITIAL_L2_BLOCK_NUM, blockHeader);
        expect(database.getTreeRoots()).toEqual({
          [MerkleTreeId.NOTE_HASH_TREE]: blockHeader.noteHashTreeRoot,
          [MerkleTreeId.NULLIFIER_TREE]: blockHeader.nullifierTreeRoot,
          [MerkleTreeId.CONTRACT_TREE]: blockHeader.contractTreeRoot,
          [MerkleTreeId.L1_TO_L2_MESSAGE_TREE]: blockHeader.l1ToL2MessageTreeRoot,
          [MerkleTreeId.ARCHIVE]: blockHeader.archiveRoot,
          [MerkleTreeId.PUBLIC_DATA_TREE]: blockHeader.publicDataTreeRoot,
        });
      });

      it('rejects getting merkle tree roots if no block set', () => {
        expect(() => database.getTreeRoots()).toThrow();
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
        const otherAddress = new CompleteAddress(address.address, Point.random(), address.partialAddress);

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
