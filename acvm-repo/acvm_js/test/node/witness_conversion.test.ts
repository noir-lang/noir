import { expect } from 'chai';
import { compressWitness, decompressWitness } from '@noir-lang/acvm_js';
import { expectedCompressedWitnessMap, expectedWitnessMap } from '../shared/witness_compression';

it('successfully compresses the witness', () => {
  const compressedWitnessMap = compressWitness(expectedWitnessMap);

  expect(compressedWitnessMap).to.be.deep.eq(expectedCompressedWitnessMap);
});

it('successfully decompresses the witness', () => {
  const witnessMap = decompressWitness(expectedCompressedWitnessMap);

  expect(witnessMap).to.be.deep.eq(expectedWitnessMap);
});
