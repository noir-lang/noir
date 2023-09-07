import { expect } from '@esm-bundle/chai';
import { initialiseResolver } from "@noir-lang/noir-source-resolver";
import newCompiler, {
  compile
} from "@noir-lang/noir_wasm";
// import { Barretenberg, RawBuffer, Crs } from '@aztec/bb.js';
import { decompressSync as gunzip } from 'fflate';
import newABICoder, { abiEncode, abiDecode } from "@noir-lang/noirc_abi";
import initACVM, {
  createBlackBoxSolver,
  executeCircuit,
  executeCircuitWithBlackBoxSolver,
  WasmBlackBoxFunctionSolver,
  WitnessMap,
  initLogLevel,
  ForeignCallHandler,
  compressWitness,
} from "@noir-lang/acvm_js";

import { Barretenberg, RawBuffer, Crs } from '@aztec/bb.js';

import * as TOML from 'smol-toml'

async function getFile(url: URL): Promise<string> {

  const response = await fetch(url)

  if (!response.ok) throw new Error('Network response was not OK');

  return await response.text();
}

const CIRCUIT_SIZE = 2 ** 19;

describe("Noir end to end test", () => {
  let numberOfThreads: number;
  let api: Barretenberg;

  beforeEach(async () => {
    numberOfThreads = navigator.hardwareConcurrency || 1;
    console.log("Will utilize number of Threads:", numberOfThreads);

    await newCompiler();
    await newABICoder();
    await initACVM();
    api = await Barretenberg.new(numberOfThreads);
  }, 15000);

  it("Compiles, Executes. Proves and Verifies", async () => {

    const base_relative_path = "../../../../";
    const noir_source_url = new URL(base_relative_path + 'nargo_cli/tests/execution_success/1_mul/src/main.nr', import.meta.url);
    const prover_toml_url = new URL(base_relative_path + 'nargo_cli/tests/execution_success/1_mul/Prover.toml', import.meta.url);

    const noir_source = await getFile(noir_source_url);
    const prover_toml = await getFile(prover_toml_url);


    initialiseResolver((id: String) => {
      return noir_source;
    });

    const inputs = TOML.parse(prover_toml);
    console.log("Prover.toml: ", inputs);

    const compile_output = await compile({});

    console.log(compile_output);
    const witnessMap = abiEncode(compile_output.abi, inputs, null);
    witnessMap.forEach((value, key) => {
      console.log(`MapValue: ${key} => ${value}`);
    });
    console.log("Witness map: ", witnessMap);
    const decoded_inputs = abiDecode(compile_output.abi, witnessMap);
    console.log("Decoded abi: ", decoded_inputs);

    console.log(compile_output);
    const compressedByteCode = Uint8Array.from(atob(compile_output.circuit), c => c.charCodeAt(0));
    // const compressedWitness = Uint8Array.from(atob(witness), c => c.charCodeAt(0));
    // // console.log(wasmCircuitBase64)


    const solvedWitness: WitnessMap = await executeCircuit(
      compressedByteCode,
      witnessMap,
      () => {
        throw Error("unexpected oracle");
      }
    );

    const compressedWitness = compressWitness(solvedWitness);
    const acirUint8Array = gunzip(compressedByteCode);
    const witnessUint8Array = gunzip(compressedWitness);

    const isRecursive = false;
    await api.commonInitSlabAllocator(CIRCUIT_SIZE);

    // Plus 1 needed!
    const crs = await Crs.new(CIRCUIT_SIZE + 1);
    await api.srsInitSrs(new RawBuffer(crs.getG1Data()), crs.numPoints, new RawBuffer(crs.getG2Data()));

    const acirComposer = await api.acirNewAcirComposer(CIRCUIT_SIZE);
    console.log("Is Recursive:", isRecursive);
    const proof = await api.acirCreateProof(
      acirComposer,
      acirUint8Array,
      witnessUint8Array,
      isRecursive
    );

  
    const verified = await api.acirVerifyProof(acirComposer, proof, isRecursive);

    console.log(verified)

    expect(verified).to.be.true;

  }).timeout(10e4);
});
