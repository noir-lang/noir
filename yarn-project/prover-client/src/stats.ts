import { type PublicKernelRequest, PublicKernelType } from '@aztec/circuit-types';
import type {
  CircuitName,
  CircuitProvingStats,
  CircuitSimulationStats,
  CircuitWitnessGenerationStats,
} from '@aztec/circuit-types/stats';
import { type Logger } from '@aztec/foundation/log';
import { type ServerProtocolArtifact } from '@aztec/noir-protocol-circuits-types';

export function emitCircuitSimulationStats(
  circuitName: CircuitName,
  duration: number,
  inputSize: number,
  outputSize: number,
  logger: Logger,
) {
  const stats: CircuitSimulationStats = {
    eventName: 'circuit-simulation',
    circuitName,
    inputSize,
    outputSize,
    duration,
  };

  logger.debug('Circuit simulation stats', stats);
}

export function emitCircuitWitnessGenerationStats(
  circuitName: CircuitName,
  duration: number,
  inputSize: number,
  outputSize: number,
  logger: Logger,
) {
  const stats: CircuitWitnessGenerationStats = {
    eventName: 'circuit-witness-generation',
    circuitName,
    inputSize,
    outputSize,
    duration,
  };

  logger.debug('Circuit witness generation stats', stats);
}

export function emitCircuitProvingStats(
  circuitName: CircuitName,
  duration: number,
  inputSize: number,
  outputSize: number,
  proofSize: number,
  logger: Logger,
) {
  const stats: CircuitProvingStats = {
    eventName: 'circuit-proving',
    circuitName,
    duration,
    inputSize,
    outputSize,
    proofSize,
  };

  logger.debug('Circuit proving stats', stats);
}

export function mapPublicKernelToCircuitName(kernelType: PublicKernelRequest['type']): CircuitName {
  switch (kernelType) {
    case PublicKernelType.SETUP:
      return 'public-kernel-setup';
    case PublicKernelType.APP_LOGIC:
      return 'public-kernel-app-logic';
    case PublicKernelType.TEARDOWN:
      return 'public-kernel-teardown';
    case PublicKernelType.TAIL:
      return 'public-kernel-tail';
    default:
      throw new Error(`Unknown kernel type: ${kernelType}`);
  }
}

export function circuitTypeToCircuitName(circuitType: ServerProtocolArtifact): CircuitName {
  switch (circuitType) {
    case 'BaseParityArtifact':
      return 'base-parity';
    case 'RootParityArtifact':
      return 'root-parity';
    case 'BaseRollupArtifact':
      return 'base-rollup';
    case 'MergeRollupArtifact':
      return 'merge-rollup';
    case 'RootRollupArtifact':
      return 'root-rollup';
    case 'PublicKernelSetupArtifact':
      return 'public-kernel-setup';
    case 'PublicKernelAppLogicArtifact':
      return 'public-kernel-app-logic';
    case 'PublicKernelTeardownArtifact':
      return 'public-kernel-teardown';
    case 'PublicKernelTailArtifact':
      return 'public-kernel-tail';
    default:
      throw new Error(`Unknown circuit type: ${circuitType}`);
  }
}
