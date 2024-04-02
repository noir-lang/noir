import {
  L2Block,
  PROVING_STATUS,
  type ProcessedTx,
  type ProverClient,
  type ProvingSuccess,
  type ProvingTicket,
} from '@aztec/circuit-types';
import { type GlobalVariables, makeEmptyProof } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';

export class DummyProver implements ProverClient {
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
      proof: makeEmptyProof(),
      block: L2Block.empty(),
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };
    return Promise.resolve(ticket);
  }

  addNewTx(_tx: ProcessedTx): Promise<void> {
    return Promise.resolve();
  }
}
