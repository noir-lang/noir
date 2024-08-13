import { TestCircuitProver } from '@aztec/bb-prover';
import {
  type BlockSimulator,
  type MerkleTreeOperations,
  type ProcessedTx,
  type ProvingTicket,
  type SimulationBlockResult,
} from '@aztec/circuit-types';
import { type Fr, type GlobalVariables } from '@aztec/circuits.js';
import { ProvingOrchestrator } from '@aztec/prover-client/orchestrator';
import { type SimulationProvider } from '@aztec/simulator';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

/**
 * Implements a block simulator using a test circuit prover under the hood, which just simulates circuits and outputs empty proofs.
 * This class is temporary and should die once we switch from tx effects to tx objects submissions, since sequencers won't have
 * the need to create L2 block headers to submit to L1. When we do that, we should also remove the references to the
 * prover-client and bb-prover packages from this package.
 */
export class BlockBuilder implements BlockSimulator {
  private orchestrator: ProvingOrchestrator;
  constructor(db: MerkleTreeOperations, simulationProvider: SimulationProvider, telemetry: TelemetryClient) {
    const testProver = new TestCircuitProver(telemetry, simulationProvider);
    this.orchestrator = new ProvingOrchestrator(db, testProver, telemetry);
  }

  startNewBlock(numTxs: number, globalVariables: GlobalVariables, l1ToL2Messages: Fr[]): Promise<ProvingTicket> {
    return this.orchestrator.startNewBlock(numTxs, globalVariables, l1ToL2Messages);
  }
  cancelBlock(): void {
    this.orchestrator.cancelBlock();
  }
  finaliseBlock(): Promise<SimulationBlockResult> {
    return this.orchestrator.finaliseBlock();
  }
  setBlockCompleted(): Promise<void> {
    return this.orchestrator.setBlockCompleted();
  }
  addNewTx(tx: ProcessedTx): Promise<void> {
    return this.orchestrator.addNewTx(tx);
  }
}

export class BlockBuilderFactory {
  constructor(private simulationProvider: SimulationProvider, private telemetry?: TelemetryClient) {}

  create(db: MerkleTreeOperations): BlockSimulator {
    return new BlockBuilder(db, this.simulationProvider, this.telemetry ?? new NoopTelemetryClient());
  }
}
