import { LevelDown, default as leveldown } from 'leveldown';
import { LevelUp, default as levelup } from 'levelup';
import { MemDown, default as memdown } from 'memdown';
import { join } from 'node:path';

import { AztecNodeConfig } from './config.js';

export const createMemDown = () => (memdown as any)() as MemDown<any, any>;
export const createLevelDown = (path: string) => (leveldown as any)(path) as LevelDown;

const DB_SUBDIR = 'aztec-node';
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
 * Opens the database for the aztec node.
 * @param config - The configuration to be used by the aztec node.
 * @returns The database for the aztec node.
 */
export async function openDb(config: AztecNodeConfig): Promise<LevelUp> {
  const nodeMetadata: NodeMetadata = {
    rollupContractAddress: config.l1Contracts.rollupAddress.toString(),
  };

  const db = levelup(config.dataDirectory ? createLevelDown(join(config.dataDirectory, DB_SUBDIR)) : createMemDown());
  const prevNodeMetadata = await getNodeMetadata(db);

  // if the rollup addresses are different, wipe the local database and start over
  if (nodeMetadata.rollupContractAddress !== prevNodeMetadata.rollupContractAddress) {
    await db.clear();
  }

  await db.put(NODE_METADATA_KEY, JSON.stringify(nodeMetadata));
  return db;
}

/**
 * Gets the metadata for the aztec node.
 * @param db - The database for the aztec node.
 * @returns Node metadata.
 */
async function getNodeMetadata(db: LevelUp): Promise<NodeMetadata> {
  try {
    const value: Buffer = await db.get(NODE_METADATA_KEY);
    return JSON.parse(value.toString('utf-8'));
  } catch {
    return {
      rollupContractAddress: '',
    };
  }
}
