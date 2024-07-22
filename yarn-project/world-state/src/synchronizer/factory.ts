import { type L1ToL2MessageSource, type L2BlockSource } from '@aztec/circuit-types';
import { type AztecKVStore } from '@aztec/kv-store';

import { MerkleTrees } from '../world-state-db/merkle_trees.js';
import { type WorldStateConfig } from './config.js';
import { ServerWorldStateSynchronizer } from './server_world_state_synchronizer.js';

export async function createWorldStateSynchronizer(
  config: WorldStateConfig,
  store: AztecKVStore,
  l2BlockSource: L2BlockSource & L1ToL2MessageSource,
) {
  const merkleTrees = await MerkleTrees.new(store);
  return new ServerWorldStateSynchronizer(store, merkleTrees, l2BlockSource, config);
}
