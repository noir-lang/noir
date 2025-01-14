/* eslint-disable @typescript-eslint/ban-ts-comment */
import { expect } from '@esm-bundle/chai';
import { TEST_LOG_LEVEL } from '../environment.js';
import { Logger } from 'tslog';
import { acvm, abi, Noir } from '@noir-lang/noir_js';

import * as TOML from 'smol-toml';
import { UltraPlonkBackend } from '@aztec/bb.js';
import { getFile } from './utils.js';
import { Field, InputMap } from '@noir-lang/noirc_abi';
import { createFileManager, compile } from '@noir-lang/noir_wasm';

const logger = new Logger({ name: 'test', minLevel: TEST_LOG_LEVEL });

const { default: initACVM } = acvm;
const { default: newABICoder } = abi;

await newABICoder();
await initACVM();

const base_relative_path = '../../../../..';
const circuit_main = 'test_programs/execution_success/assert_statement';
const circuit_recursion = 'compiler/integration-tests/circuits/recursion';

async function getCircuit(projectPath: string) {
  const fm = createFileManager('/');
  await fm.writeFile('./src/main.nr', await getFile(`${projectPath}/src/main.nr`));
  await fm.writeFile('./Nargo.toml', await getFile(`${projectPath}/Nargo.toml`));
  const result = await compile(fm);
  if (!('program' in result)) {
    throw new Error('Compilation failed');
  }
  return result.program;
}

describe('It compiles noir program code, receiving circuit bytes and abi object.', () => {
  let circuit_main_toml;

  before(async () => {
    circuit_main_toml = await new Response(await getFile(`${base_relative_path}/${circuit_main}/Prover.toml`)).text();
  });

  // TODO(https://github.com/noir-lang/noir/issues/5106): Reinstate this test.
  it.skip('Should generate valid inner proof for correct input, then verify proof within a proof', async () => {
    const main_program = await getCircuit(`${base_relative_path}/${circuit_main}`);
    const main_inputs: InputMap = TOML.parse(circuit_main_toml) as InputMap;

    const main_backend = new UltraPlonkBackend(main_program.bytecode, {}, { recursive: true });

    const { witness: main_witnessUint8Array } = await new Noir(main_program).execute(main_inputs);

    const main_proof = await main_backend.generateProof(main_witnessUint8Array);
    const main_verification = await main_backend.verifyProof(main_proof);

    logger.debug('main_verification', main_verification);

    expect(main_verification).to.be.true;

    const numPublicInputs = 1;
    const { proofAsFields, vkAsFields, vkHash } = await main_backend.generateRecursiveProofArtifacts(
      main_proof,
      numPublicInputs,
    );

    const recursion_inputs: InputMap = {
      verification_key: vkAsFields,
      proof: proofAsFields,
      public_inputs: [main_inputs.y as Field],
      key_hash: vkHash,
    };

    logger.debug('recursion_inputs', recursion_inputs);

    const recursion_program = await getCircuit(`${base_relative_path}/${circuit_recursion}`);

    const recursion_backend = new UltraPlonkBackend(recursion_program.bytecode, {}, { recursive: false });

    const { witness: recursion_witnessUint8Array } = await new Noir(recursion_program).execute(recursion_inputs);

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

    const recursion_verification = await recursion_backend.verifyProof(recursion_proof);

    logger.debug('recursion_verification', recursion_verification);

    expect(recursion_verification).to.be.true;
  }).timeout(60 * 20e3);
});
