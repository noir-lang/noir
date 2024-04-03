import { type Tx } from '@aztec/circuit-types';
import { type GlobalVariables, type Header, type PublicKernelCircuitPublicInputs } from '@aztec/circuits.js';
import { type PublicExecutor, type PublicStateDB } from '@aztec/simulator';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { type PublicKernelCircuitSimulator } from '../simulator/index.js';
import { type ContractsDataSourcePublicDB } from '../simulator/public_executor.js';
import { type AbstractPhaseManager, PublicKernelPhase } from './abstract_phase_manager.js';
import { AppLogicPhaseManager } from './app_logic_phase_manager.js';
import { SetupPhaseManager } from './setup_phase_manager.js';
import { TailPhaseManager } from './tail_phase_manager.js';
import { TeardownPhaseManager } from './teardown_phase_manager.js';

export class PhaseDidNotChangeError extends Error {
  constructor(phase: PublicKernelPhase) {
    super(`Tried to advance the phase from [${phase}] when the circuit still needs [${phase}]`);
  }
}

export class CannotTransitionToSetupError extends Error {
  constructor() {
    super('Cannot transition to setup phase');
  }
}

export class PhaseManagerFactory {
  public static phaseFromTx(
    tx: Tx,
    db: MerkleTreeOperations,
    publicExecutor: PublicExecutor,
    publicKernel: PublicKernelCircuitSimulator,
    globalVariables: GlobalVariables,
    historicalHeader: Header,
    publicContractsDB: ContractsDataSourcePublicDB,
    publicStateDB: PublicStateDB,
  ): AbstractPhaseManager | undefined {
    const data = tx.data.forPublic!;
    if (data.needsSetup) {
      return new SetupPhaseManager(
        db,
        publicExecutor,
        publicKernel,
        globalVariables,
        historicalHeader,
        publicContractsDB,
        publicStateDB,
      );
    } else if (data.needsAppLogic) {
      return new AppLogicPhaseManager(
        db,
        publicExecutor,
        publicKernel,
        globalVariables,
        historicalHeader,
        publicContractsDB,
        publicStateDB,
      );
    } else if (data.needsTeardown) {
      return new TeardownPhaseManager(
        db,
        publicExecutor,
        publicKernel,
        globalVariables,
        historicalHeader,
        publicContractsDB,
        publicStateDB,
      );
    } else {
      return undefined;
    }
  }

  public static phaseFromOutput(
    output: PublicKernelCircuitPublicInputs,
    currentPhaseManager: AbstractPhaseManager,
    db: MerkleTreeOperations,
    publicExecutor: PublicExecutor,
    publicKernel: PublicKernelCircuitSimulator,
    globalVariables: GlobalVariables,
    historicalHeader: Header,
    publicContractsDB: ContractsDataSourcePublicDB,
    publicStateDB: PublicStateDB,
  ): AbstractPhaseManager | undefined {
    if (output.needsSetup) {
      throw new CannotTransitionToSetupError();
    } else if (output.needsAppLogic) {
      if (currentPhaseManager.phase === PublicKernelPhase.APP_LOGIC) {
        throw new PhaseDidNotChangeError(currentPhaseManager.phase);
      }
      return new AppLogicPhaseManager(
        db,
        publicExecutor,
        publicKernel,
        globalVariables,
        historicalHeader,
        publicContractsDB,
        publicStateDB,
      );
    } else if (output.needsTeardown) {
      if (currentPhaseManager.phase === PublicKernelPhase.TEARDOWN) {
        throw new PhaseDidNotChangeError(currentPhaseManager.phase);
      }
      return new TeardownPhaseManager(
        db,
        publicExecutor,
        publicKernel,
        globalVariables,
        historicalHeader,
        publicContractsDB,
        publicStateDB,
      );
    } else if (currentPhaseManager.phase !== PublicKernelPhase.TAIL) {
      return new TailPhaseManager(
        db,
        publicExecutor,
        publicKernel,
        globalVariables,
        historicalHeader,
        publicContractsDB,
        publicStateDB,
      );
    } else {
      return undefined;
    }
  }
}
