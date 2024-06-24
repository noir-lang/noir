import {
  type ProofAndVerificationKey,
  type PublicInputsAndRecursiveProof,
  type PublicKernelNonTailRequest,
  type PublicKernelTailRequest,
  type ServerCircuitProver,
  makePublicInputsAndRecursiveProof,
} from '@aztec/circuit-types';
import {
  type AvmCircuitInputs,
  type BaseOrMergeRollupPublicInputs,
  type BaseParityInputs,
  type BaseRollupInputs,
  EmptyNestedData,
  type KernelCircuitPublicInputs,
  type MergeRollupInputs,
  NESTED_RECURSIVE_PROOF_LENGTH,
  type PrivateKernelEmptyInputData,
  PrivateKernelEmptyInputs,
  type Proof,
  type PublicKernelCircuitPublicInputs,
  RECURSIVE_PROOF_LENGTH,
  RootParityInput,
  type RootParityInputs,
  type RootRollupInputs,
  type RootRollupPublicInputs,
  VerificationKeyAsFields,
  VerificationKeyData,
  makeEmptyProof,
  makeEmptyRecursiveProof,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import {
  BaseParityArtifact,
  MergeRollupArtifact,
  PrivateKernelEmptyArtifact,
  RootParityArtifact,
  RootRollupArtifact,
  type ServerProtocolArtifact,
  SimulatedBaseRollupArtifact,
  SimulatedServerCircuitArtifacts,
  convertBaseParityInputsToWitnessMap,
  convertBaseParityOutputsFromWitnessMap,
  convertMergeRollupInputsToWitnessMap,
  convertMergeRollupOutputsFromWitnessMap,
  convertPrivateKernelEmptyInputsToWitnessMap,
  convertPrivateKernelEmptyOutputsFromWitnessMap,
  convertRootParityInputsToWitnessMap,
  convertRootParityOutputsFromWitnessMap,
  convertRootRollupInputsToWitnessMap,
  convertRootRollupOutputsFromWitnessMap,
  convertSimulatedBaseRollupInputsToWitnessMap,
  convertSimulatedBaseRollupOutputsFromWitnessMap,
  convertSimulatedPublicTailInputsToWitnessMap,
  convertSimulatedPublicTailOutputFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';
import { type SimulationProvider, WASMSimulator, emitCircuitSimulationStats } from '@aztec/simulator';
import { type TelemetryClient } from '@aztec/telemetry-client';

import { ProverInstrumentation } from '../instrumentation.js';
import { SimulatedPublicKernelArtifactMapping } from '../mappings/mappings.js';
import { mapPublicKernelToCircuitName } from '../stats.js';

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
  PrivateKernelEmptyArtifact: VerificationKeyAsFields.makeFake(),
  EmptyNestedArtifact: VerificationKeyAsFields.makeFake(),
};

/**
 * A class for use in testing situations (e2e, unit test etc)
 * Simulates circuits using the most efficient method and performs no proving
 */
export class TestCircuitProver implements ServerCircuitProver {
  private wasmSimulator = new WASMSimulator();
  private instrumentation: ProverInstrumentation;

  constructor(
    telemetry: TelemetryClient,
    private simulationProvider?: SimulationProvider,
    private logger = createDebugLogger('aztec:test-prover'),
  ) {
    this.instrumentation = new ProverInstrumentation(telemetry, 'TestCircuitProver');
  }

  public async getEmptyPrivateKernelProof(
    inputs: PrivateKernelEmptyInputData,
  ): Promise<PublicInputsAndRecursiveProof<KernelCircuitPublicInputs>> {
    const emptyNested = new EmptyNestedData(
      makeRecursiveProof(RECURSIVE_PROOF_LENGTH),
      VERIFICATION_KEYS['EmptyNestedArtifact'],
    );
    const kernelInputs = new PrivateKernelEmptyInputs(emptyNested, inputs.header, inputs.chainId, inputs.version);
    const witnessMap = convertPrivateKernelEmptyInputsToWitnessMap(kernelInputs);
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, PrivateKernelEmptyArtifact);
    const result = convertPrivateKernelEmptyOutputsFromWitnessMap(witness);

    return makePublicInputsAndRecursiveProof(
      result,
      makeRecursiveProof(NESTED_RECURSIVE_PROOF_LENGTH),
      VerificationKeyData.makeFake(),
    );
  }

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

    this.instrumentation.recordDuration('simulationDuration', 'base-parity', timer);

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

    this.instrumentation.recordDuration('simulationDuration', 'root-parity', timer);
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
  ): Promise<PublicInputsAndRecursiveProof<BaseOrMergeRollupPublicInputs>> {
    const timer = new Timer();
    const witnessMap = convertSimulatedBaseRollupInputsToWitnessMap(input);

    const simulationProvider = this.simulationProvider ?? this.wasmSimulator;
    const witness = await simulationProvider.simulateCircuit(witnessMap, SimulatedBaseRollupArtifact);

    const result = convertSimulatedBaseRollupOutputsFromWitnessMap(witness);

    this.instrumentation.recordDuration('simulationDuration', 'base-rollup', timer);
    emitCircuitSimulationStats(
      'base-rollup',
      timer.ms(),
      input.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );
    return makePublicInputsAndRecursiveProof(
      result,
      makeRecursiveProof(NESTED_RECURSIVE_PROOF_LENGTH),
      VerificationKeyData.makeFake(),
    );
  }
  /**
   * Simulates the merge rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getMergeRollupProof(
    input: MergeRollupInputs,
  ): Promise<PublicInputsAndRecursiveProof<BaseOrMergeRollupPublicInputs>> {
    const timer = new Timer();
    const witnessMap = convertMergeRollupInputsToWitnessMap(input);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, MergeRollupArtifact);

    const result = convertMergeRollupOutputsFromWitnessMap(witness);

    this.instrumentation.recordDuration('simulationDuration', 'merge-rollup', timer);
    emitCircuitSimulationStats(
      'merge-rollup',
      timer.ms(),
      input.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );
    return makePublicInputsAndRecursiveProof(
      result,
      makeEmptyRecursiveProof(NESTED_RECURSIVE_PROOF_LENGTH),
      VerificationKeyData.makeFake(),
    );
  }

  /**
   * Simulates the root rollup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async getRootRollupProof(
    input: RootRollupInputs,
  ): Promise<PublicInputsAndRecursiveProof<RootRollupPublicInputs>> {
    const timer = new Timer();
    const witnessMap = convertRootRollupInputsToWitnessMap(input);

    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(witnessMap, RootRollupArtifact);

    const result = convertRootRollupOutputsFromWitnessMap(witness);

    this.instrumentation.recordDuration('simulationDuration', 'root-rollup', timer);
    emitCircuitSimulationStats(
      'root-rollup',
      timer.ms(),
      input.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );
    return makePublicInputsAndRecursiveProof(
      result,
      makeEmptyRecursiveProof(NESTED_RECURSIVE_PROOF_LENGTH),
      VerificationKeyData.makeFake(),
    );
  }

  public async getPublicKernelProof(
    kernelRequest: PublicKernelNonTailRequest,
  ): Promise<PublicInputsAndRecursiveProof<PublicKernelCircuitPublicInputs>> {
    const timer = new Timer();
    const kernelOps = SimulatedPublicKernelArtifactMapping[kernelRequest.type];
    if (kernelOps === undefined) {
      throw new Error(`Unable to prove for kernel type ${kernelRequest.type}`);
    }
    const witnessMap = kernelOps.convertInputs(kernelRequest.inputs);

    const witness = await this.wasmSimulator.simulateCircuit(
      witnessMap,
      SimulatedServerCircuitArtifacts[kernelOps.artifact],
    );

    const result = kernelOps.convertOutputs(witness);
    const circuitName = mapPublicKernelToCircuitName(kernelRequest.type);
    this.instrumentation.recordDuration('simulationDuration', circuitName, timer);
    emitCircuitSimulationStats(
      circuitName,
      timer.ms(),
      kernelRequest.inputs.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );

    return makePublicInputsAndRecursiveProof(
      result,
      makeEmptyRecursiveProof(NESTED_RECURSIVE_PROOF_LENGTH),
      VerificationKeyData.makeFake(),
    );
  }

  public async getPublicTailProof(
    kernelRequest: PublicKernelTailRequest,
  ): Promise<PublicInputsAndRecursiveProof<KernelCircuitPublicInputs>> {
    const timer = new Timer();
    const witnessMap = convertSimulatedPublicTailInputsToWitnessMap(kernelRequest.inputs);
    // use WASM here as it is faster for small circuits
    const witness = await this.wasmSimulator.simulateCircuit(
      witnessMap,
      SimulatedServerCircuitArtifacts['PublicKernelTailArtifact'],
    );

    const result = convertSimulatedPublicTailOutputFromWitnessMap(witness);
    this.instrumentation.recordDuration('simulationDuration', 'public-kernel-tail', timer);
    emitCircuitSimulationStats(
      'public-kernel-tail',
      timer.ms(),
      kernelRequest.inputs.toBuffer().length,
      result.toBuffer().length,
      this.logger,
    );

    return makePublicInputsAndRecursiveProof(
      result,
      makeEmptyRecursiveProof(NESTED_RECURSIVE_PROOF_LENGTH),
      VerificationKeyData.makeFake(),
    );
  }

  getAvmProof(_inputs: AvmCircuitInputs): Promise<ProofAndVerificationKey> {
    // We can't simulate the AVM because we don't have enough context to do so (e.g., DBs).
    // We just return an empty proof and VK data.
    this.logger.debug('Skipping AVM simulation in TestCircuitProver.');
    return Promise.resolve({ proof: makeEmptyProof(), verificationKey: VerificationKeyData.makeFake() });
  }

  // Not implemented for test circuits
  public verifyProof(_1: ServerProtocolArtifact, _2: Proof): Promise<void> {
    return Promise.reject(new Error('Method not implemented.'));
  }
}
