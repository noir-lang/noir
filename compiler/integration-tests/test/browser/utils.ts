import { abiEncode } from '@noir-lang/noirc_abi';
import { executeCircuit, WitnessMap, compressWitness } from '@noir-lang/acvm_js';
import { decompressSync as gunzip } from 'fflate';
import { CompiledCircuit } from '@noir-lang/types';

export async function getFile(file_path: string): Promise<string> {
  const file_url = new URL(file_path, import.meta.url);
  const response = await fetch(file_url);

  if (!response.ok) throw new Error('Network response was not OK');

  return await response.text();
}

function base64Decode(input: string): Uint8Array {
  if (typeof Buffer !== 'undefined') {
    // Node.js environment
    return Buffer.from(input, 'base64');
  } else if (typeof atob === 'function') {
    // Browser environment
    return Uint8Array.from(atob(input), (c) => c.charCodeAt(0));
  } else {
    throw new Error('No implementation found for base64 decoding.');
  }
}

function witnessMapToUint8Array(solvedWitness: WitnessMap): Uint8Array {
  // TODO: We just want to serialize, but this will zip up the witness
  // TODO so its not ideal
  const compressedWitness = compressWitness(solvedWitness);
  return gunzip(compressedWitness);
}

// Converts an bytecode to a Uint8Array
export function acirToUint8Array(base64EncodedBytecode): Uint8Array {
  const compressedByteCode = base64Decode(base64EncodedBytecode);
  return gunzip(compressedByteCode);
}

export async function generateWitness(compiledProgram: CompiledCircuit, inputs: unknown): Promise<Uint8Array> {
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
