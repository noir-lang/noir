import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

/**
 * Configuration settings for the RPC Server.
 */
export interface RpcServerConfig {
  /**
   * The interval to wait between polling for new blocks.
   */
  l2BlockPollingIntervalMS: number;
}

/**
 * Creates an instance of SequencerClientConfig out of environment variables using sensible defaults for integration testing if not set.
 */
export function getConfigEnvVars(): RpcServerConfig {
  const { RPC_SERVER_BLOCK_POLLING_INTERVAL_MS } = process.env;

  return {
    l2BlockPollingIntervalMS: RPC_SERVER_BLOCK_POLLING_INTERVAL_MS ? +RPC_SERVER_BLOCK_POLLING_INTERVAL_MS : 1000,
  };
}

/**
 * Returns package name and version.
 */
export function getPackageInfo() {
  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../../package.json');
  const { version, name } = JSON.parse(readFileSync(packageJsonPath).toString());

  return { version, name };
}
