import { Fr } from '@aztec/foundation/fields';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type PublicStateDB } from '../../index.js';
import { PublicStorage } from './public_storage.js';

describe('avm public storage', () => {
  let publicDb: MockProxy<PublicStateDB>;
  let publicStorage: PublicStorage;

  beforeEach(() => {
    publicDb = mock<PublicStateDB>();
    publicStorage = new PublicStorage(publicDb);
  });

  describe('AVM Public Storage', () => {
    it('Reading an unwritten slot works (gets zero & DNE)', async () => {
      const contractAddress = new Fr(1);
      const slot = new Fr(2);
      // never written!
      const { exists, value: gotValue, cached } = await publicStorage.read(contractAddress, slot);
      // doesn't exist, value is zero
      expect(exists).toEqual(false);
      expect(gotValue).toEqual(Fr.ZERO);
      expect(cached).toEqual(false);
    });

    it('Should cache storage write, reading works after write', async () => {
      const contractAddress = new Fr(1);
      const slot = new Fr(2);
      const value = new Fr(3);
      // Write to cache
      publicStorage.write(contractAddress, slot, value);
      const { exists, value: gotValue, cached } = await publicStorage.read(contractAddress, slot);
      // exists because it was previously written
      expect(exists).toEqual(true);
      expect(gotValue).toEqual(value);
      expect(cached).toEqual(true);
    });

    it('Reading works on fallback to host (gets value & exists)', async () => {
      const contractAddress = new Fr(1);
      const slot = new Fr(2);
      const storedValue = new Fr(420);
      // ensure that fallback to host gets a value
      publicDb.storageRead.mockResolvedValue(storedValue);

      const { exists, value: gotValue, cached } = await publicStorage.read(contractAddress, slot);
      // it exists in the host, so it must've been written before
      expect(exists).toEqual(true);
      expect(gotValue).toEqual(storedValue);
      expect(cached).toEqual(false);
    });

    it('Reading works on fallback to parent (gets value & exists)', async () => {
      const contractAddress = new Fr(1);
      const slot = new Fr(2);
      const value = new Fr(3);
      const childStorage = new PublicStorage(publicDb, publicStorage);

      publicStorage.write(contractAddress, slot, value);
      const { exists, value: gotValue, cached } = await childStorage.read(contractAddress, slot);
      // exists because it was previously written!
      expect(exists).toEqual(true);
      expect(gotValue).toEqual(value);
      expect(cached).toEqual(true);
    });

    it('Reading works on fallback to grandparent (gets value & exists)', async () => {
      const contractAddress = new Fr(1);
      const slot = new Fr(2);
      const value = new Fr(3);
      const childStorage = new PublicStorage(publicDb, publicStorage);
      const grandChildStorage = new PublicStorage(publicDb, childStorage);

      publicStorage.write(contractAddress, slot, value);
      const { exists, value: gotValue, cached } = await grandChildStorage.read(contractAddress, slot);
      // exists because it was previously written!
      expect(exists).toEqual(true);
      expect(gotValue).toEqual(value);
      expect(cached).toEqual(true);
    });

    it('When reading from storage, should check cache, then parent, then host', async () => {
      // Store a different value in storage vs the cache, and make sure the cache is returned
      const contractAddress = new Fr(1);
      const slot = new Fr(2);
      const storedValue = new Fr(420);
      const parentValue = new Fr(69);
      const cachedValue = new Fr(1337);

      publicDb.storageRead.mockResolvedValue(storedValue);
      const childStorage = new PublicStorage(publicDb, publicStorage);

      // Cache miss falls back to host
      const { cached: cachedCacheMiss, value: valueCacheMiss } = await childStorage.read(contractAddress, slot);
      expect(valueCacheMiss).toEqual(storedValue);
      expect(cachedCacheMiss).toEqual(false);

      // Write to storage
      publicStorage.write(contractAddress, slot, parentValue);
      // Reading from child should give value written in parent
      const { cached: cachedValueFromParent, value: valueFromParent } = await childStorage.read(contractAddress, slot);
      expect(valueFromParent).toEqual(parentValue);
      expect(cachedValueFromParent).toEqual(true);

      // Now write a value directly in child
      childStorage.write(contractAddress, slot, cachedValue);

      // Reading should now give the value written in child
      const { cached: cachedChild, value: cachedResult } = await childStorage.read(contractAddress, slot);
      expect(cachedResult).toEqual(cachedValue);
      expect(cachedChild).toEqual(true);
    });
  });

  it('Should be able to merge two public storages together', async () => {
    // Checking that child's writes take precedence on marge
    const contractAddress = new Fr(1);
    const slot = new Fr(2);
    // value written initially in parent
    const value = new Fr(1);
    // value overwritten in child and later merged into parent
    const valueT1 = new Fr(2);

    // Write initial value to parent
    publicStorage.write(contractAddress, slot, value);

    const childStorage = new PublicStorage(publicDb, publicStorage);
    // Write valueT1 to child
    childStorage.write(contractAddress, slot, valueT1);

    // Parent accepts child's staged writes
    publicStorage.acceptAndMerge(childStorage);

    // Read from parent gives latest value written in child before merge (valueT1)
    const { exists, value: result } = await publicStorage.read(contractAddress, slot);
    expect(exists).toEqual(true);
    expect(result).toEqual(valueT1);
  });
});
