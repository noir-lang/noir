import { expect } from '@esm-bundle/chai';
import { Logger } from 'tslog';
import * as TOML from 'smol-toml';

import { initializeResolver } from '@noir-lang/source-resolver';
import newCompiler, { compile, init_log_level as compilerLogLevel } from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';

import { getFile } from './utils.js';
import { TEST_LOG_LEVEL } from '../environment.js';

const logger = new Logger({ name: 'test', minLevel: TEST_LOG_LEVEL });

await newCompiler();

compilerLogLevel('INFO');

const test_cases = [
  {
    case: 'tooling/nargo_cli/tests/execution_success/1_mul',
    numPublicInputs: 0,
  },
  {
    case: 'compiler/integration-tests/circuits/main',
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

  // We're ignoring this in the resolver but pass in something sensible.
  return compile('/main.nr');
}

test_cases.forEach((testInfo) => {
  const test_name = testInfo.case.split('/').pop();
  const mochaTest = new Mocha.Test(`${test_name} (Compile, Execute, Prove, Verify)`, async () => {
    const base_relative_path = '../../../../..';
    const test_case = testInfo.case;

    const noir_source = await getFile(`${base_relative_path}/${test_case}/src/main.nr`);

    let noir_program;
    try {
      noir_program = await getCircuit(noir_source);

      expect(await noir_program, 'Compile output ').to.be.an('object');
    } catch (e) {
      expect(e, 'Compilation Step').to.not.be.an('error');
      throw e;
    }

    const backend = new BarretenbergBackend(noir_program);
    const program = new Noir(noir_program, backend);

    const prover_toml = await getFile(`${base_relative_path}/${test_case}/Prover.toml`);
    const inputs = TOML.parse(prover_toml);

    // JS Proving

    const proofWithPublicInputs = await program.generateFinalProof(inputs);

    // JS verification

    const verified = await program.verifyFinalProof(proofWithPublicInputs);
    expect(verified, 'Proof fails verification in JS').to.be.true;
  });

  suite.addTest(mochaTest);
});
