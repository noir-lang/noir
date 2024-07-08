import { type PublicKernelRequest, PublicKernelType, type Tx } from '@aztec/circuit-types';
import {
  CombineHints,
  type GlobalVariables,
  type Header,
  type KernelCircuitPublicInputs,
  MAX_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  type PublicKernelCircuitPublicInputs,
  PublicKernelTailCircuitPrivateInputs,
  mergeAccumulatedData,
} from '@aztec/circuits.js';
import { type ProtocolArtifact } from '@aztec/noir-protocol-circuits-types';
import { type PublicExecutor, type PublicStateDB } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { AbstractPhaseManager } from './abstract_phase_manager.js';
import { type ContractsDataSourcePublicDB } from './public_db_sources.js';
import { type PublicKernelCircuitSimulator } from './public_kernel_circuit_simulator.js';

export class TailPhaseManager extends AbstractPhaseManager {
  constructor(
    db: MerkleTreeOperations,
    publicExecutor: PublicExecutor,
    publicKernel: PublicKernelCircuitSimulator,
    globalVariables: GlobalVariables,
    historicalHeader: Header,
    protected publicContractsDB: ContractsDataSourcePublicDB,
    protected publicStateDB: PublicStateDB,
    phase: PublicKernelType = PublicKernelType.TAIL,
  ) {
    super(db, publicExecutor, publicKernel, globalVariables, historicalHeader, phase);
  }

  override async handle(
    tx: Tx,
    previousPublicKernelOutput: PublicKernelCircuitPublicInputs,
    previousKernelArtifact: ProtocolArtifact,
  ) {
    this.log.verbose(`Processing tx ${tx.getTxHash()}`);
    const [inputs, finalKernelOutput] = await this.simulate(previousPublicKernelOutput, previousKernelArtifact).catch(
      // the abstract phase manager throws if simulation gives error in non-revertible phase
      async err => {
        await this.publicStateDB.rollbackToCommit();
        throw err;
      },
    );

    // Return a tail proving request
    const kernelRequest: PublicKernelRequest = {
      type: PublicKernelType.TAIL,
      inputs: inputs,
    };

    return {
      publicProvingRequests: [kernelRequest],
      publicKernelOutput: previousPublicKernelOutput,
      lastKernelArtifact: 'PublicKernelTailArtifact' as ProtocolArtifact,
      finalKernelOutput,
      returnValues: [],
    };
  }

  private async simulate(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousKernelArtifact: ProtocolArtifact,
  ): Promise<[PublicKernelTailCircuitPrivateInputs, KernelCircuitPublicInputs]> {
    const inputs = await this.buildPrivateInputs(previousOutput, previousKernelArtifact);
    // We take a deep copy (clone) of these to pass to the prover
    return [inputs.clone(), await this.publicKernel.publicKernelCircuitTail(inputs)];
  }

  private async buildPrivateInputs(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousKernelArtifact: ProtocolArtifact,
  ) {
    const previousKernel = this.getPreviousKernelData(previousOutput, previousKernelArtifact);

    const { validationRequests, endNonRevertibleData: nonRevertibleData, end: revertibleData } = previousOutput;

    const pendingNullifiers = mergeAccumulatedData(
      nonRevertibleData.nullifiers,
      revertibleData.nullifiers,
      MAX_NULLIFIERS_PER_TX,
    );

    const nullifierReadRequestHints = await this.hintsBuilder.getNullifierReadRequestHints(
      validationRequests.nullifierReadRequests,
      pendingNullifiers,
    );

    const nullifierNonExistentReadRequestHints = await this.hintsBuilder.getNullifierNonExistentReadRequestHints(
      validationRequests.nullifierNonExistentReadRequests,
      pendingNullifiers,
    );

    const pendingPublicDataWrites = mergeAccumulatedData(
      nonRevertibleData.publicDataUpdateRequests,
      revertibleData.publicDataUpdateRequests,
      MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
    );

    const publicDataHints = await this.hintsBuilder.getPublicDataHints(
      validationRequests.publicDataReads,
      pendingPublicDataWrites,
    );

    const publicDataReadRequestHints = this.hintsBuilder.getPublicDataReadRequestHints(
      validationRequests.publicDataReads,
      pendingPublicDataWrites,
      publicDataHints,
    );

    const currentState = await this.db.getStateReference();

    const hints = CombineHints.fromPublicData({ nonRevertibleData, revertibleData });

    return new PublicKernelTailCircuitPrivateInputs(
      previousKernel,
      nullifierReadRequestHints,
      nullifierNonExistentReadRequestHints,
      publicDataHints,
      publicDataReadRequestHints,
      currentState.partial,
      hints,
    );
  }
}
