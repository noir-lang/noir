import { type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import {
  type KernelCircuitPublicInputs,
  type PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
  type PublicKernelTailCircuitPrivateInputs,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { elapsed } from '@aztec/foundation/timer';
import {
  SimulatedServerCircuitArtifacts,
  convertSimulatedPublicInnerInputsToWitnessMap,
  convertSimulatedPublicInnerOutputFromWitnessMap,
  convertSimulatedPublicSetupInputsToWitnessMap,
  convertSimulatedPublicSetupOutputFromWitnessMap,
  convertSimulatedPublicTailInputsToWitnessMap,
  convertSimulatedPublicTailOutputFromWitnessMap,
  convertSimulatedPublicTeardownInputsToWitnessMap,
  convertSimulatedPublicTeardownOutputFromWitnessMap,
} from '@aztec/noir-protocol-circuits-types';

import { WASMSimulator } from '../providers/acvm_wasm.js';
import { type SimulationProvider } from '../providers/simulation_provider.js';
import { type PublicKernelCircuitSimulator } from './public_kernel_circuit_simulator.js';

/**
 * Implements the PublicKernelCircuitSimulator.
 */
export class RealPublicKernelCircuitSimulator implements PublicKernelCircuitSimulator {
  private log = createDebugLogger('aztec:public-kernel-simulator');

  // Some circuits are so small it is faster to use WASM
  private wasmSimulator: WASMSimulator = new WASMSimulator();

  constructor(private simulator: SimulationProvider) {}

  /**
   * Simulates the public kernel setup circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async publicKernelCircuitSetup(
    input: PublicKernelCircuitPrivateInputs,
  ): Promise<PublicKernelCircuitPublicInputs> {
    if (!input.previousKernel.publicInputs.needsSetup) {
      throw new Error(`Expected previous kernel inputs to need setup`);
    }
    const inputWitness = convertSimulatedPublicSetupInputsToWitnessMap(input);
    const [duration, witness] = await elapsed(() =>
      this.wasmSimulator.simulateCircuit(inputWitness, SimulatedServerCircuitArtifacts.PublicKernelSetupArtifact),
    );
    const result = convertSimulatedPublicSetupOutputFromWitnessMap(witness);
    this.log.debug(`Simulated public kernel setup circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'public-kernel-setup',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return result;
  }

  /**
   * Simulates the public kernel app logic circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async publicKernelCircuitAppLogic(
    input: PublicKernelCircuitPrivateInputs,
  ): Promise<PublicKernelCircuitPublicInputs> {
    if (!input.previousKernel.publicInputs.needsAppLogic) {
      throw new Error(`Expected previous kernel inputs to need app logic`);
    }
    const inputWitness = convertSimulatedPublicInnerInputsToWitnessMap(input);
    const [duration, witness] = await elapsed(() =>
      this.wasmSimulator.simulateCircuit(inputWitness, SimulatedServerCircuitArtifacts.PublicKernelAppLogicArtifact),
    );
    const result = convertSimulatedPublicInnerOutputFromWitnessMap(witness);
    this.log.debug(`Simulated public kernel app logic circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'public-kernel-app-logic',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return result;
  }

  /**
   * Simulates the public kernel teardown circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async publicKernelCircuitTeardown(
    input: PublicKernelCircuitPrivateInputs,
  ): Promise<PublicKernelCircuitPublicInputs> {
    if (!input.previousKernel.publicInputs.needsTeardown) {
      throw new Error(`Expected previous kernel inputs to need teardown`);
    }
    const inputWitness = convertSimulatedPublicTeardownInputsToWitnessMap(input);
    const [duration, witness] = await elapsed(() =>
      this.wasmSimulator.simulateCircuit(inputWitness, SimulatedServerCircuitArtifacts.PublicKernelTeardownArtifact),
    );
    const result = convertSimulatedPublicTeardownOutputFromWitnessMap(witness);
    this.log.debug(`Simulated public kernel teardown circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'public-kernel-teardown',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return result;
  }

  /**
   * Simulates the public kernel tail circuit from its inputs.
   * @param input - Inputs to the circuit.
   * @returns The public inputs as outputs of the simulation.
   */
  public async publicKernelCircuitTail(
    input: PublicKernelTailCircuitPrivateInputs,
  ): Promise<KernelCircuitPublicInputs> {
    const inputWitness = convertSimulatedPublicTailInputsToWitnessMap(input);
    const [duration, witness] = await elapsed(() =>
      this.wasmSimulator.simulateCircuit(inputWitness, SimulatedServerCircuitArtifacts.PublicKernelTailArtifact),
    );
    const result = convertSimulatedPublicTailOutputFromWitnessMap(witness);
    this.log.debug(`Simulated public kernel tail circuit`, {
      eventName: 'circuit-simulation',
      circuitName: 'public-kernel-tail',
      duration,
      inputSize: input.toBuffer().length,
      outputSize: result.toBuffer().length,
    } satisfies CircuitSimulationStats);
    return result;
  }
}
