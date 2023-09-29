import initABIWASM, { abiEncode } from '@noir-lang/noirc_abi';
import { base64Decode } from './base64_decode.js';
import initACVMWASM, { executeCircuit } from '@noir-lang/acvm_js';
import { witnessMapToUint8Array } from './serialize.js';

// Generates the witnesses needed to feed into the chosen proving system
export async function generateWitness(compiledProgram, inputs): Promise<Uint8Array> {
  if (typeof initABIWASM === 'function' && typeof initACVMWASM === 'function') {
    await initABIWASM();
    await initACVMWASM();
  }

  // Throws on ABI encoding error
  const witnessMap = abiEncode(compiledProgram.abi, inputs, null);

  // Execute the circuit to generate the rest of the witnesses and serialize
  // them into a Uint8Array.
  try {
    const solvedWitness = await executeCircuit(base64Decode(compiledProgram.bytecode), witnessMap, () => {
      throw Error('unexpected oracle during execution');
    });
    return witnessMapToUint8Array(solvedWitness);
  } catch (err) {
    throw new Error(`Circuit execution failed: ${err}`);
  }
}
