import { UnencryptedL2Log } from '@aztec/circuit-types';
import { AztecAddress, EthAddress } from '@aztec/circuits.js';
import { EventSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type CommitmentsDB, type PublicContractsDB, type PublicStateDB } from '../../index.js';
import { initL1ToL2MessageOracleInput } from '../fixtures/index.js';
import { HostStorage } from './host_storage.js';
import { AvmPersistableStateManager, type JournalData } from './journal.js';

describe('journal', () => {
  let publicDb: MockProxy<PublicStateDB>;
  let commitmentsDb: MockProxy<CommitmentsDB>;
  let journal: AvmPersistableStateManager;

  beforeEach(() => {
    publicDb = mock<PublicStateDB>();
    commitmentsDb = mock<CommitmentsDB>();
    const contractsDb = mock<PublicContractsDB>();

    const hostStorage = new HostStorage(publicDb, contractsDb, commitmentsDb);
    journal = new AvmPersistableStateManager(hostStorage);
  });

  describe('Public Storage', () => {
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

  describe('UTXOs & messages', () => {
    it('Should maintain commitments', () => {
      const utxo = new Fr(1);
      journal.writeNoteHash(utxo);

      const journalUpdates = journal.flush();
      expect(journalUpdates.newNoteHashes).toEqual([utxo]);
    });
    it('checkNullifierExists works for missing nullifiers', async () => {
      const contractAddress = new Fr(1);
      const utxo = new Fr(2);
      const exists = await journal.checkNullifierExists(contractAddress, utxo);
      expect(exists).toEqual(false);

      const journalUpdates = journal.flush();
      expect(journalUpdates.nullifierChecks).toEqual([expect.objectContaining({ nullifier: utxo, exists: false })]);
    });
    it('checkNullifierExists works for existing nullifiers', async () => {
      const contractAddress = new Fr(1);
      const utxo = new Fr(2);
      const storedLeafIndex = BigInt(42);

      commitmentsDb.getNullifierIndex.mockResolvedValue(Promise.resolve(storedLeafIndex));
      const exists = await journal.checkNullifierExists(contractAddress, utxo);
      expect(exists).toEqual(true);

      const journalUpdates = journal.flush();
      expect(journalUpdates.nullifierChecks).toEqual([expect.objectContaining({ nullifier: utxo, exists: true })]);
    });
    it('Should maintain nullifiers', async () => {
      const contractAddress = new Fr(1);
      const utxo = new Fr(2);
      await journal.writeNullifier(contractAddress, utxo);

      const journalUpdates = journal.flush();
      expect(journalUpdates.newNullifiers).toEqual([utxo]);
    });
    it('checkL1ToL2MessageExists works for missing message', async () => {
      const utxo = new Fr(2);
      const leafIndex = new Fr(42);

      const exists = await journal.checkL1ToL2MessageExists(utxo, leafIndex);
      expect(exists).toEqual(false);

      const journalUpdates = journal.flush();
      expect(journalUpdates.l1ToL2MessageChecks).toEqual([
        expect.objectContaining({ leafIndex: leafIndex, msgHash: utxo, exists: false }),
      ]);
    });
    it('checkL1ToL2MessageExists works for existing nullifiers', async () => {
      const utxo = new Fr(2);
      const leafIndex = new Fr(42);

      commitmentsDb.getL1ToL2MembershipWitness.mockResolvedValue(initL1ToL2MessageOracleInput(leafIndex.toBigInt()));
      const exists = await journal.checkL1ToL2MessageExists(utxo, leafIndex);
      expect(exists).toEqual(true);

      const journalUpdates = journal.flush();
      expect(journalUpdates.l1ToL2MessageChecks).toEqual([
        expect.objectContaining({ leafIndex: leafIndex, msgHash: utxo, exists: true }),
      ]);
    });
    it('Should maintain nullifiers', async () => {
      const contractAddress = new Fr(1);
      const utxo = new Fr(2);
      await journal.writeNullifier(contractAddress, utxo);

      const journalUpdates = journal.flush();
      expect(journalUpdates.newNullifiers).toEqual([utxo]);
    });
    it('Should maintain l1 messages', () => {
      const recipient = EthAddress.fromField(new Fr(1));
      const utxo = new Fr(2);
      journal.writeL1Message(recipient, utxo);

      const journalUpdates = journal.flush();
      expect(journalUpdates.newL1Messages).toEqual([{ recipient, content: utxo }]);
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
    const recipient = EthAddress.fromField(new Fr(42));
    const commitment = new Fr(10);
    const commitmentT1 = new Fr(20);
    const log = { address: 10n, selector: 5, data: [new Fr(5), new Fr(6)] };
    const logT1 = { address: 20n, selector: 8, data: [new Fr(7), new Fr(8)] };
    const index = new Fr(42);
    const indexT1 = new Fr(24);

    journal.writeStorage(contractAddress, key, value);
    await journal.readStorage(contractAddress, key);
    journal.writeNoteHash(commitment);
    journal.writeLog(new Fr(log.address), new Fr(log.selector), log.data);
    journal.writeL1Message(recipient, commitment);
    await journal.writeNullifier(contractAddress, commitment);
    await journal.checkNullifierExists(contractAddress, commitment);
    await journal.checkL1ToL2MessageExists(commitment, index);

    const childJournal = new AvmPersistableStateManager(journal.hostStorage, journal);
    childJournal.writeStorage(contractAddress, key, valueT1);
    await childJournal.readStorage(contractAddress, key);
    childJournal.writeNoteHash(commitmentT1);
    childJournal.writeLog(new Fr(logT1.address), new Fr(logT1.selector), logT1.data);
    childJournal.writeL1Message(recipient, commitmentT1);
    await childJournal.writeNullifier(contractAddress, commitmentT1);
    await childJournal.checkNullifierExists(contractAddress, commitmentT1);
    await childJournal.checkL1ToL2MessageExists(commitmentT1, indexT1);

    journal.acceptNestedCallState(childJournal);

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
    expect(journalUpdates.newLogs).toEqual([
      new UnencryptedL2Log(
        AztecAddress.fromBigInt(log.address),
        new EventSelector(log.selector),
        Buffer.concat(log.data.map(f => f.toBuffer())),
      ),
      new UnencryptedL2Log(
        AztecAddress.fromBigInt(logT1.address),
        new EventSelector(logT1.selector),
        Buffer.concat(logT1.data.map(f => f.toBuffer())),
      ),
    ]);
    expect(journalUpdates.newL1Messages).toEqual([
      { recipient, content: commitment },
      { recipient, content: commitmentT1 },
    ]);
    expect(journalUpdates.nullifierChecks).toEqual([
      expect.objectContaining({ nullifier: commitment, exists: true }),
      expect.objectContaining({ nullifier: commitmentT1, exists: true }),
    ]);
    expect(journalUpdates.newNullifiers).toEqual([commitment, commitmentT1]);
    expect(journalUpdates.l1ToL2MessageChecks).toEqual([
      expect.objectContaining({ leafIndex: index, msgHash: commitment, exists: false }),
      expect.objectContaining({ leafIndex: indexT1, msgHash: commitmentT1, exists: false }),
    ]);
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
    const recipient = EthAddress.fromField(new Fr(42));
    const commitment = new Fr(10);
    const commitmentT1 = new Fr(20);
    const log = { address: 10n, selector: 5, data: [new Fr(5), new Fr(6)] };
    const logT1 = { address: 20n, selector: 8, data: [new Fr(7), new Fr(8)] };
    const index = new Fr(42);
    const indexT1 = new Fr(24);

    journal.writeStorage(contractAddress, key, value);
    await journal.readStorage(contractAddress, key);
    journal.writeNoteHash(commitment);
    await journal.writeNullifier(contractAddress, commitment);
    await journal.checkNullifierExists(contractAddress, commitment);
    await journal.checkL1ToL2MessageExists(commitment, index);
    journal.writeLog(new Fr(log.address), new Fr(log.selector), log.data);
    journal.writeL1Message(recipient, commitment);

    const childJournal = new AvmPersistableStateManager(journal.hostStorage, journal);
    childJournal.writeStorage(contractAddress, key, valueT1);
    await childJournal.readStorage(contractAddress, key);
    childJournal.writeNoteHash(commitmentT1);
    await childJournal.writeNullifier(contractAddress, commitmentT1);
    await childJournal.checkNullifierExists(contractAddress, commitmentT1);
    await journal.checkL1ToL2MessageExists(commitmentT1, indexT1);
    childJournal.writeLog(new Fr(logT1.address), new Fr(logT1.selector), logT1.data);
    childJournal.writeL1Message(recipient, commitmentT1);

    journal.rejectNestedCallState(childJournal);

    // Check that the storage is reverted by reading from the journal
    const result = await journal.readStorage(contractAddress, key);
    expect(result).toEqual(value); // rather than valueT1

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

    // Check that the world state _traces_ are merged even on rejection
    expect(journalUpdates.newNoteHashes).toEqual([commitment, commitmentT1]);
    expect(journalUpdates.nullifierChecks).toEqual([
      expect.objectContaining({ nullifier: commitment, exists: true }),
      expect.objectContaining({ nullifier: commitmentT1, exists: true }),
    ]);
    expect(journalUpdates.newNullifiers).toEqual([commitment, commitmentT1]);
    expect(journalUpdates.l1ToL2MessageChecks).toEqual([
      expect.objectContaining({ leafIndex: index, msgHash: commitment, exists: false }),
      expect.objectContaining({ leafIndex: indexT1, msgHash: commitmentT1, exists: false }),
    ]);

    // Check that rejected Accrued Substate is absent
    expect(journalUpdates.newLogs).toEqual([
      new UnencryptedL2Log(
        AztecAddress.fromBigInt(log.address),
        new EventSelector(log.selector),
        Buffer.concat(log.data.map(f => f.toBuffer())),
      ),
    ]);
    expect(journalUpdates.newL1Messages).toEqual([{ recipient, content: commitment }]);
  });

  it('Can fork and merge journals', () => {
    const rootJournal = new AvmPersistableStateManager(journal.hostStorage);
    const childJournal = rootJournal.fork();

    expect(() => rootJournal.acceptNestedCallState(childJournal));
    expect(() => rootJournal.rejectNestedCallState(childJournal));
  });
});
