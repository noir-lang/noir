import { ProofData } from '@noir-lang/types';

// This is the number of bytes in a UltraPlonk proof
// minus the public inputs.
export const NUM_BYTES_IN_PROOF_WITHOUT_PUBLIC_INPUTS: number = 2144;

export function splitPublicInputsFromProof(
  proofWithPublicInputs: Uint8Array,
  numBytesInProofWithoutPublicInputs: number,
): ProofData {
  const splitIndex = proofWithPublicInputs.length - numBytesInProofWithoutPublicInputs;

  const publicInputSize = 32;
  const publicInputsConcatenated = proofWithPublicInputs.slice(0, splitIndex);

  const proof = proofWithPublicInputs.slice(splitIndex);
  const publicInputs = chunkUint8Array(publicInputsConcatenated, publicInputSize);

  return { proof, publicInputs };
}

export function reconstructProofWithPublicInputs(proofData: ProofData): Uint8Array {
  // Flatten publicInputs
  const publicInputsConcatenated = flattenUint8Arrays(proofData.publicInputs);

  // Concatenate publicInputs and proof
  const proofWithPublicInputs = Uint8Array.from([...publicInputsConcatenated, ...proofData.proof]);

  return proofWithPublicInputs;
}

function chunkUint8Array(array: Uint8Array, chunkSize: number): Uint8Array[] {
  if (array.length % chunkSize != 0) {
    throw Error(`Uint8Array cannot be cleanly split into chunks of ${chunkSize} bytes`);
  }

  const chunkedArray: Uint8Array[] = [];
  for (let i = 0; i < array.length; i += chunkSize) {
    const chunk: Uint8Array = array.slice(i, i + chunkSize);
    chunkedArray.push(chunk);
  }

  return chunkedArray;
}

function flattenUint8Arrays(arrays: Uint8Array[]): Uint8Array {
  const totalLength = arrays.reduce((acc, val) => acc + val.length, 0);
  const result = new Uint8Array(totalLength);

  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }

  return result;
}
