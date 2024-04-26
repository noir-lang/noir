import { abiEncode, InputMap } from '@noir-lang/noirc_abi';
import { base64Decode } from './base64_decode.js';
import {
  WitnessMap,
  ForeignCallHandler,
  ForeignCallInput,
  createBlackBoxSolver,
  WasmBlackBoxFunctionSolver,
  executeCircuitWithBlackBoxSolver,
} from '@noir-lang/acvm_js';
import { CompiledCircuit } from '@noir-lang/types';

let solver: Promise<WasmBlackBoxFunctionSolver>;

const getSolver = (): Promise<WasmBlackBoxFunctionSolver> => {
  if (!solver) {
    solver = createBlackBoxSolver();
  }
  return solver;
};

const defaultForeignCallHandler: ForeignCallHandler = async (name: string, args: ForeignCallInput[]) => {
  if (name == 'print') {
    // By default we do not print anything for `print` foreign calls due to a need for formatting,
    // however we provide an empty response in order to not halt execution.
    //
    // If a user needs to print values then they should provide a custom foreign call handler.
    return [];
  }
  throw Error(`Unexpected oracle during execution: ${name}(${args.join(', ')})`);
};

// Generates the witnesses needed to feed into the chosen proving system
export async function generateWitness(
  compiledProgram: CompiledCircuit,
  inputs: InputMap,
  foreignCallHandler: ForeignCallHandler = defaultForeignCallHandler,
): Promise<WitnessMap> {
  // Throws on ABI encoding error
  const witnessMap = abiEncode(compiledProgram.abi, inputs);

  // Execute the circuit to generate the rest of the witnesses and serialize
  // them into a Uint8Array.
  try {
    const solvedWitness = await executeCircuitWithBlackBoxSolver(
      await getSolver(),
      base64Decode(compiledProgram.bytecode),
      witnessMap,
      foreignCallHandler,
    );
    return solvedWitness;
  } catch (err) {
    throw new Error(`Circuit execution failed: ${err}`);
  }
}
