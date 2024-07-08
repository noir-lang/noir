import { BBNativeRollupProver, type BBProverConfig } from '@aztec/bb-prover';
import {
  BaseParityInputs,
  Fr,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
  ParityPublicInputs,
  RECURSIVE_PROOF_LENGTH,
  RootParityInput,
  RootParityInputs,
  VerificationKeyAsFields,
  makeRecursiveProof,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { randomBytes } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';
import { type Tuple } from '@aztec/foundation/serialize';
import { ProtocolCircuitVkIndexes, getVKSiblingPath, getVKTreeRoot } from '@aztec/noir-protocol-circuits-types';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { TestContext } from '../mocks/test_context.js';

const logger = createDebugLogger('aztec:bb-prover-parity');

describe('prover/bb_prover/parity', () => {
  let context: TestContext;
  let bbProver: BBNativeRollupProver;

  beforeAll(async () => {
    const buildProver = async (bbConfig: BBProverConfig) => {
      bbConfig.circuitFilter = ['BaseParityArtifact', 'RootParityArtifact'];
      bbProver = await BBNativeRollupProver.new(bbConfig, new NoopTelemetryClient());
      return bbProver;
    };
    context = await TestContext.new(logger, 1, buildProver);
  });

  afterAll(async () => {
    await context.cleanup();
  });

  it('proves the parity circuits', async () => {
    const l1ToL2Messages = makeTuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>(
      NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
      Fr.random,
    );
    const baseParityInputs = Array.from({ length: NUM_BASE_PARITY_PER_ROOT_PARITY }, (_, i) =>
      BaseParityInputs.fromSlice(l1ToL2Messages, i, getVKTreeRoot()),
    );

    // Generate the base parity proofs
    const rootInputs = await Promise.all(
      baseParityInputs.map(baseInputs => context.prover.getBaseParityProof(baseInputs)),
    );

    // Verify the base parity proofs
    await expect(
      Promise.all(rootInputs.map(input => bbProver.verifyProof('BaseParityArtifact', input.proof.binaryProof))),
    ).resolves.not.toThrow();

    // Now generate the root parity proof
    const rootParityInputs: RootParityInputs = new RootParityInputs(
      rootInputs as Tuple<RootParityInput<typeof RECURSIVE_PROOF_LENGTH>, typeof NUM_BASE_PARITY_PER_ROOT_PARITY>,
    );
    const rootOutput = await context.prover.getRootParityProof(rootParityInputs);

    // Verify the root parity proof
    await expect(bbProver.verifyProof('RootParityArtifact', rootOutput.proof.binaryProof)).resolves.not.toThrow();

    // Now test for negative cases. We will try and generate 3 invalid proofs.
    // One where a single child has an invalid proof
    // One where a child has incorrect public inputs
    // One where a child has an invalid verification key
    // In each case either the proof should fail to generate or verify

    const validVk = rootParityInputs.children[0].verificationKey;
    const baseParityVkPath = getVKSiblingPath(ProtocolCircuitVkIndexes.BaseParityArtifact);
    const validPublicInputs = rootParityInputs.children[0].publicInputs;
    const validProof = rootParityInputs.children[0].proof;

    const defectiveProofInput = new RootParityInput(
      makeRecursiveProof<typeof RECURSIVE_PROOF_LENGTH>(RECURSIVE_PROOF_LENGTH, 0x500),
      validVk,
      baseParityVkPath,
      validPublicInputs,
    );

    const shaRoot = randomBytes(32);
    shaRoot[0] = 0;

    const defectivePublicInputs = new RootParityInput(
      validProof,
      validVk,
      baseParityVkPath,
      new ParityPublicInputs(Fr.fromBuffer(shaRoot), Fr.random(), getVKTreeRoot()),
    );

    const defectiveVerificationKey = new RootParityInput(
      validProof,
      VerificationKeyAsFields.makeFake(),
      baseParityVkPath,
      validPublicInputs,
    );

    const tupleWithDefectiveProof = makeTuple(NUM_BASE_PARITY_PER_ROOT_PARITY, (i: number) => {
      if (i == 0) {
        return defectiveProofInput;
      }
      return rootParityInputs.children[i];
    });

    const tupleWithDefectiveInputs = makeTuple(NUM_BASE_PARITY_PER_ROOT_PARITY, (i: number) => {
      if (i == 0) {
        return defectivePublicInputs;
      }
      return rootParityInputs.children[i];
    });

    const tupleWithDefectiveVK = makeTuple(NUM_BASE_PARITY_PER_ROOT_PARITY, (i: number) => {
      if (i == 0) {
        return defectiveVerificationKey;
      }
      return rootParityInputs.children[i];
    });

    // Check the invalid VK scenario with an invalid witness assertion
    await expect(context.prover.getRootParityProof(new RootParityInputs(tupleWithDefectiveVK))).rejects.toThrow(
      'Failed to generate witness',
    );

    for (const t of [tupleWithDefectiveProof, tupleWithDefectiveInputs]) {
      await expect(async () => {
        const result = await context.prover.getRootParityProof(new RootParityInputs(t));
        await bbProver.verifyProof('RootParityArtifact', result.proof.binaryProof);
        fail('Proof should not be generated and verified');
      }).rejects.toThrow(/Failed to generate proof|Failed to verify proof/);
    }
  });
});
