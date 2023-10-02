/* eslint-disable @typescript-eslint/ban-ts-comment */
import { expect } from '@esm-bundle/chai';
import { TEST_LOG_LEVEL } from '../../environment.js';
import { Logger } from 'tslog';
import { initializeResolver } from '@noir-lang/source-resolver';
import newCompiler, { compile, init_log_level as compilerLogLevel } from '@noir-lang/noir_wasm';
import { acvm, abi, generateWitness } from '@noir-lang/noir_js';

import * as TOML from 'smol-toml';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';

const logger = new Logger({ name: 'test', minLevel: TEST_LOG_LEVEL });

const { default: initACVM } = acvm;
const { default: newABICoder } = abi;

await newCompiler();
await newABICoder();
await initACVM();

compilerLogLevel('INFO');

const base_relative_path = '../../../../..';
const circuit_main = 'compiler/integration-tests/test/circuits/main';
const circuit_recursion = 'compiler/integration-tests/test/circuits/recursion';

async function getFile(url: URL): Promise<string> {
  const response = await fetch(url);

  if (!response.ok) throw new Error('Network response was not OK');

  return await response.text();
}

async function getCircuit(noirSource: string) {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  initializeResolver((id: string) => {
    logger.debug('source-resolver: resolving:', id);
    return noirSource;
  });

  return compile({});
}

describe('It compiles noir program code, receiving circuit bytes and abi object.', () => {
  let circuit_main_source;
  let circuit_main_toml;
  let circuit_recursion_source;

  before(async () => {
    const circuit_main_source_url = new URL(`${base_relative_path}/${circuit_main}/src/main.nr`, import.meta.url);
    const circuit_main_toml_url = new URL(`${base_relative_path}/${circuit_main}/Prover.toml`, import.meta.url);

    circuit_main_source = await getFile(circuit_main_source_url);
    circuit_main_toml = await getFile(circuit_main_toml_url);

    const circuit_recursion_source_url = new URL(
      `${base_relative_path}/${circuit_recursion}/src/main.nr`,
      import.meta.url,
    );

    circuit_recursion_source = await getFile(circuit_recursion_source_url);
  });

  it('Should generate valid inner proof for correct input, then verify proof within a proof', async () => {
    const { circuit: main_circuit, abi: main_abi } = await getCircuit(circuit_main_source);
    const main_inputs = TOML.parse(circuit_main_toml);

    const main_program = { bytecode: main_circuit, abi: main_abi };
    const main_backend = new BarretenbergBackend(main_program);

    const main_witnessUint8Array = await generateWitness(main_program, main_inputs);

    const main_proof = await main_backend.generateIntermediateProof(main_witnessUint8Array);
    const main_verification = await main_backend.verifyIntermediateProof(main_proof);

    logger.debug('main_verification', main_verification);

    expect(main_verification).to.be.true;

    const numPublicInputs = 1;
    const { proofAsFields, vkAsFields, vkHash } = await main_backend.generateIntermediateProofArtifacts(
      main_proof,
      numPublicInputs,
    );

    const recursion_inputs = {
      verification_key: vkAsFields,
      proof: proofAsFields,
      public_inputs: [main_inputs.y],
      key_hash: vkHash,
      input_aggregation_object: ['0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0'],
    };

    logger.debug('recursion_inputs', recursion_inputs);

    const { circuit: recursion_circuit, abi: recursion_abi } = await getCircuit(circuit_recursion_source);
    const recursion_program = { bytecode: recursion_circuit, abi: recursion_abi };

    const recursion_backend = new BarretenbergBackend(recursion_program);

    const recursion_witnessUint8Array = await generateWitness(recursion_program, recursion_inputs);

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
