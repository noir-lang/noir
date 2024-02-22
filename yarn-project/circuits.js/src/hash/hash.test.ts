import { times } from '@aztec/foundation/collection';

import { AztecAddress, Fr, FunctionData, FunctionSelector, SideEffect, SideEffectLinkedToNoteHash } from '../index.js';
import { makeAztecAddress, makeVerificationKey } from '../tests/factories.js';
import {
  computeCommitmentNonce,
  computeCommitmentsHash,
  computeFunctionTreeRoot,
  computeMessageSecretHash,
  computeNullifierHash,
  computePublicDataTreeLeafSlot,
  computePublicDataTreeValue,
  computeUniqueCommitment,
  computeVarArgsHash,
  hashConstructor,
  hashVK,
  siloNoteHash,
  siloNullifier,
} from './hash.js';

describe('hash', () => {
  it('hashes VK', () => {
    const vk = makeVerificationKey();
    const res = hashVK(vk.toBuffer());
    expect(res).toMatchSnapshot();
  });

  it('computes function tree root', () => {
    const res = computeFunctionTreeRoot([new Fr(0n), new Fr(0n), new Fr(0n), new Fr(0n)]);
    expect(res).toMatchSnapshot();
  });

  it('hashes constructor info', () => {
    const functionData = new FunctionData(FunctionSelector.empty(), false, true, true);
    const argsHash = new Fr(42);
    const vkHash = Buffer.alloc(32);
    const res = hashConstructor(functionData, argsHash, vkHash);
    expect(res).toMatchSnapshot();
  });

  it('computes commitment nonce', () => {
    const nullifierZero = new Fr(123n);
    const commitmentIndex = 456;
    const res = computeCommitmentNonce(nullifierZero, commitmentIndex);
    expect(res).toMatchSnapshot();
  });

  it('computes unique commitment', () => {
    const nonce = new Fr(123n);
    const innerCommitment = new Fr(456);
    const res = computeUniqueCommitment(nonce, innerCommitment);
    expect(res).toMatchSnapshot();
  });

  it('computes siloed commitment', () => {
    const contractAddress = new AztecAddress(new Fr(123n).toBuffer());
    const uniqueCommitment = new Fr(456);
    const res = siloNoteHash(contractAddress, uniqueCommitment);
    expect(res).toMatchSnapshot();
  });

  it('computes siloed nullifier', () => {
    const contractAddress = new AztecAddress(new Fr(123n).toBuffer());
    const innerNullifier = new Fr(456);
    const res = siloNullifier(contractAddress, innerNullifier);
    expect(res).toMatchSnapshot();
  });

  it('computes public data tree value', () => {
    const value = new Fr(3n);
    const res = computePublicDataTreeValue(value);
    expect(res).toMatchSnapshot();
  });

  it('computes public data tree leaf slot', () => {
    const contractAddress = makeAztecAddress();
    const value = new Fr(3n);
    const res = computePublicDataTreeLeafSlot(contractAddress, value);
    expect(res).toMatchSnapshot();
  });

  it('hashes empty function args', () => {
    const res = computeVarArgsHash([]);
    expect(res).toMatchSnapshot();
  });

  it('hashes function args', () => {
    const args = times(8, i => new Fr(i));
    const res = computeVarArgsHash(args);
    expect(res).toMatchSnapshot();
  });

  it('hashes many function args', () => {
    const args = times(200, i => new Fr(i));
    const res = computeVarArgsHash(args);
    expect(res).toMatchSnapshot();
  });

  it('compute secret message hash', () => {
    const value = new Fr(8n);
    const hash = computeMessageSecretHash(value);
    expect(hash).toMatchSnapshot();
  });

  it('Computes an empty nullifier hash ', () => {
    const emptyNull = SideEffectLinkedToNoteHash.empty();

    const emptyHash = computeNullifierHash(emptyNull).toString();
    expect(emptyHash).toMatchSnapshot();
  });

  it('Computes an empty sideeffect hash ', () => {
    const emptySideEffect = SideEffect.empty();
    const emptyHash = computeCommitmentsHash(emptySideEffect).toString();
    expect(emptyHash).toMatchSnapshot();
  });

  it('Var args hash matches noir', () => {
    const args = times(800, i => new Fr(i));
    const res = computeVarArgsHash(args);
    expect(res).toMatchSnapshot();

    // Value used in "compute_var_args_hash" test in hash.nr
    // console.log("hash", hash);
  });
});
