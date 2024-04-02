import { Fr } from '@aztec/foundation/fields';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type CommitmentsDB } from '../../index.js';
import { Nullifiers } from './nullifiers.js';

describe('avm nullifier caching', () => {
  let commitmentsDb: MockProxy<CommitmentsDB>;
  let nullifiers: Nullifiers;

  beforeEach(() => {
    commitmentsDb = mock<CommitmentsDB>();
    nullifiers = new Nullifiers(commitmentsDb);
  });

  describe('Nullifier caching and existence checks', () => {
    it('Reading a non-existent nullifier works (gets zero & DNE)', async () => {
      const contractAddress = new Fr(1);
      const nullifier = new Fr(2);
      // never written!
      const [exists, isPending, gotIndex] = await nullifiers.checkExists(contractAddress, nullifier);
      // doesn't exist, not pending, index is zero (non-existent)
      expect(exists).toEqual(false);
      expect(isPending).toEqual(false);
      expect(gotIndex).toEqual(Fr.ZERO);
    });
    it('Should cache nullifier, existence check works after creation', async () => {
      const contractAddress = new Fr(1);
      const nullifier = new Fr(2);

      // Write to cache
      await nullifiers.append(contractAddress, nullifier);
      const [exists, isPending, gotIndex] = await nullifiers.checkExists(contractAddress, nullifier);
      // exists (in cache), isPending, index is zero (not in tree)
      expect(exists).toEqual(true);
      expect(isPending).toEqual(true);
      expect(gotIndex).toEqual(Fr.ZERO);
    });
    it('Existence check works on fallback to host (gets index, exists, not-pending)', async () => {
      const contractAddress = new Fr(1);
      const nullifier = new Fr(2);
      const storedLeafIndex = BigInt(420);

      commitmentsDb.getNullifierIndex.mockResolvedValue(Promise.resolve(storedLeafIndex));

      const [exists, isPending, gotIndex] = await nullifiers.checkExists(contractAddress, nullifier);
      // exists (in host), not pending, tree index retrieved from host
      expect(exists).toEqual(true);
      expect(isPending).toEqual(false);
      expect(gotIndex).toEqual(gotIndex);
    });
    it('Existence check works on fallback to parent (gets value, exists, is pending)', async () => {
      const contractAddress = new Fr(1);
      const nullifier = new Fr(2);
      const childNullifiers = new Nullifiers(commitmentsDb, nullifiers);

      // Write to parent cache
      await nullifiers.append(contractAddress, nullifier);
      // Get from child cache
      const [exists, isPending, gotIndex] = await childNullifiers.checkExists(contractAddress, nullifier);
      // exists (in parent), isPending, index is zero (not in tree)
      expect(exists).toEqual(true);
      expect(isPending).toEqual(true);
      expect(gotIndex).toEqual(Fr.ZERO);
    });
  });

  describe('Nullifier collision failures', () => {
    it('Cant append nullifier that already exists in cache', async () => {
      const contractAddress = new Fr(1);
      const nullifier = new Fr(2); // same nullifier for both!

      // Append a nullifier to cache
      await nullifiers.append(contractAddress, nullifier);
      // Can't append again
      await expect(nullifiers.append(contractAddress, nullifier)).rejects.toThrow(
        `Nullifier ${nullifier} at contract ${contractAddress} already exists in parent cache or host.`,
      );
    });
    it('Cant append nullifier that already exists in parent cache', async () => {
      const contractAddress = new Fr(1);
      const nullifier = new Fr(2); // same nullifier for both!

      // Append a nullifier to parent
      await nullifiers.append(contractAddress, nullifier);
      const childNullifiers = new Nullifiers(commitmentsDb, nullifiers);
      // Can't append again in child
      await expect(childNullifiers.append(contractAddress, nullifier)).rejects.toThrow(
        `Nullifier ${nullifier} at contract ${contractAddress} already exists in parent cache or host.`,
      );
    });
    it('Cant append nullifier that already exist in host', async () => {
      const contractAddress = new Fr(1);
      const nullifier = new Fr(2); // same nullifier for both!
      const storedLeafIndex = BigInt(420);

      // Nullifier exists in host
      commitmentsDb.getNullifierIndex.mockResolvedValue(Promise.resolve(storedLeafIndex));
      // Can't append to cache
      await expect(nullifiers.append(contractAddress, nullifier)).rejects.toThrow(
        `Nullifier ${nullifier} at contract ${contractAddress} already exists in parent cache or host.`,
      );
    });
  });

  describe('Nullifier cache merging', () => {
    it('Should be able to merge two nullifier caches together', async () => {
      const contractAddress = new Fr(1);
      const nullifier0 = new Fr(2);
      const nullifier1 = new Fr(3);

      // Append a nullifier to parent
      await nullifiers.append(contractAddress, nullifier0);

      const childNullifiers = new Nullifiers(commitmentsDb, nullifiers);
      // Append a nullifier to child
      await childNullifiers.append(contractAddress, nullifier1);

      // Parent accepts child's nullifiers
      nullifiers.acceptAndMerge(childNullifiers);

      // After merge, parent has both nullifiers
      const results0 = await nullifiers.checkExists(contractAddress, nullifier0);
      expect(results0).toEqual([/*exists=*/ true, /*isPending=*/ true, /*leafIndex=*/ Fr.ZERO]);
      const results1 = await nullifiers.checkExists(contractAddress, nullifier1);
      expect(results1).toEqual([/*exists=*/ true, /*isPending=*/ true, /*leafIndex=*/ Fr.ZERO]);
    });
    it('Cant merge two nullifier caches with colliding entries', async () => {
      const contractAddress = new Fr(1);
      const nullifier = new Fr(2);

      // Append a nullifier to parent
      await nullifiers.append(contractAddress, nullifier);

      // Create child cache, don't derive from parent so we can concoct a collision on merge
      const childNullifiers = new Nullifiers(commitmentsDb);
      // Append a nullifier to child
      await childNullifiers.append(contractAddress, nullifier);

      // Parent accepts child's nullifiers
      expect(() => nullifiers.acceptAndMerge(childNullifiers)).toThrow(
        `Failed to accept child call's nullifiers. Nullifier ${nullifier.toBigInt()} already exists at contract ${contractAddress.toBigInt()}.`,
      );
    });
  });
});
