import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { MemoryDB } from '../database/index.js';
import { KeyStore, TestKeyStore } from '../key_store/index.js';
import { AztecRPCServer } from './aztec_rpc_server.js';

export async function createAztecRPCServer(
  aztecNode: AztecNode,
  {
    keyStore,
    db,
  }: {
    keyStore?: KeyStore;
    db?: MemoryDB;
  } = {},
) {
  keyStore = keyStore || new TestKeyStore(await Grumpkin.new());
  db = db || new MemoryDB();

  const server = new AztecRPCServer(keyStore, aztecNode, db);
  await server.start();
  return server;
}
