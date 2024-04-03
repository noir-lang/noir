import { type Tx } from '@aztec/circuit-types';
import {
  type GlobalVariables,
  type Header,
  type Proof,
  type PublicKernelCircuitPublicInputs,
} from '@aztec/circuits.js';
import { type PublicExecutor, type PublicStateDB } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { type PublicKernelCircuitSimulator } from '../simulator/index.js';
import { type ContractsDataSourcePublicDB } from '../simulator/public_executor.js';
import { AbstractPhaseManager, PublicKernelPhase } from './abstract_phase_manager.js';

/**
 * The phase manager responsible for performing the fee preparation phase.
 */
export class AppLogicPhaseManager extends AbstractPhaseManager {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected globalVariables: GlobalVariables,
    protected historicalHeader: Header,
    protected publicContractsDB: ContractsDataSourcePublicDB,
    protected publicStateDB: PublicStateDB,
    public phase: PublicKernelPhase = PublicKernelPhase.APP_LOGIC,
  ) {
    super(db, publicExecutor, publicKernel, globalVariables, historicalHeader, phase);
  }

  override async handle(
    tx: Tx,
    previousPublicKernelOutput: PublicKernelCircuitPublicInputs,
    previousPublicKernelProof: Proof,
  ) {
    // add new contracts to the contracts db so that their functions may be found and called
    // TODO(#4073): This is catching only private deployments, when we add public ones, we'll
    // have to capture contracts emitted in that phase as well.
    // TODO(@spalladino): Should we allow emitting contracts in the fee preparation phase?
    this.log(`Processing tx ${tx.getTxHash()}`);
    await this.publicContractsDB.addNewContracts(tx);
    const [publicKernelOutput, publicKernelProof, newUnencryptedFunctionLogs, revertReason, returnValues] =
      await this.processEnqueuedPublicCalls(tx, previousPublicKernelOutput, previousPublicKernelProof).catch(
        // if we throw for any reason other than simulation, we need to rollback and drop the TX
        async err => {
          await this.publicStateDB.rollbackToCommit();
          throw err;
        },
      );

    if (revertReason) {
      await this.publicContractsDB.removeNewContracts(tx);
      await this.publicStateDB.rollbackToCheckpoint();
    } else {
      tx.unencryptedLogs.addFunctionLogs(newUnencryptedFunctionLogs);
      await this.publicStateDB.checkpoint();
    }

    return { publicKernelOutput, publicKernelProof, revertReason, returnValues };
  }
}
