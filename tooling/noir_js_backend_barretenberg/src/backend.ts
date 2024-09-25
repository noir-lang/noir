import { acirToUint8Array } from './serialize.js';
import { Backend, CompiledCircuit, ProofData, VerifierBackend } from '@noir-lang/types';
import { deflattenFields } from './public_inputs.js';
import { reconstructProofWithPublicInputs, reconstructProofWithPublicInputsHonk } from './verifier.js';
import { BackendOptions, UltraPlonkBackend, UltraHonkBackend as UltraHonkBackendInternal } from '@aztec/bb.js';
import { decompressSync as gunzip } from 'fflate';

// This is the number of bytes in a UltraPlonk proof
// minus the public inputs.
const numBytesInProofWithoutPublicInputs: number = 2144;

export class BarretenbergBackend implements Backend, VerifierBackend {
  protected backend!: UltraPlonkBackend;

  constructor(acirCircuit: CompiledCircuit, options: BackendOptions = { threads: 1 }) {
    const acirBytecodeBase64 = acirCircuit.bytecode;
    const acirUncompressedBytecode = acirToUint8Array(acirBytecodeBase64);
    this.backend = new UltraPlonkBackend(acirUncompressedBytecode, options);
  }

  /** @description Generates a proof */
  async generateProof(compressedWitness: Uint8Array): Promise<ProofData> {
    const proofWithPublicInputs = await this.backend.generateProof(gunzip(compressedWitness));

    const splitIndex = proofWithPublicInputs.length - numBytesInProofWithoutPublicInputs;

    const publicInputsConcatenated = proofWithPublicInputs.slice(0, splitIndex);
    const proof = proofWithPublicInputs.slice(splitIndex);
    const publicInputs = deflattenFields(publicInputsConcatenated);

    return { proof, publicInputs };
  }

  /**
   * Generates artifacts that will be passed to a circuit that will verify this proof.
   *
   * Instead of passing the proof and verification key as a byte array, we pass them
   * as fields which makes it cheaper to verify in a circuit.
   *
   * The proof that is passed here will have been created using a circuit
   * that has the #[recursive] attribute on its `main` method.
   *
   * The number of public inputs denotes how many public inputs are in the inner proof.
   *
   * @example
   * ```typescript
   * const artifacts = await backend.generateRecursiveProofArtifacts(proof, numOfPublicInputs);
   * ```
   */
  async generateRecursiveProofArtifacts(
    proofData: ProofData,
    numOfPublicInputs = 0,
  ): Promise<{
    proofAsFields: string[];
    vkAsFields: string[];
    vkHash: string;
  }> {
    const proof = reconstructProofWithPublicInputs(proofData);
    return this.backend.generateRecursiveProofArtifacts(proof, numOfPublicInputs);
  }

  /** @description Verifies a proof */
  async verifyProof(proofData: ProofData): Promise<boolean> {
    const proof = reconstructProofWithPublicInputs(proofData);
    return this.backend.verifyProof(proof);
  }

  async getVerificationKey(): Promise<Uint8Array> {
    return this.backend.getVerificationKey();
  }

  async destroy(): Promise<void> {
    await this.backend.destroy();
  }
}

// Buffers are prepended with their size. The size takes 4 bytes.
const serializedBufferSize = 4;
const fieldByteSize = 32;
const publicInputOffset = 3;
const publicInputsOffsetBytes = publicInputOffset * fieldByteSize;

export class UltraHonkBackend implements Backend, VerifierBackend {
  // These type assertions are used so that we don't
  // have to initialize `api` in the constructor.
  // These are initialized asynchronously in the `init` function,
  // constructors cannot be asynchronous which is why we do this.

  protected backend!: UltraHonkBackendInternal;

  constructor(acirCircuit: CompiledCircuit, options: BackendOptions = { threads: 1 }) {
    const acirBytecodeBase64 = acirCircuit.bytecode;
    const acirUncompressedBytecode = acirToUint8Array(acirBytecodeBase64);
    this.backend = new UltraHonkBackendInternal(acirUncompressedBytecode, options);
  }

  async generateProof(compressedWitness: Uint8Array): Promise<ProofData> {
    const proofWithPublicInputs = await this.backend.generateProof(gunzip(compressedWitness));
    const proofAsStrings = deflattenFields(proofWithPublicInputs.slice(4));

    const numPublicInputs = Number(proofAsStrings[1]);

    // Account for the serialized buffer size at start
    const publicInputsOffset = publicInputsOffsetBytes + serializedBufferSize;
    // Get the part before and after the public inputs
    const proofStart = proofWithPublicInputs.slice(0, publicInputsOffset);
    const publicInputsSplitIndex = numPublicInputs * fieldByteSize;
    const proofEnd = proofWithPublicInputs.slice(publicInputsOffset + publicInputsSplitIndex);
    // Construct the proof without the public inputs
    const proof = new Uint8Array([...proofStart, ...proofEnd]);

    // Fetch the number of public inputs out of the proof string
    const publicInputsConcatenated = proofWithPublicInputs.slice(
      publicInputsOffset,
      publicInputsOffset + publicInputsSplitIndex,
    );
    const publicInputs = deflattenFields(publicInputsConcatenated);

    return { proof, publicInputs };
  }

  async verifyProof(proofData: ProofData): Promise<boolean> {
    const proof = reconstructProofWithPublicInputsHonk(proofData);

    return this.backend.verifyProof(proof);
  }

  async getVerificationKey(): Promise<Uint8Array> {
    return this.backend.getVerificationKey();
  }

  // TODO(https://github.com/noir-lang/noir/issues/5661): Update this to handle Honk recursive aggregation in the browser once it is ready in the backend itself
  async generateRecursiveProofArtifacts(
    proofData: ProofData,
    numOfPublicInputs: number,
  ): Promise<{ proofAsFields: string[]; vkAsFields: string[]; vkHash: string }> {
    const proof = reconstructProofWithPublicInputsHonk(proofData);
    return this.backend.generateRecursiveProofArtifacts(proof, numOfPublicInputs);
  }

  async destroy(): Promise<void> {
    await this.backend.destroy();
  }
}
