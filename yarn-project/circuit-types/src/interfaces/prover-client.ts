import { type TxHash } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { type ConfigMappingsType, booleanConfigHelper, numberConfigHelper } from '@aztec/foundation/config';

import { type BlockProver } from './block-prover.js';
import { type MerkleTreeOperations } from './merkle_tree_operations.js';
import { type ProvingJobSource } from './proving-job.js';

/**
 * The prover configuration.
 */
export type ProverConfig = {
  /** The URL to the Aztec node to take proving jobs from */
  nodeUrl?: string;
  /** Whether to construct real proofs */
  realProofs: boolean;
  /** Whether this prover has a local prover agent */
  proverAgentEnabled: boolean;
  /** The interval agents poll for jobs at */
  proverAgentPollInterval: number;
  /** The maximum number of proving jobs to be run in parallel */
  proverAgentConcurrency: number;
  /** Jobs are retried if not kept alive for this long */
  proverJobTimeoutMs: number;
  /** The interval to check job health status */
  proverJobPollIntervalMs: number;
  /** Artificial delay to introduce to all operations to the test prover. */
  proverTestDelayMs: number;
  /** Identifier of the prover */
  proverId?: Fr;
};

export const proverConfigMappings: ConfigMappingsType<ProverConfig> = {
  nodeUrl: {
    env: 'AZTEC_NODE_URL',
    description: 'The URL to the Aztec node to take proving jobs from',
  },
  realProofs: {
    env: 'PROVER_REAL_PROOFS',
    description: 'Whether to construct real proofs',
    ...booleanConfigHelper(),
  },
  proverAgentEnabled: {
    env: 'PROVER_AGENT_ENABLED',
    description: 'Whether this prover has a local prover agent',
    ...booleanConfigHelper(true),
  },
  proverAgentPollInterval: {
    env: 'PROVER_AGENT_POLL_INTERVAL_MS',
    description: 'The interval agents poll for jobs at',
    ...numberConfigHelper(100),
  },
  proverAgentConcurrency: {
    env: 'PROVER_AGENT_CONCURRENCY',
    description: 'The maximum number of proving jobs to be run in parallel',
    ...numberConfigHelper(1),
  },
  proverJobTimeoutMs: {
    env: 'PROVER_JOB_TIMEOUT_MS',
    description: 'Jobs are retried if not kept alive for this long',
    ...numberConfigHelper(60_000),
  },
  proverJobPollIntervalMs: {
    env: 'PROVER_JOB_POLL_INTERVAL_MS',
    description: 'The interval to check job health status',
    ...numberConfigHelper(1_000),
  },
  proverId: {
    env: 'PROVER_ID',
    parseEnv: (val: string) => parseProverId(val),
    description: 'Identifier of the prover',
  },
  proverTestDelayMs: {
    env: 'PROVER_TEST_DELAY_MS',
    description: 'Artificial delay to introduce to all operations to the test prover.',
    ...numberConfigHelper(0),
  },
};

function parseProverId(str: string) {
  return Fr.fromString(str.startsWith('0x') ? str : Buffer.from(str, 'utf8').toString('hex'));
}

/**
 * The interface to the prover client.
 * Provides the ability to generate proofs and build rollups.
 * TODO(palla/prover-node): Rename this interface
 */
export interface ProverClient {
  createBlockProver(db: MerkleTreeOperations): BlockProver;

  start(): Promise<void>;

  stop(): Promise<void>;

  getProvingJobSource(): ProvingJobSource;

  updateProverConfig(config: Partial<ProverConfig>): Promise<void>;
}

export class BlockProofError extends Error {
  static #name = 'BlockProofError';
  override name = BlockProofError.#name;

  constructor(message: string, public readonly txHashes: TxHash[]) {
    super(message);
  }

  static isBlockProofError(err: any): err is BlockProofError {
    return err && typeof err === 'object' && err.name === BlockProofError.#name;
  }
}
