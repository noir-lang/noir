import { AztecAddress, Fr } from '@aztec/circuits.js';
import { computePublicDataTreeIndex } from '@aztec/circuits.js/abis';
import { MerkleTreeId } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';

import { MockProxy, mock } from 'jest-mock-extended';

import { WorldStatePublicDB } from './public_executor.js';

const DB_VALUES_SIZE = 10;

describe('world_state_public_db', () => {
  let db: MockProxy<MerkleTreeOperations>;
  let dbStorage: Map<number, Map<bigint, Buffer>>;
  let addresses: AztecAddress[];
  let slots: Fr[];
  let dbValues: Fr[];

  beforeEach(() => {
    addresses = Array(DB_VALUES_SIZE).fill(0).map(AztecAddress.random);
    slots = Array(DB_VALUES_SIZE).fill(0).map(Fr.random);
    dbValues = Array(DB_VALUES_SIZE).fill(0).map(Fr.random);
    const publicData = new Map<bigint, Buffer>(
      Array(DB_VALUES_SIZE)
        .fill(0)
        .map((_, idx: number) => {
          const index = computePublicDataTreeIndex(addresses[idx], slots[idx]);
          return [index.toBigInt(), dbValues[idx].toBuffer()];
        }),
    );
    dbStorage = new Map<number, Map<bigint, Buffer>>([[MerkleTreeId.PUBLIC_DATA_TREE, publicData]]);
    db = mock<MerkleTreeOperations>();
    db.getLeafValue.mockImplementation((treeId: MerkleTreeId, index: bigint): Promise<Buffer | undefined> => {
      const tree = dbStorage.get(treeId);
      if (!tree) {
        throw new Error('Invalid Tree Id');
      }
      return Promise.resolve(tree.get(index));
    });
  });

  it('reads unwritten value from merkle tree db', async function () {
    const publicStateDb = new WorldStatePublicDB(db);
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(dbValues[0]);
    expect(await publicStateDb.storageRead(addresses[1], slots[1])).toEqual(dbValues[1]);
  });

  it('reads uncommitted value back', async function () {
    const publicStateDb = new WorldStatePublicDB(db);
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(dbValues[0]);

    const newValue = new Fr(dbValues[0].toBigInt() + 1n);

    // write a new value to our first value
    await publicStateDb.storageWrite(addresses[0], slots[0], newValue);

    // should read back the uncommited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue);

    // other slots should be unchanged
    expect(await publicStateDb.storageRead(addresses[1], slots[1])).toEqual(dbValues[1]);
  });

  it('reads committed value back', async function () {
    const publicStateDb = new WorldStatePublicDB(db);
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(dbValues[0]);

    const newValue = new Fr(dbValues[0].toBigInt() + 1n);

    // write a new value to our first value
    await publicStateDb.storageWrite(addresses[0], slots[0], newValue);

    // commit the data
    await publicStateDb.commit();

    // should read back the commited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue);

    // other slots should be unchanged
    expect(await publicStateDb.storageRead(addresses[1], slots[1])).toEqual(dbValues[1]);
  });

  it('will not rollback a commited value', async function () {
    const publicStateDb = new WorldStatePublicDB(db);
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(dbValues[0]);

    const newValue = new Fr(dbValues[0].toBigInt() + 1n);

    // write a new value to our first value
    await publicStateDb.storageWrite(addresses[0], slots[0], newValue);

    // commit the data
    await publicStateDb.commit();

    // should read back the commited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue);

    await publicStateDb.rollback();

    // should still read back the commited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue);
  });

  it('reads original value if rolled back uncommited value', async function () {
    const publicStateDb = new WorldStatePublicDB(db);
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(dbValues[0]);

    const newValue = new Fr(dbValues[0].toBigInt() + 1n);

    // write a new value to our first value
    await publicStateDb.storageWrite(addresses[0], slots[0], newValue);

    // should read back the uncommited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue);

    // now rollback
    await publicStateDb.rollback();

    // should now read the original value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(dbValues[0]);
  });

  it('reads newly uncommitted value back', async function () {
    const publicStateDb = new WorldStatePublicDB(db);
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(dbValues[0]);

    const newValue = new Fr(dbValues[0].toBigInt() + 1n);

    // write a new value to our first value
    await publicStateDb.storageWrite(addresses[0], slots[0], newValue);

    // commit the data
    await publicStateDb.commit();

    // should read back the commited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue);

    // other slots should be unchanged
    expect(await publicStateDb.storageRead(addresses[1], slots[1])).toEqual(dbValues[1]);

    // now update the slot again
    const newValue2 = new Fr(dbValues[0].toBigInt() + 2n);
    // write a new value to our first value
    await publicStateDb.storageWrite(addresses[0], slots[0], newValue2);

    // should read back the uncommited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue2);
  });

  it('rolls back to previously commited value', async function () {
    const publicStateDb = new WorldStatePublicDB(db);
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(dbValues[0]);

    const newValue = new Fr(dbValues[0].toBigInt() + 1n);

    // write a new value to our first value
    await publicStateDb.storageWrite(addresses[0], slots[0], newValue);

    // commit the data
    await publicStateDb.commit();

    // should read back the commited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue);

    // other slots should be unchanged
    expect(await publicStateDb.storageRead(addresses[1], slots[1])).toEqual(dbValues[1]);

    // now update the slot again
    const newValue2 = new Fr(dbValues[0].toBigInt() + 2n);
    // write a new value to our first value
    await publicStateDb.storageWrite(addresses[0], slots[0], newValue2);

    // should read back the uncommited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue2);

    // rollback
    await publicStateDb.rollback();

    // should read back the previously commited value
    expect(await publicStateDb.storageRead(addresses[0], slots[0])).toEqual(newValue);
  });
});
