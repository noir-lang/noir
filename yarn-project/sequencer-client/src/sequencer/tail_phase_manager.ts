import { type Tx } from '@aztec/circuit-types';
import {
  type GlobalVariables,
  type Header,
  type KernelCircuitPublicInputs,
  type MAX_NEW_NOTE_HASHES_PER_TX,
  type Proof,
  type PublicKernelCircuitPublicInputs,
  PublicKernelTailCircuitPrivateInputs,
  type SideEffect,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { type Tuple } from '@aztec/foundation/serialize';
import { type PublicExecutor, type PublicStateDB } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { type PublicKernelCircuitSimulator } from '../simulator/index.js';
import { type ContractsDataSourcePublicDB } from '../simulator/public_executor.js';
import { AbstractPhaseManager, PublicKernelPhase } from './abstract_phase_manager.js';

export class TailPhaseManager extends AbstractPhaseManager {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected globalVariables: GlobalVariables,
    protected historicalHeader: Header,
    protected publicContractsDB: ContractsDataSourcePublicDB,
    protected publicStateDB: PublicStateDB,
    public readonly phase: PublicKernelPhase = PublicKernelPhase.TAIL,
  ) {
    super(db, publicExecutor, publicKernel, globalVariables, historicalHeader, phase);
  }

  async handle(tx: Tx, previousPublicKernelOutput: PublicKernelCircuitPublicInputs, previousPublicKernelProof: Proof) {
    this.log(`Processing tx ${tx.getTxHash()}`);
    const [finalKernelOutput, publicKernelProof] = await this.runTailKernelCircuit(
      previousPublicKernelOutput,
      previousPublicKernelProof,
    ).catch(
      // the abstract phase manager throws if simulation gives error in non-revertible phase
      async err => {
        await this.publicStateDB.rollbackToCommit();
        throw err;
      },
    );

    // commit the state updates from this transaction
    await this.publicStateDB.commit();

    return {
      publicKernelOutput: previousPublicKernelOutput,
      finalKernelOutput,
      publicKernelProof,
      revertReason: undefined,
      returnValues: undefined,
    };
  }

  private async runTailKernelCircuit(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousProof: Proof,
  ): Promise<[KernelCircuitPublicInputs, Proof]> {
    const output = await this.simulate(previousOutput, previousProof);
    // Temporary hack. Should sort them in the tail circuit.
    // TODO(#757): Enforce proper ordering of public state actions
    output.end.newNoteHashes = this.sortNoteHashes<typeof MAX_NEW_NOTE_HASHES_PER_TX>(output.end.newNoteHashes);
    return [output, makeEmptyProof()];
  }

  private async simulate(
    previousOutput: PublicKernelCircuitPublicInputs,
    previousProof: Proof,
  ): Promise<KernelCircuitPublicInputs> {
    const previousKernel = this.getPreviousKernelData(previousOutput, previousProof);

    const { validationRequests, endNonRevertibleData, end } = previousOutput;
    const nullifierReadRequestHints = await this.hintsBuilder.getNullifierReadRequestHints(
      validationRequests.nullifierReadRequests,
      endNonRevertibleData.newNullifiers,
      end.newNullifiers,
    );
    const nullifierNonExistentReadRequestHints = await this.hintsBuilder.getNullifierNonExistentReadRequestHints(
      validationRequests.nullifierNonExistentReadRequests,
      endNonRevertibleData.newNullifiers,
      end.newNullifiers,
    );
    const inputs = new PublicKernelTailCircuitPrivateInputs(
      previousKernel,
      nullifierReadRequestHints,
      nullifierNonExistentReadRequestHints,
    );
    return this.publicKernel.publicKernelCircuitTail(inputs);
  }

  private sortNoteHashes<N extends number>(noteHashes: Tuple<SideEffect, N>): Tuple<SideEffect, N> {
    return noteHashes.sort((n0, n1) => {
      if (n0.isEmpty()) {
        return 1;
      }
      return Number(n0.counter.toBigInt() - n1.counter.toBigInt());
    });
  }
}
