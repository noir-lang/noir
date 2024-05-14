import { type CircuitName, type CircuitSimulationStats } from '@aztec/circuit-types/stats';
import { type Logger } from '@aztec/foundation/log';

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
