import { expect } from '@esm-bundle/chai';
import { TEST_LOG_LEVEL } from '../../environment.js';
import { Logger } from 'tslog';
import { initializeResolver } from '@noir-lang/source-resolver';
import newCompiler, { compile, init_log_level as compilerLogLevel } from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';
import { ethers } from 'ethers';
import * as TOML from 'smol-toml';

const provider = new ethers.JsonRpcProvider('http://localhost:8545');
const logger = new Logger({ name: 'test', minLevel: TEST_LOG_LEVEL });

await newCompiler();

compilerLogLevel('INFO');

async function getFile(file_path: string): Promise<string> {
  const file_url = new URL(file_path, import.meta.url);
  const response = await fetch(file_url);

  if (!response.ok) throw new Error('Network response was not OK');

  return await response.text();
}

const FIELD_ELEMENT_BYTES = 32;

const test_cases = [
  {
    case: 'tooling/nargo_cli/tests/execution_success/1_mul',
    compiled: 'compiler/integration-tests/foundry-project/out/1_mul.sol/UltraVerifier.json',
    deployInformation: 'compiler/integration-tests/foundry-project/mul_output.json',
    numPublicInputs: 0,
  },
  {
    case: 'compiler/integration-tests/test/circuits/main',
    compiled: 'compiler/integration-tests/foundry-project/out/main.sol/UltraVerifier.json',
    deployInformation: 'compiler/integration-tests/foundry-project/main_output.json',
    numPublicInputs: 1,
  },
];

const suite = Mocha.Suite.create(mocha.suite, 'Noir end to end test');

suite.timeout(60 * 20e3); //20mins

async function getCircuit(noirSource: string) {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  initializeResolver((id: string) => {
    logger.debug('source-resolver: resolving:', id);
    return noirSource;
  });

  return compile({});
}

function separatePublicInputsFromProof(
  proof: Uint8Array,
  numPublicInputs: number,
): { proof: Uint8Array; publicInputs: Uint8Array[] } {
  const publicInputs = Array.from({ length: numPublicInputs }, (_, i) => {
    const offset = i * FIELD_ELEMENT_BYTES;
    return proof.slice(offset, offset + FIELD_ELEMENT_BYTES);
  });
  const slicedProof = proof.slice(numPublicInputs * FIELD_ELEMENT_BYTES);

  return {
    proof: slicedProof,
    publicInputs,
  };
}

test_cases.forEach((testInfo) => {
  const test_name = testInfo.case.split('/').pop();
  const mochaTest = new Mocha.Test(`${test_name} (Compile, Execute, Prove, Verify)`, async () => {
    const base_relative_path = '../../../../..';
    const test_case = testInfo.case;

    const noir_source = await getFile(`${base_relative_path}/${test_case}/src/main.nr`);

    let compile_output;
    try {
      compile_output = await getCircuit(noir_source);

      expect(await compile_output, 'Compile output ').to.be.an('object');
    } catch (e) {
      expect(e, 'Compilation Step').to.not.be.an('error');
      throw e;
    }

    const noir_program = { bytecode: compile_output.circuit, abi: compile_output.abi };
    const backend = new BarretenbergBackend(noir_program);
    const program = new Noir(noir_program, backend);

    const prover_toml = await getFile(`${base_relative_path}/${test_case}/Prover.toml`);
    const inputs = TOML.parse(prover_toml);

    // JS Proving

    const proofWithPublicInputs = await program.generateFinalProof(inputs);

    // JS verification

    const verified = await program.verifyFinalProof(proofWithPublicInputs);
    expect(verified, 'Proof fails verification in JS').to.be.true;

    // Smart contract verification

    // const compiled_contract = await getFile(`${base_relative_path}/${testInfo.compiled}`);
    // const deploy_information = await getFile(`${base_relative_path}/${testInfo.deployInformation}`);

    // const { abi } = JSON.parse(compiled_contract);
    // const { deployedTo } = JSON.parse(deploy_information);
    // const contract = new ethers.Contract(deployedTo, abi, provider);

    // const { proof, publicInputs } = separatePublicInputsFromProof(proofWithPublicInputs, testInfo.numPublicInputs);
    // const result = await contract.verify(proof, publicInputs);

    // expect(result).to.be.true;
  });

  suite.addTest(mochaTest);
});
