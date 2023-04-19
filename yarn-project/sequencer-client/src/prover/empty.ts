/* eslint-disable require-await */
import {
  AggregationObject,
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  PublicCircuitPublicInputs,
  PublicKernelPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';
import { PublicProver, RollupProver } from './index.js';

const EMPTY_PROOF_SIZE = 42;

// TODO: Silently modifying one of the inputs to inject the aggregation object is horrible.
// We should rethink these interfaces.
export class EmptyRollupProver implements RollupProver {
  async getBaseRollupProof(
    _input: BaseRollupInputs,
    publicInputs: BaseOrMergeRollupPublicInputs,
  ): Promise<UInt8Vector> {
    publicInputs.endAggregationObject = AggregationObject.makeFake();
    return new UInt8Vector(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }
  async getMergeRollupProof(
    _input: MergeRollupInputs,
    publicInputs: BaseOrMergeRollupPublicInputs,
  ): Promise<UInt8Vector> {
    publicInputs.endAggregationObject = AggregationObject.makeFake();
    return new UInt8Vector(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }
  async getRootRollupProof(_input: RootRollupInputs, publicInputs: RootRollupPublicInputs): Promise<UInt8Vector> {
    publicInputs.endAggregationObject = AggregationObject.makeFake();
    return new UInt8Vector(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }
}

export class EmptyPublicProver implements PublicProver {
  async getPublicCircuitProof(_publicInputs: PublicCircuitPublicInputs): Promise<UInt8Vector> {
    return new UInt8Vector(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }
  async getPublicKernelCircuitProof(_publicInputs: PublicKernelPublicInputs): Promise<UInt8Vector> {
    return new UInt8Vector(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }
}
