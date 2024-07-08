import { decompressSync as gunzip } from 'fflate';
import { acirToUint8Array } from './serialize.js';
import { Backend, CompiledCircuit, ProofData, VerifierBackend } from '@noir-lang/types';
import { BackendOptions } from './types.js';
import { deflattenPublicInputs } from './public_inputs.js';
import { reconstructProofWithPublicInputs } from './verifier.js';
import { type Barretenberg } from '@aztec/bb.js';

// This is the number of bytes in a UltraPlonk proof
// minus the public inputs.
const numBytesInProofWithoutPublicInputs: number = 2144;

export class BarretenbergBackend implements Backend, VerifierBackend {
  // These type assertions are used so that we don't
  // have to initialize `api` and `acirComposer` in the constructor.
  // These are initialized asynchronously in the `init` function,
  // constructors cannot be asynchronous which is why we do this.

  protected api!: Barretenberg;
  // eslint-disable-next-line  @typescript-eslint/no-explicit-any
  protected acirComposer: any;
  protected acirUncompressedBytecode: Uint8Array;

  constructor(
    acirCircuit: CompiledCircuit,
    protected options: BackendOptions = { threads: 1 },
  ) {
    const acirBytecodeBase64 = acirCircuit.bytecode;
    this.acirUncompressedBytecode = acirToUint8Array(acirBytecodeBase64);
  }

  /** @ignore */
  async instantiate(): Promise<void> {
    if (!this.api) {
      if (typeof navigator !== 'undefined' && navigator.hardwareConcurrency) {
        this.options.threads = navigator.hardwareConcurrency;
      } else {
        try {
          const os = await import('os');
          this.options.threads = os.cpus().length;
        } catch (e) {
          console.log('Could not detect environment. Falling back to one thread.', e);
        }
      }
      const { Barretenberg, RawBuffer, Crs } = await import('@aztec/bb.js');
      const api = await Barretenberg.new(this.options);

      const honkRecursion = false;
      const [_exact, _total, subgroupSize] = await api.acirGetCircuitSizes(
        this.acirUncompressedBytecode,
        honkRecursion,
      );
      const crs = await Crs.new(subgroupSize + 1);
      await api.commonInitSlabAllocator(subgroupSize);
      await api.srsInitSrs(new RawBuffer(crs.getG1Data()), crs.numPoints, new RawBuffer(crs.getG2Data()));

      this.acirComposer = await api.acirNewAcirComposer(subgroupSize);
      await api.acirInitProvingKey(this.acirComposer, this.acirUncompressedBytecode);
      this.api = api;
    }
  }

  /** @description Generates a proof */
  async generateProof(compressedWitness: Uint8Array): Promise<ProofData> {
    await this.instantiate();
    const proofWithPublicInputs = await this.api.acirCreateProof(
      this.acirComposer,
      this.acirUncompressedBytecode,
      gunzip(compressedWitness),
    );

    const splitIndex = proofWithPublicInputs.length - numBytesInProofWithoutPublicInputs;

    const publicInputsConcatenated = proofWithPublicInputs.slice(0, splitIndex);
    const proof = proofWithPublicInputs.slice(splitIndex);
    const publicInputs = deflattenPublicInputs(publicInputsConcatenated);

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
    await this.instantiate();
    const proof = reconstructProofWithPublicInputs(proofData);
    const proofAsFields = (
      await this.api.acirSerializeProofIntoFields(this.acirComposer, proof, numOfPublicInputs)
    ).slice(numOfPublicInputs);

    // TODO: perhaps we should put this in the init function. Need to benchmark
    // TODO how long it takes.
    await this.api.acirInitVerificationKey(this.acirComposer);

    // Note: If you don't init verification key, `acirSerializeVerificationKeyIntoFields`` will just hang on serialization
    const vk = await this.api.acirSerializeVerificationKeyIntoFields(this.acirComposer);

    return {
      proofAsFields: proofAsFields.map((p) => p.toString()),
      vkAsFields: vk[0].map((vk) => vk.toString()),
      vkHash: vk[1].toString(),
    };
  }

  /** @description Verifies a proof */
  async verifyProof(proofData: ProofData): Promise<boolean> {
    const proof = reconstructProofWithPublicInputs(proofData);
    await this.instantiate();
    await this.api.acirInitVerificationKey(this.acirComposer);
    return await this.api.acirVerifyProof(this.acirComposer, proof);
  }

  async getVerificationKey(): Promise<Uint8Array> {
    await this.instantiate();
    await this.api.acirInitVerificationKey(this.acirComposer);
    return await this.api.acirGetVerificationKey(this.acirComposer);
  }

  async destroy(): Promise<void> {
    if (!this.api) {
      return;
    }
    await this.api.destroy();
  }
}
