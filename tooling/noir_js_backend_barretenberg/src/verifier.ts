import { ProofData } from '@noir-lang/types';
import { flattenFieldsAsArray } from './public_inputs.js';
import {
  BackendOptions,
  BarretenbergVerifier as BarretenbergVerifierInternal,
  reconstructHonkProof,
} from '@aztec/bb.js';

export class BarretenbergVerifier {
  private verifier!: BarretenbergVerifierInternal;

  constructor(options: BackendOptions = { threads: 1 }) {
    this.verifier = new BarretenbergVerifierInternal(options);
  }

  /** @description Verifies a proof */
  async verifyProof(proofData: ProofData, verificationKey: Uint8Array): Promise<boolean> {
    const proof = reconstructProofWithPublicInputs(proofData);
    return this.verifier.verifyUltraplonkProof(proof, verificationKey);
  }

  async destroy(): Promise<void> {
    await this.verifier.destroy();
  }
}

export function reconstructProofWithPublicInputs(proofData: ProofData): Uint8Array {
  // Flatten publicInputs
  const publicInputsConcatenated = flattenFieldsAsArray(proofData.publicInputs);

  // Concatenate publicInputs and proof
  const proofWithPublicInputs = Uint8Array.from([...publicInputsConcatenated, ...proofData.proof]);

  return proofWithPublicInputs;
}

export class UltraHonkVerifier {
  private verifier!: BarretenbergVerifierInternal;

  constructor(options: BackendOptions = { threads: 1 }) {
    this.verifier = new BarretenbergVerifierInternal(options);
  }

  /** @description Verifies a proof */
  async verifyProof(proofData: ProofData, verificationKey: Uint8Array): Promise<boolean> {
    const flattenedPublicInputs = flattenFieldsAsArray(proofData.publicInputs);
    const proof = reconstructHonkProof(flattenedPublicInputs, proofData.proof);
    return this.verifier.verifyUltrahonkProof(proof, verificationKey);
  }

  async destroy(): Promise<void> {
    await this.verifier.destroy();
  }
}
