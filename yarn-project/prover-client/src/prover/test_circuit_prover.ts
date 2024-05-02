import {
  type PublicInputsAndProof,
  type PublicKernelNonTailRequest,
  type PublicKernelTailRequest,
  PublicKernelType,
  makePublicInputsAndProof,
} from '@aztec/circuit-types';
import {
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  NESTED_RECURSIVE_PROOF_LENGTH,
  type Proof,
  type PublicKernelCircuitPublicInputs,
  RECURSIVE_PROOF_LENGTH,
  RootParityInput,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
  VerificationKeyAsFields,
  makeEmptyProof,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import {
  BaseParityArtifact,
  MergeRollupArtifact,
  RootParityArtifact,
  RootRollupArtifact,
  ServerCircuitArtifacts,
  type ServerProtocolArtifact,
  SimulatedBaseRollupArtifact,
  convertBaseParityInputsToWitnessMap,
  convertBaseParityOutputsFromWitnessMap,
  convertMergeRollupInputsToWitnessMap,
  convertMergeRollupOutputsFromWitnessMap,
  convertPublicTailInputsToWitnessMap,
  convertPublicTailOutputFromWitnessMap,
  convertRootParityInputsToWitnessMap,
  convertRootParityOutputsFromWitnessMap,
  convertRootRollupInputsToWitnessMap,
  convertRootRollupOutputsFromWitnessMap,
  convertSimulatedBaseRollupInputsToWitnessMap,
  convertSimulatedBaseRollupOutputsFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';
import { type SimulationProvider, WASMSimulator } from '@aztec/simulator';

import { emitCircuitSimulationStats, mapPublicKernelToCircuitName } from '../stats.js';
import { type CircuitProver, KernelArtifactMapping } from './interface.js';

const VERIFICATION_KEYS: Record<ServerProtocolArtifact, VerificationKeyAsFields> = {
  BaseParityArtifact: VerificationKeyAsFields.makeFake(),
  RootParityArtifact: VerificationKeyAsFields.makeFake(),
  PublicKernelAppLogicArtifact: VerificationKeyAsFields.makeFake(),
  PublicKernelSetupArtifact: VerificationKeyAsFields.makeFake(),
  PublicKernelTailArtifact: VerificationKeyAsFields.makeFake(),
  PublicKernelTeardownArtifact: VerificationKeyAsFields.makeFake(),
  BaseRollupArtifact: VerificationKeyAsFields.makeFake(),
  MergeRollupArtifact: VerificationKeyAsFields.makeFake(),
  RootRollupArtifact: VerificationKeyAsFields.makeFake(),
};

/**
 * A class for use in testing situations (e2e, unit test etc)
 * Simulates circuits using the most efficient method and performs no proving
 */
export class TestCircuitProver implements CircuitProver {
  private wasmSimulator = new WASMSimulator();

  constructor(
    private simulationProvider?: SimulationProvider,
    private logger = createDebugLogger('aztec:test-prover'),
  ) {}

  /**
   * Simulates the base parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getBaseParityProof(inputs: BaseParityInputs): Promise<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>> {
    const timer = new Timer();
    const witnessMap = convertBaseParityInputsToWitnessMap(inputs);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, BaseParityArtifact);
    const result = convertBaseParityOutputsFromWitnessMap(witness);

    const rootParityInput = new RootParityInput<typeof RECURSIVE_PROOF_LENGTH>(
      makeRecursiveProof<typeof RECURSIVE_PROOF_LENGTH>(RECURSIVE_PROOF_LENGTH),
      VERIFICATION_KEYS['BaseParityArtifact'],
      result,
    );

    emitCircuitSimulationStats(
      'base-parity',
      timer.ms(),
      inputs.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );

    return Promise.resolve(rootParityInput);
  }

  /**
   * Simulates the root parity circuit from its inputs.
   * @param inputs - Inputs to the circuit.
   * @returns The public inputs of the parity circuit.
   */
  public async getRootParityProof(
    inputs: RootParityInputs,
  ): Promise<RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>> {
    const timer = new Timer();
    const witnessMap = convertRootParityInputsToWitnessMap(inputs);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, RootParityArtifact);

    const result = convertRootParityOutputsFromWitnessMap(witness);

    const rootParityInput = new RootParityInput<typeof NESTED_RECURSIVE_PROOF_LENGTH>(
      makeRecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>(NESTED_RECURSIVE_PROOF_LENGTH),
      VERIFICATION_KEYS['RootParityArtifact'],
      result,
    );

    emitCircuitSimulationStats(
      'root-parity',
      timer.ms(),
      inputs.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );

    return Promise.resolve(rootParityInput);
  }

  /**
   * Simulates the base rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getBaseRollupProof(
    input: BaseRollupInputs,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    const timer = new Timer();
    const witnessMap = convertSimulatedBaseRollupInputsToWitnessMap(input);

    const simulationProvider = this.simulationProvider ?? this.wasmSimulator;
    const witness = await simulationProvider.simulateCircuit(witnessMap, SimulatedBaseRollupArtifact);

    const result = convertSimulatedBaseRollupOutputsFromWitnessMap(witness);

    emitCircuitSimulationStats(
      'base-rollup',
      timer.ms(),
      input.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );
    return makePublicInputsAndProof(result, makeEmptyProof());
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getMergeRollupProof(
    input: MergeRollupInputs,
  ): Promise<PublicInputsAndProof<BaseOrMergeRollupPublicInputs>> {
    const timer = new Timer();
    const witnessMap = convertMergeRollupInputsToWitnessMap(input);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, MergeRollupArtifact);

    const result = convertMergeRollupOutputsFromWitnessMap(witness);

    emitCircuitSimulationStats(
      'merge-rollup',
      timer.ms(),
      input.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );
    return makePublicInputsAndProof(result, makeEmptyProof());
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getRootRollupProof(input: RootRollupInputs): Promise<PublicInputsAndProof<RootRollupPublicInputs>> {
    const timer = new Timer();
    const witnessMap = convertRootRollupInputsToWitnessMap(input);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, RootRollupArtifact);

    const result = convertRootRollupOutputsFromWitnessMap(witness);

    emitCircuitSimulationStats(
      'root-rollup',
      timer.ms(),
      input.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );
    return makePublicInputsAndProof(result, makeEmptyProof());
  }

  public async getPublicKernelProof(
    kernelRequest: PublicKernelNonTailRequest,
  ): Promise<PublicInputsAndProof<PublicKernelCircuitPublicInputs>> {
    const timer = new Timer();
    const kernelOps = KernelArtifactMapping[kernelRequest.type];
    if (kernelOps === undefined) {
      throw new Error(`Unable to prove for kernel type ${PublicKernelType[kernelRequest.type]}`);
    }
    const witnessMap = kernelOps.convertInputs(kernelRequest.inputs);

    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, ServerCircuitArtifacts[kernelOps.artifact]);

    const result = kernelOps.convertOutputs(witness);
    emitCircuitSimulationStats(
      mapPublicKernelToCircuitName(kernelRequest.type),
      timer.ms(),
      kernelRequest.inputs.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );

    return makePublicInputsAndProof(result, makeEmptyProof());
  }

  public async getPublicTailProof(
    kernelRequest: PublicKernelTailRequest,
  ): Promise<PublicInputsAndProof<KernelCircuitPublicInputs>> {
    const timer = new Timer();
    const witnessMap = convertPublicTailInputsToWitnessMap(kernelRequest.inputs);
    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(
      witnessMap,
      ServerCircuitArtifacts['PublicKernelTailArtifact'],
    );

    const result = convertPublicTailOutputFromWitnessMap(witness);
    emitCircuitSimulationStats(
      'public-kernel-tail',
      timer.ms(),
      kernelRequest.inputs.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );

    return makePublicInputsAndProof(result, makeEmptyProof());
  }

  // Not implemented for test circuits
  public verifyProof(_1: ServerProtocolArtifact, _2: Proof): Promise<void> {
    return Promise.reject(new Error('Method not implemented.'));
  }
}
