import { AztecNode } from '@aztec/circuit-types';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { TestKeyStore } from '@aztec/key-store';
import { AztecLmdbStore } from '@aztec/kv-store';

import { join } from 'path';

import { PXEServiceConfig } from '../config/index.js';
import { KVPxeDatabase } from '../database/kv_pxe_database.js';
import { PXEService } from './pxe_service.js';

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
  useLogSuffix: string | boolean | undefined = undefined,
) {
  const logSuffix =
    typeof useLogSuffix === 'boolean'
      ? useLogSuffix
        ? Math.random().toString(16).slice(2, 8)
        : undefined
      : useLogSuffix;

  const pxeDbPath = config.dataDirectory ? join(config.dataDirectory, 'pxe_data') : undefined;
  const keyStorePath = config.dataDirectory ? join(config.dataDirectory, 'pxe_key_store') : undefined;
  const l1Contracts = await aztecNode.getL1ContractAddresses();

  const keyStore = new TestKeyStore(new Grumpkin(), await AztecLmdbStore.open(l1Contracts.rollupAddress, keyStorePath));
  const db = new KVPxeDatabase(await AztecLmdbStore.open(l1Contracts.rollupAddress, pxeDbPath));

  const server = new PXEService(keyStore, aztecNode, db, config, logSuffix);

  await server.start();
  return server;
}
