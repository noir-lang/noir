import { expect } from 'chai';
import { compressWitness, decompressWitness, compressWitnessStack, decompressWitnessStack } from '@noir-lang/acvm_js';
import { expectedCompressedWitnessMap, expectedWitnessMap } from '../shared/witness_compression';
import { expectedCompressedWitnessStack, expectedWitnessStack } from '../shared/nested_acir_call';

it('successfully compresses the witness', () => {
  const compressedWitnessMap = compressWitness(expectedWitnessMap);

  expect(compressedWitnessMap).to.be.deep.eq(expectedCompressedWitnessMap);
});

it('successfully decompresses the witness', () => {
  const witnessMap = decompressWitness(expectedCompressedWitnessMap);

  expect(witnessMap).to.be.deep.eq(expectedWitnessMap);
});

it('successfully compresses the witness stack', () => {
  const compressedWitnessStack = compressWitnessStack(expectedWitnessStack);

  expect(compressedWitnessStack).to.be.deep.eq(expectedCompressedWitnessStack);
});

it('successfully decompresses the witness stack', () => {
  const witnessStack = decompressWitnessStack(expectedCompressedWitnessStack);

  expect(witnessStack).to.be.deep.eq(expectedWitnessStack);
});
