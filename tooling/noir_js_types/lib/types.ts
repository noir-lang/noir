export interface Backend {
  // Generate an outer proof. This is the proof for the circuit which will verify
  // inner proofs and or can be seen as the proof created for regular circuits.
  generateFinalProof(decompressedWitness: Uint8Array): Promise<Uint8Array>;

  // Generates an inner proof. This is the proof that will be verified
  // in another circuit.
  generateIntermediateProof(decompressedWitness: Uint8Array): Promise<Uint8Array>;

  verifyFinalProof(proof: Uint8Array): Promise<boolean>;

  verifyIntermediateProof(proof: Uint8Array): Promise<boolean>;
}

export type CompiledCircuit = {
  bytecode: string;
  abi: object;
};
