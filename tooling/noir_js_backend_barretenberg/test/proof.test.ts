import { expect } from 'chai';
import { reconstructProofWithPublicInputs, splitPublicInputsFromProof } from '../src/proofs';

it('splits off public inputs from a proof correctly', async () => {
  const proofWithPublicInputs: Uint8Array = Uint8Array.from(Array.from({ length: 96 }), (_, i) => i);

  const proofData = splitPublicInputsFromProof(proofWithPublicInputs, 32);

  expect(proofData.publicInputs[0]).to.be.deep.eq(Uint8Array.from(Array.from({ length: 32 }), (_, i) => i));
  expect(proofData.publicInputs[1]).to.be.deep.eq(Uint8Array.from(Array.from({ length: 32 }), (_, i) => i + 32));
  expect(proofData.proof).to.be.deep.eq(Uint8Array.from(Array.from({ length: 32 }), (_, i) => i + 64));
});

it('restores the original array from separate proof and public inputs', async () => {
  const proofWithPublicInputs: Uint8Array = Uint8Array.from(Array.from({ length: 96 }), (_, i) => i);

  const proofData = splitPublicInputsFromProof(proofWithPublicInputs, 32);
  const reconstructedProof = reconstructProofWithPublicInputs(proofData);

  expect(reconstructedProof).to.be.deep.eq(proofWithPublicInputs);
});
