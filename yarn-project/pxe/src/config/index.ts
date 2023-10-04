import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';

/**
 * Configuration settings for the PXE Service.
 */
export interface PXEServiceConfig {
  /**
   * The interval to wait between polling for new blocks.
   */
  l2BlockPollingIntervalMS: number;
}

/**
 * Creates an instance of PXEServiceConfig out of environment variables using sensible defaults for integration testing if not set.
 */
export function getPXEServiceConfig(): PXEServiceConfig {
  const { PXE_BLOCK_POLLING_INTERVAL_MS } = process.env;

  return {
    l2BlockPollingIntervalMS: PXE_BLOCK_POLLING_INTERVAL_MS ? +PXE_BLOCK_POLLING_INTERVAL_MS : 1000,
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
