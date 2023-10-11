import { Abi } from '@noir-lang/noirc_abi';

export interface Backend {
  // Generate an outer proof. This is the proof for the circuit which will verify
  // inner proofs and or can be seen as the proof created for regular circuits.
  generateFinalProof(decompressedWitness: Uint8Array): Promise<ProofData>;

  // Generates an inner proof. This is the proof that will be verified
  // in another circuit.
  generateIntermediateProof(decompressedWitness: Uint8Array): Promise<ProofData>;

  verifyFinalProof(proofData: ProofData): Promise<boolean>;
  verifyIntermediateProof(proofData: ProofData): Promise<boolean>;
  destroy(): Promise<void>;
}

export type ProofData = {
  publicInputs: Uint8Array[];
  proof: Uint8Array;
};

export type CompiledCircuit = {
  bytecode: string;
  abi: Abi;
};
