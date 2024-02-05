import { Fr } from '@aztec/foundation/fields';

import { MockProxy, mock } from 'jest-mock-extended';

import { CommitmentsDB, PublicContractsDB, PublicStateDB } from '../../index.js';
import { HostStorage } from './host_storage.js';
import { AvmWorldStateJournal, JournalData } from './journal.js';

describe('journal', () => {
  let publicDb: MockProxy<PublicStateDB>;
  let journal: AvmWorldStateJournal;

  beforeEach(() => {
    publicDb = mock<PublicStateDB>();
    const commitmentsDb = mock<CommitmentsDB>();
    const contractsDb = mock<PublicContractsDB>();

    const hostStorage = new HostStorage(publicDb, contractsDb, commitmentsDb);
    journal = new AvmWorldStateJournal(hostStorage);
  });

  describe('Public Storage', () => {
    it('Should cache write to storage', () => {
      // When writing to storage we should write to the storage writes map
      const contractAddress = new Fr(1);
      const key = new Fr(2);
      const value = new Fr(3);

      journal.writeStorage(contractAddress, key, value);

      const journalUpdates: JournalData = journal.flush();
      expect(journalUpdates.currentStorageValue.get(contractAddress.toBigInt())?.get(key.toBigInt())).toEqual(value);
    });

    it('When reading from storage, should check the parent first', async () => {
      // Store a different value in storage vs the cache, and make sure the cache is returned
      const contractAddress = new Fr(1);
      const key = new Fr(2);
      const storedValue = new Fr(420);
      const parentValue = new Fr(69);
      const cachedValue = new Fr(1337);

      publicDb.storageRead.mockResolvedValue(Promise.resolve(storedValue));

      const childJournal = new AvmWorldStateJournal(journal.hostStorage, journal);

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

    it('When reading from storage, should check the cache first, and be appended to read/write journal', async () => {
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

      // We expect the journal to store the access in [storedVal, cachedVal] - [time0, time1]
      const { storageReads, storageWrites }: JournalData = journal.flush();
      const contractReads = storageReads.get(contractAddress.toBigInt());
      const keyReads = contractReads?.get(key.toBigInt());
      expect(keyReads).toEqual([storedValue, cachedValue]);

      const contractWrites = storageWrites.get(contractAddress.toBigInt());
      const keyWrites = contractWrites?.get(key.toBigInt());
      expect(keyWrites).toEqual([cachedValue]);
    });
  });

  describe('UTXOs', () => {
    it('Should maintain commitments', () => {
      const utxo = new Fr(1);
      journal.writeNoteHash(utxo);

      const journalUpdates = journal.flush();
      expect(journalUpdates.newNoteHashes).toEqual([utxo]);
    });

    it('Should maintain l1 messages', () => {
      const utxo = [new Fr(1)];
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

  it('Should merge two successful journals together', async () => {
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
    const logs = [new Fr(1), new Fr(2)];
    const logsT1 = [new Fr(3), new Fr(4)];

    journal.writeStorage(contractAddress, key, value);
    await journal.readStorage(contractAddress, key);
    journal.writeNoteHash(commitment);
    journal.writeLog(logs);
    journal.writeL1Message(logs);
    journal.writeNullifier(commitment);

    const childJournal = new AvmWorldStateJournal(journal.hostStorage, journal);
    childJournal.writeStorage(contractAddress, key, valueT1);
    await childJournal.readStorage(contractAddress, key);
    childJournal.writeNoteHash(commitmentT1);
    childJournal.writeLog(logsT1);
    childJournal.writeL1Message(logsT1);
    childJournal.writeNullifier(commitmentT1);

    journal.acceptNestedWorldState(childJournal);

    const result = await journal.readStorage(contractAddress, key);
    expect(result).toEqual(valueT1);

    // Check that the storage is merged by reading from the journal
    // Check that the UTXOs are merged
    const journalUpdates: JournalData = journal.flush();

    // Check storage reads order is preserved upon merge
    // We first read value from t0, then value from t1
    const contractReads = journalUpdates.storageReads.get(contractAddress.toBigInt());
    const slotReads = contractReads?.get(key.toBigInt());
    expect(slotReads).toEqual([value, valueT1, valueT1]); // Read a third time to check storage

    // We first write value from t0, then value from t1
    const contractWrites = journalUpdates.storageWrites.get(contractAddress.toBigInt());
    const slotWrites = contractWrites?.get(key.toBigInt());
    expect(slotWrites).toEqual([value, valueT1]);

    expect(journalUpdates.newNoteHashes).toEqual([commitment, commitmentT1]);
    expect(journalUpdates.newLogs).toEqual([logs, logsT1]);
    expect(journalUpdates.newL1Messages).toEqual([logs, logsT1]);
    expect(journalUpdates.newNullifiers).toEqual([commitment, commitmentT1]);
  });

  it('Should merge failed journals together', async () => {
    // Checking public storage update journals are preserved upon journal merge,
    // But the latest state is not

    // time | journal | op     | value
    // t0 -> journal0 -> write | 1
    // t1 -> journal1 -> write | 2
    // merge journals
    // t2 -> journal0 -> read  | 1

    const contractAddress = new Fr(1);
    const key = new Fr(2);
    const value = new Fr(1);
    const valueT1 = new Fr(2);
    const commitment = new Fr(10);
    const commitmentT1 = new Fr(20);
    const logs = [new Fr(1), new Fr(2)];
    const logsT1 = [new Fr(3), new Fr(4)];

    journal.writeStorage(contractAddress, key, value);
    await journal.readStorage(contractAddress, key);
    journal.writeNoteHash(commitment);
    journal.writeLog(logs);
    journal.writeL1Message(logs);
    journal.writeNullifier(commitment);

    const childJournal = new AvmWorldStateJournal(journal.hostStorage, journal);
    childJournal.writeStorage(contractAddress, key, valueT1);
    await childJournal.readStorage(contractAddress, key);
    childJournal.writeNoteHash(commitmentT1);
    childJournal.writeLog(logsT1);
    childJournal.writeL1Message(logsT1);
    childJournal.writeNullifier(commitmentT1);

    journal.rejectNestedWorldState(childJournal);

    // Check that the storage is reverted by reading from the journal
    const result = await journal.readStorage(contractAddress, key);
    expect(result).toEqual(value); // rather than valueT1

    // Check that the UTXOs are merged
    const journalUpdates: JournalData = journal.flush();

    // Reads and writes should be preserved
    // Check storage reads order is preserved upon merge
    // We first read value from t0, then value from t1
    const contractReads = journalUpdates.storageReads.get(contractAddress.toBigInt());
    const slotReads = contractReads?.get(key.toBigInt());
    expect(slotReads).toEqual([value, valueT1, value]); // Read a third time to check storage above

    // We first write value from t0, then value from t1
    const contractWrites = journalUpdates.storageWrites.get(contractAddress.toBigInt());
    const slotWrites = contractWrites?.get(key.toBigInt());
    expect(slotWrites).toEqual([value, valueT1]);

    expect(journalUpdates.newNoteHashes).toEqual([commitment]);
    expect(journalUpdates.newLogs).toEqual([logs]);
    expect(journalUpdates.newL1Messages).toEqual([logs]);
    expect(journalUpdates.newNullifiers).toEqual([commitment]);
  });

  it('Can fork and merge journals', () => {
    const rootJournal = new AvmWorldStateJournal(journal.hostStorage);
    const childJournal = rootJournal.fork();

    expect(() => rootJournal.acceptNestedWorldState(childJournal));
    expect(() => rootJournal.rejectNestedWorldState(childJournal));
  });
});
