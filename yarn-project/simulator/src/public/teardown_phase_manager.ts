import { PublicKernelType, type PublicProvingRequest, type Tx } from '@aztec/circuit-types';
import {
  type Fr,
  type Gas,
  type GlobalVariables,
  type Header,
  type PublicKernelCircuitPublicInputs,
} from '@aztec/circuits.js';
import { type ProtocolArtifact } from '@aztec/noir-protocol-circuits-types';
import { type PublicExecutor, type PublicStateDB } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { inspect } from 'util';

import { AbstractPhaseManager, makeAvmProvingRequest } from './abstract_phase_manager.js';
import { type ContractsDataSourcePublicDB } from './public_db_sources.js';
import { type PublicKernelCircuitSimulator } from './public_kernel_circuit_simulator.js';

/**
 * The phase manager responsible for performing the fee preparation phase.
 */
export class TeardownPhaseManager extends AbstractPhaseManager {
  constructor(
    db: MerkleTreeOperations,
    publicExecutor: PublicExecutor,
    publicKernel: PublicKernelCircuitSimulator,
    globalVariables: GlobalVariables,
    historicalHeader: Header,
    protected publicContractsDB: ContractsDataSourcePublicDB,
    protected publicStateDB: PublicStateDB,
    phase: PublicKernelType = PublicKernelType.TEARDOWN,
  ) {
    super(db, publicExecutor, publicKernel, globalVariables, historicalHeader, phase);
  }

  override async handle(
    tx: Tx,
    previousPublicKernelOutput: PublicKernelCircuitPublicInputs,
    previousKernelArtifact: ProtocolArtifact,
  ) {
    this.log.verbose(`Processing tx ${tx.getTxHash()}`);
    const { publicProvingInformation, kernelOutput, lastKernelArtifact, newUnencryptedLogs, revertReason, gasUsed } =
      await this.processEnqueuedPublicCalls(tx, previousPublicKernelOutput, previousKernelArtifact).catch(
        // the abstract phase manager throws if simulation gives error in a non-revertible phase
        async err => {
          await this.publicStateDB.rollbackToCommit();
          throw err;
        },
      );
    if (revertReason) {
      await this.publicStateDB.rollbackToCheckpoint();
      tx.filterRevertedLogs(kernelOutput);
    } else {
      // TODO(#6464): Should we allow emitting contracts in the public teardown phase?
      // if so, we should insert them here
      tx.unencryptedLogs.addFunctionLogs(newUnencryptedLogs);
    }

    // Return a list of teardown proving requests
    const publicProvingRequests: PublicProvingRequest[] = publicProvingInformation.map(info => {
      return makeAvmProvingRequest(info, PublicKernelType.TEARDOWN);
    });
    return {
      publicProvingRequests,
      publicKernelOutput: kernelOutput,
      lastKernelArtifact,
      revertReason,
      returnValues: [],
      gasUsed,
    };
  }

  protected override getTransactionFee(tx: Tx, previousPublicKernelOutput: PublicKernelCircuitPublicInputs): Fr {
    const gasSettings = tx.data.constants.txContext.gasSettings;
    const gasFees = this.globalVariables.gasFees;
    // No need to add teardown limits since they are already included in end.gasUsed
    const gasUsed = previousPublicKernelOutput.end.gasUsed.add(previousPublicKernelOutput.endNonRevertibleData.gasUsed);
    const txFee = gasSettings.inclusionFee.add(gasUsed.computeFee(gasFees));
    this.log.debug(`Computed tx fee`, { txFee, gasUsed: inspect(gasUsed), gasFees: inspect(gasFees) });
    return txFee;
  }

  protected override getAvailableGas(tx: Tx, _previousPublicKernelOutput: PublicKernelCircuitPublicInputs): Gas {
    return tx.data.constants.txContext.gasSettings.getTeardownLimits();
  }
}
