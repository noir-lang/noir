import { abiEncode } from '@noir-lang/noirc_abi';
import { validateInputs } from './input_validation.js';
import { base64Decode } from './base64_decode.js';
import { WitnessMap, executeCircuit } from '@noir-lang/acvm_js';

// Generates the witnesses needed to feed into the chosen proving system
export async function generateWitness(compiledProgram, inputs): Promise<WitnessMap> {
  // Validate inputs
  const { isValid, error } = validateInputs(inputs, compiledProgram.abi);
  if (!isValid) {
    throw new Error(error?.toString());
  }
  const witnessMap = abiEncode(compiledProgram.abi, inputs, null);

  // Execute the circuit to generate the rest of the witnesses
  try {
    const solvedWitness = await executeCircuit(base64Decode(compiledProgram.bytecode), witnessMap, () => {
      throw Error('unexpected oracle during execution');
    });
    return solvedWitness;
  } catch (err) {
    throw new Error(`Circuit execution failed: ${err}`);
  }
}
