import {
  type BlockResult,
  L2Block,
  PROVING_STATUS,
  type ProcessedTx,
  type ProverClient,
  type ProvingJob,
  type ProvingJobSource,
  type ProvingRequest,
  type ProvingSuccess,
  type ProvingTicket,
} from '@aztec/circuit-types';
import { type GlobalVariables, makeEmptyProof } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

export class DummyProver implements ProverClient {
  jobs = new DummyProvingJobSource();

  public start(): Promise<void> {
    return Promise.resolve();
  }

  public stop(): Promise<void> {
    return Promise.resolve();
  }

  public static new(): Promise<DummyProver> {
    return Promise.resolve(new DummyProver());
  }

  startNewBlock(
    _numTxs: number,
    _globalVariables: GlobalVariables,
    _newL1ToL2Messages: Fr[],
    _emptyTx: ProcessedTx,
  ): Promise<ProvingTicket> {
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };
    return Promise.resolve(ticket);
  }

  addNewTx(_tx: ProcessedTx): Promise<void> {
    return Promise.resolve();
  }

  cancelBlock(): void {}

  finaliseBlock(): Promise<BlockResult> {
    return Promise.resolve({
      block: L2Block.empty(),
      proof: makeEmptyProof(),
    });
  }

  setBlockCompleted(): Promise<void> {
    return Promise.resolve();
  }

  getProvingJobSource(): ProvingJobSource {
    return this.jobs;
  }
}

class DummyProvingJobSource implements ProvingJobSource {
  getProvingJob(): Promise<ProvingJob<ProvingRequest> | null> {
    return Promise.resolve(null);
  }

  rejectProvingJob(): Promise<void> {
    return Promise.resolve();
  }

  resolveProvingJob(): Promise<void> {
    return Promise.resolve();
  }
}
