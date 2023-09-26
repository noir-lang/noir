export interface Backend {
  // Generate an outer proof. This is the proof for the circuit which will verify
  // inner proofs and or can be seen as the proof created for regular circuits.
  generateOuterProof(decompressedWitness: Uint8Array): Promise<Uint8Array>;

  // Generates an inner proof. This is the proof that will be verified
  // in another circuit.
  generateInnerProof(decompressedWitness: Uint8Array): Promise<Uint8Array>;

  verifyOuterProof(proof: Uint8Array): Promise<boolean>;

  verifyInnerProof(proof: Uint8Array): Promise<boolean>;
}
