import { randomContractInstanceWithAddress } from '@aztec/circuit-types';
import { Fr } from '@aztec/foundation/fields';
import { SerializableContractInstance } from '@aztec/types/contracts';

import { mock } from 'jest-mock-extended';

import { type PublicSideEffectTraceInterface } from '../../public/side_effect_trace_interface.js';
import { initHostStorage, initPersistableStateManager } from '../fixtures/index.js';
import {
  mockGetContractInstance,
  mockL1ToL2MessageExists,
  mockNoteHashExists,
  mockNullifierExists,
  mockStorageRead,
} from '../test_utils.js';
import { type HostStorage } from './host_storage.js';
import { type AvmPersistableStateManager } from './journal.js';

describe('journal', () => {
  const address = Fr.random();
  const utxo = Fr.random();
  const leafIndex = Fr.random();

  let hostStorage: HostStorage;
  let trace: PublicSideEffectTraceInterface;
  let persistableState: AvmPersistableStateManager;

  beforeEach(() => {
    hostStorage = initHostStorage();
    trace = mock<PublicSideEffectTraceInterface>();
    persistableState = initPersistableStateManager({ hostStorage, trace });
  });

  describe('Public Storage', () => {
    it('When reading from storage, should check the cache first, and be appended to read/write journal', async () => {
      // Store a different value in storage vs the cache, and make sure the cache is returned
      const slot = new Fr(2);
      const storedValue = new Fr(420);
      const cachedValue = new Fr(69);

      mockStorageRead(hostStorage, storedValue);

      // Get the cache first
      const cacheMissResult = await persistableState.readStorage(address, slot);
      expect(cacheMissResult).toEqual(storedValue);

      // Write to storage
      persistableState.writeStorage(address, slot, cachedValue);

      // Get the storage value
      const cachedResult = await persistableState.readStorage(address, slot);
      expect(cachedResult).toEqual(cachedValue);
      // confirm that peek works
      expect(await persistableState.peekStorage(address, slot)).toEqual(cachedResult);

      // We expect the journal to store the access in [storedVal, cachedVal] - [time0, time1]
      expect(trace.tracePublicStorageRead).toHaveBeenCalledTimes(2);
      expect(trace.tracePublicStorageRead).toHaveBeenNthCalledWith(
        /*nthCall=*/ 1,
        address,
        slot,
        storedValue,
        /*exists=*/ true,
        /*cached=*/ false,
      );
      expect(trace.tracePublicStorageRead).toHaveBeenNthCalledWith(
        /*nthCall=*/ 2,
        address,
        slot,
        cachedValue,
        /*exists=*/ true,
        /*cached=*/ true,
      );
    });
  });

  describe('UTXOs & messages', () => {
    it('checkNoteHashExists works for missing note hashes', async () => {
      const exists = await persistableState.checkNoteHashExists(address, utxo, leafIndex);
      expect(exists).toEqual(false);
      expect(trace.traceNoteHashCheck).toHaveBeenCalledTimes(1);
      expect(trace.traceNoteHashCheck).toHaveBeenCalledWith(address, utxo, leafIndex, exists);
    });

    it('checkNoteHashExists works for existing note hashes', async () => {
      mockNoteHashExists(hostStorage, leafIndex, utxo);
      const exists = await persistableState.checkNoteHashExists(address, utxo, leafIndex);
      expect(exists).toEqual(true);
      expect(trace.traceNoteHashCheck).toHaveBeenCalledTimes(1);
      expect(trace.traceNoteHashCheck).toHaveBeenCalledWith(address, utxo, leafIndex, exists);
    });

    it('writeNoteHash works', () => {
      persistableState.writeNoteHash(address, utxo);
      expect(trace.traceNewNoteHash).toHaveBeenCalledTimes(1);
      expect(trace.traceNewNoteHash).toHaveBeenCalledWith(expect.objectContaining(address), /*noteHash=*/ utxo);
    });

    it('checkNullifierExists works for missing nullifiers', async () => {
      const exists = await persistableState.checkNullifierExists(address, utxo);
      expect(exists).toEqual(false);
      expect(trace.traceNullifierCheck).toHaveBeenCalledTimes(1);
      expect(trace.traceNullifierCheck).toHaveBeenCalledWith(
        address,
        utxo,
        /*leafIndex=*/ Fr.ZERO,
        exists,
        /*isPending=*/ false,
      );
    });

    it('checkNullifierExists works for existing nullifiers', async () => {
      mockNullifierExists(hostStorage, leafIndex, utxo);
      const exists = await persistableState.checkNullifierExists(address, utxo);
      expect(exists).toEqual(true);
      expect(trace.traceNullifierCheck).toHaveBeenCalledTimes(1);
      expect(trace.traceNullifierCheck).toHaveBeenCalledWith(address, utxo, leafIndex, exists, /*isPending=*/ false);
    });

    it('writeNullifier works', async () => {
      await persistableState.writeNullifier(address, utxo);
      expect(trace.traceNewNullifier).toHaveBeenCalledWith(expect.objectContaining(address), /*nullifier=*/ utxo);
    });

    it('checkL1ToL2MessageExists works for missing message', async () => {
      const exists = await persistableState.checkL1ToL2MessageExists(address, utxo, leafIndex);
      expect(exists).toEqual(false);
      expect(trace.traceL1ToL2MessageCheck).toHaveBeenCalledTimes(1);
      expect(trace.traceL1ToL2MessageCheck).toHaveBeenCalledWith(address, utxo, leafIndex, exists);
    });

    it('checkL1ToL2MessageExists works for existing message', async () => {
      mockL1ToL2MessageExists(hostStorage, leafIndex, utxo);
      const exists = await persistableState.checkL1ToL2MessageExists(address, utxo, leafIndex);
      expect(exists).toEqual(true);
      expect(trace.traceL1ToL2MessageCheck).toHaveBeenCalledTimes(1);
      expect(trace.traceL1ToL2MessageCheck).toHaveBeenCalledWith(address, utxo, leafIndex, exists);
    });

    it('Should maintain l1 messages', () => {
      const recipient = new Fr(1);
      persistableState.writeL2ToL1Message(recipient, utxo);
      expect(trace.traceNewL2ToL1Message).toHaveBeenCalledTimes(1);
      expect(trace.traceNewL2ToL1Message).toHaveBeenCalledWith(recipient, utxo);
    });
  });

  describe('Getting contract instances', () => {
    it('Should get contract instance', async () => {
      const contractInstance = randomContractInstanceWithAddress(/*(base instance) opts=*/ {}, /*address=*/ address);
      mockGetContractInstance(hostStorage, contractInstance);
      await persistableState.getContractInstance(address);
      expect(trace.traceGetContractInstance).toHaveBeenCalledTimes(1);
      expect(trace.traceGetContractInstance).toHaveBeenCalledWith({ exists: true, ...contractInstance });
    });
    it('Can get undefined contract instance', async () => {
      const emptyContractInstance = SerializableContractInstance.empty().withAddress(address);
      await persistableState.getContractInstance(address);

      expect(trace.traceGetContractInstance).toHaveBeenCalledTimes(1);
      expect(trace.traceGetContractInstance).toHaveBeenCalledWith({ exists: false, ...emptyContractInstance });
    });
  });

  //it('Should merge two successful journals together', async () => {
  //  // Fundamentally checking that insert ordering of public storage is preserved upon journal merge
  //  // time | journal | op     | value
  //  // t0 -> journal0 -> write | 1
  //  // t1 -> journal1 -> write | 2
  //  // merge journals
  //  // t2 -> journal0 -> read  | 2

  //  const contractAddress = new Fr(1);
  //  const aztecContractAddress = AztecAddress.fromField(contractAddress);
  //  const key = new Fr(2);
  //  const value = new Fr(1);
  //  const valueT1 = new Fr(2);
  //  const recipient = EthAddress.fromField(new Fr(42));
  //  const commitment = new Fr(10);
  //  const commitmentT1 = new Fr(20);
  //  const log = { address: 10n, selector: 5, data: [new Fr(5), new Fr(6)] };
  //  const logT1 = { address: 20n, selector: 8, data: [new Fr(7), new Fr(8)] };
  //  const index = new Fr(42);
  //  const indexT1 = new Fr(24);
  //  const instance = emptyTracedContractInstance(aztecContractAddress);

  //  persistableState.writeStorage(contractAddress, key, value);
  //  await persistableState.readStorage(contractAddress, key);
  //  persistableState.writeNoteHash(contractAddress, commitment);
  //  persistableState.writeUnencryptedLog(new Fr(log.address), new Fr(log.selector), log.data);
  //  persistableState.writeL2ToL1Message(recipient, commitment);
  //  await persistableState.writeNullifier(contractAddress, commitment);
  //  await persistableState.checkNullifierExists(contractAddress, commitment);
  //  await persistableState.checkL1ToL2MessageExists(commitment, index);
  //  await persistableState.getContractInstance(aztecContractAddress);

  //  const childJournal = new AvmPersistableStateManager(persistableState.hostStorage, persistableState);
  //  childJournal.writeStorage(contractAddress, key, valueT1);
  //  await childJournal.readStorage(contractAddress, key);
  //  childJournal.writeNoteHash(contractAddress, commitmentT1);
  //  childJournal.writeUnencryptedLog(new Fr(logT1.address), new Fr(logT1.selector), logT1.data);
  //  childJournal.writeL2ToL1Message(recipient, commitmentT1);
  //  await childJournal.writeNullifier(contractAddress, commitmentT1);
  //  await childJournal.checkNullifierExists(contractAddress, commitmentT1);
  //  await childJournal.checkL1ToL2MessageExists(commitmentT1, indexT1);
  //  await childJournal.getContractInstance(aztecContractAddress);

  //  persistableState.acceptNestedCallState(childJournal);

  //  const result = await persistableState.readStorage(contractAddress, key);
  //  expect(result).toEqual(valueT1);

  //  // Check that the storage is merged by reading from the journal
  //  // Check that the UTXOs are merged
  //  const journalUpdates: JournalData = persistableState.getTrace()();

  //  // Check storage reads order is preserved upon merge
  //  // We first read value from t0, then value from t1
  //  expect(journalUpdates.storageReads).toEqual([
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      exists: true,
  //      slot: key,
  //      value: value,
  //    }),
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      exists: true,
  //      slot: key,
  //      value: valueT1,
  //    }),
  //    // Read a third time to check storage
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      exists: true,
  //      slot: key,
  //      value: valueT1,
  //    }),
  //  ]);

  //  // We first write value from t0, then value from t1
  //  expect(journalUpdates.storageWrites).toEqual([
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      slot: key,
  //      value: value,
  //    }),
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      slot: key,
  //      value: valueT1,
  //    }),
  //  ]);

  //  expect(journalUpdates.noteHashes).toEqual([
  //    expect.objectContaining({ noteHash: commitment, storageAddress: contractAddress }),
  //    expect.objectContaining({ noteHash: commitmentT1, storageAddress: contractAddress }),
  //  ]);
  //  expect(journalUpdates.newLogs).toEqual([
  //    new UnencryptedL2Log(
  //      AztecAddress.fromBigInt(log.address),
  //      new EventSelector(log.selector),
  //      Buffer.concat(log.data.map(f => f.toBuffer())),
  //    ),
  //    new UnencryptedL2Log(
  //      AztecAddress.fromBigInt(logT1.address),
  //      new EventSelector(logT1.selector),
  //      Buffer.concat(logT1.data.map(f => f.toBuffer())),
  //    ),
  //  ]);
  //  expect(journalUpdates.newL1Messages).toEqual([
  //    expect.objectContaining({ recipient, content: commitment }),
  //    expect.objectContaining({ recipient, content: commitmentT1 }),
  //  ]);
  //  expect(journalUpdates.nullifierChecks).toEqual([
  //    expect.objectContaining({ nullifier: commitment, exists: true }),
  //    expect.objectContaining({ nullifier: commitmentT1, exists: true }),
  //  ]);
  //  expect(journalUpdates.nullifiers).toEqual([
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      nullifier: commitment,
  //    }),
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      nullifier: commitmentT1,
  //    }),
  //  ]);
  //  expect(journalUpdates.l1ToL2MessageChecks).toEqual([
  //    expect.objectContaining({ leafIndex: index, msgHash: commitment, exists: false }),
  //    expect.objectContaining({ leafIndex: indexT1, msgHash: commitmentT1, exists: false }),
  //  ]);
  //  expect(persistableState.trace.gotContractInstances).toEqual([instance, instance]);
  //});

  //it('Should merge failed journals together', async () => {
  //  // Checking public storage update journals are preserved upon journal merge,
  //  // But the latest state is not

  //  // time | journal | op     | value
  //  // t0 -> journal0 -> write | 1
  //  // t1 -> journal1 -> write | 2
  //  // merge journals
  //  // t2 -> journal0 -> read  | 1

  //  const contractAddress = new Fr(1);
  //  const aztecContractAddress = AztecAddress.fromField(contractAddress);
  //  const key = new Fr(2);
  //  const value = new Fr(1);
  //  const valueT1 = new Fr(2);
  //  const recipient = EthAddress.fromField(new Fr(42));
  //  const commitment = new Fr(10);
  //  const commitmentT1 = new Fr(20);
  //  const log = { address: 10n, selector: 5, data: [new Fr(5), new Fr(6)] };
  //  const logT1 = { address: 20n, selector: 8, data: [new Fr(7), new Fr(8)] };
  //  const index = new Fr(42);
  //  const indexT1 = new Fr(24);
  //  const instance = emptyTracedContractInstance(aztecContractAddress);

  //  persistableState.writeStorage(contractAddress, key, value);
  //  await persistableState.readStorage(contractAddress, key);
  //  persistableState.writeNoteHash(contractAddress, commitment);
  //  await persistableState.writeNullifier(contractAddress, commitment);
  //  await persistableState.checkNullifierExists(contractAddress, commitment);
  //  await persistableState.checkL1ToL2MessageExists(commitment, index);
  //  persistableState.writeUnencryptedLog(new Fr(log.address), new Fr(log.selector), log.data);
  //  persistableState.writeL2ToL1Message(recipient, commitment);
  //  await persistableState.getContractInstance(aztecContractAddress);

  //  const childJournal = new AvmPersistableStateManager(persistableState.hostStorage, persistableState);
  //  childJournal.writeStorage(contractAddress, key, valueT1);
  //  await childJournal.readStorage(contractAddress, key);
  //  childJournal.writeNoteHash(contractAddress, commitmentT1);
  //  await childJournal.writeNullifier(contractAddress, commitmentT1);
  //  await childJournal.checkNullifierExists(contractAddress, commitmentT1);
  //  await persistableState.checkL1ToL2MessageExists(commitmentT1, indexT1);
  //  childJournal.writeUnencryptedLog(new Fr(logT1.address), new Fr(logT1.selector), logT1.data);
  //  childJournal.writeL2ToL1Message(recipient, commitmentT1);
  //  await childJournal.getContractInstance(aztecContractAddress);

  //  persistableState.rejectNestedCallState(childJournal);

  //  // Check that the storage is reverted by reading from the journal
  //  const result = await persistableState.readStorage(contractAddress, key);
  //  expect(result).toEqual(value); // rather than valueT1

  //  const journalUpdates: JournalData = persistableState.getTrace()();

  //  // Reads and writes should be preserved
  //  // Check storage reads order is preserved upon merge
  //  // We first read value from t0, then value from t1
  //  expect(journalUpdates.storageReads).toEqual([
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      exists: true,
  //      slot: key,
  //      value: value,
  //    }),
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      exists: true,
  //      slot: key,
  //      value: valueT1,
  //    }),
  //    // Read a third time to check storage
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      exists: true,
  //      slot: key,
  //      value: value,
  //    }),
  //  ]);

  //  // We first write value from t0, then value from t1
  //  expect(journalUpdates.storageWrites).toEqual([
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      slot: key,
  //      value: value,
  //    }),
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      slot: key,
  //      value: valueT1,
  //    }),
  //  ]);

  //  // Check that the world state _traces_ are merged even on rejection
  //  expect(journalUpdates.noteHashes).toEqual([
  //    expect.objectContaining({ noteHash: commitment, storageAddress: contractAddress }),
  //    expect.objectContaining({ noteHash: commitmentT1, storageAddress: contractAddress }),
  //  ]);
  //  expect(journalUpdates.nullifierChecks).toEqual([
  //    expect.objectContaining({ nullifier: commitment, exists: true }),
  //    expect.objectContaining({ nullifier: commitmentT1, exists: true }),
  //  ]);
  //  expect(journalUpdates.nullifiers).toEqual([
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      nullifier: commitment,
  //    }),
  //    expect.objectContaining({
  //      storageAddress: contractAddress,
  //      nullifier: commitmentT1,
  //    }),
  //  ]);
  //  expect(journalUpdates.l1ToL2MessageChecks).toEqual([
  //    expect.objectContaining({ leafIndex: index, msgHash: commitment, exists: false }),
  //    expect.objectContaining({ leafIndex: indexT1, msgHash: commitmentT1, exists: false }),
  //  ]);

  //  // Check that rejected Accrued Substate is absent
  //  expect(journalUpdates.newLogs).toEqual([
  //    new UnencryptedL2Log(
  //      AztecAddress.fromBigInt(log.address),
  //      new EventSelector(log.selector),
  //      Buffer.concat(log.data.map(f => f.toBuffer())),
  //    ),
  //  ]);
  //  expect(journalUpdates.newL1Messages).toEqual([expect.objectContaining({ recipient, content: commitment })]);
  //  expect(persistableState.trace.gotContractInstances).toEqual([instance, instance]);
  //});

  //it('Can fork and merge journals', () => {
  //  const rootJournal = new AvmPersistableStateManager(persistableState.hostStorage);
  //  const childJournal = rootJournal.fork();

  //  expect(() => rootJournal.acceptNestedCallState(childJournal));
  //  expect(() => rootJournal.rejectNestedCallState(childJournal));
  //});
});
