import { Fr } from '../../fields/fields.js';
import { poseidon2Permutation } from './index.js';

describe('poseidon2Permutation', () => {
  it('test vectors from cpp should match', () => {
    const init = [0, 1, 2, 3].map(i => new Fr(i));
    expect(poseidon2Permutation(init)).toEqual([
      new Fr(0x01bd538c2ee014ed5141b29e9ae240bf8db3fe5b9a38629a9647cf8d76c01737n),
      new Fr(0x239b62e7db98aa3a2a8f6a0d2fa1709e7a35959aa6c7034814d9daa90cbac662n),
      new Fr(0x04cbb44c61d928ed06808456bf758cbf0c18d1e15a7b6dbc8245fa7515d5e3cbn),
      new Fr(0x2e11c5cff2a22c64d01304b778d78f6998eff1ab73163a35603f54794c30847an),
    ]);
  });

  it('test vectors from Noir should match', () => {
    const init = [1n, 2n, 3n, 0x0a0000000000000000n].map(i => new Fr(i));
    expect(poseidon2Permutation(init)).toEqual([
      new Fr(0x0369007aa630f5dfa386641b15416ecb16fb1a6f45b1acb903cb986b221a891cn),
      new Fr(0x1919fd474b4e2e0f8e0cf8ca98ef285675781cbd31aa4807435385d28e4c02a5n),
      new Fr(0x0810e7e9a1c236aae4ebff7d3751d9f7346dc443d1de863977d2b81fe8c557f4n),
      new Fr(0x1f4a188575e29985b6f8ad03afc1f0759488f8835aafb6e19e06160fb64d3d4an),
    ]);
  });
});
