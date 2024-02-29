import { ProofData } from '@noir-lang/types';
import { BackendOptions } from './types.js';
import { flattenPublicInputsAsArray } from './public_inputs.js';
import { type Barretenberg } from '@aztec/bb.js';

export class BarretenbergVerifier {
  // These type assertions are used so that we don't
  // have to initialize `api` and `acirComposer` in the constructor.
  // These are initialized asynchronously in the `instantiate` function,
  // constructors cannot be asynchronous which is why we do this.

  private api!: Barretenberg;
  // eslint-disable-next-line  @typescript-eslint/no-explicit-any
  private acirComposer: any;

  constructor(
    private verificationKey: Uint8Array,
    private options: BackendOptions = { threads: 1 },
  ) {}

  /** @ignore */
  async instantiate(): Promise<void> {
    if (!this.api) {
      const { Barretenberg, RawBuffer } = await import('@aztec/bb.js');
      const api = await Barretenberg.new({ threads: this.options.threads });
      const acirComposer = await api.acirNewAcirComposer(0)
      
      await api.acirLoadVerificationKey(acirComposer, new RawBuffer(this.verificationKey));

      this.acirComposer = acirComposer;
      this.api = api;
    }
  }

  /** @description Verifies a proof */
  async verifyProof(proofData: ProofData): Promise<boolean> {
    await this.instantiate();
    
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

function reconstructProofWithPublicInputs(proofData: ProofData): Uint8Array {
  // Flatten publicInputs
  const publicInputsConcatenated = flattenPublicInputsAsArray(proofData.publicInputs);

  // Concatenate publicInputs and proof
  const proofWithPublicInputs = Uint8Array.from([...publicInputsConcatenated, ...proofData.proof]);

  return proofWithPublicInputs;
}
