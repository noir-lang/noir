import { ProofData } from '@noir-lang/types';
import { BackendOptions } from './types.js';
import { flattenPublicInputsAsArray } from './public_inputs.js';
import { type Barretenberg } from '@aztec/bb.js';

export class BarretenbergVerifier {
  // These type assertions are used so that we don't
  // have to initialize `api` and `acirComposer` in the constructor.
  // These are initialized asynchronously in the `init` function,
  // constructors cannot be asynchronous which is why we do this.

  private api!: Barretenberg;
  // eslint-disable-next-line  @typescript-eslint/no-explicit-any
  private acirComposer: any;

  constructor(private options: BackendOptions = { threads: 1 }) {}

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

      // This is the number of CRS points necessary to verify a Barretenberg proof.
      const NUM_CRS_POINTS_FOR_VERIFICATION: number = 0;
      const [api, crs] = await Promise.all([Barretenberg.new(this.options), Crs.new(NUM_CRS_POINTS_FOR_VERIFICATION)]);

      await api.commonInitSlabAllocator(NUM_CRS_POINTS_FOR_VERIFICATION);
      await api.srsInitSrs(
        new RawBuffer([] /* crs.getG1Data() */),
        NUM_CRS_POINTS_FOR_VERIFICATION,
        new RawBuffer(crs.getG2Data()),
      );

      this.acirComposer = await api.acirNewAcirComposer(NUM_CRS_POINTS_FOR_VERIFICATION);
      this.api = api;
    }
  }

  /** @description Verifies a proof */
  async verifyProof(proofData: ProofData, verificationKey: Uint8Array): Promise<boolean> {
    const { RawBuffer } = await import('@aztec/bb.js');

    await this.instantiate();
    // The verifier can be used for a variety of ACIR programs so we should not assume that it
    // is preloaded with the correct verification key.
    await this.api.acirLoadVerificationKey(this.acirComposer, new RawBuffer(verificationKey));

    const proof = reconstructProofWithPublicInputs(proofData);
    return await this.api.acirVerifyProof(this.acirComposer, proof);
  }

  async destroy(): Promise<void> {
    if (!this.api) {
      return;
    }
    await this.api.destroy();
  }
}

export function reconstructProofWithPublicInputs(proofData: ProofData): Uint8Array {
  // Flatten publicInputs
  const publicInputsConcatenated = flattenPublicInputsAsArray(proofData.publicInputs);

  // Concatenate publicInputs and proof
  const proofWithPublicInputs = Uint8Array.from([...publicInputsConcatenated, ...proofData.proof]);

  return proofWithPublicInputs;
}
