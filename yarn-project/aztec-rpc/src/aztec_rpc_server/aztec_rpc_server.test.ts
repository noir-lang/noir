import { AztecAddress, CompleteAddress, Fr } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { ConstantKeyPair, TestKeyStore } from '@aztec/key-store';
import { AztecNode } from '@aztec/types';

import { mock } from 'jest-mock-extended';

import { MemoryDB } from '../database/memory_db.js';
import { RpcServerConfig } from '../index.js';
import { AztecRPCServer } from './aztec_rpc_server.js';

describe('AztecRpcServer', function () {
  let rpcServer: AztecRPCServer;

  beforeEach(async () => {
    const keyStore = new TestKeyStore(await Grumpkin.new());
    const node = mock<AztecNode>();
    const db = new MemoryDB();
    const config: RpcServerConfig = {
      l2BlockPollingIntervalMS: 100,
    };
    rpcServer = new AztecRPCServer(keyStore, node, db, config);
  });

  it('registers an account and returns it as an account only and not as a recipient', async () => {
    const keyPair = ConstantKeyPair.random(await Grumpkin.new());
    const completeAddress = await CompleteAddress.fromPrivateKey(await keyPair.getPrivateKey());

    await rpcServer.registerAccount(await keyPair.getPrivateKey(), completeAddress);
    const accounts = await rpcServer.getAccounts();
    const recipients = await rpcServer.getRecipients();
    expect(accounts).toEqual([completeAddress]);
    expect(recipients).toEqual([]);
  });

  it('registers a recipient and returns it as a recipient only and not as an account', async () => {
    const completeAddress = await CompleteAddress.random();

    await rpcServer.registerRecipient(completeAddress);
    const accounts = await rpcServer.getAccounts();
    const recipients = await rpcServer.getRecipients();
    expect(accounts).toEqual([]);
    expect(recipients).toEqual([completeAddress]);
  });

  it('cannot register the same account twice', async () => {
    const keyPair = ConstantKeyPair.random(await Grumpkin.new());
    const completeAddress = await CompleteAddress.fromPrivateKey(await keyPair.getPrivateKey());

    await rpcServer.registerAccount(await keyPair.getPrivateKey(), completeAddress);
    await expect(async () => rpcServer.registerAccount(await keyPair.getPrivateKey(), completeAddress)).rejects.toThrow(
      `Complete address corresponding to ${completeAddress.address} already exists`,
    );
  });

  it('cannot register the same recipient twice', async () => {
    const completeAddress = await CompleteAddress.random();

    await rpcServer.registerRecipient(completeAddress);
    await expect(() => rpcServer.registerRecipient(completeAddress)).rejects.toThrow(
      `Complete address corresponding to ${completeAddress.address} already exists`,
    );
  });

  it('throws when getting public storage for non-existent contract', async () => {
    const contract = AztecAddress.random();
    await expect(async () => await rpcServer.getPublicStorageAt(contract, new Fr(0n))).rejects.toThrow(
      `Contract ${contract.toString()} is not deployed`,
    );
  });
});
