import { type ProverConfig, proverConfigMappings } from '@aztec/circuit-types';
import { type ConfigMappingsType, getConfigFromMappings } from '@aztec/foundation/config';

/**
 * The prover configuration.
 */
export type ProverClientConfig = ProverConfig & {
  /** The working directory to use for simulation/proving */
  acvmWorkingDirectory: string;
  /** The path to the ACVM binary */
  acvmBinaryPath: string;
  /** The working directory to for proving */
  bbWorkingDirectory: string;
  /** The path to the bb binary */
  bbBinaryPath: string;
};

export const proverClientConfigMappings: ConfigMappingsType<ProverClientConfig> = {
  acvmWorkingDirectory: {
    env: 'ACVM_WORKING_DIRECTORY',
    description: 'The working directory to use for simulation/proving',
  },
  acvmBinaryPath: {
    env: 'ACVM_BINARY_PATH',
    description: 'The path to the ACVM binary',
  },
  bbWorkingDirectory: {
    env: 'BB_WORKING_DIRECTORY',
    description: 'The working directory to for proving',
  },
  bbBinaryPath: {
    env: 'BB_BINARY_PATH',
    description: 'The path to the bb binary',
  },
  ...proverConfigMappings,
};

/**
 * Returns the prover configuration from the environment variables.
 * Note: If an environment variable is not set, the default value is used.
 * @returns The prover configuration.
 */
export function getProverEnvVars(): ProverClientConfig {
  return getConfigFromMappings<ProverClientConfig>(proverClientConfigMappings);
}
