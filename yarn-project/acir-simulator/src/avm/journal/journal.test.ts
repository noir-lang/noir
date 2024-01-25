import { Fr } from '@aztec/foundation/fields';

import { MockProxy, mock } from 'jest-mock-extended';

import { CommitmentsDB, PublicContractsDB, PublicStateDB } from '../../index.js';
import { HostStorage } from './host_storage.js';
import { AvmJournal, JournalData } from './journal.js';

describe('journal', () => {
  let publicDb: MockProxy<PublicStateDB>;
  let journal: AvmJournal;

  beforeEach(() => {
    publicDb = mock<PublicStateDB>();
    const commitmentsDb = mock<CommitmentsDB>();
    const contractsDb = mock<PublicContractsDB>();

    const hostStorage = new HostStorage(publicDb, contractsDb, commitmentsDb);
    journal = new AvmJournal(hostStorage);
  });

  describe('Public Storage', () => {
    it('Should cache write to storage', () => {
      // When writing to storage we should write to the storage writes map
      const contractAddress = new Fr(1);
      const key = new Fr(2);
      const value = new Fr(3);

      journal.writeStorage(contractAddress, key, value);

      const journalUpdates: JournalData = journal.flush();
      expect(journalUpdates.storageWrites.get(contractAddress.toBigInt())?.get(key.toBigInt())).toEqual(value);
    });

    it('When reading from storage, should check the parent first', async () => {
      // Store a different value in storage vs the cache, and make sure the cache is returned
      const contractAddress = new Fr(1);
      const key = new Fr(2);
      const storedValue = new Fr(420);
      const parentValue = new Fr(69);
      const cachedValue = new Fr(1337);

      publicDb.storageRead.mockResolvedValue(Promise.resolve(storedValue));

      const childJournal = new AvmJournal(journal.hostStorage, journal);

      // Get the cache miss
      const cacheMissResult = await childJournal.readStorage(contractAddress, key);
      expect(cacheMissResult).toEqual(storedValue);

      // Write to storage
      journal.writeStorage(contractAddress, key, parentValue);
      const parentResult = await childJournal.readStorage(contractAddress, key);
      expect(parentResult).toEqual(parentValue);

      // Get the parent value
      childJournal.writeStorage(contractAddress, key, cachedValue);

      // Get the storage value
      const cachedResult = await childJournal.readStorage(contractAddress, key);
      expect(cachedResult).toEqual(cachedValue);
    });

    it('When reading from storage, should check the cache first', async () => {
      // Store a different value in storage vs the cache, and make sure the cache is returned
      const contractAddress = new Fr(1);
      const key = new Fr(2);
      const storedValue = new Fr(420);
      const cachedValue = new Fr(69);

      publicDb.storageRead.mockResolvedValue(Promise.resolve(storedValue));

      // Get the cache first
      const cacheMissResult = await journal.readStorage(contractAddress, key);
      expect(cacheMissResult).toEqual(storedValue);

      // Write to storage
      journal.writeStorage(contractAddress, key, cachedValue);

      // Get the storage value
      const cachedResult = await journal.readStorage(contractAddress, key);
      expect(cachedResult).toEqual(cachedValue);
    });
  });

  describe('UTXOs', () => {
    it('Should maintain commitments', () => {
      const utxo = new Fr(1);
      journal.writeCommitment(utxo);

      const journalUpdates = journal.flush();
      expect(journalUpdates.newCommitments).toEqual([utxo]);
    });

    it('Should maintain l1 messages', () => {
      const utxo = new Fr(1);
      journal.writeL1Message(utxo);

      const journalUpdates = journal.flush();
      expect(journalUpdates.newL1Messages).toEqual([utxo]);
    });

    it('Should maintain nullifiers', () => {
      const utxo = new Fr(1);
      journal.writeNullifier(utxo);

      const journalUpdates = journal.flush();
      expect(journalUpdates.newNullifiers).toEqual([utxo]);
    });
  });

  it('Should merge two journals together', async () => {
    // Fundamentally checking that insert ordering of public storage is preserved upon journal merge
    // time | journal | op     | value
    // t0 -> journal0 -> write | 1
    // t1 -> journal1 -> write | 2
    // merge journals
    // t2 -> journal0 -> read  | 2

    const contractAddress = new Fr(1);
    const key = new Fr(2);
    const value = new Fr(1);
    const valueT1 = new Fr(2);
    const commitment = new Fr(10);
    const commitmentT1 = new Fr(20);

    journal.writeStorage(contractAddress, key, value);
    journal.writeCommitment(commitment);
    journal.writeL1Message(commitment);
    journal.writeNullifier(commitment);

    const journal1 = new AvmJournal(journal.hostStorage, journal);
    journal.writeStorage(contractAddress, key, valueT1);
    journal.writeCommitment(commitmentT1);
    journal.writeL1Message(commitmentT1);
    journal.writeNullifier(commitmentT1);

    journal1.mergeWithParent();

    // Check that the storage is merged by reading from the journal
    const result = await journal.readStorage(contractAddress, key);
    expect(result).toEqual(valueT1);

    // Check that the UTXOs are merged
    const journalUpdates: JournalData = journal.flush();
    expect(journalUpdates.newCommitments).toEqual([commitment, commitmentT1]);
    expect(journalUpdates.newL1Messages).toEqual([commitment, commitmentT1]);
    expect(journalUpdates.newNullifiers).toEqual([commitment, commitmentT1]);
  });

  it('Cannot merge a root journal, but can merge a child journal', () => {
    const rootJournal = AvmJournal.rootJournal(journal.hostStorage);
    const childJournal = AvmJournal.branchParent(rootJournal);

    expect(() => rootJournal.mergeWithParent()).toThrow();
    expect(() => childJournal.mergeWithParent());
  });
});
