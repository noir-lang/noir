const FIELD_ELEMENT_BYTES = 32;

export function separatePublicInputsFromProof(
  proof: Uint8Array,
  numPublicInputs: number,
): { proof: Uint8Array; publicInputs: Uint8Array[] } {
  const publicInputs = Array.from({ length: numPublicInputs }, (_, i) => {
    const offset = i * FIELD_ELEMENT_BYTES;
    return proof.slice(offset, offset + FIELD_ELEMENT_BYTES);
  });
  const slicedProof = proof.slice(numPublicInputs * FIELD_ELEMENT_BYTES);

  return {
    proof: slicedProof,
    publicInputs,
  };
}
