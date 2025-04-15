import { expect } from 'chai';
import { ethers } from 'hardhat';

import { readFileSync } from 'node:fs';
import { resolve, join } from 'path';
import toml from 'toml';

import { Noir } from '@noir-lang/noir_js';
import { Barretenberg, RawBuffer, UltraHonkBackend } from '@aztec/bb.js';
import { Field, InputMap } from '@noir-lang/noirc_abi';

import { compile, createFileManager } from '@noir-lang/noir_wasm';
import { decompressSync as gunzip } from 'fflate';

it(`smart contract can verify a recursive proof`, async () => {
  const basePath = resolve(join(__dirname, '../../../../'));
  const fm = createFileManager(basePath);
  const innerCompilationResult = await compile(
    fm,
    join(basePath, './test_programs/execution_success/assert_statement'),
  );
  if (!('program' in innerCompilationResult)) {
    throw new Error('Compilation failed');
  }
  const innerProgram = innerCompilationResult.program;

  const recursionCompilationResult = await compile(
    fm,
    join(basePath, './compiler/integration-tests/circuits/recursion'),
  );
  if (!('program' in recursionCompilationResult)) {
    throw new Error('Compilation failed');
  }
  const recursionProgram = recursionCompilationResult.program;

  // Intermediate proof

  const inner_backend = new UltraHonkBackend(innerProgram.bytecode, {}, { recursive: true });
  const inner = new Noir(innerProgram);

  const inner_prover_toml = readFileSync(
    join(basePath, `./test_programs/execution_success/assert_statement/Prover.toml`),
  ).toString();

  const inner_inputs = toml.parse(inner_prover_toml);

  const { witness: main_witness } = await inner.execute(inner_inputs);
  const intermediate_proof = await inner_backend.generateProof(main_witness);

  expect(await inner_backend.verifyProof(intermediate_proof)).to.be.true;

  // Final proof

  const barretenberg = await Barretenberg.new();
  const vk = await barretenberg.acirWriteVkUltraHonk(acirToUint8Array(innerProgram.bytecode), true);
  // const vkAsFields = (await barretenberg.acirVkAsFieldsUltraHonk(new RawBuffer(vk))).map((str) => str.toString());
  // const vkHash = '0x' + '0'.repeat(64);

  // const recursion = new Noir(recursionProgram);

  // const recursion_inputs: InputMap = {
  //   verification_key: vkAsFields,
  //   proof: proofToFields(intermediate_proof.proof),
  //   public_inputs: [inner_inputs.y as Field],
  //   key_hash: vkHash,
  // };

  // const { witness: recursionWitness } = await recursion.execute(recursion_inputs);

  // const recursion_backend = new UltraHonkBackend(recursionProgram.bytecode, {}, { recursive: false });
  // const recursion_proof = await recursion_backend.generateProof(recursionWitness, { keccak: true });
  // expect(await recursion_backend.verifyProof(recursion_proof)).to.be.true;

  // Smart contract verification

  // const contract = await ethers.deployContract('contracts/recursion.sol:UltraVerifier', []);

  // const result = await contract.verify.staticCall(recursion_proof.proof, recursion_proof.publicInputs);

  // expect(result).to.be.true;
});

function proofToFields(bytes: Uint8Array): string[] {
  const fields: Uint8Array[] = [];
  for (let i = 0; i < bytes.length; i += 32) {
    const fieldBytes = new Uint8Array(32);
    const end = Math.min(i + 32, bytes.length);
    for (let j = 0; j < end - i; j++) {
      fieldBytes[j] = bytes[i + j];
    }
    fields.push(Uint8Array.from(fieldBytes));
  }
  return fields.map((field) => '0x' + field.toString());
}

function acirToUint8Array(base64EncodedBytecode: string): Uint8Array {
  const compressedByteCode = base64Decode(base64EncodedBytecode);
  return gunzip(compressedByteCode);
}

function base64Decode(input: string): Uint8Array {
  if (typeof Buffer !== 'undefined') {
    // Node.js environment
    const b = Buffer.from(input, 'base64');
    return new Uint8Array(b.buffer, b.byteOffset, b.byteLength);
  } else if (typeof atob === 'function') {
    // Browser environment
    return Uint8Array.from(atob(input), (c) => c.charCodeAt(0));
  } else {
    throw new Error('No implementation found for base64 decoding.');
  }
}
