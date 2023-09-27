import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { TestKeyStore } from '@aztec/key-store';
import { AztecNode, KeyStore } from '@aztec/types';

import { PXEServiceConfig } from '../config/index.js';
import { Database, MemoryDB } from '../database/index.js';
import { PXEService } from './pxe_service.js';

/**
 * Optional information for creating an PXEService.
 */
interface CreatePXEServiceOptions {
  /**
   * A secure storage for cryptographic keys.
   */
  keyStore?: KeyStore;
  /**
   * Storage for the PXE.
   */
  db?: Database;
}

/**
 * Create and start an PXEService instance with the given AztecNode.
 * If no keyStore or database is provided, it will use TestKeyStore and MemoryDB as default values.
 * Returns a Promise that resolves to the started PXEService instance.
 *
 * @param aztecNode - The AztecNode instance to be used by the server.
 * @param config - The PXE Service Config to use
 * @param options - (Optional) Optional information for creating an PXEService.
 * @returns A Promise that resolves to the started PXEService instance.
 */
export async function createPXEService(
  aztecNode: AztecNode,
  config: PXEServiceConfig,
  { keyStore, db }: CreatePXEServiceOptions = {},
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

  const server = new PXEService(keyStore, aztecNode, db, config, logSuffix);
  await server.start();
  return server;
}
