import { NoirCompiledCircuit } from '@aztec/types/noir';

import { WasmBlackBoxFunctionSolver, createBlackBoxSolver, executeCircuitWithBlackBoxSolver } from '@noir-lang/acvm_js';
import { WitnessMap } from '@noir-lang/types';

import { SimulationProvider } from './simulation_provider.js';

let solver: Promise<WasmBlackBoxFunctionSolver>;

const getSolver = (): Promise<WasmBlackBoxFunctionSolver> => {
  if (!solver) {
    solver = createBlackBoxSolver();
  }
  return solver;
};

export class WASMSimulator implements SimulationProvider {
  async simulateCircuit(input: WitnessMap, compiledCircuit: NoirCompiledCircuit): Promise<WitnessMap> {
    // Execute the circuit on those initial witness values
    //
    // Decode the bytecode from base64 since the acvm does not know about base64 encoding
    const decodedBytecode = Buffer.from(compiledCircuit.bytecode, 'base64');
    //
    // Execute the circuit
    const _witnessMap = await executeCircuitWithBlackBoxSolver(await getSolver(), decodedBytecode, input, () => {
      throw Error('unexpected oracle during execution');
    });

    return _witnessMap;
  }
}
