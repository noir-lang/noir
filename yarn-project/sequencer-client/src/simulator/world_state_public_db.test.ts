import { MerkleTreeId } from '@aztec/circuit-types';
import { AztecAddress, Fr, PublicDataTreeLeafPreimage } from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot } from '@aztec/circuits.js/hash';
import { IndexedTreeLeafPreimage } from '@aztec/foundation/trees';
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
    const publicDataEntries = Array(DB_VALUES_SIZE)
      .fill(0)
      .map((_, idx: number) => {
        const leafSlot = computePublicDataTreeLeafSlot(addresses[idx], slots[idx]);
        return new PublicDataTreeLeafPreimage(leafSlot, dbValues[idx], Fr.ZERO, 0n);
      });
    dbStorage = new Map<number, Map<bigint, Buffer>>([
      [
        MerkleTreeId.PUBLIC_DATA_TREE,
        new Map(publicDataEntries.map((preimage, idx) => [BigInt(idx), preimage.toBuffer()])),
      ],
    ]);
    db = mock<MerkleTreeOperations>();
    db.getPreviousValueIndex.mockImplementation(
      (
        treeId: MerkleTreeId,
        leafSlot: bigint,
      ): Promise<
        | {
            index: bigint;
            alreadyPresent: boolean;
          }
        | undefined
      > => {
        const sortedByLeafSlot = publicDataEntries.slice().sort((a, b) => Number(a.getKey() - b.getKey()));
        let findResult = undefined;
        for (const preimage of sortedByLeafSlot) {
          if (preimage.getKey() > leafSlot) {
            break;
          }
          findResult = {
            index: BigInt(publicDataEntries.indexOf(preimage)),
            alreadyPresent: preimage.getKey() === leafSlot,
          };
        }

        return Promise.resolve(findResult);
      },
    );
    db.getLeafPreimage.mockImplementation((treeId: MerkleTreeId, index: bigint): Promise<IndexedTreeLeafPreimage> => {
      const tree = dbStorage.get(treeId);
      if (!tree) {
        throw new Error('Invalid Tree Id');
      }

      return Promise.resolve(PublicDataTreeLeafPreimage.fromBuffer(tree.get(index)!));
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
