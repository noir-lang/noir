import {
  BaseParityInputs,
  Fr,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  NUM_BASE_PARITY_PER_ROOT_PARITY,
  RootParityInput,
  RootParityInputs,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { createDebugLogger } from '@aztec/foundation/log';
import { type Tuple } from '@aztec/foundation/serialize';

import { TestContext } from '../mocks/test_context.js';
import { BBNativeRollupProver, type BBProverConfig } from './bb_prover.js';

const logger = createDebugLogger('aztec:bb-prover-parity');

describe('prover/bb_prover/parity', () => {
  let context: TestContext;

  beforeAll(async () => {
    const buildProver = (bbConfig: BBProverConfig) => {
      bbConfig.circuitFilter = ['BaseParityArtifact', 'RootParityArtifact'];
      return BBNativeRollupProver.new(bbConfig);
    };
    context = await TestContext.new(logger, 1, buildProver);
  }, 60_000);

  afterAll(async () => {
    await context.cleanup();
  }, 5000);

  it('proves the parity circuits', async () => {
    const l1ToL2Messages = makeTuple<Fr, typeof NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP>(
      NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
      Fr.random,
    );
    const baseParityInputs = Array.from({ length: NUM_BASE_PARITY_PER_ROOT_PARITY }, (_, i) =>
      BaseParityInputs.fromSlice(l1ToL2Messages, i),
    );

    // Generate the base parity proofs
    const rootInputs = await Promise.all(
      baseParityInputs.map(baseInputs => context.prover.getBaseParityProof(baseInputs)),
    );

    // Verify the base parity proofs
    await expect(
      Promise.all(rootInputs.map(input => context.prover.verifyProof('BaseParityArtifact', input[1]))),
    ).resolves.not.toThrow();

    // Now generate the root parity proof
    const rootChildrenInputs = rootInputs.map(rootInput => {
      const child: RootParityInput = new RootParityInput(rootInput[1], rootInput[0]);
      return child;
    });
    const rootParityInputs: RootParityInputs = new RootParityInputs(
      rootChildrenInputs as Tuple<RootParityInput, typeof NUM_BASE_PARITY_PER_ROOT_PARITY>,
    );
    const rootOutput = await context.prover.getRootParityProof(rootParityInputs);

    // Verify the root parity proof
    await expect(context.prover.verifyProof('RootParityArtifact', rootOutput[1])).resolves.not.toThrow();
  }, 100_000);
});
