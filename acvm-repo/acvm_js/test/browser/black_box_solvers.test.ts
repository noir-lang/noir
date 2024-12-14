import { expect } from '@esm-bundle/chai';
import initACVM, {
  and,
  blake2s256,
  ecdsa_secp256k1_verify,
  ecdsa_secp256r1_verify,
  sha256_compression,
  xor,
} from '@noir-lang/acvm_js';

beforeEach(async () => {
  await initACVM();
});

it('successfully calculates the bitwise AND of two fields', async () => {
  const { and_test_cases } = await import('../shared/black_box_solvers');

  for (const testCase of and_test_cases) {
    const [[lhs, rhs], expectedResult] = testCase;
    expect(and(lhs, rhs)).to.be.eq(expectedResult);
  }
});

it('successfully calculates the bitwise XOR of two fields', async () => {
  const { xor_test_cases } = await import('../shared/black_box_solvers');

  for (const testCase of xor_test_cases) {
    const [[lhs, rhs], expectedResult] = testCase;
    expect(xor(lhs, rhs)).to.be.eq(expectedResult);
  }
});

it('successfully calculates the sha256 hash', async () => {
  const { sha256_compression_test_cases } = await import('../shared/black_box_solvers');

  for (const testCase of sha256_compression_test_cases) {
    const [message, state, expectedResult] = testCase;
    const hash = sha256_compression(message, state);
    hash.forEach((value, index) => expect(value).to.be.eq(expectedResult.at(index)));
  }
});

it('successfully calculates the blake2s256 hash', async () => {
  const { blake2s256_test_cases } = await import('../shared/black_box_solvers');

  for (const testCase of blake2s256_test_cases) {
    const [preimage, expectedResult] = testCase;
    const hash = blake2s256(preimage);
    hash.forEach((value, index) => expect(value).to.be.eq(expectedResult.at(index)));
  }
});

it('successfully verifies secp256k1 ECDSA signatures', async () => {
  const { ecdsa_secp256k1_test_cases } = await import('../shared/black_box_solvers');

  for (const testCase of ecdsa_secp256k1_test_cases) {
    const [[hashed_msg, pubkey_x, pubkey_y, signature], expectedResult] = testCase;

    expect(hashed_msg.length).to.be.eq(32);
    expect(pubkey_x.length).to.be.eq(32);
    expect(pubkey_y.length).to.be.eq(32);
    expect(signature.length).to.be.eq(64);

    const result = ecdsa_secp256k1_verify(hashed_msg, pubkey_x, pubkey_y, signature);
    expect(result).to.be.eq(expectedResult);
  }
});

it('successfully verifies secp256r1 ECDSA signatures', async () => {
  const { ecdsa_secp256r1_test_cases } = await import('../shared/black_box_solvers');

  for (const testCase of ecdsa_secp256r1_test_cases) {
    const [[hashed_msg, pubkey_x, pubkey_y, signature], expectedResult] = testCase;

    expect(hashed_msg.length).to.be.eq(32);
    expect(pubkey_x.length).to.be.eq(32);
    expect(pubkey_y.length).to.be.eq(32);
    expect(signature.length).to.be.eq(64);

    const result = ecdsa_secp256r1_verify(hashed_msg, pubkey_x, pubkey_y, signature);
    expect(result).to.be.eq(expectedResult);
  }
});
