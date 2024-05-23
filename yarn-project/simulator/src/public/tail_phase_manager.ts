import { type PublicKernelRequest, PublicKernelType, type Tx } from '@aztec/circuit-types';
import {
  type Fr,
  type GlobalVariables,
  type Header,
  type KernelCircuitPublicInputs,
  type LogHash,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  type MAX_UNENCRYPTED_LOGS_PER_TX,
  type NoteHash,
  type PublicKernelCircuitPublicInputs,
  PublicKernelTailCircuitPrivateInputs,
  mergeAccumulatedData,
  sortByCounter,
} from '@aztec/circuits.js';
import { type Tuple } from '@aztec/foundation/serialize';
import { type PublicExecutor, type PublicStateDB } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { AbstractPhaseManager, PublicKernelPhase, removeRedundantPublicDataWrites } from './abstract_phase_manager.js';
import { type ContractsDataSourcePublicDB } from './public_executor.js';
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
    phase: PublicKernelPhase = PublicKernelPhase.TAIL,
  ) {
    super(db, publicExecutor, publicKernel, globalVariables, historicalHeader, phase);
  }

  override async handle(tx: Tx, previousPublicKernelOutput: PublicKernelCircuitPublicInputs) {
    this.log.verbose(`Processing tx ${tx.getTxHash()}`);
    const [inputs, finalKernelOutput] = await this.runTailKernelCircuit(previousPublicKernelOutput).catch(
      // the abstract phase manager throws if simulation gives error in non-revertible phase
      async err => {
        await this.publicStateDB.rollbackToCommit();
        throw err;
      },
    );

    // TODO(#3675): This should be done in a public kernel circuit
    finalKernelOutput.end.publicDataUpdateRequests = removeRedundantPublicDataWrites(
      finalKernelOutput.end.publicDataUpdateRequests,
    );

    // Return a tail proving request
    const request: PublicKernelRequest = {
      type: PublicKernelType.TAIL,
      inputs: inputs,
    };

    return {
      kernelRequests: [request],
      publicKernelOutput: previousPublicKernelOutput,
      finalKernelOutput,
      revertReason: undefined,
      returnValues: [],
      gasUsed: undefined,
    };
  }

  private async runTailKernelCircuit(
    previousOutput: PublicKernelCircuitPublicInputs,
  ): Promise<[PublicKernelTailCircuitPrivateInputs, KernelCircuitPublicInputs]> {
    // Temporary hack. Should sort them in the tail circuit.
    previousOutput.end.unencryptedLogsHashes = this.sortLogsHashes<typeof MAX_UNENCRYPTED_LOGS_PER_TX>(
      previousOutput.end.unencryptedLogsHashes,
    );
    const [inputs, output] = await this.simulate(previousOutput);

    // Temporary hack. Should sort them in the tail circuit.
    const noteHashes = mergeAccumulatedData(
      previousOutput.endNonRevertibleData.newNoteHashes,
      previousOutput.end.newNoteHashes,
      MAX_NEW_NOTE_HASHES_PER_TX,
    );
    output.end.newNoteHashes = this.sortNoteHashes<typeof MAX_NEW_NOTE_HASHES_PER_TX>(noteHashes);

    return [inputs, output];
  }

  private async simulate(
    previousOutput: PublicKernelCircuitPublicInputs,
  ): Promise<[PublicKernelTailCircuitPrivateInputs, KernelCircuitPublicInputs]> {
    const inputs = await this.buildPrivateInputs(previousOutput);
    // We take a deep copy (clone) of these to pass to the prover
    return [inputs.clone(), await this.publicKernel.publicKernelCircuitTail(inputs)];
  }

  private async buildPrivateInputs(previousOutput: PublicKernelCircuitPublicInputs) {
    const previousKernel = this.getPreviousKernelData(previousOutput);

    const { validationRequests, endNonRevertibleData, end } = previousOutput;

    const pendingNullifiers = mergeAccumulatedData(
      endNonRevertibleData.newNullifiers,
      end.newNullifiers,
      MAX_NEW_NULLIFIERS_PER_TX,
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
      endNonRevertibleData.publicDataUpdateRequests,
      end.publicDataUpdateRequests,
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

    return new PublicKernelTailCircuitPrivateInputs(
      previousKernel,
      nullifierReadRequestHints,
      nullifierNonExistentReadRequestHints,
      publicDataHints,
      publicDataReadRequestHints,
      currentState.partial,
    );
  }

  private sortNoteHashes<N extends number>(noteHashes: Tuple<NoteHash, N>): Tuple<Fr, N> {
    return sortByCounter(noteHashes).map(n => n.value) as Tuple<Fr, N>;
  }

  private sortLogsHashes<N extends number>(unencryptedLogsHashes: Tuple<LogHash, N>): Tuple<LogHash, N> {
    // TODO(6052): logs here may have duplicate counters from nested calls
    return sortByCounter(unencryptedLogsHashes);
  }
}
