/* eslint-disable require-await */
import {
  AggregationObject,
  BaseOrMergeRollupPublicInputs,
  BaseRollupInputs,
  MergeRollupInputs,
  Proof,
  PublicCircuitPublicInputs,
  PublicKernelPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
} from '@aztec/circuits.js';

import { PublicProver, RollupProver } from './index.js';

const EMPTY_PROOF_SIZE = 42;

// TODO: Silently modifying one of the inputs to inject the aggregation object is horrible.
// We should rethink these interfaces.

/**
 * Prover implementation that returns empty proofs and overrides aggregation objects.
 */
export class EmptyRollupProver implements RollupProver {
  /**
   * Creates an empty proof for the given input.
   * @param _input - Input to the circuit.
   * @param publicInputs - Public inputs of the circuit obtained via simulation, modified by this call.
   */
  async getBaseRollupProof(_input: BaseRollupInputs, publicInputs: BaseOrMergeRollupPublicInputs): Promise<Proof> {
    publicInputs.endAggregationObject = AggregationObject.makeFake();
    return new Proof(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }

  /**
   * Creates an empty proof for the given input.
   * @param _input - Input to the circuit.
   * @param publicInputs - Public inputs of the circuit obtained via simulation, modified by this call.
   */
  async getMergeRollupProof(_input: MergeRollupInputs, publicInputs: BaseOrMergeRollupPublicInputs): Promise<Proof> {
    publicInputs.endAggregationObject = AggregationObject.makeFake();
    return new Proof(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }
  /**
   * Creates an empty proof for the given input.
   * @param _input - Input to the circuit.
   * @param publicInputs - Public inputs of the circuit obtained via simulation, modified by this call.
   */
  async getRootRollupProof(_input: RootRollupInputs, publicInputs: RootRollupPublicInputs): Promise<Proof> {
    publicInputs.endAggregationObject = AggregationObject.makeFake();
    return new Proof(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }
}

/**
 * Prover implementation that returns empty proofs.
 */
export class EmptyPublicProver implements PublicProver {
  /**
   * Creates an empty proof for the given input.
   * @param _publicInputs - Public inputs obtained via simulation.
   */
  async getPublicCircuitProof(_publicInputs: PublicCircuitPublicInputs): Promise<Proof> {
    return new Proof(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }

  /**
   * Creates an empty proof for the given input.
   * @param _publicInputs - Public inputs obtained via simulation.
   */
  async getPublicKernelCircuitProof(_publicInputs: PublicKernelPublicInputs): Promise<Proof> {
    return new Proof(Buffer.alloc(EMPTY_PROOF_SIZE, 0));
  }
}
