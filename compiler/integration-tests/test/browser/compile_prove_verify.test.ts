import { expect } from '@esm-bundle/chai';
import * as TOML from 'smol-toml';

import newCompiler, {
  CompiledProgram,
  PathToFileSourceMap,
  compile,
  init_log_level as compilerLogLevel,
} from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { InputMap } from '@noir-lang/noirc_abi';
import { BarretenbergBackend } from '@noir-lang/backend_barretenberg';

import { getFile } from './utils.js';

await newCompiler();

compilerLogLevel('INFO');

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

function getCircuit(noirSource: string): CompiledProgram {
  const sourceMap = new PathToFileSourceMap();
  sourceMap.add_source_code('main.nr', noirSource);

  // We're ignoring this in the resolver but pass in something sensible.
  const result = compile('main.nr', undefined, undefined, sourceMap);
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

    const noir_source = await getFile(`${base_relative_path}/${test_case}/src/main.nr`);

    let noir_program: CompiledProgram;
    try {
      noir_program = getCircuit(noir_source);

      expect(noir_program, 'Compile output ').to.be.an('object');
    } catch (e) {
      expect(e, 'Compilation Step').to.not.be.an('error');
      throw e;
    }

    const backend = new BarretenbergBackend(noir_program);
    const program = new Noir(noir_program, backend);

    const prover_toml = await getFile(`${base_relative_path}/${test_case}/Prover.toml`);
    const inputs: InputMap = TOML.parse(prover_toml) as InputMap;

    // JS Proving

    const proofWithPublicInputs = await program.generateFinalProof(inputs);

    // JS verification

    const verified = await program.verifyFinalProof(proofWithPublicInputs);
    expect(verified, 'Proof fails verification in JS').to.be.true;
  });

  suite.addTest(mochaTest);
});
