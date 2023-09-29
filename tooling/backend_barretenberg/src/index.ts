/* eslint-disable  @typescript-eslint/no-explicit-any */
// eslint-disable-next-line @typescript-eslint/ban-ts-comment

// import { Barretenberg, Crs, RawBuffer } from '@aztec/bb.js/dest/browser/types/index.js';
import { decompressSync as gunzip } from 'fflate';
import { Backend, CompiledCircuit } from '@noir-lang/types';

// Since this is a simple function, we can use feature detection to
// see if we are in the nodeJs environment or the browser environment.
export function base64Decode(input: string): Uint8Array {
  return Uint8Array.from(atob(input), (c) => c.charCodeAt(0));
}

// Converts an bytecode to a Uint8Array
export function acirToUint8Array(base64EncodedBytecode): Uint8Array {
  const compressedByteCode = base64Decode(base64EncodedBytecode);
  return gunzip(compressedByteCode);
}

export class BarretenbergBackend implements Backend {
  // These type assertions are used so that we don't
  // have to initialize `api` and `acirComposer` in the constructor.
  // These are initialized asynchronously in the `init` function,
  // constructors cannot be asynchronous which is why we do this.
  private api: any;
  private acirComposer: any;
  private acirUncompressedBytecode: Uint8Array;
  private numberOfThreads = 1;

  constructor(acirCircuit: CompiledCircuit, numberOfThreads = 1) {
    const acirBytecodeBase64 = acirCircuit.bytecode;
    this.numberOfThreads = numberOfThreads;
    this.acirUncompressedBytecode = acirToUint8Array(acirBytecodeBase64);
  }

  async instantiate(): Promise<void> {
    if (!this.api) {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      //@ts-ignore
      const { Barretenberg, RawBuffer, Crs } = await import('@aztec/bb.js');
      const api = await Barretenberg.new(this.numberOfThreads);

      const [_exact, _total, subgroupSize] = await api.acirGetCircuitSizes(this.acirUncompressedBytecode);
      const crs = await Crs.new(subgroupSize + 1);
      await api.commonInitSlabAllocator(subgroupSize);
      await api.srsInitSrs(new RawBuffer(crs.getG1Data()), crs.numPoints, new RawBuffer(crs.getG2Data()));

      this.acirComposer = await api.acirNewAcirComposer(subgroupSize);
      this.api = api;
    }
  }

  // Generate an outer proof. This is the proof for the circuit which will verify
  // inner proofs and or can be seen as the proof created for regular circuits.
  //
  // The settings for this proof are the same as the settings for a "normal" proof
  // ie one that is not in the recursive setting.
  async generateFinalProof(decompressedWitness: Uint8Array, optimizeForVerifyInCircuit = false): Promise<Uint8Array> {
    await this.instantiate();
    const proof = await this.api.acirCreateProof(
      this.acirComposer,
      this.acirUncompressedBytecode,
      decompressedWitness,
      optimizeForVerifyInCircuit,
    );

    return proof;
  }

  // Generates an inner proof. This is the proof that will be verified
  // in another circuit.
  //
  // This is sometimes referred to as a recursive proof.
  // We avoid this terminology as the only property of this proof
  // that matters, is the fact that it is easy to verify in another
  // circuit. We _could_ choose to verify this proof in the CLI.
  //
  // We set `makeEasyToVerifyInCircuit` to true, which will tell the backend to
  // generate the proof using components that will make the proof
  // easier to verify in a circuit.
  async generateIntermediateProof(witness: Uint8Array): Promise<Uint8Array> {
    const optimizeForVerifyInCircuit = true;
    return this.generateFinalProof(witness, optimizeForVerifyInCircuit);
  }

  // Generates artifacts that will be passed to a circuit that will verify this proof.
  //
  // Instead of passing the proof and verification key as a byte array, we pass them
  // as fields which makes it cheaper to verify in a circuit.
  //
  // The proof that is passed here will have been created using the `generateInnerProof`
  // method.
  //
  // The number of public inputs denotes how many public inputs are in the inner proof.
  async generateIntermediateProofArtifacts(
    proof: Uint8Array,
    numOfPublicInputs = 0,
  ): Promise<{
    proofAsFields: string[];
    vkAsFields: string[];
    vkHash: string;
  }> {
    await this.instantiate();
    const proofAsFields = await this.api.acirSerializeProofIntoFields(this.acirComposer, proof, numOfPublicInputs);

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

  async verifyIntermediateProof(proof: Uint8Array): Promise<boolean> {
    const optimizeForVerifyInCircuit = true;
    return this.verifyFinalProof(proof, optimizeForVerifyInCircuit);
  }

  async verifyFinalProof(proof: Uint8Array, optimizeForVerifyInCircuit = false): Promise<boolean> {
    await this.instantiate();
    await this.api.acirInitVerificationKey(this.acirComposer);
    return await this.api.acirVerifyProof(this.acirComposer, proof, optimizeForVerifyInCircuit);
  }

  async destroy(): Promise<void> {
    await this.api.destroy();
  }
}
