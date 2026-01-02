import { expect } from '@esm-bundle/chai';
import { TEST_LOG_LEVEL } from '../environment.js';
import { Logger } from 'tslog';
import { acvm, abi, Noir } from '@noir-lang/noir_js';

import { Barretenberg, deflattenFields, UltraHonkBackend, UltraHonkVerifierBackend } from '@aztec/bb.js';
import { getFile } from './utils.js';
import { InputMap } from '@noir-lang/noirc_abi';
import { createFileManager, compile } from '@noir-lang/noir_wasm';

const logger = new Logger({ name: 'test', minLevel: TEST_LOG_LEVEL });
const debugLogger = logger.debug.bind(logger);

const { default: initACVM } = acvm;
const { default: newABICoder } = abi;

await newABICoder();
await initACVM();

const base_relative_path = '../../../../..';
const circuit_main = 'compiler/integration-tests/circuits/assert_lt';
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
  it('Should generate valid inner proof for correct input, then verify proof within a proof', async () => {
    const main_inputs: InputMap = {
      x: '2',
      y: '3',
    };

    const main_program = await getCircuit(`${base_relative_path}/${circuit_main}`);
    const main_backend = new UltraHonkBackend(main_program.bytecode, { logger: debugLogger }, { recursive: true });

    const { witness: main_witnessUint8Array } = await new Noir(main_program).execute(main_inputs);

    const { proof: intermediateProof, publicInputs: intermediatePublicInputs } =
      await main_backend.generateProof(main_witnessUint8Array);

    // Get verification key for inner circuit as fields
    const innerCircuitVerificationKey = await main_backend.getVerificationKey();
    main_backend.destroy();

    const barretenbergAPI = await Barretenberg.new({ threads: 1 });
    const vkAsFields = await barretenbergAPI.vkAsFields({ verificationKey: innerCircuitVerificationKey });
    const vkHash = await barretenbergAPI.poseidon2Hash({ inputs: vkAsFields.fields });
    barretenbergAPI.destroy();

    const recursion_inputs: InputMap = {
      verification_key: vkAsFields.fields.map((field) => field.toString()),
      proof: deflattenFields(intermediateProof),
      public_inputs: intermediatePublicInputs,
      key_hash: vkHash.hash.toString(),
    };

    logger.debug('recursion_inputs', recursion_inputs);

    const recursion_program = await getCircuit(`${base_relative_path}/${circuit_recursion}`);
    const recursion_backend = new UltraHonkBackend(
      recursion_program.bytecode,
      { logger: debugLogger },
      { recursive: false },
    );

    const { witness: recursion_witnessUint8Array } = await new Noir(recursion_program).execute(recursion_inputs);

    logger.debug('About to generate proof...');
    const recursion_proof = await recursion_backend.generateProof(recursion_witnessUint8Array);
    const verificationKey = await recursion_backend.getVerificationKey();

    const verifierBackend = new UltraHonkVerifierBackend({ logger: debugLogger }, { recursive: false });
    const recursion_verification = await verifierBackend.verifyProof({ ...recursion_proof, verificationKey });

    logger.debug('recursion_verification', recursion_verification);

    expect(recursion_verification).to.be.true;
  }).timeout(60 * 20e3);
});
