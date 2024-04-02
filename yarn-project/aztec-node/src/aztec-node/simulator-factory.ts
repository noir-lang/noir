import { type DebugLogger } from '@aztec/foundation/log';
import { NativeACVMSimulator, type SimulationProvider, WASMSimulator } from '@aztec/simulator';

import * as fs from 'fs/promises';

import { type AztecNodeConfig } from './config.js';

export async function getSimulationProvider(
  config: AztecNodeConfig,
  logger?: DebugLogger,
): Promise<SimulationProvider> {
  if (config.acvmBinaryPath && config.acvmWorkingDirectory) {
    try {
      await fs.access(config.acvmBinaryPath, fs.constants.R_OK);
      await fs.mkdir(config.acvmWorkingDirectory, { recursive: true });
      logger?.(`Using native ACVM at ${config.acvmBinaryPath} and working directory ${config.acvmWorkingDirectory}`);
      return new NativeACVMSimulator(config.acvmWorkingDirectory, config.acvmBinaryPath);
    } catch {
      logger?.(`Failed to access ACVM at ${config.acvmBinaryPath}, falling back to WASM`);
    }
  }
  logger?.('Using WASM ACVM simulation');
  return new WASMSimulator();
}
