import { Tx } from '@aztec/circuit-types';
import { GlobalVariables, Header, Proof, PublicKernelCircuitPublicInputs } from '@aztec/circuits.js';
import { PublicExecutor, PublicStateDB } from '@aztec/simulator';
import { MerkleTreeOperations } from '@aztec/world-state';

import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ContractsDataSourcePublicDB } from '../simulator/public_executor.js';
import { AbstractPhaseManager, PublicKernelPhase } from './abstract_phase_manager.js';
import { FailedTx } from './processed_tx.js';

/**
 * The phase manager responsible for performing the fee preparation phase.
 */
export class AppLogicPhaseManager extends AbstractPhaseManager {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected publicProver: PublicProver,
    protected globalVariables: GlobalVariables,
    protected historicalHeader: Header,
    protected publicContractsDB: ContractsDataSourcePublicDB,
    protected publicStateDB: PublicStateDB,
    public phase: PublicKernelPhase = PublicKernelPhase.APP_LOGIC,
  ) {
    super(db, publicExecutor, publicKernel, publicProver, globalVariables, historicalHeader, phase);
  }

  async handle(
    tx: Tx,
    previousPublicKernelOutput: PublicKernelCircuitPublicInputs,
    previousPublicKernelProof: Proof,
  ): Promise<{
    /**
     * the output of the public kernel circuit for this phase
     */
    publicKernelOutput: PublicKernelCircuitPublicInputs;
    /**
     * the proof of the public kernel circuit for this phase
     */
    publicKernelProof: Proof;
  }> {
    // add new contracts to the contracts db so that their functions may be found and called
    // TODO(#4073): This is catching only private deployments, when we add public ones, we'll
    // have to capture contracts emitted in that phase as well.
    // TODO(@spalladino): Should we allow emitting contracts in the fee preparation phase?
    this.log(`Processing tx ${tx.getTxHash()}`);
    await this.publicContractsDB.addNewContracts(tx);
    this.log(`Executing enqueued public calls for tx ${tx.getTxHash()}`);
    const [publicKernelOutput, publicKernelProof, newUnencryptedFunctionLogs] = await this.processEnqueuedPublicCalls(
      tx,
      previousPublicKernelOutput,
      previousPublicKernelProof,
    );
    tx.unencryptedLogs.addFunctionLogs(newUnencryptedFunctionLogs);

    // commit the state updates from this transaction
    await this.publicStateDB.commit();

    return { publicKernelOutput, publicKernelProof };
  }

  async rollback(tx: Tx, err: unknown): Promise<FailedTx> {
    this.log.warn(`Error processing tx ${tx.getTxHash()}: ${err}`);
    // remove contracts on failure
    await this.publicContractsDB.removeNewContracts(tx);
    // rollback any state updates from this failed transaction
    await this.publicStateDB.rollback();
    return {
      tx,
      error: err instanceof Error ? err : new Error('Unknown error'),
    };
  }
}
