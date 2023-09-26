/* eslint-disable  @typescript-eslint/no-explicit-any */
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import { Barretenberg, Crs, RawBuffer } from '@aztec/bb.js';
// TODO: This should be re-exported from @aztec/bb-js
import { Ptr } from '@aztec/bb.js/dest/browser/types';
import { acirToUint8Array } from '../../src/index.js';
import { Backend } from './backend_interface.js';

export class BarretenbergBackend implements Backend {
  // These type assertions are used so that we don't
  // have to initialize `api` and `acirComposer` in the constructor.
  // These are initialized asynchronously in the `init` function,
  // constructors cannot be asynchronous which is why we do this.
  api = {} as Barretenberg;
  acirComposer = {} as Ptr;
  acirUncompressedBytecode: Uint8Array;

  private constructor(acirCircuit: { bytecode: string }) {
    const acirBytecodeBase64 = acirCircuit.bytecode;
    this.acirUncompressedBytecode = acirToUint8Array(acirBytecodeBase64);
  }

  static async initialize(acirCircuit: { bytecode: string }): Promise<BarretenbergBackend> {
    const backend = new BarretenbergBackend(acirCircuit);
    await backend.init();
    return backend;
  }

  private async init(): Promise<void> {
    const numThreads = 4;

    const { api, composer } = await this.initBarretenberg(numThreads, this.acirUncompressedBytecode);

    this.api = api;
    this.acirComposer = composer;
  }

  private async initBarretenberg(numThreads: number, acirUncompressedBytecode: Uint8Array) {
    const api = await Barretenberg.new(numThreads);

    const [_exact, _total, subgroupSize] = await api.acirGetCircuitSizes(acirUncompressedBytecode);
    const crs = await Crs.new(subgroupSize + 1);
    await api.commonInitSlabAllocator(subgroupSize);
    await api.srsInitSrs(new RawBuffer(crs.getG1Data()), crs.numPoints, new RawBuffer(crs.getG2Data()));

    const acirComposer = await api.acirNewAcirComposer(subgroupSize);
    return { api: api, composer: acirComposer };
  }

  // Generate an outer proof. This is the proof for the circuit which will verify
  // inner proofs and or can be seen as the proof created for regular circuits.
  //
  // The settings for this proof are the same as the settings for a "normal" proof
  // ie one that is not in the recursive setting.
  async generateFinalProof(decompressedWitness: Uint8Array): Promise<Uint8Array> {
    const makeEasyToVerifyInCircuit = false;
    return this.generateProof(decompressedWitness, makeEasyToVerifyInCircuit);
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
    const makeEasyToVerifyInCircuit = true;
    return this.generateProof(witness, makeEasyToVerifyInCircuit);
  }

  async generateProof(decompressedWitness: Uint8Array, makeEasyToVerifyInCircuit: boolean): Promise<Uint8Array> {
    const proof = await this.api.acirCreateProof(
      this.acirComposer,
      this.acirUncompressedBytecode,
      decompressedWitness,
      makeEasyToVerifyInCircuit,
    );

    return proof;
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

  async verifyFinalProof(proof: Uint8Array): Promise<boolean> {
    const makeEasyToVerifyInCircuit = false;
    const verified = await this.verifyProof(proof, makeEasyToVerifyInCircuit);
    return verified;
  }

  async verifyIntermediateProof(proof: Uint8Array): Promise<boolean> {
    const makeEasyToVerifyInCircuit = true;
    return this.verifyProof(proof, makeEasyToVerifyInCircuit);
  }

  async verifyProof(proof: Uint8Array, makeEasyToVerifyInCircuit: boolean): Promise<boolean> {
    await this.api.acirInitVerificationKey(this.acirComposer);
    return await this.api.acirVerifyProof(this.acirComposer, proof, makeEasyToVerifyInCircuit);
  }

  async destroy(): Promise<void> {
    await this.api.destroy();
  }
}
