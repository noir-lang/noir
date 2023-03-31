import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { Point } from '@aztec/foundation';
import { L2Block, UnverifiedData } from '@aztec/l2-block';
import { randomBytes } from 'crypto';
import { TxAuxData } from '../aztec_rpc_server/tx_aux_data/tx_aux_data.js';
import { Database } from '../database/database.js';
import { MemoryDB } from '../database/memory_db.js';
import { Synchroniser } from './synchroniser.js';
import { mock } from 'jest-mock-extended';

describe('Synchroniser', () => {
  let grumpkin: Grumpkin;
  let aztecNode: ReturnType<typeof mock<AztecNode>>;
  let database: Database;
  let synchroniser: Synchroniser;
  const ownerPrivKey = randomBytes(32);
  const allMockedTxAuxData: TxAuxData[] = [];

  beforeAll(async () => {
    const wasm = await BarretenbergWasm.new();
    grumpkin = new Grumpkin(wasm);
    const ownerPubKey = Point.fromBuffer(grumpkin.mul(Grumpkin.generator, ownerPrivKey));

    // create array of 10 random blocks and 10 random unverified data
    const mockedBlocks = Array(10)
      .fill(0)
      .map((_, i) => L2Block.random(i));
    const mockedUnverifiedData = Array(10)
      .fill(0)
      .map(() => {
        const mockedTxAuxData = createArrayOfRandomTxAuxData();
        allMockedTxAuxData.push(...mockedTxAuxData);
        return createRandomUnverifiedData(mockedTxAuxData, ownerPubKey, grumpkin);
      });

    aztecNode = mock<AztecNode>();
    aztecNode.getBlocks.mockResolvedValueOnce(mockedBlocks);
    aztecNode.getUnverifiedData.mockResolvedValueOnce(mockedUnverifiedData);

    database = new MemoryDB();
    synchroniser = new Synchroniser(aztecNode, database);
    await synchroniser.addAccount(ownerPrivKey);
  });

  it('Synchroniser synchronises', async () => {
    synchroniser.start();
    await synchroniser.stop();
    // check all the mocked txAuxData are in the database
    for (const txAuxData of allMockedTxAuxData) {
      const txAuxDataDao = database.getStorageAt(txAuxData.contractAddress, txAuxData.storageSlot);
      expect(txAuxDataDao).toBeDefined();
    }
  });
});

function createArrayOfRandomTxAuxData(): TxAuxData[] {
  const numTxAuxData = Math.floor(Math.random() * 10) + 1;
  return Array(numTxAuxData)
    .fill(0)
    .map(() => TxAuxData.random());
}

function createRandomUnverifiedData(
  mockedTxAuxData: TxAuxData[],
  ownerPubKey: Point,
  grumpkin: Grumpkin,
): UnverifiedData {
  const ephPrivKey = randomBytes(32);
  const encryptedMockedTxAuxData = mockedTxAuxData.map(txAuxData => {
    return txAuxData.toEncryptedBuffer(ownerPubKey, ephPrivKey, grumpkin);
  });
  return new UnverifiedData(encryptedMockedTxAuxData);
}
