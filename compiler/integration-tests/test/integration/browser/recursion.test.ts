/* eslint-disable @typescript-eslint/ban-ts-comment */
import { expect } from "@esm-bundle/chai";
import { TEST_LOG_LEVEL } from "../../environment.js";
import { Logger } from "tslog";
import { initializeResolver } from "@noir-lang/source-resolver";
import newCompiler, {
  compile,
  init_log_level as compilerLogLevel,
} from "@noir-lang/noir_wasm";
import { decompressSync as gunzip } from "fflate";
import { acvm, abi, generateWitness } from "@noir-lang/noir_js";

// @ts-ignore
import { Barretenberg, RawBuffer, Crs } from "@aztec/bb.js";

import * as TOML from "smol-toml";

const logger = new Logger({ name: "test", minLevel: TEST_LOG_LEVEL });

const { default: initACVM } = acvm;
const { default: newABICoder } = abi;

await newCompiler();
await newABICoder();
await initACVM();

compilerLogLevel("INFO");

const numberOfThreads = navigator.hardwareConcurrency || 1;

const base_relative_path = "../../../../..";
const circuit_main = "compiler/integration-tests/test/circuits/main";
const circuit_recursion = "compiler/integration-tests/test/circuits/recursion";

async function getFile(url: URL): Promise<string> {
  const response = await fetch(url);

  if (!response.ok) throw new Error("Network response was not OK");

  return await response.text();
}

const CIRCUIT_SIZE = 2 ** 19;

const api = await Barretenberg.new(numberOfThreads);
await api.commonInitSlabAllocator(CIRCUIT_SIZE);
// Plus 1 needed!
const crs = await Crs.new(CIRCUIT_SIZE + 1);
await api.srsInitSrs(
  new RawBuffer(crs.getG1Data()),
  crs.numPoints,
  new RawBuffer(crs.getG2Data()),
);

const acirComposer = await api.acirNewAcirComposer(CIRCUIT_SIZE);

async function getCircuit(noirSource) {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  initializeResolver((id: string) => {
    logger.debug("source-resolver: resolving:", id);
    return noirSource;
  });

  return compile({});
}

async function generateProof(
  acirUint8Array: Uint8Array,
  witnessUint8Array: Uint8Array,
  optimizeForRecursion: boolean,
) {
  // This took ~6.5 minutes!
  return api.acirCreateProof(
    acirComposer,
    acirUint8Array,
    witnessUint8Array,
    optimizeForRecursion,
  );
}

async function verifyProof(proof: Uint8Array, optimizeForRecursion: boolean) {
  await api.acirInitVerificationKey(acirComposer);
  const verified = await api.acirVerifyProof(
    acirComposer,
    proof,
    optimizeForRecursion,
  );
  return verified;
}

describe("It compiles noir program code, receiving circuit bytes and abi object.", () => {
  let circuit_main_source;
  let circuit_main_toml;
  let circuit_recursion_source;

  before(async () => {
    const circuit_main_source_url = new URL(
      `${base_relative_path}/${circuit_main}/src/main.nr`,
      import.meta.url,
    );
    const circuit_main_toml_url = new URL(
      `${base_relative_path}/${circuit_main}/Prover.toml`,
      import.meta.url,
    );

    circuit_main_source = await getFile(circuit_main_source_url);
    circuit_main_toml = await getFile(circuit_main_toml_url);

    const circuit_recursion_source_url = new URL(
      `${base_relative_path}/${circuit_recursion}/src/main.nr`,
      import.meta.url,
    );

    circuit_recursion_source = await getFile(circuit_recursion_source_url);
  });

  it("Should generate valid inner proof for correct input, then verify proof within a proof", async () => {
    const { circuit: main_circuit, abi: main_abi } =
      await getCircuit(circuit_main_source);
    const main_inputs = TOML.parse(circuit_main_toml);

    const main_witnessUint8Array = await generateWitness(
      {
        bytecode: main_circuit,
        abi: main_abi
      },
      main_inputs,
    );
    const main_compressedByteCode = Uint8Array.from(atob(main_circuit), (c) =>
      c.charCodeAt(0),
    );
    const main_acirUint8Array = gunzip(main_compressedByteCode);

    const optimizeMainProofForRecursion = true;

    const main_proof = await generateProof(
      main_acirUint8Array,
      main_witnessUint8Array,
      optimizeMainProofForRecursion,
    );

    const main_verification = await verifyProof(
      main_proof,
      optimizeMainProofForRecursion,
    );

    logger.debug("main_verification", main_verification);

    expect(main_verification).to.be.true;

    const numPublicInputs = 1;
    const proofAsFields = (
      await api.acirSerializeProofIntoFields(
        acirComposer,
        main_proof,
        numPublicInputs,
      )
    ).map((p) => p.toString());

    const vk = await api.acirSerializeVerificationKeyIntoFields(acirComposer);

    const vkAsFields = vk[0].map((vk) => vk.toString());
    const vkHash = vk[1].toString();

    const recursion_inputs = {
      verification_key: vkAsFields,
      proof: proofAsFields,
      public_inputs: [main_inputs.y],
      key_hash: vkHash,
      // eslint-disable-next-line prettier/prettier
      input_aggregation_object: ["0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0"]
    };

    logger.debug("recursion_inputs", recursion_inputs);

    const { circuit: recursion_circuit, abi: recursion_abi } = await getCircuit(
      circuit_recursion_source,
    );

    const recursion_witnessUint8Array = await generateWitness(
      {
        bytecode: recursion_circuit,
        abi: recursion_abi
      },
      recursion_inputs,
    );

    const recursion_compressedByteCode = Uint8Array.from(
      atob(recursion_circuit),
      (c) => c.charCodeAt(0),
    );

    const recursion_acirUint8Array = gunzip(recursion_compressedByteCode);

    const optimizeRecursionProofForRecursion = false;

    const recursion_proof = await generateProof(
      recursion_acirUint8Array,
      recursion_witnessUint8Array,
      optimizeRecursionProofForRecursion,
    );

    const recursion_numPublicInputs = 1;

    const recursion_proofAsFields = (
      await api.acirSerializeProofIntoFields(
        acirComposer,
        recursion_proof,
        recursion_numPublicInputs,
      )
    ).map((p) => p.toString());

    logger.debug("recursion_proofAsFields", recursion_proofAsFields);

    const recursion_verification = await verifyProof(recursion_proof, false);

    logger.debug("recursion_verification", recursion_verification);

    expect(recursion_verification).to.be.true;
  }).timeout(60 * 20e3);
});
