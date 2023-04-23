/**
 * World State synchroniser configuration values.
 */
export interface WorldStateConfig {
  /**
   * The frequency in which to check.
   */
  checkInterval: number;

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
  const { WS_CHECK_INTERVAL, WS_L2_BLOCK_QUEUE_SIZE } = process.env;
  const envVars: WorldStateConfig = {
    checkInterval: WS_CHECK_INTERVAL ? +WS_CHECK_INTERVAL : 100,
    l2QueueSize: WS_L2_BLOCK_QUEUE_SIZE ? +WS_L2_BLOCK_QUEUE_SIZE : 1000,
  };
  return envVars;
}
