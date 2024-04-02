import { type AztecNode } from '@aztec/circuit-types';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { randomBytes } from '@aztec/foundation/crypto';
import { TestKeyStore } from '@aztec/key-store';
import { AztecLmdbStore } from '@aztec/kv-store/lmdb';
import { initStoreForRollup } from '@aztec/kv-store/utils';
import { getCanonicalClassRegisterer } from '@aztec/protocol-contracts/class-registerer';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { getCanonicalInstanceDeployer } from '@aztec/protocol-contracts/instance-deployer';
import { getCanonicalMultiCallEntrypointContract } from '@aztec/protocol-contracts/multi-call-entrypoint';

import { join } from 'path';

import { type PXEServiceConfig } from '../config/index.js';
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
    typeof useLogSuffix === 'boolean' ? (useLogSuffix ? randomBytes(3).toString('hex') : undefined) : useLogSuffix;

  const pxeDbPath = config.dataDirectory ? join(config.dataDirectory, 'pxe_data') : undefined;
  const keyStorePath = config.dataDirectory ? join(config.dataDirectory, 'pxe_key_store') : undefined;
  const l1Contracts = await aztecNode.getL1ContractAddresses();

  const keyStore = new TestKeyStore(
    new Grumpkin(),
    await initStoreForRollup(AztecLmdbStore.open(keyStorePath), l1Contracts.rollupAddress),
  );
  const db = new KVPxeDatabase(await initStoreForRollup(AztecLmdbStore.open(pxeDbPath), l1Contracts.rollupAddress));

  const server = new PXEService(keyStore, aztecNode, db, config, logSuffix);
  for (const contract of [
    getCanonicalClassRegisterer(),
    getCanonicalInstanceDeployer(),
    getCanonicalMultiCallEntrypointContract(),
    getCanonicalGasToken(l1Contracts.gasPortalAddress),
  ]) {
    await server.registerContract(contract);
  }

  await server.start();
  return server;
}
