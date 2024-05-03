import { type AztecNode } from '@aztec/circuit-types';
import { randomBytes } from '@aztec/foundation/crypto';
import { TestKeyStore } from '@aztec/key-store';
import { AztecLmdbStore } from '@aztec/kv-store/lmdb';
import { initStoreForRollup } from '@aztec/kv-store/utils';
import { getCanonicalClassRegisterer } from '@aztec/protocol-contracts/class-registerer';
import { getCanonicalGasToken } from '@aztec/protocol-contracts/gas-token';
import { getCanonicalInstanceDeployer } from '@aztec/protocol-contracts/instance-deployer';
import { getCanonicalKeyRegistry } from '@aztec/protocol-contracts/key-registry';
import { getCanonicalMultiCallEntrypointContract } from '@aztec/protocol-contracts/multi-call-entrypoint';

import { join } from 'path';

import { type PXEServiceConfig } from '../config/index.js';
import { KVPxeDatabase } from '../database/kv_pxe_database.js';
import { BBNativeProofCreator } from '../kernel_prover/bb_prover/bb_native_proof_creator.js';
import { type ProofCreator } from '../kernel_prover/interface/proof_creator.js';
import { TestProofCreator } from '../kernel_prover/test/test_circuit_prover.js';
import { PXEService } from './pxe_service.js';

/**
 * Create and start an PXEService instance with the given AztecNode.
 * If no keyStore or database is provided, it will use TestKeyStore and MemoryDB as default values.
 * Returns a Promise that resolves to the started PXEService instance.
 *
 * @param aztecNode - The AztecNode instance to be used by the server.
 * @param config - The PXE Service Config to use
 * @param options - (Optional) Optional information for creating an PXEService.
 * @param proofCreator - An optional proof creator to use in place of any other configuration
 * @returns A Promise that resolves to the started PXEService instance.
 */
export async function createPXEService(
  aztecNode: AztecNode,
  config: PXEServiceConfig,
  useLogSuffix: string | boolean | undefined = undefined,
  proofCreator?: ProofCreator,
) {
  const logSuffix =
    typeof useLogSuffix === 'boolean' ? (useLogSuffix ? randomBytes(3).toString('hex') : undefined) : useLogSuffix;

  const pxeDbPath = config.dataDirectory ? join(config.dataDirectory, 'pxe_data') : undefined;
  const keyStorePath = config.dataDirectory ? join(config.dataDirectory, 'pxe_key_store') : undefined;
  const l1Contracts = await aztecNode.getL1ContractAddresses();

  const keyStore = new TestKeyStore(
    await initStoreForRollup(AztecLmdbStore.open(keyStorePath), l1Contracts.rollupAddress),
  );
  const db = new KVPxeDatabase(await initStoreForRollup(AztecLmdbStore.open(pxeDbPath), l1Contracts.rollupAddress));

  // (@PhilWindle) Temporary validation until WASM is implemented
  let prover: ProofCreator | undefined = proofCreator;
  if (!prover) {
    if (config.proverEnabled && (!config.bbBinaryPath || !config.bbWorkingDirectory)) {
      throw new Error(`Prover must be configured with binary path and working directory`);
    }
    prover = !config.proverEnabled
      ? new TestProofCreator()
      : new BBNativeProofCreator(config.bbBinaryPath!, config.bbWorkingDirectory!);
  }

  const server = new PXEService(keyStore, aztecNode, db, prover, config, logSuffix);
  for (const contract of [
    getCanonicalClassRegisterer(),
    getCanonicalInstanceDeployer(),
    getCanonicalMultiCallEntrypointContract(),
    getCanonicalGasToken(l1Contracts.gasPortalAddress),
    getCanonicalKeyRegistry(),
  ]) {
    await server.registerContract(contract);
  }

  await server.start();
  return server;
}
