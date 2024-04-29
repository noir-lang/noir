import { tmpdir } from 'os';

/**
 * The prover configuration.
 */
export interface ProverConfig {
  /** The working directory to use for simulation/proving */
  acvmWorkingDirectory: string;
  /** The path to the ACVM binary */
  acvmBinaryPath: string;
  /** The working directory to for proving */
  bbWorkingDirectory: string;
  /** The path to the bb binary */
  bbBinaryPath: string;
  /** How many agents to start */
  proverAgents: number;
  /** Enable proving. If true, must set bb env vars */
  realProofs: boolean;
}

/**
 * Returns the prover configuration from the environment variables.
 * Note: If an environment variable is not set, the default value is used.
 * @returns The prover configuration.
 */
export function getProverEnvVars(): ProverConfig {
  const {
    ACVM_WORKING_DIRECTORY = tmpdir(),
    ACVM_BINARY_PATH = '',
    BB_WORKING_DIRECTORY = tmpdir(),
    BB_BINARY_PATH = '',
    PROVER_AGENTS = '1',
    PROVER_REAL_PROOFS = '',
  } = process.env;

  const parsedProverAgents = parseInt(PROVER_AGENTS, 10);
  const proverAgents = Number.isSafeInteger(parsedProverAgents) ? parsedProverAgents : 0;

  return {
    acvmWorkingDirectory: ACVM_WORKING_DIRECTORY,
    acvmBinaryPath: ACVM_BINARY_PATH,
    bbBinaryPath: BB_BINARY_PATH,
    bbWorkingDirectory: BB_WORKING_DIRECTORY,
    proverAgents,
    realProofs: ['1', 'true'].includes(PROVER_REAL_PROOFS),
  };
}
