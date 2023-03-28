import {
  BaseRollupInputs,
  BaseRollupPublicInputs,
  MergeRollupInputs,
  MergeRollupPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';
import { Prover } from './index.js';

/* eslint-disable */

export class MockProver implements Prover {
  async getBaseRollupProof(input: BaseRollupInputs, publicInputs: BaseRollupPublicInputs): Promise<UInt8Vector> {
    return new UInt8Vector(Buffer.alloc(0));
  }
  async getMergeRollupProof(input: MergeRollupInputs, publicInputs: MergeRollupPublicInputs): Promise<UInt8Vector> {
    return new UInt8Vector(Buffer.alloc(0));
  }
  async getRootRollupProof(input: RootRollupInputs, publicInputs: RootRollupPublicInputs): Promise<UInt8Vector> {
    return new UInt8Vector(Buffer.alloc(0));
  }
}
