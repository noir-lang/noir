import { expect } from "@esm-bundle/chai";
import initACVM, {
  compressWitness,
  decompressWitness,
} from "@noir-lang/acvm_js";
import {
  expectedCompressedWitnessMap,
  expectedWitnessMap,
} from "../shared/witness_compression";

beforeEach(async () => {
  await initACVM();
});

it("successfully compresses the witness", async () => {
  const compressedWitnessMap = compressWitness(expectedWitnessMap);

  expect(compressedWitnessMap).to.be.deep.eq(expectedCompressedWitnessMap);
});

it("successfully decompresses the witness", async () => {
  const witnessMap = decompressWitness(expectedCompressedWitnessMap);

  expect(witnessMap).to.be.deep.eq(expectedWitnessMap);
});
