// @ts-ignore
import { Barretenberg, Crs, RawBuffer } from '@aztec/bb.js';
// TODO: This should be re-exported from @aztec/bb-js
import { Ptr } from '@aztec/bb.js/dest/browser/types';
import { acirToUint8Array, acvm, generateWitness, witnessMapToUint8Array } from '../../src/index.js';
import { WitnessMap, getPublicParametersWitness } from '@noir-lang/acvm_js';
import { base64Decode } from '../../src/base64_decode.js';
import double_verify_json from '../noir_compiled_examples/double_verify_proof/target/double_verify_proof.json' assert { type: 'json' };

export class Backend {
  // These type assertions are used so that we don't
  // have to initialize `api` and `acirComposer` in the constructor.
  // These are initialized asynchronously in the `init` function,
  // constructors cannot be asynchronous which is why we do this.
  api = {} as Barretenberg;
  acirComposer = {} as Ptr;
  acirUncompressedBytecode: Uint8Array;

  constructor(acirBytecodeBase64: string) {
    this.acirUncompressedBytecode = acirToUint8Array(acirBytecodeBase64);
  }
  async init() {
    const numThreads = 4;

    const { api, composer } = await this.initBarretenberg(numThreads, this.acirUncompressedBytecode);

    this.api = api;
    this.acirComposer = composer;
  }

  async initBarretenberg(numThreads: number, acirUncompressedBytecode: Uint8Array) {
    const api = await Barretenberg.new(numThreads);

    const [exact, total, subgroupSize] = await api.acirGetCircuitSizes(acirUncompressedBytecode);
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
  async generateOuterProof(decompressedWitness: Uint8Array) {
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
  async generateInnerProof(witness: Uint8Array) {
    const makeEasyToVerifyInCircuit = true;
    return this.generateProof(witness, makeEasyToVerifyInCircuit);
  }

  async generateProof(decompressedWitness: Uint8Array, makeEasyToVerifyInCircuit: boolean) {
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
  async generateInnerProofArtifacts(proof: Uint8Array, numOfPublicInputs = 0) {
    const proofAsFields = await this.api.acirSerializeProofIntoFields(this.acirComposer, proof, numOfPublicInputs);

    // TODO: perhaps we should put this in the init function. Need to benchmark
    // TODO how long it takes.
    await this.api.acirInitVerificationKey(this.acirComposer);

    // Note: If you don't init verification key, `acirSerializeVerificationKeyIntoFields`` will just hang on serialization
    const vk = await this.api.acirSerializeVerificationKeyIntoFields(this.acirComposer);

    return {
      proofAsFields: proofAsFields.map((p: any) => p.toString()),
      vkAsFields: vk[0].map((vk: any) => vk.toString()),
      vkHash: vk[1].toString(),
    };
  }

  async verifyOuterProof(proof: Uint8Array) {
    const makeEasyToVerifyInCircuit = false;
    const verified = await this.verifyProof(proof, makeEasyToVerifyInCircuit);
    return verified;
  }

  async verifyInnerProof(proof: Uint8Array) {
    const makeEasyToVerifyInCircuit = true;
    return this.verifyProof(proof, makeEasyToVerifyInCircuit);
  }

  async verifyProof(proof: Uint8Array, makeEasyToVerifyInCircuit: boolean) {
    await this.api.acirInitVerificationKey(this.acirComposer);
    return await this.api.acirVerifyProof(this.acirComposer, proof, makeEasyToVerifyInCircuit);
  }

  async destroy() {
    await this.api.destroy();
  }
}

// This uses a 2 to 1 recursive backend.
// We assume all the circuits on the base layer are homogenous
// for simplicity, but we can change it, it will just require a lot more
// memory.
// TODO: Note that this API is vastly different from the one in the backend
// TODO my only concern is the usage of compiledProgram and bytecode not being consistent
//
// TODO: This API will change depending on whether we can aggregate different
// TODO circuits. If not, then API is fine. If so, then we cannot just save the proof
// TODO because each proof could be associated with a different circuit.
export class RecursiveBackend {
  backend: Backend;
  innerProofs: any[] = [];
  acirCompressedBytecode: Uint8Array;
  //
  compiledProgram: any;

  constructor(compiledProgram: any) {
    const acirBytecodeBase64 = compiledProgram.bytecode as string;
    this.backend = new Backend(acirBytecodeBase64);
    this.compiledProgram = compiledProgram;

    // TODO: This is needed because acvm reads in compressed bytecode
    // TODO: while backends read in uncompressed bytecode.
    // TODO: We should fix up the API to get rid of this discrepancy.
    //
    // Its only the raw barretenberg API that requires the uncompressed bytecode
    // The acir tests for example use the compressed bytecode.
    // We could:
    // - change the barretenberg raw API to use compressed bytecode
    // - make acvm take in uncompressed bytecode
    // - create an API over the raw barretenberg API that takes in compressed bytecode
    // I'm leaning towards modifying the acvm api to use uncompressed bytecode
    this.acirCompressedBytecode = base64Decode(acirBytecodeBase64);
  }

  async init() {
    await this.backend.init();
  }

  // Creates a new inner proof, given just the inputs
  //
  // This is a nice APi, though it requires the abi being known
  // by this class. This is the reason that the constructor takes in
  // a compiledProgram.
  async newInnerProof(inputs: any) {
    const solvedWitness = await generateWitness(this.compiledProgram, inputs);
    await this.newInnerProofFromSolvedWitness(solvedWitness);
  }

  private async newInnerProofFromSolvedWitness(solvedWitness: WitnessMap) {
    const serializedWitness = witnessMapToUint8Array(solvedWitness);
    const proof = await this.backend.generateInnerProof(serializedWitness);
    const numPublicParameters = getPublicParametersWitness(this.acirCompressedBytecode, solvedWitness).size;
    const innerProofArtifacts = await this.backend.generateInnerProofArtifacts(proof, numPublicParameters);
    this.innerProofs.push({ proofArtifacts: innerProofArtifacts, numPublicParameters: numPublicParameters });
  }

  // Recursively verifies a list of inner proofs.
  //
  //
  async finalize(): Promise<Uint8Array> {
    if (this.innerProofs.length == 0) {
      throw new Error('No inner proofs have been created');
    }
    if (this.innerProofs.length == 1) {
      throw new Error('Currently we do not support the edge case of 1 proof being created. Fix would be to pad.');
    }

    // TODO(Maxim): I this should be fixed in ACVM and barretenberg.
    if (this.innerProofs.length > 2) {
      throw new Error(
        'Currently we only support recursively verifying 2 proofs, where each proof must not contain a recursive verification opcode. Barretenberg API does not support this.',
      );
    }

    // Assuming we have the same circuit.
    const proofA = this.innerProofs[0];
    const proofB = this.innerProofs[1];

    this.backend = new Backend(double_verify_json.bytecode);
    await this.backend.init();
    await generateWitness(double_verify_json, { verification_key: proofA.proofArtifacts.vkAsFields, y: '3' });

    return new Uint8Array();

    // This is sort of broken at the moment because barretenberg does not accept proofs
    // which themselves have aggregation objects. So we can only have a tree which has two leaves.
    // So doing a tree style recursive aggregation does not really work.
    //
    // Need to discuss further what the API should look like.
    //
    // If it was not broken, one use-case is we would push all proofs into a queue and verify
    // each pair of proofs using an inner double-verify proof, until we get to the last layer.
    throw new Error('Its not clear if we should do a tree based recursion algorithm or something closer to IVF');
  }
}
