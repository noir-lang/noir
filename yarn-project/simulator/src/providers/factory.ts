import { type DebugLogger, createDebugLogger } from '@aztec/foundation/log';

import * as fs from 'fs/promises';

import { NativeACVMSimulator } from './acvm_native.js';
import { WASMSimulator } from './acvm_wasm.js';
import { type SimulationProvider } from './simulation_provider.js';

export type SimulationProviderConfig = {
  acvmBinaryPath?: string;
  acvmWorkingDirectory?: string;
};

export function getSimulationProviderConfigFromEnv() {
  const { ACVM_BINARY_PATH, ACVM_WORKING_DIRECTORY } = process.env;
  return {
    acvmWorkingDirectory: ACVM_WORKING_DIRECTORY ? ACVM_WORKING_DIRECTORY : undefined,
    acvmBinaryPath: ACVM_BINARY_PATH ? ACVM_BINARY_PATH : undefined,
  };
}

export async function createSimulationProvider(
  config: SimulationProviderConfig,
  logger: DebugLogger = createDebugLogger('aztec:simulator'),
): Promise<SimulationProvider> {
  if (config.acvmBinaryPath && config.acvmWorkingDirectory) {
    try {
      await fs.access(config.acvmBinaryPath, fs.constants.R_OK);
      await fs.mkdir(config.acvmWorkingDirectory, { recursive: true });
      logger.info(`Using native ACVM at ${config.acvmBinaryPath} and working directory ${config.acvmWorkingDirectory}`);
      return new NativeACVMSimulator(config.acvmWorkingDirectory, config.acvmBinaryPath);
    } catch {
      logger.warn(`Failed to access ACVM at ${config.acvmBinaryPath}, falling back to WASM`);
    }
  }
  logger.info('Using WASM ACVM simulation');
  return new WASMSimulator();
}
