import { BB_RESULT, executeBbClientIvcProof, verifyClientIvcProof } from '@aztec/bb-prover';
import { ClientIvcProof } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

import { jest } from '@jest/globals';
import { encode } from '@msgpack/msgpack';
import fs from 'fs/promises';
import os from 'os';
import path from 'path';
import { fileURLToPath } from 'url';

import {
  MOCK_MAX_COMMITMENTS_PER_TX,
  MockAppCreatorCircuit,
  MockAppReaderCircuit,
  MockPrivateKernelInitCircuit,
  MockPrivateKernelInnerCircuit,
  MockPrivateKernelResetCircuit,
  MockPrivateKernelTailCircuit,
  witnessGenCreatorAppMockCircuit,
  witnessGenMockPrivateKernelInitCircuit,
  witnessGenMockPrivateKernelInnerCircuit,
  witnessGenMockPrivateKernelResetCircuit,
  witnessGenMockPrivateKernelTailCircuit,
  witnessGenReaderAppMockCircuit,
} from './index.js';

/* eslint-disable camelcase */

const logger = createDebugLogger('aztec:clientivc-integration');

jest.setTimeout(120_000);

describe('Client IVC Integration', () => {
  let bbWorkingDirectory: string;
  let bbBinaryPath: string;

  beforeEach(async () => {
    // Create a temp working dir
    bbWorkingDirectory = await fs.mkdtemp(path.join(os.tmpdir(), 'bb-client-ivc-integration-'));
    bbBinaryPath = path.join(path.dirname(fileURLToPath(import.meta.url)), '../../../barretenberg/cpp/build/bin', 'bb');
  });

  async function createClientIvcProof(witnessStack: Uint8Array[], bytecodes: string[]): Promise<ClientIvcProof> {
    await fs.writeFile(
      path.join(bbWorkingDirectory, 'acir.msgpack'),
      encode(bytecodes.map(bytecode => Buffer.from(bytecode, 'base64'))),
    );

    await fs.writeFile(path.join(bbWorkingDirectory, 'witnesses.msgpack'), encode(witnessStack));
    const provingResult = await executeBbClientIvcProof(
      bbBinaryPath,
      bbWorkingDirectory,
      path.join(bbWorkingDirectory, 'acir.msgpack'),
      path.join(bbWorkingDirectory, 'witnesses.msgpack'),
      logger.info,
    );

    if (provingResult.status === BB_RESULT.FAILURE) {
      throw new Error(provingResult.reason);
    }

    return ClientIvcProof.readFromOutputDirectory(bbWorkingDirectory);
  }

  // This test will verify a client IVC proof of a simple tx:
  // 1. Run a mock app that creates two commitments
  // 2. Run the init kernel to process the app run
  // 3. Run the tail kernel to finish the client IVC chain.
  it('Should generate a verifiable client IVC proof from a simple mock tx', async () => {
    const tx = {
      number_of_calls: '0x1',
    };
    // Witness gen app and kernels
    const appWitnessGenResult = await witnessGenCreatorAppMockCircuit({ commitments_to_create: ['0x1', '0x2'] });

    const initWitnessGenResult = await witnessGenMockPrivateKernelInitCircuit({
      app_inputs: appWitnessGenResult.publicInputs,
      tx,
    });

    const tailWitnessGenResult = await witnessGenMockPrivateKernelTailCircuit({
      prev_kernel_public_inputs: initWitnessGenResult.publicInputs,
    });
    // Create client IVC proof
    const bytecodes = [
      MockAppCreatorCircuit.bytecode,
      MockPrivateKernelInitCircuit.bytecode,
      MockPrivateKernelTailCircuit.bytecode,
    ];
    const witnessStack = [appWitnessGenResult.witness, initWitnessGenResult.witness, tailWitnessGenResult.witness];

    const proof = await createClientIvcProof(witnessStack, bytecodes);
    await proof.writeToOutputDirectory(bbWorkingDirectory);
    const verifyResult = await verifyClientIvcProof(bbBinaryPath, bbWorkingDirectory, logger.info);

    expect(verifyResult.status).toEqual(BB_RESULT.SUCCESS);
  });

  // This test will verify a client IVC proof of a more complex tx:
  // 1. Run a mock app that creates two commitments
  // 2. Run the init kernel to process the app run
  // 3. Run a mock app that reads one of those commitments
  // 4. Run the inner kernel to process the second app run
  // 5. Run the reset kernel to process the read request emitted by the reader app
  // 6. Run the tail kernel to finish the client IVC chain
  it('Should generate a verifiable client IVC proof from a complex mock tx', async () => {
    const tx = {
      number_of_calls: '0x2',
    };
    // Witness gen app and kernels
    const creatorAppWitnessGenResult = await witnessGenCreatorAppMockCircuit({ commitments_to_create: ['0x1', '0x2'] });
    const readerAppWitnessGenRult = await witnessGenReaderAppMockCircuit({ commitments_to_read: ['0x2', '0x0'] });

    const initWitnessGenResult = await witnessGenMockPrivateKernelInitCircuit({
      app_inputs: creatorAppWitnessGenResult.publicInputs,
      tx,
    });
    const innerWitnessGenResult = await witnessGenMockPrivateKernelInnerCircuit({
      prev_kernel_public_inputs: initWitnessGenResult.publicInputs,
      app_inputs: readerAppWitnessGenRult.publicInputs,
    });

    const resetWitnessGenResult = await witnessGenMockPrivateKernelResetCircuit({
      prev_kernel_public_inputs: innerWitnessGenResult.publicInputs,
      commitment_read_hints: [
        '0x1', // Reader reads commitment 0x2, which is at index 1 of the created commitments
        MOCK_MAX_COMMITMENTS_PER_TX.toString(), // Pad with no-ops
        MOCK_MAX_COMMITMENTS_PER_TX.toString(),
        MOCK_MAX_COMMITMENTS_PER_TX.toString(),
      ],
    });

    const tailWitnessGenResult = await witnessGenMockPrivateKernelTailCircuit({
      prev_kernel_public_inputs: resetWitnessGenResult.publicInputs,
    });

    // Create client IVC proof
    const bytecodes = [
      MockAppCreatorCircuit.bytecode,
      MockPrivateKernelInitCircuit.bytecode,
      MockAppReaderCircuit.bytecode,
      MockPrivateKernelInnerCircuit.bytecode,
      MockPrivateKernelResetCircuit.bytecode,
      MockPrivateKernelTailCircuit.bytecode,
    ];
    const witnessStack = [
      creatorAppWitnessGenResult.witness,
      initWitnessGenResult.witness,
      readerAppWitnessGenRult.witness,
      innerWitnessGenResult.witness,
      resetWitnessGenResult.witness,
      tailWitnessGenResult.witness,
    ];

    const proof = await createClientIvcProof(witnessStack, bytecodes);
    await proof.writeToOutputDirectory(bbWorkingDirectory);
    const verifyResult = await verifyClientIvcProof(bbBinaryPath, bbWorkingDirectory, logger.info);

    expect(verifyResult.status).toEqual(BB_RESULT.SUCCESS);
  });
});
