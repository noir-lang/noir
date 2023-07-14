/**
 * World State synchroniser configuration values.
 */
export interface WorldStateConfig {
  /**
   * The frequency in which to check.
   */
  worldStateBlockCheckIntervalMS: number;

  /**
   * Size of queue of L2 blocks to store.
   */
  l2QueueSize: number;
}

/**
 * Returns the configuration values for the world state synchroniser.
 * @returns The configuration values for the world state synchroniser.
 */
export function getConfigEnvVars(): WorldStateConfig {
  const { WS_BLOCK_CHECK_INTERVAL_MS, WS_L2_BLOCK_QUEUE_SIZE } = process.env;
  const envVars: WorldStateConfig = {
    worldStateBlockCheckIntervalMS: WS_BLOCK_CHECK_INTERVAL_MS ? +WS_BLOCK_CHECK_INTERVAL_MS : 100,
    l2QueueSize: WS_L2_BLOCK_QUEUE_SIZE ? +WS_L2_BLOCK_QUEUE_SIZE : 1000,
  };
  return envVars;
}
