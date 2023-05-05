import { AztecNode } from '@aztec/aztec-node';
import { Grumpkin } from '@aztec/barretenberg.js/crypto';
import { KeyStore, TestKeyStore } from '@aztec/key-store';
import { Database, MemoryDB } from '../database/index.js';
import { AztecRPCServer } from './aztec_rpc_server.js';

/**
 * Optional information for creating an AztecRPCServer.
 */
interface CreateAztecRPCServerOptions {
  /**
   * A secure storage for cryptographic keys.
   */
  keyStore?: KeyStore;
  /**
   * Storage for the RPC server.
   */
  db?: Database;
}

/**
 * Create and start an AztecRPCServer instance with the given AztecNode.
 * If no keyStore or database is provided, it will use TestKeyStore and MemoryDB as default values.
 * Returns a Promise that resolves to the started AztecRPCServer instance.
 *
 * @param aztecNode - The AztecNode instance to be used by the server.
 * @param options - (Optional) Optional information for creating an AztecRPCServer.
 * @returns A Promise that resolves to the started AztecRPCServer instance.
 */
export async function createAztecRPCServer(aztecNode: AztecNode, { keyStore, db }: CreateAztecRPCServerOptions = {}) {
  keyStore = keyStore || new TestKeyStore(await Grumpkin.new());
  db = db || new MemoryDB();

  const server = new AztecRPCServer(keyStore, aztecNode, db);
  await server.start();
  return server;
}
