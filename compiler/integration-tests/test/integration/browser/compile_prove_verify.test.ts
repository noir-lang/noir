import { expect } from "@esm-bundle/chai";
import { initializeResolver } from "@noir-lang/source-resolver";
import newCompiler, {
  compile,
  init_log_level as compilerLogLevel,
} from "@noir-lang/noir_wasm";
import { acvm, abi } from "@noir-lang/noir_js";
import { Barretenberg, RawBuffer, Crs } from "@aztec/bb.js";
import { decompressSync as gunzip } from "fflate";
import { ethers } from "ethers";
import * as TOML from "smol-toml";

const mnemonic = "test test test test test test test test test test test junk";
const provider = new ethers.JsonRpcProvider("http://localhost:8545");
const walletMnemonic = ethers.Wallet.fromPhrase(mnemonic);
const wallet = walletMnemonic.connect(provider);

const { default: initACVM, executeCircuit, compressWitness } = acvm;
const { default: newABICoder, abiEncode } = abi;

type WitnessMap = acvm.WitnessMap;

await newCompiler();
await newABICoder();
await initACVM();

compilerLogLevel("DEBUG");

async function getFile(url: URL): Promise<string> {
  const response = await fetch(url);

  if (!response.ok) throw new Error("Network response was not OK");

  return await response.text();
}

const CIRCUIT_SIZE = 2 ** 19;

const test_cases = [
  {
    case: "tooling/nargo_cli/tests/execution_success/1_mul",
    compiled: "foundry-project/out/1_mul.sol/UltraVerifier.json",
    address: "MUL_CONTRACT_ADDRESS",
    publicInputsLength: 0,
  },
  {
    case: "tooling/nargo_cli/tests/execution_success/double_verify_proof",
    compiled: "foundry-project/out/double_verify.sol/UltraVerifier.json",
    address: "DV_CONTRACT_ADDRESS",
    publicInputsLength: 16 * 32,
  },
];

const numberOfThreads = navigator.hardwareConcurrency || 1;

const suite = Mocha.Suite.create(mocha.suite, "Noir end to end test");

suite.timeout(60 * 20e3); //20mins

test_cases.forEach((testInfo) => {
  const test_name = testInfo.case.split("/").pop();
  const mochaTest = new Mocha.Test(
    `${test_name} (Compile, Execute, Prove, Verify)`,
    async () => {
      const base_relative_path = "../../../../..";
      const test_case = testInfo.case;

      const noir_source_url = new URL(
        `${base_relative_path}/${test_case}/src/main.nr`,
        import.meta.url,
      );
      const prover_toml_url = new URL(
        `${base_relative_path}/${test_case}/Prover.toml`,
        import.meta.url,
      );
      const compiled_contract_url = new URL(
        `${base_relative_path}/${testInfo.compiled}`,
        import.meta.url,
      );

      const noir_source = await getFile(noir_source_url);
      const prover_toml = await getFile(prover_toml_url);
      const compiled_contract = await getFile(compiled_contract_url);

      const { abi } = JSON.parse(compiled_contract);

      const contract = new ethers.Contract(testInfo.address, abi, wallet);

      expect(noir_source).to.be.a.string;

      initializeResolver((id: string) => {
        console.log("Resolving:", id);
        return noir_source;
      });

      const inputs = TOML.parse(prover_toml);

      expect(inputs, "Prover.toml").to.be.an("object");

      let compile_output;

      try {
        compile_output = await compile({});

        expect(await compile_output, "Compile output ").to.be.an("object");
      } catch (e) {
        expect(e, "Compilation Step").to.not.be.an("error");
        throw e;
      }

      let witnessMap: WitnessMap;
      try {
        witnessMap = abiEncode(compile_output.abi, inputs, null);
      } catch (e) {
        expect(e, "Abi Encoding Step").to.not.be.an("error");
        throw e;
      }

      let solvedWitness: WitnessMap;
      let compressedByteCode;
      try {
        compressedByteCode = Uint8Array.from(
          atob(compile_output.circuit),
          (c) => c.charCodeAt(0),
        );

        solvedWitness = await executeCircuit(
          compressedByteCode,
          witnessMap,
          () => {
            throw Error("unexpected oracle");
          },
        );
      } catch (e) {
        expect(e, "Abi Encoding Step").to.not.be.an("error");
        throw e;
      }

      try {
        const compressedWitness = compressWitness(solvedWitness);
        const acirUint8Array = gunzip(compressedByteCode);
        const witnessUint8Array = gunzip(compressedWitness);

        const isRecursive = true;
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

        // This took ~6.5 minutes!
        const proof = await api.acirCreateProof(
          acirComposer,
          acirUint8Array,
          witnessUint8Array,
          isRecursive,
        );

        // And this took ~5 minutes!
        const verified = await api.acirVerifyProof(
          acirComposer,
          proof,
          isRecursive,
        );

        expect(verified).to.be.true;

        try {
          let result;
          if (testInfo.publicInputsLength === 0) {
            result = await contract.verify(proof, []);
          } else {
            const publicInputs = proof.slice(0, testInfo.publicInputsLength);
            const slicedProof = proof.slice(testInfo.publicInputsLength);
            result = await contract.verify(slicedProof, [publicInputs]);
          }

          expect(result).to.be.true;
        } catch (error) {
          console.error("Error while submitting the proof:", error);
          throw error;
        }
      } catch (e) {
        expect(e, "Proving and Verifying").to.not.be.an("error");
        throw e;
      }
    },
  );

  suite.addTest(mochaTest);
});
