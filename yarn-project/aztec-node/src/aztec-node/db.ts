import { LogFn } from '@aztec/foundation/log';

import { LevelDown, default as leveldown } from 'leveldown';
import { LevelUp, default as levelup } from 'levelup';
import { RootDatabase, open } from 'lmdb';
import { MemDown, default as memdown } from 'memdown';
import { mkdir } from 'node:fs/promises';
import { join } from 'node:path';

import { AztecNodeConfig } from './config.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;
export const createLevelDown = (path: string) => (leveldown as any)(path) as LevelDown;

const DB_SUBDIR = 'aztec-node-db';
const WORLD_STATE_SUBDIR = 'aztec-world-state-db';
const NODE_METADATA_KEY = '@@aztec_node_metadata';

/**
 * The metadata for an aztec node.
 */
type NodeMetadata = {
  /**
   * The address of the rollup contract on L1
   */
  rollupContractAddress: string;
};

/**
 * Opens the database for the aztec node. If a data directory is specified, then this attempts to create it.
 * @param config - The configuration to be used by the aztec node.
 * @throws If `config.dataDirectory` is set and the directory cannot be created.
 * @returns The database for the aztec node.
 */
export async function openDb(
  config: AztecNodeConfig,
  log: LogFn,
): Promise<[nodeDb: RootDatabase, worldStateDb: LevelUp]> {
  const nodeMetadata: NodeMetadata = {
    rollupContractAddress: config.l1Contracts.rollupAddress.toString(),
  };

  let nodeDb: RootDatabase;
  let worldStateDb: LevelUp;

  if (config.dataDirectory) {
    const nodeDir = join(config.dataDirectory, DB_SUBDIR);
    const worldStateDir = join(config.dataDirectory, WORLD_STATE_SUBDIR);
    // this throws if we don't have permissions to create the directory
    await mkdir(nodeDir, { recursive: true });
    await mkdir(worldStateDir, { recursive: true });

    log(`Opening aztec-node database at ${nodeDir}`);
    nodeDb = open(nodeDir, {});

    log(`Opening world-state database at ${worldStateDir}`);
    worldStateDb = levelup(createLevelDown(worldStateDir));
  } else {
    log('Opening temporary databases');
    // not passing a path will use a temp file that gets deleted when the process exits
    nodeDb = open({});
    worldStateDb = levelup(createMemDown());
  }

  await checkNodeMetadataAndClear(nodeDb, worldStateDb, nodeMetadata, log);
  return [nodeDb, worldStateDb];
}

/**
 * Checks the node metadata and clears the database if the rollup contract address has changed.
 * @param nodeDb - The database for the aztec node.
 * @param nodeMetadata - The metadata for the aztec node.
 */
async function checkNodeMetadataAndClear(
  nodeDb: RootDatabase,
  worldStateDb: LevelUp,
  nodeMetadata: NodeMetadata,
  log: LogFn,
): Promise<void> {
  const metadataDB = nodeDb.openDB<NodeMetadata, string>('metadata', {});
  try {
    const existing = metadataDB.get(NODE_METADATA_KEY);
    // if the rollup addresses are different, wipe the local database and start over
    if (!existing || existing.rollupContractAddress !== nodeMetadata.rollupContractAddress) {
      log('Rollup contract address has changed, clearing databases');
      await Promise.all([nodeDb.clearAsync(), worldStateDb.clear()]);
    }
    await metadataDB.put(NODE_METADATA_KEY, nodeMetadata);
  } finally {
    await metadataDB.close();
  }
}
