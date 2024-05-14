import { expect } from '@esm-bundle/chai';
import * as TOML from 'smol-toml';

import { compile, createFileManager } from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { InputMap } from '@noir-lang/noirc_abi';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';

import { getFile } from './utils.js';

const test_cases = [
  {
    case: 'test_programs/execution_success/1_mul',
    numPublicInputs: 0,
  },
  {
    case: 'test_programs/execution_success/assert_statement',
    numPublicInputs: 1,
  },
];

const suite = Mocha.Suite.create(mocha.suite, 'Noir end to end test');

suite.timeout(60 * 20e3); //20mins

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

test_cases.forEach((testInfo) => {
  const test_name = testInfo.case.split('/').pop();
  const mochaTest = new Mocha.Test(`${test_name} (Compile, Execute, Prove, Verify)`, async () => {
    const base_relative_path = '../../../../..';
    const test_case = testInfo.case;

    let noir_program;
    try {
      noir_program = await getCircuit(`${base_relative_path}/${test_case}`);

      expect(noir_program, 'Compile output ').to.be.an('object');
    } catch (e) {
      expect(e, 'Compilation Step').to.not.be.an('error');
      throw e;
    }

    const backend = new BarretenbergBackend(noir_program);
    const program = new Noir(noir_program, backend);

    const prover_toml = await new Response(await getFile(`${base_relative_path}/${test_case}/Prover.toml`)).text();
    const inputs: InputMap = TOML.parse(prover_toml) as InputMap;

    // JS Proving

    const proofWithPublicInputs = await program.generateProof(inputs);

    // JS verification

    const verified = await program.verifyProof(proofWithPublicInputs);
    expect(verified, 'Proof fails verification in JS').to.be.true;
  });

  suite.addTest(mochaTest);
});
