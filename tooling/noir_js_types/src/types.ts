import { Abi } from '@noir-lang/noirc_abi';

export interface Backend {
  generateFinalProof(decompressedWitness: Uint8Array): Promise<ProofData>;
  generateIntermediateProof(decompressedWitness: Uint8Array): Promise<ProofData>;
  verifyFinalProof(proofData: ProofData): Promise<boolean>;
  verifyIntermediateProof(proofData: ProofData): Promise<boolean>;
  destroy(): Promise<void>;
}

export type BackendOptions = {
  threads: number;
};

export type ProofData = {
  publicInputs: Uint8Array[];
  proof: Uint8Array;
};

export type CompiledCircuit = {
  bytecode: string;
  abi: Abi;
};
