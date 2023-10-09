interface BackendInternal {
  circuit: CompiledCircuit;
  generateFinalProof(decompressedWitness: Uint8Array): Promise<ProofData>;
  verifyFinalProof(proofData: ProofData): Promise<boolean>;
  instantiate(): Promise<void>;
  destroy(): Promise<void>;
}

export type ProofArtifacts = {
  proofAsFields: string[];
  vkAsFields: string[];
  vkHash: string;
};

export interface Backend extends BackendInternal {
  generateIntermediateProof(decompressedWitness: Uint8Array): Promise<ProofData>;
  verifyIntermediateProof(proofData: ProofData): Promise<boolean>;
  generateIntermediateProofArtifacts(proofData: ProofData, numOfPublicInputs: number): Promise<ProofArtifacts>;
}

export type BackendOptions = {
  numOfThreads: number;
};

export type ProofData = {
  publicInputs: Uint8Array[];
  proof: Uint8Array;
};

export type CompiledCircuit = {
  bytecode: string;
  abi: object;
};
