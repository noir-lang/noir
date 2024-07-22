import { type ProverConfig } from '@aztec/circuit-types';

import { tmpdir } from 'os';

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
  /** True to disable proving altogether. */
  disableProver: boolean;
};

/**
 * Returns the prover configuration from the environment variables.
 * Note: If an environment variable is not set, the default value is used.
 * @returns The prover configuration.
 */
export function getProverEnvVars(): ProverClientConfig {
  const {
    AZTEC_NODE_URL,
    ACVM_WORKING_DIRECTORY = tmpdir(),
    ACVM_BINARY_PATH = '',
    BB_WORKING_DIRECTORY = tmpdir(),
    BB_BINARY_PATH = '',
    PROVER_DISABLED = '',
    /** @deprecated */
    PROVER_AGENTS = '1',
    PROVER_AGENT_ENABLED = '1',
    PROVER_AGENT_CONCURRENCY = PROVER_AGENTS,
    PROVER_AGENT_POLL_INTERVAL_MS = '100',
    PROVER_REAL_PROOFS = '',
    PROVER_JOB_TIMEOUT_MS = '60000',
    PROVER_JOB_POLL_INTERVAL_MS = '1000',
  } = process.env;

  const realProofs = ['1', 'true'].includes(PROVER_REAL_PROOFS);
  const proverAgentEnabled = ['1', 'true'].includes(PROVER_AGENT_ENABLED);
  const proverAgentConcurrency = safeParseNumber(PROVER_AGENT_CONCURRENCY, 1);
  const proverAgentPollInterval = safeParseNumber(PROVER_AGENT_POLL_INTERVAL_MS, 100);
  const proverJobTimeoutMs = safeParseNumber(PROVER_JOB_TIMEOUT_MS, 60000);
  const proverJobPollIntervalMs = safeParseNumber(PROVER_JOB_POLL_INTERVAL_MS, 1000);
  const disableProver = ['1', 'true'].includes(PROVER_DISABLED);

  return {
    acvmWorkingDirectory: ACVM_WORKING_DIRECTORY,
    acvmBinaryPath: ACVM_BINARY_PATH,
    bbBinaryPath: BB_BINARY_PATH,
    bbWorkingDirectory: BB_WORKING_DIRECTORY,
    realProofs,
    disableProver,
    proverAgentEnabled,
    proverAgentPollInterval,
    proverAgentConcurrency,
    nodeUrl: AZTEC_NODE_URL,
    proverJobPollIntervalMs,
    proverJobTimeoutMs,
  };
}

function safeParseNumber(value: string, defaultValue: number): number {
  const parsedValue = parseInt(value, 10);
  return Number.isSafeInteger(parsedValue) ? parsedValue : defaultValue;
}
