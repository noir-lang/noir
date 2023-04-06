import { AcirSimulator } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { BarretenbergWasm } from '@aztec/barretenberg.js/wasm';
import { mock } from 'jest-mock-extended';
import { Database, MemoryDB } from '../database/index.js';
import { ConstantKeyPair } from '../key_store/index.js';
import { Synchroniser } from './synchroniser.js';

describe('Synchroniser', () => {
  let grumpkin: Grumpkin;
  let aztecNode: ReturnType<typeof mock<AztecNode>>;
  let database: Database;
  let simulator: AcirSimulator;
  let synchroniser: Synchroniser;

  beforeAll(async () => {
    const wasm = await BarretenbergWasm.new();
    grumpkin = new Grumpkin(wasm);

    aztecNode = mock<AztecNode>();
    aztecNode.getUnverifiedData.mockResolvedValue([]);

    database = new MemoryDB();
    simulator = mock<AcirSimulator>();
    synchroniser = new Synchroniser(aztecNode, database, simulator, wasm);
  });

  it('Should create account state', async () => {
    const account = ConstantKeyPair.random(grumpkin);
    const address = account.getPublicKey().toAddress();

    expect(synchroniser.getAccount(address)).toBeUndefined();

    await synchroniser.addAccount(await account.getPrivateKey());

    expect(synchroniser.getAccount(address)!.getPublicKey()).toEqual(account.getPublicKey());
  });
});
