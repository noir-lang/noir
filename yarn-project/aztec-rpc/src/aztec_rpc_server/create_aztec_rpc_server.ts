import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { TestKeyStore } from '@aztec/key-store';
import { AztecNode, KeyStore } from '@aztec/types';

import { RpcServerConfig } from '../config/index.js';
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
 * @param config - The Rpc Server Config to use
 * @param options - (Optional) Optional information for creating an AztecRPCServer.
 * @returns A Promise that resolves to the started AztecRPCServer instance.
 */
export async function createAztecRPCServer(
  aztecNode: AztecNode,
  config: RpcServerConfig,
  { keyStore, db }: CreateAztecRPCServerOptions = {},
  useLogSuffix: string | boolean | undefined = undefined,
) {
  const logSuffix =
    typeof useLogSuffix === 'boolean'
      ? useLogSuffix
        ? Math.random().toString(16).slice(2, 8)
        : undefined
      : useLogSuffix;

  keyStore = keyStore || new TestKeyStore(await Grumpkin.new());
  db = db || new MemoryDB(logSuffix);

  const server = new AztecRPCServer(keyStore, aztecNode, db, config, logSuffix);
  await server.start();
  return server;
}
