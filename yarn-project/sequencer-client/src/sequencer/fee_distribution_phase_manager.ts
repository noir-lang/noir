import { Tx } from '@aztec/circuit-types';
import { GlobalVariables, Header, Proof, PublicCallRequest, PublicKernelPublicInputs } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { PublicExecutor, PublicStateDB } from '@aztec/simulator';
import { MerkleTreeOperations } from '@aztec/world-state';

import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ContractsDataSourcePublicDB } from '../simulator/public_executor.js';
import { AbstractPhaseManager } from './abstract_phase_manager.js';
import { FailedTx } from './processed_tx.js';

/**
 * The phase manager responsible for performing the fee preparation phase.
 */
export class FeeDistributionPhaseManager extends AbstractPhaseManager {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected publicProver: PublicProver,
    protected globalVariables: GlobalVariables,
    protected historicalHeader: Header,
    protected publicContractsDB: ContractsDataSourcePublicDB,
    protected publicStateDB: PublicStateDB,

    protected log = createDebugLogger('aztec:sequencer:fee-distribution'),
  ) {
    super(db, publicExecutor, publicKernel, publicProver, globalVariables, historicalHeader);
  }

  // this is a no-op for now
  extractEnqueuedPublicCalls(_tx: Tx): PublicCallRequest[] {
    return [];
  }

  // this is a no-op for now
  async handle(
    tx: Tx,
    previousPublicKernelOutput?: PublicKernelPublicInputs,
    previousPublicKernelProof?: Proof,
  ): Promise<{
    /**
     * the output of the public kernel circuit for this phase
     */
    publicKernelOutput?: PublicKernelPublicInputs;
    /**
     * the proof of the public kernel circuit for this phase
     */
    publicKernelProof?: Proof;
  }> {
    this.log.debug(`Handle ${await tx.getTxHash()} with no-op`);
    return {
      publicKernelOutput: previousPublicKernelOutput,
      publicKernelProof: previousPublicKernelProof,
    };
  }

  nextPhase() {
    return null;
  }

  async rollback(tx: Tx, err: unknown): Promise<FailedTx> {
    this.log.warn(`Error processing tx ${await tx.getTxHash()}: ${err}`);
    return {
      tx,
      error: err instanceof Error ? err : new Error('Unknown error'),
    };
  }
}
