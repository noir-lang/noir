import { abiEncode } from '@noir-lang/noirc_abi';
import { base64Decode } from './base64_decode.js';
import { executeCircuit } from '@noir-lang/acvm_js';
import { witnessMapToUint8Array } from './serialize.js';

// Generates the witnesses needed to feed into the chosen proving system
export async function generateWitness(compiledProgram, inputs): Promise<Uint8Array> {
  // Throws on ABI encoding error
  const witnessMap = abiEncode(compiledProgram.abi, inputs, null);

  const compressedByteCode = base64Decode(compiledProgram.bytecode);

  // Execute the circuit to generate the rest of the witnesses and serialize
  // them into a Uint8Array.
  try {
    const solvedWitness = await executeCircuit(compressedByteCode, witnessMap, () => {
      throw Error('unexpected oracle during execution');
    });
    return witnessMapToUint8Array(solvedWitness);
  } catch (err) {
    throw new Error(`Circuit execution failed: ${err}`);
  }
}
