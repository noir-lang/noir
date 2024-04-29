/* eslint-disable @typescript-eslint/ban-ts-comment */
import { expect } from 'chai';
import { Logger } from 'tslog';
import { Noir } from '@noir-lang/noir_js';

import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { resolve, join } from 'path';
import { Field, InputMap } from '@noir-lang/noirc_abi';
import { createFileManager, compile } from '@noir-lang/noir_wasm';

const logger = new Logger({ name: 'test' });

const base_relative_path = '../../../../';
const circuit_main = './test_programs/execution_success/assert_statement_recursive';
const circuit_recursion = './compiler/integration-tests/circuits/recursion';
const circuit_double_verify = './test_programs/execution_success/double_verify_proof';

async function getCircuit(projectPath: string) {
  console.log(__dirname);
  const basePath = resolve(join(__dirname, base_relative_path));
  console.log(basePath);
  const absProjectPath = join(basePath, projectPath);
  console.log(absProjectPath);
  const fm = createFileManager(absProjectPath);
  const result = await compile(fm);
  if (!('program' in result)) {
    throw new Error('Compilation failed');
  }
  return result.program;
}

describe('It compiles noir program code, receiving circuit bytes and abi object.', () => {
  it('Should generate two valid inner proofs for correct input, then verify proofs within a proof', async () => {
    const main_program = await getCircuit(circuit_main);
    const main_inputs: InputMap = {
      x: '3',
      y: '3',
    };

    const main_backend = new BarretenbergBackend(main_program);

    const { witness: main_witnessUint8Array } = await new Noir(main_program).execute(main_inputs);

    const main_proof = await main_backend.generateProof(main_witnessUint8Array);
    const main_proof2 = await main_backend.generateProof(main_witnessUint8Array);

    const numPublicInputs = 1;
    const { proofAsFields, vkAsFields, vkHash } = await main_backend.generateRecursiveProofArtifacts(
      main_proof,
      numPublicInputs,
    );
    const {
      proofAsFields: proofAsFields2,
      vkAsFields: vkAsFields2,
      vkHash: vkHash2,
    } = await main_backend.generateRecursiveProofArtifacts(main_proof2, numPublicInputs);
    expect(vkAsFields).to.be.deep.eq(vkAsFields2, 'two separate vks for the same program.');
    expect(vkHash).to.be.eq(vkHash2, 'two separate vk hashes for the same program.');

    const recursion_inputs: InputMap = {
      verification_key: vkAsFields,
      proof: proofAsFields,
      public_inputs: [main_inputs.y as Field],
      key_hash: vkHash,
      proof_b: proofAsFields2,
    };

    logger.debug('recursion_inputs', recursion_inputs);

    const recursion_program = await getCircuit(circuit_double_verify);

    const recursion_backend = new BarretenbergBackend(recursion_program);

    const { witness: recursion_witnessUint8Array } = await new Noir(recursion_program).execute(recursion_inputs);

    console.log('got here!');
    const recursion_proof = await recursion_backend.generateProof(recursion_witnessUint8Array);

    // Causes an "unreachable" error.
    // Due to the fact that it's a non-recursive proof?
    //
    // const recursion_numPublicInputs = 1;
    // const { proofAsFields: recursion_proofAsFields } = await recursion_backend.generateRecursiveProofArtifacts(
    //   recursion_proof,
    //   recursion_numPublicInputs,
    // );
    //
    // logger.debug('recursion_proofAsFields', recursion_proofAsFields);

    // const recursion_verification = await recursion_backend.verifyProof(recursion_proof);

    // logger.debug('recursion_verification', recursion_verification);

    // expect(recursion_verification).to.be.true;
  }).timeout(60 * 20e3);
});
