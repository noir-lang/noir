import { NoirCompiledCircuit } from '@aztec/types/noir';

import { WitnessMap } from '@noir-lang/types';

/**
 * Low level simulation interface
 */
export interface SimulationProvider {
  simulateCircuit(input: WitnessMap, compiledCircuit: NoirCompiledCircuit): Promise<WitnessMap>;
}
