import { times } from '@aztec/foundation/collection';
import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import { AztecAddress, Fr, MAX_ARGS_LENGTH } from '../index.js';
import { makeAztecAddress } from '../tests/factories.js';
import {
  computeNoteHashNonce,
  computePublicDataTreeLeafSlot,
  computePublicDataTreeValue,
  computeSecretHash,
  computeUniqueNoteHash,
  computeVarArgsHash,
  siloNoteHash,
  siloNullifier,
} from './hash.js';

describe('hash', () => {
  setupCustomSnapshotSerializers(expect);

  it('computes note hash nonce', () => {
    const nullifierZero = new Fr(123n);
    const noteHashIndex = 456;
    const res = computeNoteHashNonce(nullifierZero, noteHashIndex);
    expect(res).toMatchSnapshot();
  });

  it('computes unique note hash', () => {
    const nonce = new Fr(123n);
    const innerNoteHash = new Fr(456);
    const res = computeUniqueNoteHash(nonce, innerNoteHash);
    expect(res).toMatchSnapshot();
  });

  it('computes siloed note hash', () => {
    const contractAddress = new AztecAddress(new Fr(123n).toBuffer());
    const uniqueNoteHash = new Fr(456);
    const res = siloNoteHash(contractAddress, uniqueNoteHash);
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
    const hash = computeSecretHash(value);
    expect(hash).toMatchSnapshot();
  });

  it('Var args hash matches noir', () => {
    const args = times(MAX_ARGS_LENGTH, i => new Fr(i));
    const res = computeVarArgsHash(args);
    expect(res).toMatchSnapshot();

    // Value used in "compute_var_args_hash" test in hash.nr
    // console.log("hash", hash);
  });
});
