export interface Backend {
  // Generate an outer proof. This is the proof for the circuit which will verify
  // inner proofs and or can be seen as the proof created for regular circuits.
  generateProof(decompressedWitness: Uint8Array, optimizeForVerifyInCircuit?: boolean): Promise<Uint8Array>;

  // Generates a child proof. Child Proof will be verified in another Circuit.
  generateChildProof(decompressedWitness: Uint8Array): Promise<Uint8Array>;

  verifyProof(proof: Uint8Array, optimizeForVerifyInCircuit?: boolean): Promise<boolean>;

  // Verifies a child proof.
  verifyChildProof(proof: Uint8Array): Promise<boolean>;
}

export type CompiledCircuit = {
  bytecode: string;
  abi: object;
};
