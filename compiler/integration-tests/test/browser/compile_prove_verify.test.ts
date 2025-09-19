import { expect } from '@esm-bundle/chai';
import * as TOML from 'smol-toml';
import { Logger } from 'tslog';
import { TEST_LOG_LEVEL } from '../environment.js';

import { compile, createFileManager } from '@noir-lang/noir_wasm';
import { Noir } from '@noir-lang/noir_js';
import { InputMap } from '@noir-lang/noirc_abi';
import { UltraHonkBackend, UltraHonkVerifierBackend } from '@aztec/bb.js';

import { getFile } from './utils.js';

const logger = new Logger({ name: 'test', minLevel: TEST_LOG_LEVEL });
const debugLogger = logger.debug.bind(logger);

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

describe('Noir end to end test', function () {
  this.timeout(60 * 20e3); // 20 minutes

  it('a_1_mul (Compile, Execute, Prove, Verify)', async () => {
    const base_relative_path = '../../../../..';
    const test_case = 'test_programs/execution_success/a_1_mul';

    const noir_program = await getCircuit(`${base_relative_path}/${test_case}`);
    expect(noir_program).to.be.an('object');

    const prover_toml = await new Response(await getFile(`${base_relative_path}/${test_case}/Prover.toml`)).text();
    const inputs: InputMap = TOML.parse(prover_toml) as InputMap;

    const program = new Noir(noir_program);
    const { witness } = await program.execute(inputs);

    const backend = new UltraHonkBackend(noir_program.bytecode, { logger: debugLogger });
    const proof = await backend.generateProof(witness);
    await backend.destroy();

    const vkBackend = new UltraHonkBackend(noir_program.bytecode, { logger: debugLogger });
    const verificationKey = await vkBackend.getVerificationKey();
    await vkBackend.destroy();

    const verifier_backend = new UltraHonkVerifierBackend();
    const verified = await verifier_backend.verifyProof({ ...proof, verificationKey });
    expect(verified).to.be.true;
  });

  it('assert_statement (Compile, Execute, Prove, Verify)', async () => {
    const base_relative_path = '../../../../..';
    const test_case = 'test_programs/execution_success/assert_statement';

    const noir_program = await getCircuit(`${base_relative_path}/${test_case}`);
    expect(noir_program).to.be.an('object');

    const prover_toml = await new Response(await getFile(`${base_relative_path}/${test_case}/Prover.toml`)).text();
    const inputs: InputMap = TOML.parse(prover_toml) as InputMap;

    const program = new Noir(noir_program);
    const { witness } = await program.execute(inputs);

    const backend = new UltraHonkBackend(noir_program.bytecode, { logger: debugLogger });
    const proof = await backend.generateProof(witness);
    const verificationKey = await backend.getVerificationKey();
    await backend.destroy();

    const verifier_backend = new UltraHonkVerifierBackend();
    const verified = await verifier_backend.verifyProof({ ...proof, verificationKey });
    expect(verified).to.be.true;
  });
});
