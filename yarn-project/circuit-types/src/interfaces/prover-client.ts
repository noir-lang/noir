import { type BlockProver } from './block-prover.js';
import { type ProvingJobSource } from './proving-job.js';

/**
 * The interface to the prover client.
 * Provides the ability to generate proofs and build rollups.
 */
export interface ProverClient extends BlockProver {
  start(): Promise<void>;

  stop(): Promise<void>;

  getProvingJobSource(): ProvingJobSource;
}
