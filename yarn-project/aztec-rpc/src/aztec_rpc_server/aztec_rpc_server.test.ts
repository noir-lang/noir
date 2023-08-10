import { AztecAddress, CircuitsWasm, Fr, Point, PrivateKey } from '@aztec/circuits.js';
import { computeContractAddressFromPartial } from '@aztec/circuits.js/abis';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { ConstantKeyPair, TestKeyStore } from '@aztec/key-store';
import { AztecNode } from '@aztec/types';

import { MockProxy, mock } from 'jest-mock-extended';

import { MemoryDB } from '../database/memory_db.js';
import { RpcServerConfig } from '../index.js';
import { AztecRPCServer } from './aztec_rpc_server.js';

describe('AztecRpcServer', function () {
  let wasm: CircuitsWasm;
  let keyStore: TestKeyStore;
  let db: MemoryDB;
  let node: MockProxy<AztecNode>;
  let rpcServer: AztecRPCServer;

  beforeEach(async () => {
    keyStore = new TestKeyStore(await Grumpkin.new());
    node = mock<AztecNode>();
    db = new MemoryDB();
    const config: RpcServerConfig = {
      l2BlockPollingIntervalMS: 100,
    };
    rpcServer = new AztecRPCServer(keyStore, node, db, config);
    wasm = await CircuitsWasm.get();
  });

  it('registers a public key in the db when adding a new account', async () => {
    const keyPair = ConstantKeyPair.random(await Grumpkin.new());
    const pubKey = keyPair.getPublicKey();
    const partialAddress = Fr.random();
    const address = computeContractAddressFromPartial(wasm, pubKey, partialAddress);

    await rpcServer.addAccount(await keyPair.getPrivateKey(), address, partialAddress);
    expect(await db.getPublicKeyAndPartialAddress(address)).toEqual([pubKey, partialAddress]);
  });

  it('refuses to add an account with incorrect address for given partial address and privkey', async () => {
    const privateKey = PrivateKey.random();
    const partialAddress = Fr.random();
    const address = AztecAddress.random();

    await expect(rpcServer.addAccount(privateKey, address, partialAddress)).rejects.toThrowError(/cannot be derived/);
  });

  it('refuses to add an account with incorrect address for given partial address and pubkey', async () => {
    const publicKey = Point.random();
    const partialAddress = Fr.random();
    const address = AztecAddress.random();

    await expect(rpcServer.addPublicKeyAndPartialAddress(address, publicKey, partialAddress)).rejects.toThrowError(
      /cannot be derived/,
    );
  });

  it('cannot add the same account twice', async () => {
    const keyPair = ConstantKeyPair.random(await Grumpkin.new());
    const pubKey = keyPair.getPublicKey();
    const partialAddress = Fr.random();
    const address = computeContractAddressFromPartial(wasm, pubKey, partialAddress);

    await rpcServer.addAccount(await keyPair.getPrivateKey(), address, partialAddress);
    await expect(async () =>
      rpcServer.addAccount(await keyPair.getPrivateKey(), address, partialAddress),
    ).rejects.toThrow(`Account ${address} already exists`);
  });

  it('throws when getting public storage for non-existent contract', async () => {
    const contract = AztecAddress.random();
    await expect(async () => await rpcServer.getPublicStorageAt(contract, new Fr(0n))).rejects.toThrow(
      `Contract ${contract.toString()} is not deployed`,
    );
  });
});
