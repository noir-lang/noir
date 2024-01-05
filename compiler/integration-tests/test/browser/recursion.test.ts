/* eslint-disable @typescript-eslint/ban-ts-comment */
import { expect } from '@esm-bundle/chai';
import { TEST_LOG_LEVEL } from '../environment.js';
import { Logger } from 'tslog';
import newCompiler, {
  CompiledProgram,
  PathToFileSourceMap,
  compile,
  init_log_level as compilerLogLevel,
} from '@noir-lang/noir_wasm';
import { acvm, abi, Noir } from '@noir-lang/noir_js';

import * as TOML from 'smol-toml';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { getFile } from './utils.js';
import { Field, InputMap } from '@noir-lang/noirc_abi';

const logger = new Logger({ name: 'test', minLevel: TEST_LOG_LEVEL });

const { default: initACVM } = acvm;
const { default: newABICoder } = abi;

await newCompiler();
await newABICoder();
await initACVM();

compilerLogLevel('INFO');

const base_relative_path = '../../../../..';
const circuit_main = 'test_programs/execution_success/assert_statement';
const circuit_recursion = 'compiler/integration-tests/circuits/recursion';

function getCircuit(noirSource: string): CompiledProgram {
  const sourceMap = new PathToFileSourceMap();
  sourceMap.add_source_code('main.nr', noirSource);
  const result = compile('main.nr', undefined, undefined, sourceMap);
  if (!('program' in result)) {
    throw new Error('Compilation failed');
  }

  return result.program;
}

describe('It compiles noir program code, receiving circuit bytes and abi object.', () => {
  let circuit_main_source;
  let circuit_main_toml;
  let circuit_recursion_source;

  before(async () => {
    circuit_main_source = await getFile(`${base_relative_path}/${circuit_main}/src/main.nr`);
    circuit_main_toml = await getFile(`${base_relative_path}/${circuit_main}/Prover.toml`);

    circuit_recursion_source = await getFile(`${base_relative_path}/${circuit_recursion}/src/main.nr`);
  });

  it('Should generate valid inner proof for correct input, then verify proof within a proof', async () => {
    const main_program = getCircuit(circuit_main_source);
    const main_inputs: InputMap = TOML.parse(circuit_main_toml) as InputMap;

    const main_backend = new BarretenbergBackend(main_program);

    const { witness: main_witnessUint8Array } = await new Noir(main_program).execute(main_inputs);

    const main_proof = await main_backend.generateIntermediateProof(main_witnessUint8Array);
    const main_verification = await main_backend.verifyIntermediateProof(main_proof);

    logger.debug('main_verification', main_verification);

    expect(main_verification).to.be.true;

    const numPublicInputs = 1;
    const { proofAsFields, vkAsFields, vkHash } = await main_backend.generateIntermediateProofArtifacts(
      main_proof,
      numPublicInputs,
    );

    const recursion_inputs: InputMap = {
      verification_key: vkAsFields,
      proof: proofAsFields,
      public_inputs: [main_inputs.y as Field],
      key_hash: vkHash,
      input_aggregation_object: ['0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0'],
    };

    logger.debug('recursion_inputs', recursion_inputs);

    const recursion_program = await getCircuit(circuit_recursion_source);

    const recursion_backend = new BarretenbergBackend(recursion_program);

    const { witness: recursion_witnessUint8Array } = await new Noir(recursion_program).execute(recursion_inputs);

    const recursion_proof = await recursion_backend.generateFinalProof(recursion_witnessUint8Array);

    // Causes an "unreachable" error.
    // Due to the fact that it's a non-recursive proof?
    //
    // const recursion_numPublicInputs = 1;
    // const { proofAsFields: recursion_proofAsFields } = await recursion_backend.generateIntermediateProofArtifacts(
    //   recursion_proof,
    //   recursion_numPublicInputs,
    // );
    //
    // logger.debug('recursion_proofAsFields', recursion_proofAsFields);

    const recursion_verification = await recursion_backend.verifyFinalProof(recursion_proof);

    logger.debug('recursion_verification', recursion_verification);

    expect(recursion_verification).to.be.true;
  }).timeout(60 * 20e3);
});
