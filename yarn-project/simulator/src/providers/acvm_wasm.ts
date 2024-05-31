import { foreignCallHandler } from '@aztec/noir-protocol-circuits-types';
import { type NoirCompiledCircuit } from '@aztec/types/noir';

import { executeCircuit } from '@noir-lang/acvm_js';
import { type WitnessMap } from '@noir-lang/types';

import { type SimulationProvider } from './simulation_provider.js';

export class WASMSimulator implements SimulationProvider {
  async simulateCircuit(input: WitnessMap, compiledCircuit: NoirCompiledCircuit): Promise<WitnessMap> {
    // Execute the circuit on those initial witness values
    //
    // Decode the bytecode from base64 since the acvm does not know about base64 encoding
    const decodedBytecode = Buffer.from(compiledCircuit.bytecode, 'base64');
    //
    // Execute the circuit
    const _witnessMap = await executeCircuit(
      decodedBytecode,
      input,
      foreignCallHandler, // handle calls to debug_log
    );

    return _witnessMap;
  }
}
