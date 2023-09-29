import { expect } from "@esm-bundle/chai";
import { initializeResolver } from "@noir-lang/source-resolver";
import newCompiler, { compile } from "@noir-lang/noir_wasm";
import { acvm, abi } from "@noir-lang/noir_js";
import { getRandomMap, getRandomString, getRandomUint8Array } from "./helper";
import * as TOML from "smol-toml";

const { default: initACVM, executeCircuit, compressWitness } = acvm;
const { default: newABICoder, abiEncode } = abi;

const iterations = 5;

describe("compile fuzz test", () => {
  for (let i = 0; i < iterations; i++) {
    it("should error compile() gracefully", async () => {
      await newCompiler();
      await newABICoder();
      await initACVM();
      const fuzzedNoirSource = getRandomString(
        Math.floor(Math.random() * Math.pow(10, i)),
      );
      initializeResolver(() => fuzzedNoirSource);
      let compileOutput;
      let errorOccurred = false;
      try {
        compileOutput = await compile({});
        expect(compileOutput).to.be.an("object");
      } catch (e) {
        expect(e.toString().includes("RuntimeError: unreachable executed")).to
          .be.true;
        errorOccurred = true;
      }
      expect(
        errorOccurred,
        "Compilation should have failed with the fuzzed input",
      ).to.be.true;
    });
  }
});

describe("abiEncode fuzz test", () => {
  for (let i = 0; i < iterations; i++) {
    it(`should error abiEncode() gracefully`, async () => {
      let errorTriggered = false;
      const stringLength = Math.pow(10, i);
      const randomString = getRandomString(stringLength);
      const fuzzedTOML = `${randomString} = "${randomString}"`;
      try {
        const inputs = TOML.parse(fuzzedTOML);
        expect(inputs).to.be.an("object");
        abiEncode(randomString, inputs, null);
      } catch (e) {
        console.log({ e });
        expect(e.toString().includes("Error: invalid type")).to.be.true;
        errorTriggered = true;
      }

      expect(errorTriggered).to.be.true;
    });
  }
});

// timeouts and errors in acvm_js
xdescribe("executeCircuit fuzz test", function () {
  this.timeout(120e3);

  for (let i = 0; i < iterations; i++) {
    it(`should error executeCircuit() gracefully`, async () => {
      let errorTriggered = false;
      const numberLength = Math.pow(10, i);
      try {
        const compressedByteCode = getRandomUint8Array(numberLength);
        const witnessMap = getRandomMap(numberLength);
        await executeCircuit(compressedByteCode, witnessMap, () => {
          throw Error("unexpected oracle");
        }).catch();
      } catch (e) {
        expect(e.toString().includes("RuntimeError: unreachable executed")).to
          .be.true;
        errorTriggered = true;
      }

      expect(errorTriggered).to.be.true;
    });
  }
});

describe("compressWitness fuzz test", () => {
  for (let i = 0; i < iterations; i++) {
    it(`should error compressWitness() gracefully`, async () => {
      let errorTriggered = false;
      const mapLength = Math.pow(10, i);
      const randomMap = getRandomMap(mapLength);
      try {
        const compressedWitness = compressWitness(randomMap);
        expect(compressedWitness).to.be.an("object");
      } catch (e) {
        expect(e.toString().includes("RuntimeError: unreachable executed")).to
          .be.true;
        errorTriggered = true;
      }

      expect(errorTriggered).to.be.true;
    });
  }
});
